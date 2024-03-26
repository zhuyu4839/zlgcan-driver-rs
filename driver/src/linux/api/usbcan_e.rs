// #![allow(non_upper_case_globals)]
// #![allow(non_camel_case_types)]
// #![allow(non_snake_case)]
// #![allow(unused)]
//
// #[cfg(not(debug_assertions))]
// include!(concat!(env!("OUT_DIR"), "/usbcan_e.rs"));
//
// #[cfg(debug_assertions)]
// include!("bindings/usbcan_e.rs");

use std::ffi::{c_uchar, c_uint};
use dlopen2::symbor::{Symbol, SymBorApi};
use log::{debug, warn};
use common::can::CanChlCfg;
use common::can::channel::{ZCanChlCfgDetail, ZCanChlError, ZCanChlStatus};
use common::can::constant::ZCanFrameType;
use common::can::frame::ZCanFrame;
use common::device::{IProperty, ZCanDeviceType, ZDeviceInfo};
use common::error::ZCanError;
use crate::constant::{INVALID_CHANNEL_HANDLE, STATUS_OK};

#[allow(non_snake_case)]
#[derive(SymBorApi)]
pub(crate) struct USBCANEApi<'a> {
    /// DEVICE_HANDLE ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// INT ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO* pInfo);
    ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// CHANNEL_HANDLE ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfgDetail) -> c_uint>,
    /// INT ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// INT ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// INT ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// INT ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);
    ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, msg: c_uchar) -> c_uint>,
    /// INT ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, INT wait_time);
    ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// INT ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    //ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, len: c_uint) -> c_uint>,
    /// INT ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, INT wait_time);
    //ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, size: c_uint, timeout: c_uint) -> c_uint>,

    /// IProperty* GetIProperty(DEVICE_HANDLE device_handle);   //获取属性接口
    GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// INT ReleaseIProperty(IProperty * pIProperty);
    ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,
}

impl USBCANEApi<'_> {
    #[inline(always)]
    pub(crate) fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } as u32 {
            STATUS_OK => Ok(1),
            code => Err(ZCanError::new(code, format!("ZLGCAN - {} open failed", dev_type))),
        }
    }
    #[inline(always)]
    pub(crate) fn close(&self, dev_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_CloseDevice)(dev_hdl) } as u32 {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code as u32, "ZLGCAN - {} close failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn read_device_info(&self, dev_hdl: u32) -> Result<ZDeviceInfo, ZCanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(dev_hdl, &mut info) } as u32 {
            STATUS_OK => Ok(info),
            code => Err(ZCanError::new(code as u32, "ZLGCAN - read device info failed".to_string())),
        }
    }
    // #[inline(always)]
    // pub(crate) fn is_online(&self, _dev_hdl: u32) -> Result<bool, ZCanError> {
    //     Err(ZCanError::new(0, "ZLGCAN - method not supported by device".to_string()))
    // }
    #[inline(always)]
    pub(crate) fn init_can_chl(&self, dev_hdl: u32, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        unsafe {
            let cfg = ZCanChlCfgDetail::from(cfg);
            match (self.ZCAN_InitCAN)(dev_hdl, channel as u32, &cfg) as u32 {
                INVALID_CHANNEL_HANDLE => Err(ZCanError::new(INVALID_CHANNEL_HANDLE, format!("ZLGCAN - `InitCAN` channel: {} failed", channel))),
                handler => {
                    match (self.ZCAN_StartCAN)(handler) as u32 {
                        STATUS_OK => Ok(handler),
                        code => Err(ZCanError::new(code, format!("ZLGCAN - `StartCAN` channel: {} failed", channel))),
                    }
                }
            }
        }
    }
    #[inline(always)]
    pub(crate) fn reset_can_chl(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ResetCAN)(chl_hdl) } as u32 {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - CAN channel reset failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn read_can_chl_status(&self, chl_hdl: u32) -> Result<ZCanChlStatus, ZCanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(chl_hdl, &mut status) } as u32 {
            STATUS_OK => Ok(status),
            code =>Err(ZCanError::new(code, "ZLGCAN - read CAN channel status failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn read_can_chl_error(&self, chl_hdl: u32) -> Result<ZCanChlError, ZCanError> {
        let mut info: ZCanChlError = ZCanChlError::from_v2(Default::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(chl_hdl, &mut info) } as u32  {
            STATUS_OK => Ok(info),
            code =>Err(ZCanError::new(code, "ZLGCAN - read CAN channel error info failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn clear_can_buffer(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(chl_hdl) } as u32 {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - clear CAN channel's buffer failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn get_can_num(&self, chl_hdl: u32, msg: ZCanFrameType) -> u32 {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(chl_hdl, msg as u8) };
        debug!("ZLGCAN - get receive {} number: {}.", msg, ret);
        ret as u32
    }
    #[inline(always)]
    pub(crate) fn receive_can(&self, chl_hdl: u32, size: u32, timeout: Option<u32>, resize: impl Fn(&mut Vec<ZCanFrame>, usize)) -> Vec<ZCanFrame> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_Receive)(chl_hdl, frames.as_mut_ptr(), size, timeout.unwrap_or(50)) };
        let ret = ret as u32;
        if ret < size {
            warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        frames
    }
    #[inline(always)]
    pub(crate) fn transmit_can(&self, chl_hdl: u32, frames: Vec<ZCanFrame>) -> u32 {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
        let ret = ret as u32;
        if ret < len {
            warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        ret
    }
    // #[inline(always)]
    // pub(crate) fn receive_canfd(&self, chl_hdl: u32, size: u32, timeout: Option<u32>, resize: fn(&mut Vec<ZCanFdFrame>, usize)) -> Vec<ZCanFdFrame> {
    //     let mut frames = Vec::new();
    //     // frames.resize_with(size as usize, Default::default);
    //     resize(&mut frames, size as usize);
    //
    //     let ret = unsafe { (self.ZCAN_ReceiveFD)(chl_hdl, frames.as_mut_ptr(), size, timeout.unwrap_or(50)) };
    //     let ret = ret as u32;
    //     if ret < size {
    //         warn!("ZLGCAN - receive CANFD frame expect: {}, actual: {}!", size, ret);
    //     }
    //     frames
    // }
    // #[inline(always)]
    // pub(crate) fn transmit_canfd(&self, chl_hdl: u32, frames: Vec<ZCanFdFrame>) -> u32 {
    //     let len = frames.len() as u32;
    //     let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frames.as_ptr(), len) };
    //     let ret = ret as u32;
    //     if ret < len {
    //         warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
    //     }
    //     ret
    // }
}

#[allow(dead_code)]
impl USBCANEApi<'_> {
    pub(crate) unsafe fn get_property(&self, dev_hdl: u32) -> Option<IProperty> {
        let ret = (self.GetIProperty)(dev_hdl);
        if ret.is_null() {
            None
        }
        else {
            Some(*ret)
        }
    }
    pub(crate) unsafe fn release_property(self, p: &IProperty) -> Result<(), ZCanError> {
        match (self.ReleaseIProperty)(p) {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "release_property"))),
        }
    }
}

