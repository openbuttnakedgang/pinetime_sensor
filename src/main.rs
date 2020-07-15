#![no_std]
#![no_main]

#[macro_use]
extern crate log;

#[macro_use]
mod macros;
mod emblog;
mod sys;
mod backlight;
mod battery;
mod delay;

use cortex_m::asm;
use cortex_m_rt::entry;

use nrf52832_hal::gpio::{
//     p0, 
//     Floating, 
//     Input, 
    Level, 
//     Output, 
//     Pin, 
//     PushPull
};
// use nrf52832_hal::prelude::*;
use nrf52832_hal::{
    self as hal, 
    pac,
    Twim,
    twim,
    target::twim0::frequency
};

// sensor module
use embedded_hal::blocking::delay::DelayUs;
mod hrs3300;
mod ppg_processor;
use core::sync::atomic;
static GLOBAL_ALS: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
static GLOBAL_HRS: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
static GLOBAL_SUM: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
static GLOBAL_FIL: atomic::AtomicI32 = atomic::AtomicI32::new(0_i32);

#[entry]
fn main() -> ! {
    emblog::init_with_level(log::Level::Trace).unwrap();

    error!("Test error log lvl");
    warn! ("Test warn log lvl");
    info! ("Test info log lvl");
    debug!("Test debug log lvl");
    trace!("Test trace log lvl");

    let pac::Peripherals {
        CLOCK: clock_peripheral,
        // FICR,
        P0: p0_peripheral,
        // RADIO,
        SAADC: saadc_peripheral,
        // SPIM1,
        TIMER0: timer0_peripheral,
        // TIMER1: timer1_peripheral,
        // TIMER2,
        TWIM0: twim0_peripheral,
        // SPIM1: spim1_peripheral,
        ..
    } = pac::Peripherals::take().unwrap();    

    // Set up clocks. On reset, the high frequency clock is already used,
    // but we also need to switch to the external HF oscillator. This is
    // needed for Bluetooth to work.
    let _clocks = hal::clocks::Clocks::new(clock_peripheral).enable_ext_hfosc();


    // Set up GPIO peripheral
    let gpio = hal::gpio::p0::Parts::new(p0_peripheral);
    
    let mut sensor;
    {
        // P0.06 : I²C SDA
        let sda = gpio.p0_06.into_floating_input().degrade();
        // P0.07 : I²C SCL
        let scl = gpio.p0_07.into_floating_input().degrade();
        // pins for TWIM0
        let pins = twim::Pins { scl, sda };    
        let twim_driver = Twim::new(twim0_peripheral, pins, frequency::FREQUENCY_A::K400);
        sensor = hrs3300::I2cDriver::new(twim_driver);
    }   

    // Enable backlight
    #[allow(unused)]
    let backlight = backlight::Backlight::init(
        gpio.p0_14.into_push_pull_output(Level::High).degrade(),
        gpio.p0_22.into_push_pull_output(Level::High).degrade(),
        gpio.p0_23.into_push_pull_output(Level::High).degrade(),
        1,
    );

    // Battery status
    #[allow(unused)]
    let battery = battery::BatteryStatus::init(
        gpio.p0_12.into_floating_input(),
        gpio.p0_31.into_floating_input(),
        saadc_peripheral,
    ); 

    // Set up delay provider on TIMER0
    let mut delay_provider = delay::TimerDelay::new(timer0_peripheral);
    match try_hrs3300(&mut sensor, &mut delay_provider) 
    {
        Result::Err(err) => {
            match err {
                twim::Error::TxBufferTooLong => error!("\tTxBufferTooLong\n"),
                twim::Error::RxBufferTooLong => error!("\tRxBufferTooLong\n"),
                twim::Error::Transmit => error!("\tTransmit\n"),
                twim::Error::Receive => error!("\tReceive\n"),
                twim::Error::DMABufferNotInDataMemory => error!("\tDMABufferNotInDataMemory\n"),
            }
        },
        Result::Ok(_) => info!("HRS3300 usage successful!")
    }
    
    loop {
        asm::wfi();
    }
}

fn try_hrs3300<T, E> (
    sensor: &mut hrs3300::I2cDriver<T>, 
    delay_provider: &mut delay::TimerDelay) -> Result<(), E> 
where
    T:  embedded_hal::blocking::i2c::Write::<Error = E> + 
        embedded_hal::blocking::i2c::Read::<Error = E> + 
        embedded_hal::blocking::i2c::WriteRead::<Error = E>,
    E:  core::fmt::Debug
{       
    info!("HRS3300 usage starts");

    sensor.init()?;

    sensor.set_hrs_active(true)?;

    sensor.set_osc_active(true)?;

    // let mut ppg_processor = ppg_processor::PpgFilter::new();
    
    for _ in 0..1000000 {
        let raw_sample = sensor.read_raw_sample()?;

        GLOBAL_HRS.store(raw_sample.hrs, atomic::Ordering::Relaxed);
        GLOBAL_ALS.store(raw_sample.als,  atomic::Ordering::Relaxed);
        GLOBAL_SUM.store(raw_sample.get_sum(), atomic::Ordering::Relaxed);
        // GLOBAL_FIL.store(
        //     ppg_processor.consume_value(raw_sample) as i32, 
        //     atomic::Ordering::Relaxed);
            
        delay_provider.delay_us(sensor.get_adc_wait_time_us());
    }

    info!("HRS3300 osc deactivation:");
    sensor.set_osc_active(false)?;

    info!("HRS3300 sensor off:");
    sensor.set_hrs_active(false)?;

    Ok(())
}