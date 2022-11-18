use defmt::{info, warn};

#[derive(PartialEq, Eq)]
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

    pub fn receive_word(&mut self, val: u8) -> Option<u32> {
        let mut success = false;
        let mut result = None;

        if (val & 0x80) == 0 {
            match val {
                b'S' => {
                    self.state = TimeSyncState::Magic1;
                    success = true;
                }
                b'b' => {
                    if TimeSyncState::Magic1 == self.state {
                        self.state = TimeSyncState::Receiving;
                        self.value = 0;
                        info!("TimeSyncState::Receiving");
                        success = true;
                    }
                }
                b'c' => {
                    if TimeSyncState::Receiving == self.state {
                        self.state = TimeSyncState::Begin;
                        result = Some(self.value);
                        info!("write_value_callback: {}", self.value);
                        success = true;
                    }
                }
                _ => {}
            }
        } else {
            if TimeSyncState::Receiving == self.state {
                let val: u32 = (val & 0x7f) as u32;
                self.value = (self.value << 7) | val;
                success = true;
            }
        }

        if !success {
            warn!("Received wrong value: {}", val);
            self.state = TimeSyncState::Begin;
        }

        result
    }
}
