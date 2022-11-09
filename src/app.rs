use crate::timesync::TimeSync;
use crate::timewriter::TimeWriter;
use cortex_m::{asm::wfi, peripheral::NVIC};
use defmt::{info, warn};
use embedded_hal::serial::Write;
use nb::block;
use stm32f1xx_hal::{
    adc::{Adc, Align, SampleTime},
    device::ADC1,
    gpio::{Analog, PA0},
    pac::{Interrupt, Peripherals, USART1, USART2},
    prelude::*,
    rtc::Rtc,
    serial::{Config, Rx, Serial, Tx},
};

static mut APPLICATION: Option<Application> = None;

pub struct Application {
    rtc: Rtc,
    time_sync: TimeSync,
    time_writer: TimeWriter,
    serial1_tx: Tx<USART1>,
    serial1_rx: Rx<USART1>,
    serial2_tx: Tx<USART2>,
    adc: Adc<ADC1>,
    adc_ch0: PA0<Analog>,
}

impl Application {
    fn init() {
        let dp = Peripherals::take().unwrap();

        let mut flash = dp.FLASH.constrain();
        let rcc = dp.RCC.constrain();
        let clocks = rcc
            .cfgr
            // .use_hse(8_000_000.Hz())
            // .sysclk(72_000_000.Hz())
            // .hclk(72_000_000.Hz())
            // .adcclk(12_000_000.Hz())
            // .pclk1(36_000_000.Hz())
            // .pclk2(72_000_000.Hz())
            .use_hse(8_000_000.Hz())
            .sysclk(8_000_000.Hz())
            .hclk(8_000_000.Hz())
            .adcclk(4_000_000.Hz())
            .pclk1(8_000_000.Hz())
            .pclk2(8_000_000.Hz())
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
            Config::default().baudrate(9_600.bps()),
            clocks,
        )
        .split();

        let mut adc = Adc::adc1(dp.ADC1, clocks);
        adc.set_align(Align::Right);
        adc.set_sample_time(SampleTime::T_239);
        let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

        let application = Application {
            rtc,
            time_sync: TimeSync::new(),
            time_writer: TimeWriter::new(),
            serial1_tx,
            serial1_rx,
            serial2_tx,
            adc,
            adc_ch0,
        };

        unsafe {
            APPLICATION = Some(application);
        }
    }

    fn enable_interrupts() {
        unsafe {
            NVIC::unmask(Interrupt::RTC);
            NVIC::unmask(Interrupt::USART1);
        }
    }

    fn run_internal(&mut self) -> ! {
        loop {
            wfi();
        }
    }

    fn on_rtc_internal(&mut self) {
        info!("RTC");
        self.rtc.clear_second_flag();
        let timestamp = self.rtc.current_time();
        self.time_writer.write(timestamp, |val| {
            block!(self.serial2_tx.write(val)).unwrap();
        });
    }

    fn on_usart1_internal(&mut self) {
        info!("USART1");
        if let Ok(val) = self.serial1_rx.read() {
            self.time_sync.receive_word(val, |val| {
                self.rtc.set_time(val);
                info!("Set time: {}", val);
            });
        } else {
            warn!("USART1 receive error.");
        }
    }

    fn logger_write_internal(&mut self, bytes: &[u8]) {
        for b in bytes {
            block!(self.serial1_tx.write(*b)).ok();
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

    pub unsafe fn on_rtc() {
        cortex_m::interrupt::free(|_| {
            Self::get_application().on_rtc_internal();
        });
    }

    pub unsafe fn on_usart1() {
        cortex_m::interrupt::free(|_| {
            Self::get_application().on_usart1_internal();
        });
    }

    pub unsafe fn logger_write(bytes: &[u8]) {
        Self::get_application().logger_write_internal(bytes);
    }
}
