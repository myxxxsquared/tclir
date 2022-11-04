use embedded_hal::serial::Write;
use nb::block;
use stm32f1xx_hal::{
    pac,
    prelude::*,
    rtc::Rtc,
    serial::{Config, Serial},
};

use crate::timesync::TimeSync;
use crate::timewriter::TimeWriter;

static mut APPLICATION: Option<Application> = None;

pub struct Application {
    rtc: Rtc,
    time_sync: TimeSync,
    time_writer: TimeWriter,
    serial1_tx: stm32f1xx_hal::serial::Tx<stm32f1xx_hal::pac::USART1>,
    serial1_rx: stm32f1xx_hal::serial::Rx<stm32f1xx_hal::pac::USART1>,
    serial2_tx: stm32f1xx_hal::serial::Tx<stm32f1xx_hal::pac::USART2>,
}

impl Application {
    fn init() {
        let dp = pac::Peripherals::take().unwrap();

        let mut flash = dp.FLASH.constrain();
        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            .use_hse(8_000_000.Hz())
            .sysclk(72_000_000.Hz())
            .hclk(72_000_000.Hz())
            .adcclk(12_000_000.Hz())
            .pclk1(36_000_000.Hz())
            .pclk2(72_000_000.Hz())
            .freeze(&mut flash.acr);

        let mut gpioa = dp.GPIOA.split();
        let mut afio = dp.AFIO.constrain();

        let mut pwr = dp.PWR;
        let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut pwr);
        let mut rtc = Rtc::new(dp.RTC, &mut backup_domain);
        rtc.select_frequency(1.Hz());
        rtc.listen_seconds();

        let (serial1_tx, mut serial1_rx) = Serial::usart1(
            dp.USART1,
            (
                gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh),
                gpioa.pa10.into_floating_input(&mut gpioa.crh),
            ),
            &mut afio.mapr,
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .split();
        serial1_rx.listen();

        let (serial2_tx, _) = Serial::usart2(
            dp.USART2,
            (
                gpioa.pa2.into_alternate_push_pull(&mut gpioa.crl),
                gpioa.pa3.into_floating_input(&mut gpioa.crl),
            ),
            &mut afio.mapr,
            Config::default().baudrate(115_200.bps()),
            clocks,
        )
        .split();

        let application = Application {
            rtc,
            time_sync: TimeSync::new(),
            time_writer: TimeWriter::new(),
            serial1_tx,
            serial1_rx,
            serial2_tx,
        };

        unsafe {
            APPLICATION = Some(application);
            cortex_m::peripheral::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::RTC);
            cortex_m::peripheral::NVIC::unmask(stm32f1xx_hal::pac::Interrupt::USART1);
        }
    }

    fn run_internal(&mut self) -> ! {
        loop {}
    }

    fn on_rtc_internal(&mut self) {
        self.rtc.clear_second_flag();
        let timestamp = self.rtc.current_time();
        self.time_writer.write(timestamp, |val| {
            block!(self.serial2_tx.write(val)).unwrap();
        });
    }

    fn on_usart1_internal(&mut self) {
        if let Ok(val) = self.serial1_rx.read() {
            self.time_sync.receive_word(val, |val| {
                self.rtc.set_time(val);
            });
        }
    }

    pub unsafe fn run() -> ! {
        Self::init();
        Self::get_application().run_internal();
    }

    pub unsafe fn on_rtc() {
        Self::get_application().on_rtc_internal();
    }

    pub unsafe fn on_usart1() {
        Self::get_application().on_usart1_internal();
    }

    fn logger_write_internal(&mut self, bytes: &[u8]) {
        for b in bytes {
            block!(self.serial1_tx.write(*b)).ok();
        }
    }

    pub unsafe fn logger_write(bytes: &[u8]) {
        Self::get_application().logger_write_internal(bytes);
    }

    unsafe fn get_application() -> &'static mut Application {
        if let Some(application) = APPLICATION.as_mut() {
            return application;
        } else {
            panic!("No application");
        }
    }
}
