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
    spim,
    gpio,
    target::twim0::frequency
};

// sensor module
mod hrs3300;

// display module
use embedded_graphics::{
    style,
    prelude::*,
    pixelcolor,
    primitives::rectangle,
    fonts
};
use embedded_hal::digital::v2::OutputPin;
const LCD_W: u16 = 240;
const LCD_H: u16 = 240;
const BACKGROUND_COLOR: pixelcolor::Rgb565 = pixelcolor::Rgb565::new(0, 0b000111, 0);
const MARGIN: u16 = 10;

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

    // let mut display_driver;
    // {
    //     // Set up SPI pins
    //     let spi_clk = gpio.p0_02
    //         .into_push_pull_output(Level::Low).degrade();
    //     let spi_mosi = gpio.p0_03
    //         .into_push_pull_output(Level::Low).degrade();
    //     let spi_miso = gpio.p0_04
    //         .into_floating_input().degrade();
    //     let spi_pins = spim::Pins {
    //         sck: spi_clk,
    //         miso: Some(spi_miso),
    //         mosi: Some(spi_mosi)
    //     };

    //     // Set up LCD pins
    //     // LCD_RS - data/clock pin      (P0.18) 	Clock/data pin (CD)
    //     let lcd_data_clock = gpio.p0_18
    //         .into_push_pull_output(Level::Low);
    //     // LCD_CS - chip select         (P0.25) 	Chip select
    //     let mut lcd_chip_select = gpio.p0_25
    //         .into_push_pull_output(Level::Low);
    //     // LCD_RESET - reset            (P0.26) 	Display reset
    //     let lcd_reset = gpio.p0_26
    //         .into_push_pull_output(Level::Low);

    //     // Initialize SPI
    //     let spi_interface = spim::Spim::new(
    //         spim1_peripheral, 
    //         spi_pins, 
    //         // Use SPI at 8MHz (the fastest clock available on the nRF52832)
    //         // because otherwise refreshing will be super slow.
    //         spim::Frequency::M8, 
    //         // SPI must be used in mode 3. Mode 0 (the default) won't work.
    //         spim::MODE_3, 
    //         0);

    //     // Chip select must be held low while driving the display. It must be high
    //     // when using other SPI devices on the same bus (such as external flash
    //     // storage) so that the display controller won't respond to the wrong
    //     // commands.

    //     lcd_chip_select.set_low().unwrap();

    //     // Set up delay provider on TIMER0
    //     let delay_provider = delay::TimerDelay::new(timer0_peripheral);
    //     // Initialize LCD
    //     display_driver = st7789::ST7789::new(
    //         spi_interface, 
    //         lcd_data_clock, lcd_reset, 
    //         LCD_W, LCD_H, delay_provider);

    //     display_driver.init().unwrap();
    //     display_driver.set_orientation(&st7789::Orientation::Portrait).unwrap();

    //     // Draw something onto the LCD
    //     let backdrop_style = style::PrimitiveStyleBuilder::new()
    //         .fill_color(BACKGROUND_COLOR)
    //         .build();
    //     rectangle::Rectangle::new(
    //         Point::new(0, 0), 
    //         Point::new(LCD_W as i32, LCD_H as i32)
    //     )
    //         .into_styled(backdrop_style)
    //         .draw(&mut display_driver)
    //         .unwrap();

    //     // Choose text style
    //     let text_style = style::TextStyleBuilder::new(fonts::Font12x16)
    //         .text_color(pixelcolor::Rgb565::WHITE)
    //         .background_color(BACKGROUND_COLOR);

    //     // Draw text
    //     fonts::Text::new("HRS data ...", Point::new(10, 10))
    //         .into_styled(text_style.build())
    //         .draw(&mut display_driver)
    //         .unwrap();

    //     // Draw text
    //     fonts::Text::new("20%", Point::new(10, 10 + 16 + MARGIN as i32))
    //         .into_styled(text_style.build())
    //         .draw(&mut display_driver)
    //         .unwrap();
    // }

    // Battery status
    let battery = battery::BatteryStatus::init(
        gpio.p0_12.into_floating_input(),
        gpio.p0_31.into_floating_input(),
        saadc_peripheral,
    ); 

    // Set up delay provider on TIMER0
    let mut delay_provider = delay::TimerDelay::new(timer0_peripheral);
    match try_hrs3300(&mut sensor, &mut delay_provider) {
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

    sensor.set_adc_wait_time(hrs3300::ADCWaitTime::Ms100)?;

    sensor.set_gain(hrs3300::Gain::X8)?;

    sensor.set_resolution(hrs3300::BitsResolution::_18)?;

    sensor.set_hrs_active(true)?;

    sensor.set_osc_active(true)?;

    let gains = [
        hrs3300::Gain::X1,
        hrs3300::Gain::X2,
        hrs3300::Gain::X4,
        hrs3300::Gain::X8,
        hrs3300::Gain::X64,
    ];

    for gain in gains.iter() {
        sensor.set_gain(*gain)?;

        let mut sum0 = 0_u32;
        let mut sum1 = 0_u32;

        let count = 1000_usize;
        for _ in 0..count {
            sum0 += sensor.get_ch0()?;
            sum1 += sensor.get_ch1()?;
        }

        println!("g: {:?}, 0: {}, 1: {}", gain, sum0 / count as u32, sum1 / count as u32);
    }

    // let mut values = [(0_u32, 0_u32); 20];

    // for value in values.iter_mut() {
    //     value.0 = sensor.get_ch0()?;
    //     value.1 = sensor.get_ch1()?;
    //     delay_provider.delay_us(sensor.get_adc_wait_time_us());
    // }

    // println!("__");
    // for value in values.iter() {
    //     println!("{:?}", value);
    // }
    // println!("__");

    info!("HRS3300 osc deactivation:");
    sensor.set_osc_active(false)?;

    info!("HRS3300 sensor off:");
    sensor.set_hrs_active(false)?;

    Ok(())
}