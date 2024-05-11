use crate::error::ZCanError;

pub enum ZLinMode {
    Slave = 0,
    Master = 1,
}

impl TryFrom<u8> for ZLinMode {
    type Error = ZCanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZLinMode::Slave),
            1 => Ok(ZLinMode::Master),
            _ => Err(ZCanError::ParamNotSupported),
        }
    }
}

pub enum ZLinDataType {
    TypeData = 0,
    TypeError = 1,
    TypeEvent = 2,
}

impl TryFrom<u8> for ZLinDataType {
    type Error = ZCanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZLinDataType::TypeData),
            1 => Ok(ZLinDataType::TypeError),
            2 => Ok(ZLinDataType::TypeEvent),
            _ => Err(ZCanError::ParamNotSupported),
        }
    }
}

pub enum ZLinEventType {
    Wakeup = 1,
    EnterSleep = 2,
    ExitSleep = 3,
}

impl TryFrom<u8> for ZLinEventType {
    type Error = ZCanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZLinEventType::Wakeup),
            1 => Ok(ZLinEventType::EnterSleep),
            2 => Ok(ZLinEventType::ExitSleep),
            _ => Err(ZCanError::ParamNotSupported),
        }
    }
}

pub enum ZLinCheckSumMode {
    Classic = 1,
    Enhance = 2,
    Auto = 3,
}

impl TryFrom<u8> for ZLinCheckSumMode {
    type Error = ZCanError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ZLinCheckSumMode::Classic),
            1 => Ok(ZLinCheckSumMode::Enhance),
            2 => Ok(ZLinCheckSumMode::Auto),
            _ => Err(ZCanError::ParamNotSupported),
        }
    }
}
