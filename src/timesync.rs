enum TimeSyncState {
    Begin,
    Magic1,
    Receiving,
}

pub struct TimeSync {
    state: TimeSyncState,
    value: u32,
}

impl TimeSync {
    pub fn new() -> TimeSync {
        TimeSync {
            state: TimeSyncState::Begin,
            value: 0,
        }
    }

    pub fn receive_word(&mut self, val: u8, write_value_callback: impl FnOnce(u32) -> ()) {
        if (val & 0x80) == 0 {
            match val {
                b'S' => {
                    self.state = TimeSyncState::Magic1;
                    return;
                }
                b'b' => {
                    if let TimeSyncState::Magic1 = self.state {
                        self.state = TimeSyncState::Receiving;
                        self.value = 0;
                        return;
                    }
                }
                b'c' => {
                    write_value_callback(self.value);
                }
                _ => {}
            }
        } else {
            if let TimeSyncState::Receiving = self.state {
                let val: u32 = (val & 0x7f) as u32;
                self.value = (self.value << 7) | val;
                return;
            }
        }
        self.state = TimeSyncState::Begin;
    }
}
