use crate::app::Application;
use cortex_m_rt::entry;
use stm32f1xx_hal::pac::interrupt;

#[entry]
unsafe fn main() -> ! {
    Application::run();
}

#[interrupt]
unsafe fn USART1() {
    Application::on_usart1();
}

#[interrupt]
unsafe fn TIM2() {
    Application::on_tim2();
}
