use crate::timewriter::TimeWriter;
use crate::{leds::LEDS, timesync::TimeSync};
use cortex_m::{asm::wfi, peripheral::NVIC};
use defmt::{info, warn};
use embedded_hal::serial::Write;
use nb::block;
use stm32f1xx_hal::{
    adc::{Adc, Align, SampleTime},
    device::ADC1,
    gpio::{gpioa, gpiob, gpioc, gpiod, Analog, PinState, PA0},
    pac::{Interrupt, Peripherals, USART1},
    prelude::*,
    rtc::Rtc,
    serial::{Config, Rx, Serial, Tx},
};

static mut APPLICATION: Option<Application> = None;

// LOGGING_SERIAL is not in APPLICATION because anywhere will call LOGGING_SERIAL
static mut LOGGING_SERIAL: Option<Tx<USART1>> = None;

pub struct Application {
    rtc: Rtc,
    time_sync: TimeSync,
    time_writer: TimeWriter,
    serial1_rx: Rx<USART1>,
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
            .use_hse(8_000_000.Hz())
            .sysclk(8_000_000.Hz())
            .hclk(8_000_000.Hz())
            .adcclk(4_000_000.Hz())
            .pclk1(8_000_000.Hz())
            .pclk2(8_000_000.Hz())
            .freeze(&mut flash.acr);

        let gpioa::Parts {
            pa0,
            pa1,
            pa2,
            pa3,
            pa4,
            pa5,
            pa6,
            pa7,
            pa8,
            pa9,
            pa10,
            pa12,
            pa13,
            pa14,
            pa15,
            crh: mut acrh,
            crl: mut acrl,
            ..
        } = dp.GPIOA.split();
        let gpiob::Parts {
            pb0,
            pb1,
            pb3,
            pb4,
            pb5,
            pb6,
            pb7,
            pb8,
            pb9,
            pb10,
            pb11,
            pb12,
            crh: mut bcrh,
            crl: mut bcrl,
            ..
        } = dp.GPIOB.split();
        let gpioc::Parts {
            pc4,
            pc5,
            pc10,
            pc11,
            pc12,
            crh: mut ccrh,
            crl: mut ccrl,
            ..
        } = dp.GPIOC.split();
        let gpiod::Parts {
            pd2, crl: mut dcrl, ..
        } = dp.GPIOD.split();
        let mut afio = dp.AFIO.constrain();

        let mut pwr = dp.PWR;
        let mut backup_domain = rcc.bkp.constrain(dp.BKP, &mut pwr);
        let mut rtc = Rtc::new(dp.RTC, &mut backup_domain);
        rtc.select_frequency(1.Hz());
        rtc.listen_seconds();

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
                1.kHz(),
                &clocks,
            )
            .split();
        pwm.enable();

        let mut adc = Adc::adc1(dp.ADC1, clocks);
        adc.set_align(Align::Right);
        adc.set_sample_time(SampleTime::T_239);
        let adc_ch0 = pa0.into_analog(&mut acrl);

        let (pa13, pa14, pa15, pb3, pb4) = afio.mapr.disable_jtag(pa13, pa14, pa15, pb3, pb4);

        let p11 = pa15.into_push_pull_output_with_state(&mut acrh, PinState::High);
        let p12 = pc12.into_push_pull_output_with_state(&mut ccrh, PinState::High);
        let p13 = pc11.into_push_pull_output_with_state(&mut ccrh, PinState::High);
        let p14 = pb3.into_push_pull_output_with_state(&mut bcrl, PinState::High);
        let p15 = pb6.into_push_pull_output_with_state(&mut bcrl, PinState::High);
        let p16 = pb5.into_push_pull_output_with_state(&mut bcrl, PinState::High);
        let p17 = pb7.into_push_pull_output_with_state(&mut bcrl, PinState::High);
        let p21 = pa14.into_push_pull_output_with_state(&mut acrh, PinState::High);
        let p22 = pc10.into_push_pull_output_with_state(&mut ccrh, PinState::High);
        let p23 = pa13.into_push_pull_output_with_state(&mut acrh, PinState::High);
        let p24 = pd2.into_push_pull_output_with_state(&mut dcrl, PinState::High);
        let p25 = pb4.into_push_pull_output_with_state(&mut bcrl, PinState::High);
        let p26 = pb9.into_push_pull_output_with_state(&mut bcrh, PinState::High);
        let p27 = pb8.into_push_pull_output_with_state(&mut bcrh, PinState::High);
        let p31 = pb12.into_push_pull_output_with_state(&mut bcrh, PinState::High);
        let p32 = pb10.into_push_pull_output_with_state(&mut bcrh, PinState::High);
        let p33 = pb1.into_push_pull_output_with_state(&mut bcrl, PinState::High);
        let p34 = pc4.into_push_pull_output_with_state(&mut ccrl, PinState::High);
        let p35 = pa6.into_push_pull_output_with_state(&mut acrl, PinState::High);
        let p36 = pa5.into_push_pull_output_with_state(&mut acrl, PinState::High);
        let p37 = pa2.into_push_pull_output_with_state(&mut acrl, PinState::High);
        let p41 = pb11.into_push_pull_output_with_state(&mut bcrh, PinState::High);
        let p42 = pb0.into_push_pull_output_with_state(&mut bcrl, PinState::High);
        let p43 = pc5.into_push_pull_output_with_state(&mut ccrl, PinState::High);
        let p44 = pa7.into_push_pull_output_with_state(&mut acrl, PinState::High);
        let p45 = pa4.into_push_pull_output_with_state(&mut acrl, PinState::High);
        let p46 = pa3.into_push_pull_output_with_state(&mut acrl, PinState::High);
        let p47 = pa1.into_push_pull_output_with_state(&mut acrl, PinState::High);
        let pcol = pa12.into_push_pull_output_with_state(&mut acrh, PinState::High);

        let leds = LEDS::new(
            p11, p12, p13, p14, p15, p16, p17, p21, p22, p23, p24, p25, p26, p27, p31, p32, p33,
            p34, p35, p36, p37, p41, p42, p43, p44, p45, p46, p47, pcol,
        );

        let application = Application {
            rtc,
            time_sync: TimeSync::new(),
            time_writer: TimeWriter::new(leds, pwm),
            serial1_rx,
            adc,
            adc_ch0,
        };

        unsafe {
            APPLICATION = Some(application);
            LOGGING_SERIAL = Some(serial1_tx);
        }
    }

    fn enable_interrupts() {
        unsafe {
            NVIC::unmask(Interrupt::RTC);
            NVIC::unmask(Interrupt::USART1);
        }
    }

    fn read_sensor(&mut self) -> u16 {
        self.adc.read(&mut self.adc_ch0).unwrap()
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
        self.time_writer.update_time(timestamp);
        let val = self.read_sensor();
        info!("light: {}", val);
        self.time_writer.update_brightness(val);
    }

    fn on_usart1_internal(&mut self) {
        info!("USART1");
        if let Ok(val) = self.serial1_rx.read() {
            if let Some(t) = self.time_sync.receive_word(val) {
                self.rtc.set_time(t);
                info!("Set time: {}", val);
            }
        } else {
            warn!("USART1 receive error.");
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
        for b in bytes {
            if let Some(ref mut logging_serial) = LOGGING_SERIAL {
                block!(logging_serial.write(*b)).ok();
            }
        }
    }
}
