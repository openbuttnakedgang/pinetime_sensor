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

use nrf52832_hal::gpio::{p0, Floating, Input, Level, Output, Pin, PushPull};
use nrf52832_hal::prelude::*;
use nrf52832_hal::{self as hal, pac};


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
    
    loop {
        asm::nop();
    }
}
