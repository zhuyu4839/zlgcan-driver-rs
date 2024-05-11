use std::ffi::{c_uchar, c_uint};
use dlopen2::symbor::{Symbol, SymBorApi};
use log::{debug, warn};
use zlgcan_common::can::{
    CanChlCfg,
    ZCanChlCfgDetail, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus,
    ZCanFrameType,
    ZCanFrame
};
use zlgcan_common::device::{IProperty, ZCanDeviceType, ZDeviceInfo};
use zlgcan_common::error::ZCanError;
use crate::constant::{INVALID_CHANNEL_HANDLE, INVALID_DEVICE_HANDLE, STATUS_OK};
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};

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

impl ZDeviceApi<u32, u32> for USBCANEApi<'_> {
    fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } as u32 {
            INVALID_DEVICE_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_OpenDevice".to_string(), INVALID_DEVICE_HANDLE)),
            handler => Ok(handler),
        }
    }

    fn close(&self, dev_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_CloseDevice)(dev_hdl) } as u32 {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_CloseDevice".to_string(), code)),
        }
    }

    fn read_device_info(&self, dev_hdl: u32) -> Result<ZDeviceInfo, ZCanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(dev_hdl, &mut info) } as u32 {
            STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_GetDeviceInf".to_string(), code)),
        }
    }

    fn get_property(&self, dev_hdl: u32) -> Result<IProperty, ZCanError> {
        let ret = unsafe { (self.GetIProperty)(dev_hdl) };
        if ret.is_null() {
            Err(ZCanError::MethodExecuteFailed("GetIProperty".to_string(), 0))
        }
        else {
            unsafe { Ok(*ret) }
        }
    }

    fn release_property(&self, p: &IProperty) -> Result<(), ZCanError> {
        match unsafe { (self.ReleaseIProperty)(p) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ReleaseIProperty".to_string(), code)),
        }
    }
}

impl ZCanApi<u32, u32> for USBCANEApi<'_> {
    fn init_can_chl(&self, dev_hdl: u32, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        unsafe {
            let cfg = ZCanChlCfgDetail::try_from(cfg)?;
            match (self.ZCAN_InitCAN)(dev_hdl, channel as u32, &cfg) as u32 {
                INVALID_CHANNEL_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_InitCAN".to_string(), INVALID_CHANNEL_HANDLE)),
                handler => {
                    match (self.ZCAN_StartCAN)(handler) as u32 {
                        STATUS_OK => Ok(handler),
                        code => Err(ZCanError::MethodExecuteFailed("ZCAN_StartCAN".to_string(), code)),
                    }
                }
            }
        }
    }

    fn reset_can_chl(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ResetCAN)(chl_hdl) } as u32 {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ResetCAN".to_string(), code)),
        }
    }

    fn read_can_chl_status(&self, chl_hdl: u32) -> Result<ZCanChlStatus, ZCanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(chl_hdl, &mut status) } as u32 {
            STATUS_OK => Ok(status),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelStatus".to_string(), code)),
        }
    }

    fn read_can_chl_error(&self, chl_hdl: u32) -> Result<ZCanChlError, ZCanError> {
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(chl_hdl, &mut info) } as u32  {
            STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelErrInfo".to_string(), code)),
        }
    }

    fn clear_can_buffer(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(chl_hdl) } as u32 {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ClearBuffer".to_string(), code)),
        }
    }

    fn get_can_num(&self, chl_hdl: u32, can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(chl_hdl, can_type as u8) };
        debug!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        Ok(ret as u32)
    }

    fn receive_can(&self, chl_hdl: u32, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCanFrame>, usize)) -> Result<Vec<ZCanFrame>, ZCanError> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_Receive)(chl_hdl, frames.as_mut_ptr(), size, timeout) };
        let ret = ret as u32;
        if ret < size {
            warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_can(&self, chl_hdl: u32, frames: Vec<ZCanFrame>) -> Result<u32, ZCanError> {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
        let ret = ret as u32;
        if ret < len {
            warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }
}

impl<DH, CH> ZLinApi<DH, CH> for USBCANEApi<'_> {}
impl<DH> ZCloudApi<DH> for USBCANEApi<'_> {}

