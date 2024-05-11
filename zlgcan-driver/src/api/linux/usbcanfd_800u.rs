use dlopen2::symbor::{Symbol, SymBorApi};
use std::ffi::{c_char, c_uchar, c_uint, c_void, CString};

use zlgcan_common::can::{CanChlCfg, ZCanChlCfgDetail, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus, ZCanChlType, ZCanFdFrame, ZCanFrame, ZCanFrameType};
use zlgcan_common::device::{CmdPath, IProperty, ZCanDeviceType, ZDeviceInfo};
use zlgcan_common::error::ZCanError;
use zlgcan_common::utils::c_str_to_string;

use crate::constant::{STATUS_OK, INVALID_CHANNEL_HANDLE, INVALID_DEVICE_HANDLE};
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};

#[allow(non_snake_case)]
#[derive(SymBorApi)]
pub(crate) struct USBCANFD800UApi<'a> {
    /// DEVICE_HANDLE FUNC_CALL ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO* pInfo);
    ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_IsDeviceOnLine(DEVICE_HANDLE device_handle);
    //ZCAN_IsDeviceOnLine: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,

    /// CHANNEL_HANDLE FUNC_CALL ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, ZCanChlCfgDetail) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);    //type:TYPE_CAN, TYPE_CANFD, TYPE_ALL_DATA
    ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, can_type: c_uchar) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFdFrame, size: c_uint, timeout: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_TransmitData(DEVICE_HANDLE device_handle, ZCANDataObj* pTransmit, UINT len);
    // ZCAN_TransmitData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, data: *const ZCANDataObj, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveData(DEVICE_HANDLE device_handle, ZCANDataObj* pReceive, UINT len, int wait_time DEF(-1));
    // ZCAN_ReceiveData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, frames: *mut ZCANDataObj, size: c_uint, timeout: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_SetValue(DEVICE_HANDLE device_handle, const char* path, const void* value);
    // ZCAN_SetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char, value: *const c_void) -> c_uint>,
    /// const void* FUNC_CALL ZCAN_GetValue(DEVICE_HANDLE device_handle, const char* path);
    // ZCAN_GetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char) -> *const c_void>,
    /// IProperty* FUNC_CALL GetIProperty(DEVICE_HANDLE device_handle);
    GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// UINT FUNC_CALL ReleaseIProperty(IProperty * pIProperty);
    ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetReference(UINT DeviceType, UINT nDevIndex, UINT nChnlIndex, UINT nRefType, void* pData);
    ZCAN_GetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, cmd: c_uint, value: *mut c_void) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetReference(UINT DeviceType, UINT nDevIndex, UINT nChnlIndex, UINT nRefType, void* pData);
    ZCAN_SetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, cmd: c_uint, value: *const c_void) -> c_uint>,
}

impl ZDeviceApi<u32, u32> for USBCANFD800UApi<'_> {
    fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            INVALID_DEVICE_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_OpenDevice".to_string(), INVALID_DEVICE_HANDLE)),
            v => Ok(v),
        }
    }

    fn close(&self, dev_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_CloseDevice)(dev_hdl) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_CloseDevice".to_string(), code)),
        }
    }

    fn read_device_info(&self, dev_hdl: u32) -> Result<ZDeviceInfo, ZCanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(dev_hdl, &mut info) } {
            STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_GetDeviceInf".to_string(), code)),
        }
    }

    fn get_property(&self, chl_hdl: u32) -> Result<IProperty, ZCanError> {
        let ret = unsafe { (self.GetIProperty)(chl_hdl) };
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

    // fn set_reference(&self, chl_hdl: u32, cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
    //     match unsafe { (self.ZCAN_SetReference)(dev_type as u32, dev_idx, cmd, value) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::MethodExecuteFailed("ZCAN_SetReference".to_string(), code)),
    //     }
    // }

    // fn get_reference(&self, chl_hdl: u32, cmd_path: &CmdPath, value: *mut c_void) -> Result<(), ZCanError> {
    //     match unsafe { (self.ZCAN_GetReference)(dev_type as u32, dev_idx, cmd, value) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::MethodExecuteFailed("ZCAN_GetReference".to_string(), code)),
    //     }
    // }

    fn set_values(&self, chl_hdl: u32, values: Vec<(CmdPath, *const c_char)>) -> Result<(), ZCanError> {
        unsafe {
            let p = self.get_property(chl_hdl)?;
            match p.SetValue {
                Some(f) => {
                    for (cmd, value) in values {
                        let path = cmd.get_path();
                        // let _path = format!("{}/{}", path, channel);
                        let _path = CString::new(path).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                        match f(_path.as_ptr(), value) {
                            1 => (),
                            _ => log::warn!("ZLGCAN - set `{}` failed!", path),
                        }
                    }

                    let _ = self.release_property(&p).is_err_and(|e| -> bool {
                        log::warn!("{}", e);
                        true
                    });
                    Ok(())
                },
                None => Err(ZCanError::MethodNotSupported),
            }
        }
    }

    fn get_values(&self, chl_hdl: u32, channel: u8, paths: Vec<CmdPath>) -> Result<Vec<String>, ZCanError> {
        unsafe {
            let p = self.get_property(chl_hdl)?;
            match p.GetValue {
                Some(f) => {
                    let mut result = Vec::new();
                    for cmd in paths {
                        let path = cmd.get_path();
                        let _path = CString::new(format!("{}/{}", path, channel)).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                        let ret = f(_path.as_ptr());
                        let v = c_str_to_string(ret)?;
                        result.push(v);
                    }

                    let _ = self.release_property(&p).is_err_and(|e| -> bool {
                        log::warn!("{}", e);
                        true
                    });

                    Ok(result)
                },
                None => Err(ZCanError::MethodNotSupported),
            }
        }
    }
}

impl ZCanApi<u32, u32> for USBCANFD800UApi<'_> {
    #[allow(unused_variables)]
    fn init_can_chl(&self, dev_hdl: u32, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        let dev_type = cfg.device_type()?;
        unsafe {
            // configure the clock
            if let Some(clock) = cfg.clock() {
                todo!()
                // let clock_path = CmdPath::new_path("clock");
                // self.set_check_value(dev_hdl, &clock_path, clock.to_string().as_str(), dev_type)?;
            }
            // set channel resistance status
            if dev_type.has_resistance() {
                todo!()
                // let state = (cfg.extra().resistance() as u32).to_string();
                // let resistance_path = format!("{}/{}", channel, INTERNAL_RESISTANCE);
                // let resistance_path = CmdPath::new_path(resistance_path.as_str());
                // self.set_check_value(dev_hdl, &resistance_path, state.as_str(), dev_type)?;
            }
            // set channel protocol
            let can_type = cfg.can_type()?;
            // let protocol = (can_type as u32).to_string();
            // let protocol_path = format!("{}/{}", channel, PROTOCOL);
            // let protocol_path = CmdPath::new_path(protocol_path.as_str());
            // self.set_check_value(dev_hdl, &protocol_path, protocol.as_str(), dev_type)?;

            // set channel bitrate
            // let bitrate = cfg.bitrate();
            // let abitrate_path = format!("{}/{}", channel, CANFD_ABIT_BAUD_RATE);
            // let abitrate_path = CmdPath::new_path(abitrate_path.as_str());
            // self.set_check_value(dev_hdl, &abitrate_path, bitrate.to_string().as_str(), dev_type)?;
            match can_type {
                ZCanChlType::CANFD_ISO | ZCanChlType::CANFD_NON_ISO => {
                    todo!()
                    // let dbitrate = cfg.extra().dbitrate().unwrap_or(bitrate).to_string();
                    // let dbitrate_path = format!("{}/{}", channel, CANFD_DBIT_BAUD_RATE);
                    // let dbitrate_path = CmdPath::new_path(dbitrate_path.as_str());
                    // self.set_check_value(dev_hdl, &dbitrate_path, dbitrate.as_str(), dev_type)?;
                },
                _ => {},
            }

            let cfg = ZCanChlCfgDetail::try_from(cfg)?;
            match (self.ZCAN_InitCAN)(dev_hdl, channel as u32, cfg) {
                INVALID_CHANNEL_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_InitCAN".to_string(), INVALID_CHANNEL_HANDLE)),
                handler => {
                    match (self.ZCAN_StartCAN)(handler) {
                        STATUS_OK => Ok(handler),
                        code => Err(ZCanError::MethodExecuteFailed("ZCAN_InitCAN".to_string(), code)),
                    }
                }
            }
        }
    }

    fn reset_can_chl(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ResetCAN)(chl_hdl) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ResetCAN".to_string(), code)),
        }
    }

    fn read_can_chl_status(&self, chl_hdl: u32) -> Result<ZCanChlStatus, ZCanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(chl_hdl, &mut status) } {
            STATUS_OK => Ok(status),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelStatus".to_string(), code)),
        }
    }

    fn read_can_chl_error(&self, chl_hdl: u32) -> Result<ZCanChlError, ZCanError> {
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(chl_hdl, &mut info) } {
            STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelErrInfo".to_string(), code)),
        }
    }

    fn clear_can_buffer(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(chl_hdl) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ClearBuffer".to_string(), code)),
        }
    }

    fn get_can_num(&self, chl_hdl: u32, can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(chl_hdl, can_type as u8) };
        log::debug!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        Ok(ret)
    }

    fn receive_can(&self, chl_hdl: u32, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCanFrame>, usize)) -> Result<Vec<ZCanFrame>, ZCanError> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_Receive)(chl_hdl, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_can(&self, chl_hdl: u32, frames: Vec<ZCanFrame>) -> Result<u32, ZCanError> {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
        if ret < len {
            log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }

    fn receive_canfd(&self, chl_hdl: u32, size: u32, timeout: u32, resize: fn(&mut Vec<ZCanFdFrame>, usize)) -> Result<Vec<ZCanFdFrame>, ZCanError> {
        let mut frames = Vec::new();
        // frames.resize_with(size as usize, Default::default);
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_ReceiveFD)(chl_hdl, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive CANFD frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_canfd(&self, chl_hdl: u32, frames: Vec<ZCanFdFrame>) -> Result<u32, ZCanError> {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frames.as_ptr(), len) };
        if ret < len {
            log::warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }
}

impl<DH, CH> ZLinApi<DH, CH> for USBCANFD800UApi<'_> {

}
impl<DH> ZCloudApi<DH> for USBCANFD800UApi<'_> {}
