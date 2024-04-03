use std::collections::HashMap;
use std::ffi::{c_uchar, c_uint, c_ushort};
use std::mem::ManuallyDrop;
use crate::can::frame::ZCanHeaderV1;
use super::constant::{BRP, CANERR_FRAME_LENGTH, SJW, SMP, TSEG1, TSEG2, ZCanChlMode, ZCanChlType, ZCanFilterType};

/// Linux USBCAN USBCAN_4E(8_E) USBCANFD_800U and windows
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanChlCfg {
    acc_code: c_uint,
    acc_mask: c_uint,
    #[allow(dead_code)]
    reserved: c_uint,
    filter: c_uchar,
    timing0: c_uchar,
    timing1: c_uchar,
    mode: c_uchar,
}
impl ZCanChlCfg {
    #[inline(always)]
    pub fn new(mode: ZCanChlMode, timing0: u32, timing1: u32, filter: ZCanFilterType, acc_code: Option<u32>, acc_mask: Option<u32>) -> Self {
        Self {
            acc_code: acc_code.unwrap_or(0),
            acc_mask: acc_mask.unwrap_or(0xFFFFFFFF),
            reserved: Default::default(),
            filter: filter as u8,
            timing0: timing0 as u8,
            timing1: timing1 as u8,
            mode: mode as u8,
        }
    }
}

/// Linux USBCAN_4E_8E USBCANFD_800U and windows
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanFdChlCfgV1 {
    acc_code: c_uint,
    acc_mask: c_uint,
    timing0: c_uint,    // abit_timing when USBCANFD
    timing1: c_uint,    // dbit_timing when USBCANFD
    brp: c_uint,
    filter: c_uchar,
    mode: c_uchar,
    #[allow(dead_code)]
    pad: c_ushort,
    #[allow(dead_code)]
    reserved: c_uint,
}
impl ZCanFdChlCfgV1 {
    #[inline(always)]
    pub fn new(mode: ZCanChlMode, timing0: u32, timing1: u32, filter: ZCanFilterType, acc_code: Option<u32>, acc_mask: Option<u32>, brp: Option<u32>) -> Self {
        Self {
            acc_code: acc_code.unwrap_or(0),
            acc_mask: acc_mask.unwrap_or(0xFFFFFFFF),
            timing0,
            timing1,
            brp: brp.unwrap_or(0),
            filter: filter as u8,
            mode: mode as u8,
            pad: Default::default(),
            reserved: Default::default(),
        }
    }
}

/// Linux USBCANFD
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanFdChlCfgSet {
    tseg1: c_uchar,
    tseg2: c_uchar,
    sjw: c_uchar,
    smp: c_uchar,
    brp: c_ushort,
}

impl From<&HashMap<String, u32>> for ZCanFdChlCfgSet {
    fn from(value: &HashMap<String, u32>) -> Self {
        let tseg1 = value.get(TSEG1).expect(format!("ZLGCAN - `{}` is not configured in file!", TSEG1).as_str());
        let tseg2 = value.get(TSEG2).expect(format!("ZLGCAN - `{}` is not configured in file!", TSEG2).as_str());
        let sjw = value.get(SJW).expect(format!("ZLGCAN - `{}` is not configured in file!", SJW).as_str());
        let smp = value.get(SMP).expect(format!("ZLGCAN - `{}` is not configured in file!", SMP).as_str());
        let brp = value.get(BRP).expect(format!("ZLGCAN - `{}` is not configured in file!", BRP).as_str());

        Self::new(*tseg1, *tseg2, *sjw, *smp, *brp)
    }
}

impl ZCanFdChlCfgSet {
    #[inline(always)]
    pub fn new(tseg1: u32, tseg2: u32, sjw: u32, smp: u32, brp: u32) -> Self {
        Self {
            tseg1: tseg1 as u8,
            tseg2: tseg2 as u8,
            sjw: sjw as u8,
            smp: smp as u8,
            brp: brp as u16,
        }
    }
}
/// Linux USBCANFD
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZCanFdChlCfgV2 {
    #[doc = "< clock(Hz)"]
    clk: c_uint,
    #[doc = "< bit0-normal/listen_only, bit1-ISO/BOSCH"]
    mode: c_uint,
    aset: ZCanFdChlCfgSet,
    dset: ZCanFdChlCfgSet,
}
impl ZCanFdChlCfgV2 {
    #[inline(always)]
    pub fn new(can_type: ZCanChlType, mode: ZCanChlMode, clock: u32, aset: ZCanFdChlCfgSet, dset: ZCanFdChlCfgSet) -> Self {
        let mut mode = mode as u32;
        match can_type {
            ZCanChlType::CANFD_NON_ISO => mode |= 2,
            _ => {},
        }
        Self {
            clk: clock,
            mode,
            aset,
            dset,
        }
    }
}
/// end of Linux USBCANFD

#[repr(C)]
pub union ZCanChlCfgV1Union {
    can: ZCanChlCfg,
    canfd: ZCanFdChlCfgV1,
}
impl ZCanChlCfgV1Union {
    #[inline(always)]
    pub fn from_can(can: ZCanChlCfg) -> Self {
        Self { can }
    }
    #[inline(always)]
    pub fn from_canfd(canfd: ZCanFdChlCfgV1) -> Self {
        Self { canfd }
    }
}

/// CAN channel configuration v1.
/// used windows and USBCAN_4E(8_E) or USBCANFD-800U on Linux
#[repr(C)]
pub struct ZCanChlCfgV1 {
    can_type: c_uint,
    cfg: ZCanChlCfgV1Union,
}
impl ZCanChlCfgV1 {
    #[inline(always)]
    pub fn new(can_type: ZCanChlType, cfg: ZCanChlCfgV1Union) -> Self {
        Self { can_type: can_type as u32, cfg }
    }
}

/// CAN channel configuration v2.
/// used USBCAN USBCANFD on Linux
#[repr(C)]
pub union ZCanChlCfgV2 {
    can: ZCanChlCfg,
    canfd: ZCanFdChlCfgV2
}
impl ZCanChlCfgV2 {
    #[inline(always)]
    pub fn from_can(can: ZCanChlCfg) -> Self {
        Self { can }
    }
    #[inline(always)]
    pub fn from_canfd(canfd: ZCanFdChlCfgV2) -> Self {
        Self { canfd }
    }
}

#[repr(C)]
pub union ZCanChlCfgDetail {
    v1: ManuallyDrop<ZCanChlCfgV1>,
    v2: ManuallyDrop<ZCanChlCfgV2>,
}

impl From<ZCanChlCfgV1> for ZCanChlCfgDetail {
    fn from(value: ZCanChlCfgV1) -> Self {
        Self { v1: ManuallyDrop::new(value) }
    }
}

impl From<ZCanChlCfgV2> for ZCanChlCfgDetail {
    fn from(value: ZCanChlCfgV2) -> Self {
        Self { v2: ManuallyDrop::new(value) }
    }
}

impl ZCanChlCfgDetail {
    #[deprecated(since = "0.2.3-Beta3", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v1(v1: ZCanChlCfgV1) -> Self {
        Self { v1: ManuallyDrop::new(v1) }
    }
    #[deprecated(since = "0.2.3-Beta3", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v2(v2: ZCanChlCfgV2) -> Self {
        Self { v2: ManuallyDrop::new(v2) }
    }
}

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanChlStatus {
    pub errInterrupt: c_uchar,  /**< not used(for backward compatibility) */
    pub regMode: c_uchar,       /**< not used */
    pub regStatus: c_uchar,     /**< not used */
    pub regALCapture: c_uchar,  /**< not used */
    pub regECCapture: c_uchar,  /**< not used */
    pub regEWLimit: c_uchar,    /**< not used */
    pub regRECounter: c_uchar,  /**< RX errors */
    pub regTECounter: c_uchar,  /**< TX errors */
    pub Reserved: c_uint,
}

/// used by USBCANFD on linux
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanChlErrorV1 {
    pub hdr: ZCanHeaderV1,
    pub data: [c_uchar; CANERR_FRAME_LENGTH],
}

/// USBCAN USBCAN-4_E/8_E USBCAN-800U in linux and windows
#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct ZCanChlErrorV2 {
    pub error_code: c_uint,
    pub passive_ErrData: [c_uchar; 3usize],
    pub arLost_ErrData: c_uchar,
}

#[repr(C)]
pub union ZCanChlError {
    v1: ZCanChlErrorV1,
    v2: ZCanChlErrorV2,
}

impl From<ZCanChlErrorV1> for ZCanChlError {
    fn from(value: ZCanChlErrorV1) -> Self {
        Self { v1: value }
    }
}

impl From<&ZCanChlError> for ZCanChlErrorV1 {
    fn from(value: &ZCanChlError) -> Self {
        unsafe { value.v1 }
    }
}

impl From<ZCanChlErrorV2> for ZCanChlError {
    fn from(value: ZCanChlErrorV2) -> Self {
        Self { v2: value }
    }
}

impl From<&ZCanChlError> for ZCanChlErrorV2 {
    fn from(value: &ZCanChlError) -> Self {
        unsafe { value.v2 }
    }
}

impl ZCanChlError {
    #[deprecated(since = "0.2.3-Beta2", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v1(v1: ZCanChlErrorV1) -> Self {
        Self { v1 }
    }
    #[deprecated(since = "0.2.3-Beta2", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn from_v2(v2: ZCanChlErrorV2) -> Self {
        Self { v2 }
    }
    #[deprecated(since = "0.2.3-Beta2", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn get_v1(&self) -> ZCanChlErrorV1 {
        unsafe { self.v1 }
    }
    #[deprecated(since = "0.2.3-Beta2", note = "Please use `from` to convert!")]
    #[inline(always)]
    pub fn get_v2(&self) -> ZCanChlErrorV2 {
        unsafe { self.v2 }
    }
}
