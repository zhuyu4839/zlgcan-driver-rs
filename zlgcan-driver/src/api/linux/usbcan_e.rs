use std::collections::HashMap;
use std::ffi::{c_uchar, c_uint, CString};
use dlopen2::symbor::{Symbol, SymBorApi};
use zlgcan_common::can::{CanChlCfg, ZCanChlCfgV1, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus, ZCanFrameType, ZCanFrameV3};
use zlgcan_common::device::{Handler, IProperty, SetValueFunc, ZCanDeviceType, ZDeviceInfo};
use zlgcan_common::error::ZCanError;
use zlgcan_common::utils::system_timestamp;
use crate::constant::{channel_bitrate, channel_work_mode};
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};

#[allow(non_snake_case)]
#[derive(Debug, SymBorApi)]
pub(crate) struct USBCANEApi<'a> {
    /// DEVICE_HANDLE ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    pub(crate) ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// INT ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    pub(crate) ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO* pInfo);
    pub(crate) ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// CHANNEL_HANDLE ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    pub(crate) ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfgV1) -> c_uint>,
    /// INT ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    pub(crate) ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// INT ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    pub(crate) ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// INT ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    pub(crate) ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// INT ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    pub(crate) ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrameV3, len: c_uint) -> c_uint>,
    /// INT ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);
    pub(crate) ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, msg: c_uchar) -> c_uint>,
    /// INT ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, INT wait_time);
    pub(crate) ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrameV3, size: c_uint, timeout: c_uint) -> c_uint>,
    /// INT ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    //ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, len: c_uint) -> c_uint>,
    /// INT ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, INT wait_time);
    //ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, size: c_uint, timeout: c_uint) -> c_uint>,

    /// IProperty* GetIProperty(DEVICE_HANDLE device_handle);   //获取属性接口
    pub(crate) GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// INT ReleaseIProperty(IProperty * pIProperty);
    pub(crate) ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,
}

impl USBCANEApi<'_> {
    pub(crate) const INVALID_DEVICE_HANDLE: u32 = 0;
    pub(crate) const INVALID_CHANNEL_HANDLE: u32 = 0;
    pub(crate) const STATUS_OK: u32 = 0;
    pub(crate) fn init_can_chl_ex(
        &self,
        dev_hdl: &mut Handler,
        channels: u8,
        cfg: &Vec<CanChlCfg>,
        timestamps: &mut HashMap<u8, u64>,
    ) -> Result<(), ZCanError> {
        let p = self.get_property(dev_hdl.device_handler())?;
        let set_value_func = p.SetValue;
        let mut error = None;
        for (idx, cfg) in cfg.iter().enumerate() {
            let idx = idx as u8;
            if idx >= channels {
                log::warn!("ZLGCAN - the length of CAN channel configuration is out of channels!");
                break;
            }

            if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                self.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                dev_hdl.remove_can(idx);
            }

            match self.start_channel(dev_hdl, idx, set_value_func, cfg) {
                Ok(()) => {
                    timestamps.insert(idx, system_timestamp());
                },
                Err(e) => {
                    error = Some(e);
                    break;
                }
            }
        }
        self.release_property(&p)?;

        match error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }
    #[inline]
    pub(crate) fn start_channel(
        &self,
        dev_hdl: &mut Handler,
        channel: u8,
        set_value_func: SetValueFunc,
        cfg: &CanChlCfg
    ) -> Result<(), ZCanError> {
        let chl_hdl = self.init_can_chl(dev_hdl.device_handler(), channel, cfg)?; // ZCAN_InitCAN
        dev_hdl.add_can(channel, chl_hdl);
        // self.usbcan_4e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
        self.set_channel(channel, set_value_func, cfg)?;

        match unsafe { (self.ZCAN_StartCAN)(chl_hdl) as u32 } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_StartCAN".to_string(), code)),
        }
    }

    pub(crate) fn set_channel(
        &self,
        channel: u8,
        func: SetValueFunc,
        cfg: &CanChlCfg
    ) -> Result<(), ZCanError> {
        unsafe {
            let func = func.ok_or(ZCanError::MethodNotSupported)?;
            let cmd_path = CString::new(channel_bitrate(channel))
                .map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
            let bitrate = CString::new(cfg.bitrate().to_string())
                .map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
            match func(cmd_path.as_ptr(), bitrate.as_ptr()) as u32 {
                Self::STATUS_OK => Ok(()),
                code => Err(ZCanError::MethodExecuteFailed(format!("{:?}, SetValue failed", cmd_path), code)),
            }?;

            let cmd_path = CString::new(channel_work_mode(channel))
                .map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
            let mode = CString::new(cfg.mode().to_string())
                .map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
            match func(cmd_path.as_ptr(), mode.as_ptr()) as u32 {
                Self::STATUS_OK => Ok(()),
                code => Err(ZCanError::MethodExecuteFailed(format!("{:?}, SetValue failed", cmd_path), code)),
            }
        }
    }
}

impl ZDeviceApi for USBCANEApi<'_> {
    type DeviceHandler = u32;
    type ChannelHandler = u32;
    fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } as u32 {
            Self::INVALID_DEVICE_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_OpenDevice".to_string(), Self::INVALID_DEVICE_HANDLE)),
            handler => Ok(handler),
        }
    }

    fn close(&self, dev_hdl: Self::DeviceHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_CloseDevice)(dev_hdl) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_CloseDevice".to_string(), code)),
        }
    }

    fn read_device_info(&self, dev_hdl: Self::DeviceHandler) -> Result<ZDeviceInfo, ZCanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(dev_hdl, &mut info) } as u32 {
            Self::STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_GetDeviceInf".to_string(), code)),
        }
    }

    fn get_property(&self, dev_hdl: Self::DeviceHandler) -> Result<IProperty, ZCanError> {
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
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ReleaseIProperty".to_string(), code)),
        }
    }
}

impl ZCanApi for USBCANEApi<'_> {
    type DeviceHandler = u32;
    type ChannelHandler = u32;
    type Frame = ZCanFrameV3;
    type FdFrame = ();
    fn init_can_chl(&self, dev_hdl: Self::DeviceHandler, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        unsafe {
            let dev_type = cfg.device_type()?;
            let handler = match dev_type {
                ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                    match (self.ZCAN_InitCAN)(dev_hdl, channel as u32, std::ptr::null()) as u32 {
                        Self::INVALID_CHANNEL_HANDLE =>
                            Err(ZCanError::MethodExecuteFailed("ZCAN_InitCAN".to_string(), Self::INVALID_CHANNEL_HANDLE)),
                        handler => Ok(handler),
                    }
                },
                ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                    let cfg = ZCanChlCfgV1::try_from(cfg)?;
                    match (self.ZCAN_InitCAN)(dev_hdl, channel as u32, &cfg) as u32 {
                        Self::INVALID_CHANNEL_HANDLE =>
                            Err(ZCanError::MethodExecuteFailed("ZCAN_InitCAN".to_string(), Self::INVALID_CHANNEL_HANDLE)),
                        handler => {
                            match (self.ZCAN_StartCAN)(handler) as u32 {
                                Self::STATUS_OK => Ok(handler),
                                code => Err(ZCanError::MethodExecuteFailed("ZCAN_StartCAN".to_string(), code)),
                            }
                        }
                    }
                },
                _ => Err(ZCanError::DeviceNotSupported),
            }?;

            Ok(handler)
        }
    }

    fn reset_can_chl(&self, chl_hdl: Self::ChannelHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ResetCAN)(chl_hdl) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ResetCAN".to_string(), code)),
        }
    }

    fn read_can_chl_status(&self, chl_hdl: Self::ChannelHandler) -> Result<ZCanChlStatus, ZCanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(chl_hdl, &mut status) } as u32 {
            Self::STATUS_OK => Ok(status),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelStatus".to_string(), code)),
        }
    }

    fn read_can_chl_error(&self, chl_hdl: Self::ChannelHandler) -> Result<ZCanChlError, ZCanError> {
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(chl_hdl, &mut info) } as u32  {
            Self::STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelErrInfo".to_string(), code)),
        }
    }

    fn clear_can_buffer(&self, chl_hdl: Self::ChannelHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(chl_hdl) } as u32 {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ClearBuffer".to_string(), code)),
        }
    }

    fn get_can_num(&self, chl_hdl: Self::ChannelHandler, can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(chl_hdl, can_type as u8) };
        log::debug!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        Ok(ret as u32)
    }

    fn receive_can(&self, chl_hdl: Self::ChannelHandler, size: u32, timeout: u32, resize: impl Fn(&mut Vec<Self::Frame>, usize)) -> Result<Vec<Self::Frame>, ZCanError> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_Receive)(chl_hdl, frames.as_mut_ptr(), size, timeout) };
        let ret = ret as u32;
        if ret < size {
            log::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_can(&self, chl_hdl: Self::ChannelHandler, frames: Vec<Self::Frame>) -> Result<u32, ZCanError> {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
        let ret = ret as u32;
        if ret < len {
            log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }
}

impl ZLinApi for USBCANEApi<'_> {
    type DeviceHandler = u32;
    type ChannelHandler = u32;
}
impl ZCloudApi for USBCANEApi<'_> {
    type DeviceHandler = u32;
    type ChannelHandler = u32;
}

#[cfg(test)]
mod tests {
    use std::ffi::CString;
    use dlopen2::symbor::{Library, SymBorApi};
    // use zlgcan_common::can::{CanChlCfgFactory, ZCanChlMode, ZCanChlType};
    use zlgcan_common::device::{ZCanDeviceType, ZDeviceInfo};
    // use crate::api::ZDeviceApi;
    use super::USBCANEApi;

    #[test]
    fn usbcan_4e_u() {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN_4E_U;
        let dev_idx = 0;
        let so_path = "library/linux/x86_64/libusbcan-4e.so";

        let lib = Library::open(so_path).expect("ZLGCAN - could not open library");
        let mut handlers = Vec::new();

        unsafe {
            let api = USBCANEApi::load(&lib).expect("ZLGCAN - could not load symbols!");
            let dev_hdl = (api.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0);
            if dev_hdl == USBCANEApi::INVALID_DEVICE_HANDLE {
                println!("Can't open the device!");
                return;
            }
            let mut dev_info = ZDeviceInfo::default();
            let ret = (api.ZCAN_GetDeviceInf)(dev_hdl, &mut dev_info);
            if ret != USBCANEApi::STATUS_OK {
                println!("Can't get the device info!");
                return;
            }

            let p = (api.GetIProperty)(dev_hdl);
            if p.is_null() {
                println!("Get property failed!");
                return;
            }
            let func = (*p).SetValue.expect("Can't get SetValue function!");

            for chl in 0..4 {
                let chl_hdl = (api.ZCAN_InitCAN)(dev_hdl, chl, std::ptr::null());
                if chl_hdl == USBCANEApi::INVALID_CHANNEL_HANDLE {
                    println!("Init channel: {} failed!", chl);
                    break;
                }
                handlers.push(chl_hdl);
                let ret = (api.ZCAN_ResetCAN)(chl_hdl);
                if ret != USBCANEApi::STATUS_OK {
                    println!("Reset channel: {} failed!", chl);
                }

                let path = CString::new(format!("info/channel/channel_{}/baud_rate", chl)).unwrap();
                let bitrate = CString::new(500_000.to_string()).unwrap();

                // let func = (*p).SetValue.expect("Can't get SetValue function!");
                let ret = func(path.as_ptr(), bitrate.as_ptr());

                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("SetValue failed: {}!", ret);
                    break;
                }

                let ret = (api.ZCAN_StartCAN)(chl_hdl);
                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("ZCAN_StartCAN failed!");
                    break;
                }
            }

            let ret = (api.ReleaseIProperty)(p);
            if ret as u32 != USBCANEApi::STATUS_OK {
                println!("ReleaseIProperty failed!");
            }

            for handler in handlers {
                let ret = (api.ZCAN_ResetCAN)(handler);
                if ret as u32 != USBCANEApi::STATUS_OK {
                    println!("ZCAN_ResetCAN failed!");
                }
            }

            let ret = (api.ZCAN_CloseDevice)(dev_hdl);
            if ret as u32 != USBCANEApi::STATUS_OK {
                println!("ZCAN_CloseDevice failed!");
            }
        }
    }
}
