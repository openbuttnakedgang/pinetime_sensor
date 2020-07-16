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
use nrf52832_hal::{
    twim,
    timer,
};

mod init;

// sensor module
use embedded_hal::blocking::delay::DelayUs;
mod hrs3300;
mod ppg_processor;
use core::sync::atomic;
static GLOBAL_ALS: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
static GLOBAL_HRS: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
static GLOBAL_SUM: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);

// display module
#[allow(non_snake_case)]
mod ST7789_wrapper;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::blocking::spi;

#[entry]
fn main() -> ! {
    emblog::init_with_level(log::Level::Trace).unwrap();

    error!("Test error log lvl");
    warn! ("Test warn log lvl");
    info! ("Test info log lvl");
    debug!("Test debug log lvl");
    trace!("Test trace log lvl");
    
    #[allow(unused)]
    let init::Components {
        mut display_wrapper, 
        mut sensor, 
        mut backlight, 
        mut battery,
        mut delay_provider
    } = init::Components::new();

    match try_st7789(&mut display_wrapper, &mut delay_provider) {
        Result::Err(err) => {
            println!("error! {:?}", err);
        },
        Result::Ok(()) => {
            println!("display success!");
        }
    } 
    
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

fn try_hrs3300<D> (
        sensor: &mut hrs3300::I2cDriver, 
        delay_provider: &mut delay::TimerDelay<D>
    ) 
-> Result<(), twim::Error>
where
    D:  timer::Instance
{       
    info!("HRS3300 usage starts");

    sensor.init()?;

    sensor.set_hrs_active(true)?;

    sensor.set_osc_active(true)?;
    
    for _ in 0..5000 {
        let raw_sample = sensor.read_raw_sample()?;

        GLOBAL_HRS.store(raw_sample.hrs, atomic::Ordering::Relaxed);
        GLOBAL_ALS.store(raw_sample.als,  atomic::Ordering::Relaxed);
        GLOBAL_SUM.store(raw_sample.get_sum(), atomic::Ordering::Relaxed);
            
        delay_provider.delay_us(sensor.get_adc_wait_time_us());
    }

    info!("HRS3300 osc deactivation:");
    sensor.set_osc_active(false)?;

    info!("HRS3300 sensor off:");
    sensor.set_hrs_active(false)?;

    Ok(())
}

fn try_st7789<RST, SPI, DC, DELAY, E, T> (
    display: &mut ST7789_wrapper::SPIDriver<RST, SPI, DC, DELAY>,
    delay_provider: &mut delay::TimerDelay<T>
)
-> Result<(), st7789::Error<SPI::Error, DC::Error, RST::Error>>
where
    SPI: spi::Write<u8>,
    DC: OutputPin<Error = E>,
    RST: OutputPin<Error = E>,
    DELAY: DelayUs<u32>,
    E: core::fmt::Debug,
    T: timer::Instance
{
    display.init()?;
    display.draw_backgound()?;
    display.draw_axes()?;
    display.count_sin();
    display.draw_sin()?;

    for _ in 0..100 {
        display.clear_sin()?;
        display.rotate_sin();
        display.draw_axes()?;
        display.draw_sin()?;
        delay_provider.delay_us(33_000);
    }

    display.display_driver.hard_reset()?;
    
    Ok(())
}