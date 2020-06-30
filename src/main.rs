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
    p0, 
    Floating, 
    Input, 
    Level, 
    Output, 
    Pin, 
    PushPull
};
use nrf52832_hal::prelude::*;
use nrf52832_hal::{
    self as hal, 
    pac,
    Twim,
    twim
};


#[entry]
fn main() -> ! {
    emblog::init_with_level(log::Level::Trace).unwrap();

    error!("Test error log lvl");
    warn! ("Test warn log lvl");
    info! ("Test info log lvl");
    debug!("Test debug log lvl");
    trace!("Test trace log lvl");

    let pac::Peripherals {
        CLOCK,
        FICR,
        P0,
        RADIO,
        SAADC,
        SPIM1,
        TIMER0,
        TIMER1,
        TIMER2,
        TWIM0,
        ..
    } = pac::Peripherals::take().unwrap();    

    // Set up clocks. On reset, the high frequency clock is already used,
    // but we also need to switch to the external HF oscillator. This is
    // needed for Bluetooth to work.
    let _clocks = hal::clocks::Clocks::new(CLOCK).enable_ext_hfosc();

    // Set up delay provider on TIMER0
    let delay = delay::TimerDelay::new(TIMER0);

    // Set up GPIO peripheral
    let gpio = hal::gpio::p0::Parts::new(P0);
    
    //----------------------------- Sensor init ---------------------------------------
    // up to 800 kHz
    // 525nm green
    trace!("sensor init starts");
    // P0.06 : I²C SDA
    let sda = gpio.p0_06.into_floating_input().degrade();
    // P0.07 : I²C SCL
    let scl = gpio.p0_07.into_floating_input().degrade();
    // pins for TWIM0
    let pins = twim::Pins { scl, sda };
    // sensor instance
    let mut sensor = Twim::new(TWIM0, pins, 
        nrf52832_hal::target::twim0::frequency::FREQUENCY_A::K400);

    //sensor setup
    const sensor_addr: u8 = 0x44;
    const BUFF_LEN: usize = 8;
    let mut read_buff = [0_u8; BUFF_LEN];
    let mut write_buff = [0_u8; BUFF_LEN];
    match sensor.write_read(sensor_addr, &write_buff, &mut read_buff) {
        core::result::Result::Err(err) => {
            match err {
                twim::Error::TxBufferTooLong => error!("TxBufferTooLong"),
                twim::Error::RxBufferTooLong => error!("RxBufferTooLong"),
                twim::Error::Transmit => error!("Transmit"),
                twim::Error::Receive => error!("Receive"),
                twim::Error::DMABufferNotInDataMemory => error!("DMABufferNotInDataMemory"),
            }
        },
        core::result::Result::Ok(_) => trace!("Sent ok")
    }
    

    trace!("sensor init ends");    
    //----------------------------- Sensor init end---------------------------------------

    // Enable backlight
    let backlight = backlight::Backlight::init(
        gpio.p0_14.into_push_pull_output(Level::High).degrade(),
        gpio.p0_22.into_push_pull_output(Level::High).degrade(),
        gpio.p0_23.into_push_pull_output(Level::High).degrade(),
        1,
    );

    // Battery status
    let battery = battery::BatteryStatus::init(
        gpio.p0_12.into_floating_input(),
        gpio.p0_31.into_floating_input(),
        SAADC,
    );
    
    //----------------------------- Sensor interact ---------------------------------------
    //----------------------------- Sensor interact end ---------------------------------------
    
    loop {
        asm::nop();
    }
}