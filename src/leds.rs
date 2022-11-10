use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::gpio::{
    Output, PushPull, PA1, PA12, PA13, PA14, PA15, PA2, PA3, PA4, PA5, PA6, PA7, PB0, PB1, PB10,
    PB11, PB12, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PC10, PC11, PC12, PC4, PC5, PD2,
};

#[derive(Debug)]
pub struct DigitInvalidCharError;

#[derive(Clone, Copy, PartialEq)]
pub enum LEDState {
    Y,
    N,
}

pub trait DigitChar {
    const V1: LEDState;
    const V2: LEDState;
    const V3: LEDState;
    const V4: LEDState;
    const V5: LEDState;
    const V6: LEDState;
    const V7: LEDState;
}

macro_rules! decl_digit_char {
    ($name:ident, $v1:ident, $v2:ident, $v3:ident, $v4:ident, $v5:ident, $v6:ident, $v7:ident) => {
        pub struct $name;
        impl DigitChar for $name {
            const V1: LEDState = LEDState::$v1;
            const V2: LEDState = LEDState::$v2;
            const V3: LEDState = LEDState::$v3;
            const V4: LEDState = LEDState::$v4;
            const V5: LEDState = LEDState::$v5;
            const V6: LEDState = LEDState::$v6;
            const V7: LEDState = LEDState::$v7;
        }
    };
}

decl_digit_char!(DigitCharSpace, N, N, N, N, N, N, N);
decl_digit_char!(DigitChar0, Y, Y, Y, N, Y, Y, Y);
decl_digit_char!(DigitChar1, N, Y, N, N, Y, N, N);
decl_digit_char!(DigitChar2, Y, Y, N, Y, N, Y, Y);
decl_digit_char!(DigitChar3, Y, Y, N, Y, Y, N, Y);
decl_digit_char!(DigitChar4, N, Y, Y, Y, Y, N, Y);
decl_digit_char!(DigitChar5, Y, N, Y, Y, Y, N, Y);
decl_digit_char!(DigitChar6, Y, N, Y, Y, Y, Y, Y);
decl_digit_char!(DigitChar7, Y, Y, N, N, Y, N, N);
decl_digit_char!(DigitChar8, Y, Y, Y, Y, Y, Y, Y);
decl_digit_char!(DigitChar9, Y, Y, Y, Y, Y, N, Y);

pub trait Digit {
    fn set_digit<T: DigitChar>(&mut self, c: T);
}

pub trait DigitSetChar {
    fn set_chr(&mut self, c: u8) -> Result<(), DigitInvalidCharError>;
}

impl<T: Digit> DigitSetChar for T {
    fn set_chr(&mut self, c: u8) -> Result<(), DigitInvalidCharError> {
        match c {
            b' ' => self.set_digit(DigitCharSpace),
            b'0' => self.set_digit(DigitChar0),
            b'1' => self.set_digit(DigitChar1),
            b'2' => self.set_digit(DigitChar2),
            b'3' => self.set_digit(DigitChar3),
            b'4' => self.set_digit(DigitChar4),
            b'5' => self.set_digit(DigitChar5),
            b'6' => self.set_digit(DigitChar6),
            b'7' => self.set_digit(DigitChar7),
            b'8' => self.set_digit(DigitChar8),
            b'9' => self.set_digit(DigitChar9),
            _ => {
                return Err(DigitInvalidCharError);
            }
        };
        Ok(())
    }
}

fn set_led(pin: &mut impl OutputPin, state: LEDState) {
    match state {
        LEDState::Y => pin.set_high().ok(),
        LEDState::N => pin.set_low().ok(),
    };
}

impl<P1, P2, P3, P4, P5, P6, P7> Digit
    for (
        &mut P1,
        &mut P2,
        &mut P3,
        &mut P4,
        &mut P5,
        &mut P6,
        &mut P7,
    )
where
    P1: OutputPin,
    P2: OutputPin,
    P3: OutputPin,
    P4: OutputPin,
    P5: OutputPin,
    P6: OutputPin,
    P7: OutputPin,
{
    fn set_digit<T: DigitChar>(&mut self, _: T) {
        set_led(self.0, T::V1);
        set_led(self.1, T::V2);
        set_led(self.2, T::V3);
        set_led(self.3, T::V4);
        set_led(self.4, T::V5);
        set_led(self.5, T::V6);
        set_led(self.6, T::V7);
    }
}

pub trait Colon {
    fn set_colon(self, state: LEDState);
}

impl<T: OutputPin> Colon for &mut T {
    fn set_colon(self, state: LEDState) {
        set_led(self, state);
    }
}

pub type P11 = PA15<Output<PushPull>>;
pub type P12 = PC12<Output<PushPull>>;
pub type P13 = PC11<Output<PushPull>>;
pub type P14 = PB3<Output<PushPull>>;
pub type P15 = PB6<Output<PushPull>>;
pub type P16 = PB5<Output<PushPull>>;
pub type P17 = PB7<Output<PushPull>>;
pub type P21 = PA14<Output<PushPull>>;
pub type P22 = PC10<Output<PushPull>>;
pub type P23 = PA13<Output<PushPull>>;
pub type P24 = PD2<Output<PushPull>>;
pub type P25 = PB4<Output<PushPull>>;
pub type P26 = PB9<Output<PushPull>>;
pub type P27 = PB8<Output<PushPull>>;
pub type P31 = PB12<Output<PushPull>>;
pub type P32 = PB10<Output<PushPull>>;
pub type P33 = PB1<Output<PushPull>>;
pub type P34 = PC4<Output<PushPull>>;
pub type P35 = PA6<Output<PushPull>>;
pub type P36 = PA5<Output<PushPull>>;
pub type P37 = PA2<Output<PushPull>>;
pub type P41 = PB11<Output<PushPull>>;
pub type P42 = PB0<Output<PushPull>>;
pub type P43 = PC5<Output<PushPull>>;
pub type P44 = PA7<Output<PushPull>>;
pub type P45 = PA4<Output<PushPull>>;
pub type P46 = PA3<Output<PushPull>>;
pub type P47 = PA1<Output<PushPull>>;
pub type PCOL = PA12<Output<PushPull>>;

pub struct LEDS {
    p11: P11,
    p12: P12,
    p13: P13,
    p14: P14,
    p15: P15,
    p16: P16,
    p17: P17,
    p21: P21,
    p22: P22,
    p23: P23,
    p24: P24,
    p25: P25,
    p26: P26,
    p27: P27,
    p31: P31,
    p32: P32,
    p33: P33,
    p34: P34,
    p35: P35,
    p36: P36,
    p37: P37,
    p41: P41,
    p42: P42,
    p43: P43,
    p44: P44,
    p45: P45,
    p46: P46,
    p47: P47,
    pcol: PCOL,
}

impl LEDS {
    pub fn new(
        p11: P11,
        p12: P12,
        p13: P13,
        p14: P14,
        p15: P15,
        p16: P16,
        p17: P17,
        p21: P21,
        p22: P22,
        p23: P23,
        p24: P24,
        p25: P25,
        p26: P26,
        p27: P27,
        p31: P31,
        p32: P32,
        p33: P33,
        p34: P34,
        p35: P35,
        p36: P36,
        p37: P37,
        p41: P41,
        p42: P42,
        p43: P43,
        p44: P44,
        p45: P45,
        p46: P46,
        p47: P47,
        pcol: PCOL,
    ) -> Self {
        Self {
            p11,
            p12,
            p13,
            p14,
            p15,
            p16,
            p17,
            p21,
            p22,
            p23,
            p24,
            p25,
            p26,
            p27,
            p31,
            p32,
            p33,
            p34,
            p35,
            p36,
            p37,
            p41,
            p42,
            p43,
            p44,
            p45,
            p46,
            p47,
            pcol,
        }
    }

    pub fn digit1<'a>(&'a mut self) -> impl Digit + 'a {
        (
            &mut self.p11,
            &mut self.p12,
            &mut self.p13,
            &mut self.p14,
            &mut self.p15,
            &mut self.p16,
            &mut self.p17,
        )
    }

    pub fn digit2<'a>(&'a mut self) -> impl Digit + 'a {
        (
            &mut self.p21,
            &mut self.p22,
            &mut self.p23,
            &mut self.p24,
            &mut self.p25,
            &mut self.p26,
            &mut self.p27,
        )
    }

    pub fn digit3<'a>(&'a mut self) -> impl Digit + 'a {
        (
            &mut self.p31,
            &mut self.p32,
            &mut self.p33,
            &mut self.p34,
            &mut self.p35,
            &mut self.p36,
            &mut self.p37,
        )
    }

    pub fn digit4<'a>(&'a mut self) -> impl Digit + 'a {
        (
            &mut self.p41,
            &mut self.p42,
            &mut self.p43,
            &mut self.p44,
            &mut self.p45,
            &mut self.p46,
            &mut self.p47,
        )
    }

    pub fn colon<'a>(&'a mut self) -> impl Colon + 'a {
        return &mut self.pcol;
    }
}
