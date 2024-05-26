use std::ffi::{c_uchar, c_uint, c_ushort};
use crate::error::ZCanError;
use super::constant::{CAN_ID_FLAG, CAN_FRAME_LENGTH, CANFD_FRAME_LENGTH, ZCanHdrInfoField, CAN_EFF_MASK, CAN_EFF_FLAG, CAN_RTR_FLAG, CAN_ERR_FLAG, CANFD_BRS, CANFD_ESI};

#[repr(C)]
pub struct USBCanEUAutoTransFrame {
    pub interval: u32,
    pub can_id: u32,
    pub is_extend: bool,
    pub is_remote: bool,
    pub length: u8,
    pub data: *const u8,
}

#[repr(C)]
pub struct USBCanEUWhiteList {
    pub is_extend: bool,
    pub start: u32,
    pub stop: u32,
}

pub trait NewZCanFrame {
    fn new<T>(
        can_id: u32,
        channel: u8,
        data: T,
        info: ZCanHdrInfo
    ) -> Result<Self, ZCanError>
        where
            T: AsRef<[u8]>,
            Self: Sized;
}

/// used by usbcan
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFrameV1 {
    pub(crate) can_id: c_uint,
    pub(crate) timestamp: c_uint,
    pub(crate) time_flag: c_uchar,     // 0 - timestamp not valuable; 1 timestamp valuable
    pub(crate) send_type: c_uchar,
    pub(crate) rem_flag: c_uchar,      //是否是远程帧
    pub(crate) ext_flag: c_uchar,      //是否是扩展帧
    pub(crate) len: c_uchar,
    pub(crate) data: [c_uchar; CAN_FRAME_LENGTH],
    #[allow(dead_code)]
    pub(crate) reserved: [c_uchar; 3],
}

impl NewZCanFrame for ZCanFrameV1 {
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo) -> Result<Self, ZCanError>
        where
            T: AsRef<[u8]> {
        zcan_frame_new(can_id, channel, data, info, |id, _chl, data, len, info| {
            Ok(Self {
                can_id: id,
                timestamp: Default::default(),
                time_flag: Default::default(),
                send_type: info.get_field(ZCanHdrInfoField::TxMode),
                rem_flag: info.get_field(ZCanHdrInfoField::IsRemoteFrame),
                ext_flag: info.get_field(ZCanHdrInfoField::IsExtendFrame),
                len,
                data: data.try_into().map_err(|_| ZCanError::ParamNotSupported)?,
                reserved: Default::default(),
            })
        })
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanHdrInfo {
    mode: c_uchar,  // U8 txm : 4; /**< TX-mode, @see ZCAN_TX_MODE */
                    // U8 fmt : 4; /**< 0-CAN2.0, 1-CANFD */
    flag: c_uchar,  // U8 sdf : 1; /**< 0-data_frame, 1-remote_frame */
                    // U8 sef : 1; /**< 0-std_frame, 1-ext_frame */
                    // U8 err : 1; /**< error flag */
                    // U8 brs : 1; /**< bit-rate switch */
                    // U8 est : 1; /**< error state */
                    // 5~7bit not used
    #[allow(dead_code)]
    pad: c_ushort,  // U16 pad : 16;
}

impl ZCanHdrInfo {
    /// It may result in unexpected errors that setting value out of range.
    /// ZCanFrameInfoField::TxMode 0~15
    /// ZCanFrameInfoField::FrameType 0~15
    /// Others: 0~1
    #[inline(always)]
    pub(crate) fn set_field(&mut self, field: ZCanHdrInfoField, value: u8) -> &mut Self {
        let value = value as u32;
        match field {
            ZCanHdrInfoField::TxMode => self.mode = (self.mode & 0xF0) | ((value & 0x0F) as u8), // self.mode = (self.mode & 0xF0) | ((value & 0x0F) as u8) << 0,
            ZCanHdrInfoField::FrameType => self.mode = (self.mode & 0x0F) | ((value & 0x0F) as u8) << 4,
            ZCanHdrInfoField::IsRemoteFrame => self.flag = (self.flag & (0xFF - 1)) | ((value & 0x01) as u8), // self.flag = (self.flag & (0xFE)) | ((value & 0x01) as u8) << 0,
            ZCanHdrInfoField::IsExtendFrame => self.flag = (self.flag & (0xFF - (1 << 1))) | ((value & 0x01) as u8) << 1,
            ZCanHdrInfoField::IsErrorFrame => self.flag = (self.flag & (0xFF - (1 << 2))) | ((value & 0x01) as u8) << 2,
            ZCanHdrInfoField::IsBitrateSwitch => self.flag = (self.flag & (0xFF - (1 << 3))) | ((value & 0x01) as u8) << 3,
            ZCanHdrInfoField::IsErrorStateIndicator => self.flag = (self.flag & (0xFF - (1 << 4))) | ((value & 0x01) as u8) << 4,
        }
        self
    }

    #[inline(always)]
    pub(crate) fn get_field(&self, field: ZCanHdrInfoField) -> u8 {
        match field {
            ZCanHdrInfoField::TxMode => self.mode & 0x0F,     //(self.mode & 0x0F) >> 0,
            ZCanHdrInfoField::FrameType => (self.mode & 0xF0) >> 4,
            ZCanHdrInfoField::IsRemoteFrame => self.flag & (1 << 0),   // (self.flag & (1 << 0)) >> 0,
            ZCanHdrInfoField::IsExtendFrame => (self.flag & (1 << 1)) >> 1,
            ZCanHdrInfoField::IsErrorFrame => (self.flag & (1 << 2)) >> 2,
            ZCanHdrInfoField::IsBitrateSwitch => (self.flag & (1 << 3)) >> 3,
            ZCanHdrInfoField::IsErrorStateIndicator => (self.flag & (1 << 4)) >> 4,
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanHeaderV1 {
    pub(crate) timestamp: c_uint,  //**< timestamp */
    pub(crate) can_id: c_uint,     //**< CAN-ID */
    pub(crate) info: ZCanHdrInfo,  //**< @see ZCAN_MSG_INF */
    #[allow(dead_code)]
    pub(crate) pad: c_ushort,
    pub(crate) channel: c_uchar,   //**< channel */
    pub(crate) len: c_uchar,       //**< data length */
}

/// used by linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFrameV2 {
    pub(crate) hdr: ZCanHeaderV1,
    pub(crate) data: [c_uchar; CAN_FRAME_LENGTH],
}

impl NewZCanFrame for ZCanFrameV2 {
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo) -> Result<Self, ZCanError>
        where
            T: AsRef<[u8]> {
        zcan_frame_new(can_id, channel, data, info, |id, chl, data, len, info| {
            Ok(Self {
                hdr: ZCanHeaderV1 {
                    timestamp: Default::default(),
                    can_id: id,
                    info,
                    pad: Default::default(),
                    channel: chl,
                    len,
                },
                data: data.try_into().map_err(|_| ZCanError::ParamNotSupported)?,
            })
        })
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanHeaderV2 {
    pub(crate) can_id: c_uint,
    pub(crate) can_len: c_uchar,
    pub(crate) flag: c_uchar,
    #[allow(dead_code)]
    pub(crate) __res0: c_uchar,  /* reserved / padding */
    #[allow(dead_code)]
    pub(crate) __res1: c_uchar,  /* reserved / padding */
}

/// used by usbcanfd-800u usbcan-4-E usbcan-8-E and windows
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFrameV3 {
    pub(crate) hdr: ZCanHeaderV2,
    pub(crate) data: [c_uchar; CAN_FRAME_LENGTH],
    pub(crate) ts_or_mode: c_uint,       // timestamp when received
}

impl NewZCanFrame for ZCanFrameV3 {
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo) -> Result<Self, ZCanError>
        where
            T: AsRef<[u8]> {
        // zcan_frame_new2::<CAN_FRAME_LENGTH, T, Self>(can_id, channel, data, info,  |id, _chl, data, len, info| -> Self {
        zcan_frame_new2(can_id, channel, data, info,  |id, _chl, data, len, info| {
            Ok(Self {
                hdr: ZCanHeaderV2 {
                    can_id: id,
                    can_len: len,
                    flag: Default::default(),
                    __res0: Default::default(),
                    __res1: Default::default(),
                },
                data: data.try_into().map_err(|_| ZCanError::ParamNotSupported)?,
                ts_or_mode: info.get_field(ZCanHdrInfoField::TxMode) as u32,
            })
        })
    }
}

#[repr(C)]
pub union ZCanFrame {
    v1: ZCanFrameV1,
    v2: ZCanFrameV2,
    v3: ZCanFrameV3,
}

impl From<ZCanFrameV1> for ZCanFrame {
    fn from(value: ZCanFrameV1) -> Self {
        Self { v1: value }
    }
}

impl From<&ZCanFrame> for ZCanFrameV1 {
    fn from(value: &ZCanFrame) -> Self {
        unsafe { value.v1 }
    }
}

impl From<ZCanFrameV2> for ZCanFrame {
    fn from(value: ZCanFrameV2) -> Self {
        Self { v2: value }
    }
}

impl From<&ZCanFrame> for ZCanFrameV2 {
    fn from(value: &ZCanFrame) -> Self {
        unsafe { value.v2 }
    }
}

impl From<ZCanFrameV3> for ZCanFrame {
    fn from(value: ZCanFrameV3) -> Self {
        Self { v3: value }
    }
}

impl From<&ZCanFrame> for ZCanFrameV3 {
    fn from(value: &ZCanFrame) -> Self {
        unsafe { value.v3 }
    }
}

impl ZCanFrame {
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v1(v1: ZCanFrameV1) -> Self {
        Self { v1 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn get_v1(&self) -> &ZCanFrameV1 {
        unsafe { &self.v1 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v2(v2: ZCanFrameV2) -> Self {
        Self { v2 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn get_v2(&self) -> &ZCanFrameV2 {
        unsafe { &self.v2 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v3(v3: ZCanFrameV3) -> Self {
        Self { v3 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn get_v3(&self) -> &ZCanFrameV3 {
        unsafe { &self.v3 }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct CanFdData {
    pub(crate) data: [c_uchar; CANFD_FRAME_LENGTH],
}

impl Default for CanFdData {
    fn default() -> Self {
        Self { data: [Default::default(); CANFD_FRAME_LENGTH] }
    }
}

/// used by USBCANFD on linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFdFrameV1 {
    pub(crate) hdr: ZCanHeaderV1,
    pub(crate) data: CanFdData,
}

impl NewZCanFrame for ZCanFdFrameV1 {
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo) -> Result<Self, ZCanError>
        where
            T: AsRef<[u8]> {
        zcanfd_frame_new(can_id, channel, data, info, |id, chl, data, len, info| {
            Ok(Self {
                hdr: ZCanHeaderV1 {
                    timestamp: Default::default(),
                    can_id: id,
                    info,
                    pad: Default::default(),
                    channel: chl,
                    len,
                },
                data: CanFdData {data: data.try_into().map_err(|_| ZCanError::ParamNotSupported)? },
            })
        })
    }
}

/// used by windows and USBCAN-4E|8E USBCANFD-800U
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanFdFrameV2 {
    pub(crate) hdr: ZCanHeaderV2,
    pub(crate) data: CanFdData,
    pub(crate) ts_or_mode: c_uint,       // timestamp when received
}

impl NewZCanFrame for ZCanFdFrameV2 {
    fn new<T>(can_id: u32, channel: u8, data: T, info: ZCanHdrInfo) -> Result<Self, ZCanError>
        where
            T: AsRef<[u8]> {
        // zcan_frame_new2::<CANFD_FRAME_LENGTH, T, Self>(can_id, channel, data, info, |id, _chl, data, len, info| -> Self {
        zcanfd_frame_new2(can_id, channel, data, info, |id, _chl, data, len, info| {
            let mut flag: u8 = Default::default();
            if info.get_field(ZCanHdrInfoField::IsBitrateSwitch) > 0 {
                flag |= CANFD_BRS;
            }
            if info.get_field(ZCanHdrInfoField::IsErrorStateIndicator) > 0 {
                flag |= CANFD_ESI;
            }

            Ok(Self {
                hdr: ZCanHeaderV2 {
                    can_id: id,
                    can_len: len,
                    flag,
                    __res0: Default::default(),
                    __res1: Default::default(),
                },
                data: CanFdData { data: data.try_into().map_err(|_| ZCanError::ParamNotSupported)? },
                ts_or_mode: info.get_field(ZCanHdrInfoField::TxMode) as u32,
            })
        })
    }
}

#[repr(C)]
pub union ZCanFdFrame {
    v1: ZCanFdFrameV1,
    v2: ZCanFdFrameV2,
}

impl From<ZCanFdFrameV1> for ZCanFdFrame {
    fn from(value: ZCanFdFrameV1) -> Self {
        Self { v1: value }
    }
}

impl From<&ZCanFdFrame> for ZCanFdFrameV1 {
    fn from(value: &ZCanFdFrame) -> Self {
        unsafe { value.v1 }
    }
}

impl From<ZCanFdFrameV2> for ZCanFdFrame {
    fn from(value: ZCanFdFrameV2) -> Self {
        Self { v2: value }
    }
}

impl From<&ZCanFdFrame> for ZCanFdFrameV2 {
    fn from(value: &ZCanFdFrame) -> Self {
        unsafe { value.v2 }
    }
}

impl ZCanFdFrame {
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v1(v1: ZCanFdFrameV1) -> Self {
        Self { v1 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v2(v2: ZCanFdFrameV2) -> Self {
        Self { v2 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn get_v1(&self) -> &ZCanFdFrameV1 {
        unsafe { &self.v1 }
    }
    #[deprecated(since = "0.2.3-Beta1", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn get_v2(&self) -> &ZCanFdFrameV2 {
        unsafe { &self.v2 }
    }
}

fn zcan_frame_new<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, ZCanError>
) -> Result<R, ZCanError>
    where
        T: AsRef<[u8]> {
    match can_id {
        0..=CAN_ID_FLAG => {
            let mut data = Vec::from(data.as_ref());
            let len = data.len();
            data.resize(CAN_FRAME_LENGTH, 0);
            if (can_id & CAN_EFF_MASK) > 0 {
                info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
            }
            match len {
                0..=CAN_FRAME_LENGTH => {
                    callback(can_id, channel, data, len as u8, info)
                },
                _ => Err(ZCanError::ParamNotSupported),
            }
        },
        _ => Err(ZCanError::ParamNotSupported),
    }
}

fn zcanfd_frame_new<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, ZCanError>
) -> Result<R, ZCanError>
    where
        T: AsRef<[u8]> {
    if let 0..=CAN_ID_FLAG = can_id {
        let mut data = Vec::from(data.as_ref());
        let len = data.len();
        data.resize(CANFD_FRAME_LENGTH, 0);
        if (can_id & CAN_EFF_MASK) > 0 {
            info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
        }
        if let 0..=CANFD_FRAME_LENGTH = len {
            callback(can_id, channel, data, len as u8, info)
        }
        else {
            Err(ZCanError::ParamNotSupported)
        }
    } else {
        Err(ZCanError::ParamNotSupported)
    }
}

// pub(self) fn zcan_frame_new2<const MAX_LEN: usize, T, R>(can_id: u32, channel: u8, data: T, mut info: ZCanHdrInfo,
//                                    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> R) -> Option<R>
fn zcan_frame_new2<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, ZCanError>
) -> Result<R, ZCanError>
    where
        T: AsRef<[u8]> {
    match can_id {
        0..=CAN_ID_FLAG => {
            let mut data = Vec::from(data.as_ref());
            let len = data.len();
            data.resize(CAN_FRAME_LENGTH, 0);
            if (can_id & CAN_EFF_MASK) > 0 {
                info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
            }
            match len {
                0..=CAN_FRAME_LENGTH => {
                    let mut can_id = can_id;
                    if info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0 {
                        can_id |= CAN_EFF_FLAG;
                    }
                    if info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0 {
                        can_id |= CAN_RTR_FLAG;
                    }
                    if info.get_field(ZCanHdrInfoField::IsErrorFrame) > 0 {
                        can_id |= CAN_ERR_FLAG;
                    }
                    callback(can_id, channel, data, len as u8, info)
                },
                _ => Err(ZCanError::ParamNotSupported),
            }
        },
        _ => Err(ZCanError::ParamNotSupported),
    }
}

fn zcanfd_frame_new2<T, R>(
    can_id: u32,
    channel: u8,
    data: T,
    mut info: ZCanHdrInfo,
    callback: impl Fn(u32, u8, Vec<u8>, u8, ZCanHdrInfo) -> Result<R, ZCanError>
) -> Result<R, ZCanError>
    where
        T: AsRef<[u8]> {
    if let 0..=CAN_ID_FLAG = can_id {
        let mut data = Vec::from(data.as_ref());
        let len = data.len();
        data.resize(CANFD_FRAME_LENGTH, 0);
        if (can_id & CAN_EFF_MASK) > 0 {
            info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
        }
        if let 0..=CANFD_FRAME_LENGTH = len {
            let mut can_id = can_id;
            if info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0 {
                can_id |= CAN_EFF_FLAG;
            }
            if info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0 {
                can_id |= CAN_RTR_FLAG;
            }
            if info.get_field(ZCanHdrInfoField::IsErrorFrame) > 0 {
                can_id |= CAN_ERR_FLAG;
            }
            callback(can_id, channel, data, len as u8, info)
        } else {
            Err(ZCanError::ParamNotSupported)
        }
    }
    else {
        Err(ZCanError::ParamNotSupported)
    }
}

#[cfg(test)]
mod tests {
    use crate::can::constant::{ZCanFrameType, ZCanTxMode};
    use super::{ZCanHdrInfo, ZCanHdrInfoField};

    #[test]
    fn frame_info() {
        let info: ZCanHdrInfo = Default::default();
        assert_eq!(info.mode, 0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::Normal as u8);
        assert_eq!(info.mode, 0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::Once as u8);
        assert_eq!(info.mode, 1);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::SelfReception as u8);
        assert_eq!(info.mode, 2);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::TxMode, ZCanTxMode::SelfReceptionOnce as u8);
        assert_eq!(info.mode, 3);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CAN as u8);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CANFD as u8);
        assert_eq!(info.mode, 0x10);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsRemoteFrame, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsRemoteFrame, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x01);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsExtendFrame, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x02);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsErrorFrame, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsErrorFrame, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x04);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x08);
        assert_eq!(info.pad, 0);

        let mut info: ZCanHdrInfo = Default::default();
        info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 0);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0);
        assert_eq!(info.pad, 0);
        info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 1);
        assert_eq!(info.mode, 0x0);
        assert_eq!(info.flag, 0x10);
        assert_eq!(info.pad, 0);
    }
}

