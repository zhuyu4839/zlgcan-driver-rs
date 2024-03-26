use std::fmt::{Display, Formatter};

pub(crate) const BITRATE_CFG_FILENAME: &str = "bitrate.cfg.yaml";
pub(crate) const TIMING0: &str = "timing0";
pub(crate) const TIMING1: &str = "timing1";
pub(crate) const TSEG1: &str = "tseg1";     // Time Segment 1
pub(crate) const TSEG2: &str = "tseg2";     // Time Segment 2
pub(crate) const SJW: &str = "sjw";         // Synchronization Jump Width
pub(crate) const SMP: &str = "smp";         // Sampling specifies
pub(crate) const BRP: &str = "brp";         // BaudRate Pre-scale

pub(crate) const CAN_EFF_FLAG: u32 = 0x80000000; /* EFF/SFF is set in the MSB */
pub(crate) const CAN_RTR_FLAG: u32 = 0x40000000; /* remote transmission request */
pub(crate) const CAN_ERR_FLAG: u32 = 0x20000000; /* error message frame */
// pub(crate) const CAN_SFF_MASK: u32 = 0x000007FF; /* standard frame format (SFF) */
pub(crate) const CAN_ID_FLAG: u32 = 0x1FFFFFFF; /* id */
pub(crate) const CAN_EFF_MASK: u32 = 0x1FFF800;
pub(crate) const CANFD_BRS: u8 = 0x01; /* bit rate switch (second bitrate for payload data) */
pub(crate) const CANFD_ESI: u8 = 0x02; /* error state indicator of the transmitting node */

pub const CAN_FRAME_LENGTH: usize = 8;
pub const CANERR_FRAME_LENGTH: usize = 8;
pub const CANFD_FRAME_LENGTH: usize = 64;

/// Then CAN frame type used in crate.
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum ZCanFrameType {
    CAN = 0,
    CANFD = 1,
    ALL = 2,
}

impl Display for ZCanFrameType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CAN => writeln!(f, "CAN"),
            Self::CANFD => writeln!(f, "CANFD"),
            Self::ALL => writeln!(f, "CAN|CANFD"),
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanChlMode {
    #[default]
    Normal = 0,
    ListenOnly = 1,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone)]
pub enum ZCanFdStd {
    CANFD_ISO = 0,
    CANFD_NON_ISO = 1,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanChlType {
    #[default]
    CAN = 0,
    CANFD_ISO = 1,
    CANFD_NON_ISO = 2,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanFilterType {
    #[default]
    Double = 0,
    Single = 1,
}

#[derive(Debug, Default, Copy, Clone)]
pub enum ZCanTxMode {
    #[default]
    Normal = 0,             //**< normal transmission */
    Once = 1,               //**< single-shot transmission */
    SelfReception = 2,      //**< self reception */
    SelfReceptionOnce = 3,  //**< single-shot transmission & self reception */
}

#[derive(Debug, Copy, Clone)]
pub enum ZCanHdrInfoField {
    TxMode = 1,
    FrameType = 2,
    IsRemoteFrame = 3,
    IsExtendFrame = 4,
    IsErrorFrame = 5,
    IsBitrateSwitch = 6,
    IsErrorStateIndicator = 7,
}
/// The reference for Linux device
#[allow(dead_code)]
pub enum Reference {
    Filter = 0x14,          // filter setting; @see ZCAN_Filter and ZCanFilterTable
    SkdSend = 0x16,         // timed send setting; @see ZCAN_TTX
    SkdSendStatus = 0x17,   // timed send status; 0-disable, 1-enable
    Resistance = 0x18,      // terminal resistance; 0-disable, 1-enable
    Timeout = 0x44,         // send timeout; range 0~4000ms
}
