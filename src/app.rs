use crate::timewriter::TimeWriter;
use crate::{leds::LEDS, timesync::TimeSync};
use cortex_m::{asm::wfi, peripheral::NVIC};
use defmt::{info, warn};
use embedded_hal::serial::Write;
use nb::block;
use stm32f1xx_hal::{
    adc::{Adc, Align, SampleTime},
    device::ADC1,
    gpio::{Analog, PinState, PA0},
    pac::{Interrupt, Peripherals, USART1},
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

        let mut gpioa = dp.GPIOA.split();
        let mut gpiob = dp.GPIOB.split();
        let mut gpioc = dp.GPIOC.split();
        let mut gpiod = dp.GPIOD.split();
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

        let mut pwm = dp
            .TIM1
            .pwm_hz(
                gpioa.pa8.into_alternate_push_pull(&mut gpioa.crh),
                &mut afio.mapr,
                1.kHz(),
                &clocks,
            )
            .split();
        pwm.enable();

        let mut adc = Adc::adc1(dp.ADC1, clocks);
        adc.set_align(Align::Right);
        adc.set_sample_time(SampleTime::T_239);
        let adc_ch0 = gpioa.pa0.into_analog(&mut gpioa.crl);

        let (pa13, pa14, pa15, pb3, pb4) = afio
            .mapr
            .disable_jtag(gpioa.pa13, gpioa.pa14, gpioa.pa15, gpiob.pb3, gpiob.pb4);

        let p11 = pa15.into_push_pull_output_with_state(&mut gpioa.crh, PinState::High);
        let p12 = gpioc
            .pc12
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::High);
        let p13 = gpioc
            .pc11
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::High);
        let p14 = pb3.into_push_pull_output_with_state(&mut gpiob.crl, PinState::High);
        let p15 = gpiob
            .pb6
            .into_push_pull_output_with_state(&mut gpiob.crl, PinState::High);
        let p16 = gpiob
            .pb5
            .into_push_pull_output_with_state(&mut gpiob.crl, PinState::High);
        let p17 = gpiob
            .pb7
            .into_push_pull_output_with_state(&mut gpiob.crl, PinState::High);
        let p21 = pa14.into_push_pull_output_with_state(&mut gpioa.crh, PinState::High);
        let p22 = gpioc
            .pc10
            .into_push_pull_output_with_state(&mut gpioc.crh, PinState::High);
        let p23 = pa13.into_push_pull_output_with_state(&mut gpioa.crh, PinState::High);
        let p24 = gpiod
            .pd2
            .into_push_pull_output_with_state(&mut gpiod.crl, PinState::High);
        let p25 = pb4.into_push_pull_output_with_state(&mut gpiob.crl, PinState::High);
        let p26 = gpiob
            .pb9
            .into_push_pull_output_with_state(&mut gpiob.crh, PinState::High);
        let p27 = gpiob
            .pb8
            .into_push_pull_output_with_state(&mut gpiob.crh, PinState::High);
        let p31 = gpiob
            .pb12
            .into_push_pull_output_with_state(&mut gpiob.crh, PinState::High);
        let p32 = gpiob
            .pb10
            .into_push_pull_output_with_state(&mut gpiob.crh, PinState::High);
        let p33 = gpiob
            .pb1
            .into_push_pull_output_with_state(&mut gpiob.crl, PinState::High);
        let p34 = gpioc
            .pc4
            .into_push_pull_output_with_state(&mut gpioc.crl, PinState::High);
        let p35 = gpioa
            .pa6
            .into_push_pull_output_with_state(&mut gpioa.crl, PinState::High);
        let p36 = gpioa
            .pa5
            .into_push_pull_output_with_state(&mut gpioa.crl, PinState::High);
        let p37 = gpioa
            .pa2
            .into_push_pull_output_with_state(&mut gpioa.crl, PinState::High);
        let p41 = gpiob
            .pb11
            .into_push_pull_output_with_state(&mut gpiob.crh, PinState::High);
        let p42 = gpiob
            .pb0
            .into_push_pull_output_with_state(&mut gpiob.crl, PinState::High);
        let p43 = gpioc
            .pc5
            .into_push_pull_output_with_state(&mut gpioc.crl, PinState::High);
        let p44 = gpioa
            .pa7
            .into_push_pull_output_with_state(&mut gpioa.crl, PinState::High);
        let p45 = gpioa
            .pa4
            .into_push_pull_output_with_state(&mut gpioa.crl, PinState::High);
        let p46 = gpioa
            .pa3
            .into_push_pull_output_with_state(&mut gpioa.crl, PinState::High);
        let p47 = gpioa
            .pa1
            .into_push_pull_output_with_state(&mut gpioa.crl, PinState::High);
        let pcol = gpioa
            .pa12
            .into_push_pull_output_with_state(&mut gpioa.crh, PinState::High);

        let leds = LEDS::new(
            p11, p12, p13, p14, p15, p16, p17, p21, p22, p23, p24, p25, p26, p27, p31, p32, p33,
            p34, p35, p36, p37, p41, p42, p43, p44, p45, p46, p47, pcol,
        );

        let application = Application {
            rtc,
            time_sync: TimeSync::new(),
            time_writer: TimeWriter::new(leds, pwm),
            serial1_tx,
            serial1_rx,
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
        if timestamp % 10 == 0 {
            let val = self.read_sensor();
            info!("light: {}", val);
            self.time_writer.update_brightness(val);
        }
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
