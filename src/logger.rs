use crate::app::Application;
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};
use defmt::{error, global_logger, Logger};

#[panic_handler]
fn fn_on_panic(panic_info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();
    unsafe {
        ApplicationLogger::panic_acquire();
    }

    error!("PANIC!!!");

    if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
        error!("INFO: {}", s);
    } else {
        error!("INFO: not string");
    }

    if let Some(location) = panic_info.location() {
        error!(
            "LOCATION: {}, {}",
            location.file().as_bytes(),
            location.line()
        );
    }
    fn_after_panic();
}

fn fn_after_panic() -> ! {
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
static mut LOGGER_PANIC: bool = false;

#[global_logger]
struct ApplicationLogger;

impl ApplicationLogger {
    unsafe fn panic_acquire() {
        LOGGER_PANIC = true;
    }
}

unsafe impl Logger for ApplicationLogger {
    fn acquire() {
        if !unsafe { LOGGER_PANIC } {
            let val = LOGGER_ACQUIRED.swap(true, Ordering::Relaxed);
            if val {
                panic!("Logger re-entrance.");
            }
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
