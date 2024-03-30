
pub enum ZLinMode {
    Slave = 0,
    Master = 1,
}

impl From<u8> for ZLinMode {
    fn from(value: u8) -> Self {
        match value {
            0 => ZLinMode::Slave,
            1 => ZLinMode::Master,
            _ => panic!("value is out of range!"),
        }
    }
}

pub enum ZLinDataType {
    Data = 0,
    Error = 1,
    Event = 2,
}

impl From<u8> for ZLinDataType {
    fn from(value: u8) -> Self {
        match value {
            0 => ZLinDataType::Data,
            1 => ZLinDataType::Error,
            2 => ZLinDataType::Event,
            _ => panic!("value is out of range!"),
        }
    }
}

pub enum ZLinEventType {
    Wakeup = 1,
    EnterSleep = 2,
    ExitSleep = 3,
}

impl From<u8> for ZLinEventType {
    fn from(value: u8) -> Self {
        match value {
            0 => ZLinEventType::Wakeup,
            1 => ZLinEventType::EnterSleep,
            2 => ZLinEventType::ExitSleep,
            _ => panic!("value is out of range!"),
        }
    }
}

pub enum ZLinCheckSumMode {
    Classic = 1,
    Enhance = 2,
    Auto = 3,
}

impl From<u8> for ZLinCheckSumMode {
    fn from(value: u8) -> Self {
        match value {
            0 => ZLinCheckSumMode::Classic,
            1 => ZLinCheckSumMode::Enhance,
            2 => ZLinCheckSumMode::Auto,
            _ => panic!("value is out of range!"),
        }
    }
}
