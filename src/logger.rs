use crate::app::Application;
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
use defmt::{global_logger, Logger};

#[panic_handler]
fn fn_on_panic(panic_info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();

    let logger_write = |val| {
        unsafe { Application::logger_write(val) };
    };
    logger_write(b"PANIC: ");
    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        logger_write(s.as_bytes());
    } else {
        logger_write(b"no info");
    }
    logger_write(b"\n");
    if let Some(location) = panic_info.location() {
        logger_write(b"FILE: ");
        logger_write(location.file().as_bytes());
        logger_write(b"\n LINE: 0x");
        let lineno = location.line();
        let mut lineno_str = [0u8; 8];
        for i in 0..8 {
            lineno_str[i] = match ((lineno >> (28 - 4 * i)) & 0xf) as u8 {
                val if val < 10 => b'0' + val,
                val => b'a' - 10 + val,
            }
        }
        logger_write(&lineno_str);
        logger_write(b"\n");
    }
    loop {
        let mut scb = unsafe { cortex_m::Peripherals::steal() }.SCB;
        let pwr = unsafe { stm32f1xx_hal::pac::Peripherals::steal() }.PWR;
        scb.set_sleepdeep();
        pwr.cr.modify(|_, w| w.pdds().set_bit());
        pwr.cr.modify(|_, w| w.cwuf().clear_bit());
        cortex_m::asm::wfi();
    }
}

static COUNT: AtomicUsize = AtomicUsize::new(0);
defmt::timestamp!("{=usize}", COUNT.fetch_add(1, Ordering::Relaxed));

static LOGGER_ACQUIRED: AtomicBool = AtomicBool::new(false);

#[global_logger]
struct ApplicationLogger;

unsafe impl Logger for ApplicationLogger {
    fn acquire() {
        let val = LOGGER_ACQUIRED.swap(true, Ordering::Relaxed);
        if val {
            panic!("Logger re-entrance.");
        }
    }

    unsafe fn flush() {}

    unsafe fn release() {
        LOGGER_ACQUIRED.store(false, Ordering::Relaxed);
    }

    unsafe fn write(bytes: &[u8]) {
        Application::logger_write(bytes);
    }
}
