pub trait Status: Copy + Default {
    fn status_value(self) -> (Option<u32>, bool);
    fn next_status(self, value: u32) -> Self;
    fn interrupt_status(self) -> Self;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TCLIRSenderStatus {
    Standby,
    WaitForNext,
    BeginSignal,
    BeginSignalWait,
    SendingBitHigh(u32),
    SendingBitLow(u32, bool),
    EndSignalHigh,
    EndSignalLow,
    BeginSignal2,
    BeginSignalWait2,
    SendingBitHigh2(u32),
    SendingBitLow2(u32, bool),
    EndSignalHigh2,
    EndSignalLow2,
}

pub const LENGTH_B: u32 = 8;
pub const LENGTH_BW: u32 = 8;
pub const LENGTH_E: u32 = 16;
pub const LENGTH_H: u32 = 1;
pub const LENGTH_0: u32 = 2;
pub const LENGTH_1: u32 = 4;

impl Default for TCLIRSenderStatus {
    fn default() -> Self {
        Self::Standby
    }
}

impl Status for TCLIRSenderStatus {
    fn status_value(self) -> (Option<u32>, bool) {
        match self {
            Self::Standby => (None, false),
            Self::WaitForNext => (Some(LENGTH_E), false),
            Self::BeginSignal => (Some(LENGTH_B), true),
            Self::BeginSignalWait => (Some(LENGTH_BW), false),
            Self::SendingBitHigh(_) => (Some(LENGTH_H), true),
            Self::SendingBitLow(_, value) => (Some(if value { LENGTH_1 } else { LENGTH_0 }), false),
            Self::EndSignalHigh => (Some(LENGTH_H), true),
            Self::EndSignalLow => (Some(LENGTH_E), false),
            Self::BeginSignal2 => (Some(LENGTH_B), true),
            Self::BeginSignalWait2 => (Some(LENGTH_BW), false),
            Self::SendingBitHigh2(_) => (Some(LENGTH_H), true),
            Self::SendingBitLow2(_, value) => (Some(if value { LENGTH_1 } else { LENGTH_0 }), false),
            Self::EndSignalHigh2 => (Some(LENGTH_H), true),
            Self::EndSignalLow2 => (Some(LENGTH_E), false),
        }
    }

    fn next_status(self, value: u32) -> Self {
        match self {
            Self::Standby => Self::Standby,
            Self::WaitForNext => Self::BeginSignal,
            Self::BeginSignal => Self::BeginSignalWait,
            Self::BeginSignalWait => Self::SendingBitHigh(0),
            Self::SendingBitHigh(loc) => Self::SendingBitLow(loc, ((value >> (23 - loc)) & 1) != 0),
            Self::SendingBitLow(loc, _) => {
                if loc < 23 {
                    Self::SendingBitHigh(loc + 1)
                } else {
                    Self::EndSignalHigh
                }
            }
            Self::EndSignalHigh => Self::EndSignalLow,
            Self::EndSignalLow => Self::BeginSignal2,
            Self::BeginSignal2 => Self::BeginSignalWait2,
            Self::BeginSignalWait2 => Self::SendingBitHigh2(0),
            Self::SendingBitHigh2(loc) => Self::SendingBitLow2(loc, ((value >> (23 - loc)) & 1) != 0),
            Self::SendingBitLow2(loc, _) => {
                if loc < 23 {
                    Self::SendingBitHigh2(loc + 1)
                } else {
                    Self::EndSignalHigh2
                }
            }
            Self::EndSignalHigh2 => Self::EndSignalLow2,
            Self::EndSignalLow2 => Self::Standby,
        }
    }

    fn interrupt_status(self) -> Self {
        Self::WaitForNext
    }
}

pub struct IRSender<TStatus>
where
    TStatus: Status,
{
    status: TStatus,
    remaining_length: Option<u32>,

    sending_value: u32,
}

impl<TStatus> IRSender<TStatus>
where
    TStatus: Status,
{
    pub fn new() -> Self {
        Self {
            status: TStatus::default(),
            remaining_length: None,
            sending_value: 0,
        }
    }

    pub fn next(&mut self) -> Option<bool> {
        match self.remaining_length {
            None => None,
            Some(remaining_length) => {
                if remaining_length > 1 {
                    self.remaining_length = Some(remaining_length - 1);
                    None
                } else {
                    let next_status = self.status.next_status(self.sending_value);
                    let (remaining_length, value) = next_status.status_value();
                    self.status = next_status;
                    self.remaining_length = remaining_length;
                    Some(value)
                }
            }
        }
    }

    pub fn set_value(&mut self, value: u32) -> bool {
        self.sending_value = value;
        let next_status = self.status.interrupt_status();
        let (remaining_length, value) = next_status.status_value();
        self.status = next_status;
        self.remaining_length = remaining_length;
        value
    }
}

pub type TCLIRSender = IRSender<TCLIRSenderStatus>;
