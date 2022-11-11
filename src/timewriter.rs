use stm32f1xx_hal::{device::TIM1, timer::PwmChannel};

use crate::leds::{Colon, DigitSetChar, LEDState, LEDS};

struct DayTime {
    hour: u8,
    minute: u8,
    _second: u8,
}

const MINUTE_LENGTH: u32 = 60;
const HOUR_LENGTH: u32 = MINUTE_LENGTH * 60;
const DAY_LENGTH: u32 = HOUR_LENGTH * 24;

pub struct TimeWriter {
    brightness: Brightness,
    first_run: bool,
    leds: LEDS,
    pwm: PwmChannel<TIM1, 0>,
}

#[repr(u32)]
#[derive(Copy, Clone, PartialEq)]
enum Brightness {
    B0 = 5,   // < 100
    B1 = 30,  // 100 ~ 1000
    B2 = 100, // > 1000
}

impl TimeWriter {
    pub fn new(leds: LEDS, pwm: PwmChannel<TIM1, 0>) -> TimeWriter {
        TimeWriter {
            brightness: Brightness::B2,
            first_run: true,
            leds,
            pwm,
        }
    }

    fn to_time(timestamp: u32) -> DayTime {
        let timestamp = timestamp % DAY_LENGTH;
        let hour = timestamp / HOUR_LENGTH;
        let timestamp = timestamp % HOUR_LENGTH;
        let minute = timestamp / MINUTE_LENGTH;
        let timestamp = timestamp % MINUTE_LENGTH;
        let second = timestamp;

        DayTime {
            hour: hour as u8,
            minute: minute as u8,
            _second: second as u8,
        }
    }

    fn to_digits(val: u8) -> (u8, u8) {
        let mut val1 = val / 10;
        if val1 >= 10 {
            val1 = 9;
        }
        let val2 = val % 10;
        let val1 = val1 + b'0';
        let val2 = val2 + b'0';
        (val1, val2)
    }

    // fn write_hide(&mut self) {
    //     self.leds.digit1().set_digit(DigitCharSpace);
    //     self.leds.digit2().set_digit(DigitCharSpace);
    //     self.leds.digit3().set_digit(DigitCharSpace);
    //     self.leds.digit4().set_digit(DigitCharSpace);
    //     self.leds.colon().set_colon(LEDState::N);
    // }

    fn write_brightness(&mut self) {
        let pwm_max = self.pwm.get_max_duty();
        let pwm = pwm_max - pwm_max / 1000 * (self.brightness as u8 as u16);
        self.pwm.set_duty(pwm);
    }

    pub fn write_time(&mut self, timestamp: u32) {
        let DayTime { hour, minute, .. } = Self::to_time(timestamp);
        let (hour1, hour2) = Self::to_digits(hour);
        let (minute1, minute2) = Self::to_digits(minute);
        self.leds.digit4().set_chr(hour1).unwrap();
        self.leds.digit3().set_chr(hour2).unwrap();
        self.leds.digit2().set_chr(minute1).unwrap();
        self.leds.digit1().set_chr(minute2).unwrap();
        self.leds.colon().set_colon(if timestamp % 2 == 0 {
            LEDState::Y
        } else {
            LEDState::N
        });
    }

    pub fn update_time(&mut self, timestamp: u32) {
        if self.first_run {
            self.write_brightness();
        }
        self.write_time(timestamp);
    }

    pub fn update_brightness(&mut self, val: u16) {
        let new_brightness;
        if val < 100 {
            new_brightness = Brightness::B0;
        } else if val < 1000 {
            new_brightness = Brightness::B1;
        } else {
            new_brightness = Brightness::B2;
        }
        if self.brightness != new_brightness {
            self.brightness = new_brightness;
            self.write_brightness();
        }
    }
}
