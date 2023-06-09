use cortex_m::{asm::wfi, peripheral::NVIC};
use defmt::{info, warn};
use embedded_hal::serial::Write;
use nb::block;
use stm32f1xx_hal::{
    device::{TIM1, TIM2},
    gpio::gpioa,
    pac::{Interrupt, Peripherals, USART1},
    prelude::*,
    serial::{Config, Rx, Serial, Tx},
    timer::{CounterUs, Event, PwmChannel},
};

use crate::irsender::TCLIRSender;
use crate::receiver::Receiver;

static mut APPLICATION: Option<Application> = None;

// LOGGING_SERIAL is not in APPLICATION because anywhere will call LOGGING_SERIAL
static mut LOGGING_SERIAL: Option<Tx<USART1>> = None;

pub struct Application {
    serial1_rx: Rx<USART1>,
    pwm: PwmChannel<TIM1, 0>,
    timer: CounterUs<TIM2>,
    irsender: TCLIRSender,
    receiver: Receiver,
}

impl Application {
    fn init() {
        let dp = Peripherals::take().unwrap();

        let mut flash = dp.FLASH.constrain();
        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8_000_000.Hz())
            .sysclk(16_000_000.Hz())
            .hclk(16_000_000.Hz())
            .pclk1(16_000_000.Hz())
            .pclk2(16_000_000.Hz())
            .freeze(&mut flash.acr);

        let gpioa::Parts {
            pa8,
            pa9,
            pa10,
            crh: mut acrh,
            ..
        } = dp.GPIOA.split();
        let mut afio = dp.AFIO.constrain();

        let (serial1_tx, mut serial1_rx) = Serial::usart1(
            dp.USART1,
            (
                pa9.into_alternate_push_pull(&mut acrh),
                pa10.into_floating_input(&mut acrh),
            ),
            &mut afio.mapr,
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .split();
        serial1_rx.listen();

        let mut pwm = dp
            .TIM1
            .pwm_hz(
                pa8.into_alternate_push_pull(&mut acrh),
                &mut afio.mapr,
                38.kHz(),
                &clocks,
            )
            .split();
        pwm.set_duty(0);
        pwm.enable();

        let mut timer = dp.TIM2.counter_us(&clocks);
        timer.start(500.micros()).unwrap();
        timer.listen(Event::Update);

        let application = Application {
            pwm,
            serial1_rx,
            timer,
            irsender: TCLIRSender::new(),
            receiver: Receiver::new(),
        };

        unsafe {
            APPLICATION = Some(application);
            LOGGING_SERIAL = Some(serial1_tx);
        }
    }

    fn enable_interrupts() {
        unsafe {
            NVIC::unmask(Interrupt::USART1);
            NVIC::unmask(Interrupt::TIM2);
        }
    }

    fn run_internal(&mut self) -> ! {
        loop {
            wfi();
        }
    }

    fn on_usart1_internal(&mut self) {
        if let Ok(val) = self.serial1_rx.read() {
            if let Some(value) = self.receiver.receive(val) {
                self.irsender.set_value(value);
            }
        } else {
            warn!("USART1 receive error.");
        }
    }

    fn on_tim2_internal(&mut self) {
        self.timer.wait().ok();
        if let Some(value) = self.irsender.next() {
            self.set_pwm_status(value);
        }
    }

    fn set_pwm_status(&mut self, status: bool) {
        if status {
            self.pwm.set_duty(self.pwm.get_max_duty() / 4);
        } else {
            self.pwm.set_duty(0);
        }
    }

    unsafe fn get_application() -> &'static mut Application {
        if let Some(application) = APPLICATION.as_mut() {
            return application;
        } else {
            panic!("No application");
        }
    }

    pub unsafe fn run() -> ! {
        let mut app = None;
        cortex_m::interrupt::free(|_| {
            Self::init();
            Self::enable_interrupts();
            app = Some(Self::get_application());
            info!("initialized.");
        });
        app.unwrap().run_internal();
    }

    pub unsafe fn on_usart1() {
        cortex_m::interrupt::free(|_| {
            Self::get_application().on_usart1_internal();
        });
    }

    pub unsafe fn on_tim2() {
        cortex_m::interrupt::free(|_| {
            Self::get_application().on_tim2_internal();
        });
    }

    pub unsafe fn logger_write(bytes: &[u8]) {
        for b in bytes {
            if let Some(ref mut logging_serial) = LOGGING_SERIAL {
                block!(logging_serial.write(*b)).ok();
            }
        }
    }
}
