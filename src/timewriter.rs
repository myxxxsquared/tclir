

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
}

impl TimeWriter {
    pub fn new() -> TimeWriter {
        TimeWriter {
            writing_value: *b"$001,00.00.00#",
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

    pub fn write(&mut self, timestamp: u32, mut writer: impl FnMut(u8) -> ()) {
        let DayTime {
            hour,
            minute,
            second,
            ..
        } = Self::to_time(timestamp);
        self.set_loc(hour, 5);
        self.set_loc(minute, 8);
        self.set_loc(second, 11);
        for i in 0..14 {
            writer(self.writing_value[i]);
        }
    }
}
