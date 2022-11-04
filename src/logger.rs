use crate::app::Application;
use core::panic::PanicInfo;
use defmt::{global_logger, Logger};

#[panic_handler]
fn fn_on_panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[global_logger]
struct ApplicationLogger;

unsafe impl Logger for ApplicationLogger {
    fn acquire() {}

    unsafe fn flush() {}

    unsafe fn release() {}

    unsafe fn write(bytes: &[u8]) {
        Application::logger_write(bytes);
    }
}
