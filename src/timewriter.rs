struct DayTime {
    hour: u8,
    minute: u8,
    second: u8,
}

const MINUTE_LENGTH: u32 = 60;
const HOUR_LENGTH: u32 = MINUTE_LENGTH * 60;
const DAY_LENGTH: u32 = HOUR_LENGTH * 24;

pub struct TimeWriter {
    writing_value: [u8; 14],
    writing_hide: [u8; 6],
    writing_brightness: [u8; 7],
    current_brightness: Brightness,
    current_showing: bool,
    first_run: bool,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq)]
pub enum Brightness {
    B0 = 0,
    B1 = 1,
    B2 = 2,
    B3 = 3,
    B4 = 4,
    B5 = 5,
    B6 = 6,
    B7 = 7,
}

impl TimeWriter {
    pub fn new() -> TimeWriter {
        TimeWriter {
            writing_value: *b"$001,00.00.00#",
            writing_hide: *b"$001,#",
            writing_brightness: *b"$001,0%",
            current_brightness: Brightness::B7,
            current_showing: true,
            first_run: true,
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
            second: second as u8,
        }
    }

    fn set_loc(&mut self, val: u8, loc: usize) {
        let mut val1 = val / 10;
        if val1 >= 10 {
            val1 = 9;
        }
        let val2 = val % 10;
        let val1 = val1 + b'0';
        let val2 = val2 + b'0';
        self.writing_value[loc] = val1;
        self.writing_value[loc + 1] = val2;
    }

    fn write_time(&mut self, timestamp: u32, mut writer: impl FnMut(u8) -> ()) {
        let DayTime {
            hour,
            minute,
            second,
            ..
        } = Self::to_time(timestamp);
        self.set_loc(hour, 5);
        self.set_loc(minute, 8);
        self.set_loc(second, 11);
        for b in self.writing_value {
            writer(b);
        }
    }

    fn write_hide(&mut self, mut writer: impl FnMut(u8) -> ()) {
        for b in self.writing_hide {
            writer(b);
        }
    }

    fn write_brightness(&mut self, brightness: Brightness, mut writer: impl FnMut(u8) -> ()) {
        self.writing_brightness[5] = (brightness as u8) + b'0';
        for b in self.writing_brightness {
            writer(b);
        }
    }

    pub fn update_time(&mut self, timestamp: u32, mut writer: impl FnMut(u8) -> ()) {
        if self.current_showing {
            self.write_time(timestamp, &mut writer);
        } else {
            self.write_hide(&mut writer);
        }
        if self.first_run {
            self.first_run = false;
            self.write_brightness(self.current_brightness, &mut writer);
        }
    }

    pub fn update_brightness(&mut self, val: u16, mut writer: impl FnMut(u8) -> ()) {
        let mut new_brightness = self.current_brightness;
        if val < 400 {
            new_brightness = Brightness::B0;
            self.current_showing = false;
        } else {
            new_brightness = Brightness::B7;
            self.current_showing = true;
        }
        if self.current_brightness != new_brightness {
            self.current_brightness = new_brightness;
            self.write_brightness(new_brightness, &mut writer);
        }
    }
}
