#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Idle,
    ReceivedS,
    ReceivedB,
    Received1,
    Received2,
    Received3,
}

pub struct Receiver {
    current_value: u32,
    status: Status,
}

impl Receiver {
    pub fn new() -> Self {
        Self {
            current_value: 0,
            status: Status::Idle,
        }
    }

    pub fn receive(&mut self, val: u8) -> Option<u32> {
        match self.status {
            Status::Idle => {
                if val == b'S' {
                    self.status = Status::ReceivedS;
                }
            }
            Status::ReceivedS => {
                if val == b'b' {
                    self.status = Status::ReceivedB;
                } else {
                    self.status = Status::Idle;
                }
            }
            Status::ReceivedB => {
                self.status = Status::Received1;
                self.current_value = val as u32;
            }
            Status::Received1 => {
                self.status = Status::Received2;
                self.current_value |= (val as u32) << 8;
            }
            Status::Received2 => {
                self.status = Status::Received3;
                self.current_value |= (val as u32) << 16;
            }
            Status::Received3 => {
                self.status = Status::Idle;
                self.current_value |= (val as u32) << 24;
                return Some(self.current_value);
            }
        }
        None
    }
}
