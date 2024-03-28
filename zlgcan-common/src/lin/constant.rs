
pub enum ZLinMode {
    Slave = 0,
    Master = 1,
}

pub enum ZLinDataType {
    Data = 0,
    Error = 1,
    Event = 2,
}

pub enum ZLinEventType {
    Wakeup = 1,
    EnterSleep = 2,
    ExitSleep = 3,
}

pub enum ZLinCheckSumMode {
    Classic = 1,
    Enhance = 2,
    Auto = 3,
}
