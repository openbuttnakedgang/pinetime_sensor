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
#[allow(unused)]
use nrf52832_hal::prelude::*;
#[allow(unused)]
use nrf52832_hal::{
    self as hal, 
    pac,
    Twim,
    twim,
    spim,
    gpio,
    target::twim0::frequency
};

// sensor module
mod hrs3300;

// display module
mod ST7789_wrapper;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::spi;
use embedded_hal::blocking::delay::DelayUs;

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
        SPIM1: spim1_peripheral,
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
    let backlight = backlight::Backlight::init(
        gpio.p0_14.into_push_pull_output(Level::High).degrade(),
        gpio.p0_22.into_push_pull_output(Level::High).degrade(),
        gpio.p0_23.into_push_pull_output(Level::High).degrade(),
        1,
    );

    let mut display_wrapper;
    {
        // Set up SPI pins
        let spi_clk = gpio.p0_02
            .into_push_pull_output(Level::Low).degrade();
        let spi_mosi = gpio.p0_03
            .into_push_pull_output(Level::Low).degrade();
        let spi_miso = gpio.p0_04
            .into_floating_input().degrade();
        let spi_pins = spim::Pins { 
            sck: spi_clk, 
            miso: Some(spi_miso), 
            mosi: Some(spi_mosi) 
        };        

        // Set up LCD pins
        // LCD_RS - data/clock pin      (P0.18) 	Clock/data pin (CD)
        let lcd_data_clock = gpio.p0_18
            .into_push_pull_output(Level::Low);
        // LCD_CS - chip select         (P0.25) 	Chip select
        let mut lcd_chip_select = gpio.p0_25
            .into_push_pull_output(Level::Low);
        // LCD_RESET - reset            (P0.26) 	Display reset
        let lcd_reset = gpio.p0_26
            .into_push_pull_output(Level::Low);

        // Initialize SPI
        let spi_interface = spim::Spim::new(
            spim1_peripheral, 
            spi_pins, 
            // Use SPI at 8MHz (the fastest clock available on the nRF52832)
            // because otherwise refreshing will be super slow.
            spim::Frequency::M8, 
            // SPI must be used in mode 3. Mode 0 (the default) won't work.
            spim::MODE_3, 
            0);

        // Chip select must be held low while driving the display. It must be high
        // when using other SPI devices on the same bus (such as external flash
        // storage) so that the display controller won't respond to the wrong
        // commands.
        lcd_chip_select.set_low().unwrap();

        // Set up delay provider on TIMER0
        let delay_provider = crate::delay::TimerDelay::new(timer0_peripheral);
        // Initialize LCD
        let mut display_driver = st7789::ST7789::new(
                spi_interface, 
                lcd_data_clock, lcd_reset, 
                ST7789_wrapper::LCD_W, ST7789_wrapper::LCD_H, 
                delay_provider);

        display_wrapper = ST7789_wrapper::SPIDriver::new(display_driver);
    }        
    display_wrapper.init();
    display_wrapper.draw_text();

    // Battery status
    let battery = battery::BatteryStatus::init(
        gpio.p0_12.into_floating_input(),
        gpio.p0_31.into_floating_input(),
        saadc_peripheral,
    ); 
    
    loop {
        asm::wfi();
    }
}

// fn try_st7789<RST, SPI, DC, DELAY> (display: ST7789_wrapper::SPIDriver<RST, SPI, DC, DELAY>) 
//     -> Result<(), core::fmt::Debug> 
// where
//     SPI: spi::Write<u8>,
//     DC: OutputPin,
//     RST: OutputPin,
//     DELAY: DelayUs<u32>,
// {
//     Ok(())
// }