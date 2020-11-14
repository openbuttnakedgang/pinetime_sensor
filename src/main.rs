#![no_std]
#![no_main]
#![allow(unused_imports)]

#[macro_use]
mod macros;
mod sys;

use cortex_m_rt::entry;

use {
    core::{
        panic::PanicInfo,
        sync::atomic::{compiler_fence, Ordering},
    },
    nrf52832_hal as hal,
};

use nrf52832_hal::{
    gpio::*,
    pac,
    twim::{self, Twim},
};

#[entry]
fn main() -> ! {
    mmain()
}

#[no_mangle]
#[inline(never)]
fn mmain() -> ! {
    println!("TEST!");
    println!("TEST!");
    println!("TEST!");
    println!("TEST!");
    println!("TEST!");
    println!("TEST!");
    println!("TEST!");

    let p = pac::Peripherals::take().unwrap();
    let port0 = p0::Parts::new(p.P0);

    let sda = port0.p0_06.into_floating_input().degrade();
    let scl = port0.p0_07.into_floating_input().degrade();

    let pins = twim::Pins { scl, sda };

    let i2c = Twim::new(p.TWIM0, pins, twim::Frequency::K400);

    let mut sensor = hrs3300::Hrs3300::new(i2c);
    sensor.init().unwrap();
    sensor.enable_hrs().unwrap();
    sensor.enable_oscillator().unwrap();

    loop {
        //asm::wfi();
        let hrs = sensor.read_hrs().unwrap();
        unsafe { HRS = hrs; }
        let als = sensor.read_als().unwrap();
        unsafe { ALS = als; }
        println!("HRS: {}, ALS: {}", hrs, als);
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    }
}

#[no_mangle]
pub static mut HRS: u32 = 0;
#[no_mangle]
pub static mut ALS: u32 = 0;