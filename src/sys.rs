
use core::panic::PanicInfo;
use core::sync::atomic::Ordering;

#[allow(unused)]
use cortex_m::asm::{self, bkpt, delay, wfi};
#[allow(unused)]
use cortex_m::interrupt::{disable as int_disable, enable as int_enable};
#[allow(unused)]
use cortex_m_rt::{entry, exception, ExceptionFrame};


pub fn reset() -> ! {
    cortex_m::interrupt::disable();
    cortex_m::peripheral::SCB::sys_reset();
}

#[panic_handler]
#[inline(never)]
fn panic(info: &PanicInfo) -> ! {
    int_disable();

    println!("Panic handler! Reseting...");
    println!("Panic info : {}", info);

    loop {
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[inline(never)]
fn hardfault_normal(ef: &ExceptionFrame) -> ! {
    println!("HardFault handler! Reseting...");
    println!("ExceptionFrame : {:?}", ef);

    loop {
        core::sync::atomic::compiler_fence(Ordering::SeqCst);
    }
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    int_disable();
    hardfault_normal(ef)
}


