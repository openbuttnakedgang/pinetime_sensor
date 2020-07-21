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
    pac,
};

mod init;

// sensor module
use embedded_hal::blocking::delay::DelayUs;
mod hrs3300;
mod ppg_processor;
use core::sync::atomic;
#[no_mangle]
static GLOBAL_ALS: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
#[no_mangle]
static GLOBAL_HRS: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
#[no_mangle]
static GLOBAL_SUM: atomic::AtomicU32 = atomic::AtomicU32::new(0_u32);
type SensorTimerType = pac::TIMER0;
type SensorDelayProviderType = delay::TimerDelay<SensorTimerType>;

// display module
#[allow(non_snake_case)]
mod display;
type DisplayTimerType = pac::TIMER1;
type DisplayDelayProviderType = delay::TimerDelay<DisplayTimerType>;

#[entry]
fn main() -> ! {
    #[allow(unused)]
    let init::Components {
        mut display_wrapper, 
        mut sensor, 
        mut backlight, 
        mut battery,
        mut delay_provider
    } = init::Components::new();

    // try_scan_display(&mut sensor, &mut display_wrapper, &mut delay_provider).expect("trying scan and display");    
    try_hrs3300(&mut sensor, &mut delay_provider).unwrap();

    loop {
        asm::wfi();
    }
}

#[no_mangle]
fn try_scan_display(
    sensor: &mut hrs3300::Sensor, 
    display: &mut display::DisplayDriver, 
    delay_provider: &mut SensorDelayProviderType
)
-> Result<(), core::convert::Infallible>
{
    // init sensor
    sensor.init().unwrap();
    sensor.set_hrs_active(true).unwrap();
    sensor.set_osc_active(true).unwrap();

    // init display
    display.init().unwrap();
    display.draw_backgound().unwrap();
    display.draw_axes().unwrap();
    display.count_sin();
    display.draw_sin().unwrap();

    println!("start");
    let mut samples = 0_u32;    
    let mut display_updates = 0_u32;    

    let mut time_us = 0_u64;
    let sensor_update_time = 12_000_u64; // value < 0.1 ms
    let display_update_time = 432_000_u64; // 432 ms
    // scan and draw samples
    for _ in 0..1_000_000_u64 {
        delay_provider.delay_us(1000_u32);
        time_us += 1000_u64;

        if time_us % display_update_time == 0 {
            display.update().unwrap();

            time_us += display_update_time;
            display_updates += 1;
        } else if time_us % sensor_update_time == 0 {        
            let raw_sample = sensor.read_raw_sample().unwrap();

            GLOBAL_HRS.store(raw_sample.hrs, atomic::Ordering::Relaxed);
            GLOBAL_ALS.store(raw_sample.als,  atomic::Ordering::Relaxed);
            GLOBAL_SUM.store(raw_sample.get_sum(), atomic::Ordering::Relaxed);
            
            samples += 1;
            println!("samples: {}, display: {}", samples, display_updates);
        } 
    }

    GLOBAL_HRS.store(0, atomic::Ordering::Relaxed);
    GLOBAL_ALS.store(0,  atomic::Ordering::Relaxed);
    GLOBAL_SUM.store(0, atomic::Ordering::Relaxed);

    // turn off sensor
    sensor.set_osc_active(false).unwrap();
    sensor.set_hrs_active(false).unwrap();

    // turn off display
    display.display_driver.hard_reset().unwrap();

    Ok(())
}

#[allow(unused)]
fn try_hrs3300(sensor: &mut hrs3300::Sensor, delay_provider: &mut SensorDelayProviderType) 
-> Result<(), hrs3300::SensorError>
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

#[allow(unused)]
fn try_st7789(display: &mut display::DisplayDriver, delay_provider: &mut SensorDelayProviderType)
-> Result<(), display::DisplayErrorType>
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

    display.display_driver.hard_reset().unwrap();
    
    Ok(())
}