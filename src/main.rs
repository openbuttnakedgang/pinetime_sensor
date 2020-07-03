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
mod hrs3300;

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
        // TIMER0,
        // TIMER1,
        // TIMER2,
        TWIM0: twim0_peripheral,
        ..
    } = pac::Peripherals::take().unwrap();    

    // Set up clocks. On reset, the high frequency clock is already used,
    // but we also need to switch to the external HF oscillator. This is
    // needed for Bluetooth to work.
    let _clocks = hal::clocks::Clocks::new(clock_peripheral).enable_ext_hfosc();

    // Set up delay provider on TIMER0
    // let delay = delay::TimerDelay::new(TIMER0);

    // Set up GPIO peripheral
    let gpio_peripheral = hal::gpio::p0::Parts::new(p0_peripheral);

    // P0.06 : I²C SDA
    let sda = gpio_peripheral.p0_06.into_floating_input().degrade();
    // P0.07 : I²C SCL
    let scl = gpio_peripheral.p0_07.into_floating_input().degrade();
    // pins for TWIM0
    let pins = twim::Pins { scl, sda };
    
    let twim_driver = Twim::new(twim0_peripheral, pins, frequency::FREQUENCY_A::K400);

    let mut sensor = hrs3300::I2cDriver::new(twim_driver);
    match hrs3300::try_hrs3300(&mut sensor) {
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

    // Enable backlight
    let backlight = backlight::Backlight::init(
        gpio_peripheral.p0_14.into_push_pull_output(Level::High).degrade(),
        gpio_peripheral.p0_22.into_push_pull_output(Level::High).degrade(),
        gpio_peripheral.p0_23.into_push_pull_output(Level::High).degrade(),
        1,
    );

    // Battery status
    let battery = battery::BatteryStatus::init(
        gpio_peripheral.p0_12.into_floating_input(),
        gpio_peripheral.p0_31.into_floating_input(),
        saadc_peripheral,
    );
    
    loop {
        asm::nop();
    }
}