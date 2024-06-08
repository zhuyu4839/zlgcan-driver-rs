use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ushort, c_void, CString};
use dlopen2::symbor::{Symbol, SymBorApi};
use zlgcan_common::can::{CanChlCfg, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus, ZCanChlType, ZCanFdFrameV2, ZCanFrameV3, ZCanFrameType, ZCanChlCfgV1};
use zlgcan_common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use zlgcan_common::device::{CmdPath, IProperty, ZCanDeviceType, ZCanError, ZDeviceInfo};
use zlgcan_common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};
use zlgcan_common::utils::c_str_to_string;

use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};
use crate::constant::{STATUS_OFFLINE, STATUS_ONLINE, INTERNAL_RESISTANCE, PROTOCOL, CANFD_ABIT_BAUD_RATE, CANFD_DBIT_BAUD_RATE, BAUD_RATE, CLOCK};

#[allow(non_snake_case)]
#[derive(Debug, SymBorApi)]
pub(crate) struct Api<'a> {
    /// DEVICE_HANDLE FUNC_CALL ZCAN_OpenDevice(UINT device_type, UINT device_index, UINT reserved);
    ZCAN_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_index: c_uint, reserved: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_CloseDevice(DEVICE_HANDLE device_handle);
    ZCAN_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetDeviceInf(DEVICE_HANDLE device_handle, ZCAN_DEVICE_INFO *pInfo);
    ZCAN_GetDeviceInf: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, dev_info: *const ZDeviceInfo) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_IsDeviceOnLine(DEVICE_HANDLE device_handle);
    ZCAN_IsDeviceOnLine: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_TransmitData(DEVICE_HANDLE device_handle, ZCANDataObj* pTransmit, UINT len);
    // ZCAN_TransmitData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, data: *const ZCANDataObj, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveData(DEVICE_HANDLE device_handle, ZCANDataObj* pReceive, UINT len, int wait_time DEF(-1));
    // ZCAN_ReceiveData: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, data: *mut ZCANDataObj, timeout: c_uint) -> c_uint>,

    /// UINT FUNC_CALL ZCAN_SetValue(DEVICE_HANDLE device_handle, const char* path, const void* value);
    ZCAN_SetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char, value: *const c_void) -> c_uint>,
    /// const void* FUNC_CALL ZCAN_GetValue(DEVICE_HANDLE device_handle, const char* path);
    ZCAN_GetValue: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, path: *const c_char) -> *const c_void>,
    /// IProperty* FUNC_CALL GetIProperty(DEVICE_HANDLE device_handle);
    GetIProperty: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint) -> *const IProperty>,
    /// UINT FUNC_CALL ReleaseIProperty(IProperty * pIProperty);
    ReleaseIProperty: Symbol<'a, unsafe extern "C" fn(p: *const IProperty) -> c_uint>,

    /// CHANNEL_HANDLE FUNC_CALL ZCAN_InitCAN(DEVICE_HANDLE device_handle, UINT can_index, ZCAN_CHANNEL_INIT_CONFIG* pInitConfig);
    ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfgV1) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_StartCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_StartCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ResetCAN(CHANNEL_HANDLE channel_handle);
    ZCAN_ResetCAN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ClearBuffer(CHANNEL_HANDLE channel_handle);
    ZCAN_ClearBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelErrInfo(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_ERR_INFO* pErrInfo);
    ZCAN_ReadChannelErrInfo: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, *mut ZCanChlError) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReadChannelStatus(CHANNEL_HANDLE channel_handle, ZCAN_CHANNEL_STATUS* pCANStatus);
    ZCAN_ReadChannelStatus: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, *mut ZCanChlStatus) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetReceiveNum(CHANNEL_HANDLE channel_handle, BYTE type);//type:TYPE_CAN, TYPE_CANFD, TYPE_ALL_DATA
    ZCAN_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, can_type: c_uchar) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Transmit(CHANNEL_HANDLE channel_handle, ZCAN_Transmit_Data* pTransmit, UINT len);
    ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrameV3, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFrameV3, len: c_uint, timeout: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrameV2, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFdFrameV2, len: c_uint, timeout: c_uint) -> c_uint>,

    /// void FUNC_CALL ZCLOUD_SetServerInfo(const char* httpSvr, unsigned short httpPort, const char* authSvr, unsigned short authPort);
    ZCLOUD_SetServerInfo: Symbol<'a, unsafe extern "C" fn(http: *const c_char, port1: c_ushort, auth: *const c_char, port2: c_ushort)>,
    /// UINT FUNC_CALL ZCLOUD_ConnectServer(const char* username, const char* password); // return 0:success, 1:failure, 2:https error, 3:user login info error, 4:mqtt connection error, 5:no device
    ZCLOUD_ConnectServer: Symbol<'a, unsafe extern "C" fn(username: *const c_char, password: *const c_char) -> c_uint>,
    /// bool FUNC_CALL ZCLOUD_IsConnected();
    ZCLOUD_IsConnected: Symbol<'a, unsafe extern "C" fn() -> bool>,
    /// UINT FUNC_CALL ZCLOUD_DisconnectServer(); // return 0:success, 1:failure
    ZCLOUD_DisconnectServer: Symbol<'a, unsafe extern "C" fn() -> c_uint>,
    /// const ZCLOUD_USER_DATA* FUNC_CALL ZCLOUD_GetUserData(int update DEF(0));
    ZCLOUD_GetUserData: Symbol<'a, unsafe extern "C" fn(update: c_int) -> *const ZCloudUserData>,
    /// UINT FUNC_CALL ZCLOUD_ReceiveGPS(DEVICE_HANDLE device_handle, ZCLOUD_GPS_FRAME* pReceive, UINT len, int wait_time DEF(-1));
    ZCLOUD_ReceiveGPS: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, frames: *mut ZCloudGpsFrame, len: c_uint, timeout: c_uint) -> c_uint>,

    /// CHANNEL_HANDLE FUNC_CALL ZCAN_InitLIN(DEVICE_HANDLE device_handle, UINT can_index, PZCAN_LIN_INIT_CONFIG pLINInitConfig);
    ZCAN_InitLIN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZLinChlCfg) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_StartLIN(CHANNEL_HANDLE channel_handle);
    ZCAN_StartLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ResetLIN(CHANNEL_HANDLE channel_handle);
    ZCAN_ResetLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_TransmitLIN(CHANNEL_HANDLE channel_handle, PZCAN_LIN_MSG pSend, UINT Len);
    ZCAN_TransmitLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZLinFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_GetLINReceiveNum(CHANNEL_HANDLE channel_handle);
    ZCAN_GetLINReceiveNum: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveLIN(CHANNEL_HANDLE channel_handle, PZCAN_LIN_MSG pReceive, UINT Len,int WaitTime);
    ZCAN_ReceiveLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZLinFrame, len: c_uint, timeout: c_uint) -> c_uint>,
    // UINT FUNC_CALL ZCAN_ClearLINBuffer(CHANNEL_HANDLE channel_handle);
    // ZCAN_ClearLINBuffer: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINSlaveMsg(CHANNEL_HANDLE channel_handle, PZCAN_LIN_MSG pSend, UINT nMsgCount);
    ZCAN_SetLINSlaveMsg: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZLinFrame, size: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ClearLINSlaveMsg(CHANNEL_HANDLE channel_handle, BYTE* pLINID, UINT nIDCount);
    ZCAN_ClearLINSlaveMsg: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, ids: *const c_uchar, size: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_WakeUpLIN(CHANNEL_HANDLE channel_handle);
    ZCAN_WakeUpLIN: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINSubscribe(CHANNEL_HANDLE channel_handle, PZCAN_LIN_SUBSCIBE_CFG pSend, UINT nSubscribeCount);
    ZCAN_SetLINSubscribe: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, cfg: *const ZLinSubscribe, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINPublish(CHANNEL_HANDLE channel_handle, PZCAN_LIN_PUBLISH_CFG pSend, UINT nPublishCount);
    ZCAN_SetLINPublish: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, cfg: *const ZLinPublish, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_SetLINPublishEx(CHANNEL_HANDLE channel_handle, PZCAN_LIN_PUBLISH_CFG_EX pSend, UINT nPublishCount);
    ZCAN_SetLINPublishEx: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, cfg: *const ZLinPublishEx, len: c_uint) -> c_uint>,

    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_ControlEX(DEVICE_HANDLE device_handle, ZCAN_UDS_DATA_DEF dataType,
    //                                              const ZCAN_UDS_CTRL_REQ *ctrl, ZCAN_UDS_CTRL_RESP *resp);
    // ZCAN_UDS_ControlEX: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, data_type: ZCAN_UDS_DATA_DEF, ctrl: *const ZCAN_UDS_CTRL_REQ, resp: *mut ZCAN_UDS_CTRL_RESP) -> c_uint>,
    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_RequestEX(DEVICE_HANDLE device_handle, const ZCANUdsRequestDataObj *requestData,
    //                                              ZCAN_UDS_RESPONSE *resp, BYTE *dataBuf, UINT dataBufSize);
    // ZCAN_UDS_RequestEX: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, data: *const ZCANUdsRequestDataObj, resp: *mut ZCAN_UDS_CTRL_RESP, buff: *mut c_uchar, buff_size: c_uint) -> c_uint>,
    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_Control(DEVICE_HANDLE device_handle, const ZCAN_UDS_CTRL_REQ *ctrl, ZCAN_UDS_CTRL_RESP *resp);
    // ZCAN_UDS_Control: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, ctrl: *const ZCAN_UDS_CTRL_REQ, resp: *mut ZCAN_UDS_CTRL_RESP) -> c_uint>,
    // ZCAN_RET_STATUS FUNC_CALL ZCAN_UDS_Request(DEVICE_HANDLE device_handle, const ZCAN_UDS_REQUEST *req,
    //                                            ZCAN_UDS_RESPONSE *resp, BYTE *dataBuf, UINT dataBufSize);
    // ZCAN_UDS_Request: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, req: *const ZCAN_UDS_REQUEST, resp: *mut ZCAN_UDS_RESPONSE, buff: *mut c_uchar, buff_size: c_uint) -> c_uint>,
}

impl Api<'_> {
    const INVALID_DEVICE_HANDLE: u32 = 0;
    const INVALID_CHANNEL_HANDLE: u32 = 0;
    const STATUS_OK: u32 = 1;
}

impl ZDeviceApi for Api<'_> {
    type DeviceHandler = u32;
    type ChannelHandler = u32;
    fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            Self::INVALID_DEVICE_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_OpenDevice".to_string(), Self::INVALID_DEVICE_HANDLE)),
            v => Ok(v),
        }
    }
    fn close(&self, dev_hdl: DeviceHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_CloseDevice)(dev_hdl) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_CloseDevice".to_string(), code)),
        }
    }
    fn read_device_info(&self, dev_hdl: DeviceHandler) -> Result<ZDeviceInfo, ZCanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(dev_hdl, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_GetDeviceInf".to_string(), code)),
        }
    }
    fn is_online(&self, dev_hdl: DeviceHandler) -> Result<bool, ZCanError> {
        unsafe {
            match (self.ZCAN_IsDeviceOnLine)(dev_hdl) {
                STATUS_ONLINE => Ok(true),
                STATUS_OFFLINE => Ok(false),
                code => Err(ZCanError::MethodExecuteFailed("ZCAN_IsDeviceOnLine".to_string(), code)),
            }
        }
    }
    fn get_property(&self, dev_hdl: DeviceHandler) -> Result<IProperty, ZCanError> {
        unsafe {
            let ret = (self.GetIProperty)(dev_hdl);
            if ret.is_null() {
                Err(ZCanError::MethodExecuteFailed("GetIProperty".to_string(), 0))
            } else {
                Ok(*ret)
            }
        }
    }
    fn release_property(&self, p: &IProperty) -> Result<(), ZCanError> {
        unsafe {
            match (self.ReleaseIProperty)(p) {
                Self::STATUS_OK => Ok(()),
                code => Err(ZCanError::MethodExecuteFailed("ReleaseIProperty".to_string(), code)),
            }
        }
    }
    fn get_value(&self, dev_type: ZCanDeviceType, dev_hdl: DeviceHandler, cmd_path: &CmdPath) -> Result<*const c_void, ZCanError> {
        unsafe {
            let path = cmd_path.get_path();
            let path = CString::new(path).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
            if dev_type.get_value_support() {
                let ret = (self.ZCAN_GetValue)(dev_hdl, path.as_ptr() as *const c_char);
                if ret.is_null() {
                    Err(ZCanError::MethodExecuteFailed("ZCAN_GetValue".to_string(), 0))
                } else {
                    Ok(ret)
                }
            } else {
                Err(ZCanError::MethodNotSupported)
            }
        }
    }
    fn set_value(&self, dev_hdl: DeviceHandler, cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        unsafe {
            let path = cmd_path.get_path();
            let _path = CString::new(path).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
            // let _value = CString::new(value).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
            match (self.ZCAN_SetValue)(dev_hdl, _path.as_ptr() as *const c_char, value) {
                Self::STATUS_OK => Ok(()),
                code=> Err(ZCanError::MethodExecuteFailed("ZCAN_SetValue".to_string(), code)),
            }
        }
    }
    fn set_values(&self, dev_hdl: DeviceHandler, values: Vec<(CmdPath, *const c_char)>) -> Result<(), ZCanError> {
        unsafe {
            let p = self.get_property(dev_hdl)?;
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
    fn get_values(&self, dev_hdl: DeviceHandler, channel: u8, paths: Vec<CmdPath>) -> Result<Vec<String>, ZCanError> {
        unsafe {
            let p = self.get_property(dev_hdl)?;
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

impl ZCanApi for Api<'_> {
    type DeviceHandler = u32;
    type ChannelHandler = u32;
    type Frame = ZCanFrameV3;
    type FdFrame = ZCanFdFrameV2;
    fn init_can_chl(&self, dev_hdl: DeviceHandler, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        let dev_type = cfg.device_type()?;
        unsafe {
            if !matches!(dev_type, ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2) {
                // configure the clock
                if let Some(clock) = cfg.clock() {
                    let clock_path = CmdPath::new_path(CLOCK);
                    let value = CString::new(clock.to_string()).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                    self.set_value(dev_hdl, &clock_path, value.as_ptr() as *const c_void)?;
                }
                // set channel resistance status
                if dev_type.has_resistance() {
                    let state = (cfg.extra().resistance() as u32).to_string();
                    let resistance_path = format!("{}/{}", channel, INTERNAL_RESISTANCE);
                    let resistance_path = CmdPath::new_path(resistance_path.as_str());
                    let value = CString::new(state).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                    self.set_value(dev_hdl, &resistance_path, value.as_ptr() as *const c_void)?;
                }
                // set channel protocol
                let can_type = cfg.can_type()?;
                let protocol = (can_type as u32).to_string();
                let protocol_path = format!("{}/{}", channel, PROTOCOL);
                let protocol_path = CmdPath::new_path(protocol_path.as_str());
                let value = CString::new(protocol).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                self.set_value(dev_hdl, &protocol_path, value.as_ptr() as *const c_void)?;

                // set channel bitrate
                let bitrate = cfg.bitrate();
                if dev_type.canfd_support() {
                    let abitrate_path = format!("{}/{}", channel, CANFD_ABIT_BAUD_RATE);
                    let abitrate_path = CmdPath::new_path(abitrate_path.as_str());
                    let value = CString::new(bitrate.to_string()).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                    self.set_value(dev_hdl, &abitrate_path, value.as_ptr() as *const c_void)?;
                    match can_type {
                        ZCanChlType::CANFD_ISO | ZCanChlType::CANFD_NON_ISO => {
                            let dbitrate = cfg.extra().dbitrate().unwrap_or(bitrate).to_string();
                            let dbitrate_path = format!("{}/{}", channel, CANFD_DBIT_BAUD_RATE);
                            let dbitrate_path = CmdPath::new_path(dbitrate_path.as_str());
                            let value = CString::new(dbitrate).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                            self.set_value(dev_hdl, &dbitrate_path, value.as_ptr() as *const c_void)?;
                        },
                        _ => {},
                    }
                }
                else {
                    let bitrate_path = format!("{}/{}", channel, BAUD_RATE);
                    let bitrate_path = CmdPath::new_path(bitrate_path.as_str());
                    let value = CString::new(bitrate.to_string()).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                    self.set_value(dev_hdl, &bitrate_path, value.as_ptr() as *const c_void)?;
                }
            }

            let cfg = ZCanChlCfgV1::try_from(cfg)?;
            match (self.ZCAN_InitCAN)(dev_hdl, channel as u32, &cfg) {
                Self::INVALID_CHANNEL_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_InitCAN".to_string(), Self::INVALID_CHANNEL_HANDLE)),
                handler => match (self.ZCAN_StartCAN)(handler) {
                    Self::STATUS_OK => Ok(handler),
                    code => Err(ZCanError::MethodExecuteFailed("ZCAN_StartCAN".to_string(), code)),
                }
            }
        }
    }

    fn reset_can_chl(&self, chl_hdl: ChannelHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ResetCAN)(chl_hdl) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ResetCAN".to_string(), code)),
        }
    }

    fn read_can_chl_status(&self, chl_hdl: ChannelHandler) -> Result<ZCanChlStatus, ZCanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(chl_hdl, &mut status) } {
            Self::STATUS_OK => Ok(status),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelStatus".to_string(), code)),
        }
    }

    fn read_can_chl_error(&self, chl_hdl: ChannelHandler) -> Result<ZCanChlError, ZCanError> {
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(chl_hdl, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ReadChannelErrInfo".to_string(), code)),
        }
    }

    fn clear_can_buffer(&self, chl_hdl: ChannelHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(chl_hdl) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ClearBuffer".to_string(), code)),
        }
    }

    fn get_can_num(&self, chl_hdl: ChannelHandler, can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(chl_hdl, can_type as u8) };
        log::debug!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        Ok(ret)
    }

    fn receive_can(&self, chl_hdl: ChannelHandler, size: u32, timeout: u32, resize: impl Fn(&mut Vec<Self::Frame>, usize)) -> Result<Vec<Self::Frame>, ZCanError> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_Receive)(chl_hdl, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_can(&self, chl_hdl: ChannelHandler, frames: Vec<Self::Frame>) -> Result<u32, ZCanError> {
        let len = frames.len() as u32;
        // method 1
        // let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
        // if ret < len {
        //     log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        // }
        // Ok(ret)
        // method 2
        // let mut boxed_slice: Box<[ZCanFrame]> = frames.into_boxed_slice();
        // let array: *mut ZCanFrame = boxed_slice.as_mut_ptr();
        // // let ptr = frames.as_ptr();
        // let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, array, len) };
        // std::mem::forget(boxed_slice);
        // if ret < len {
        //     log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        // }
        // Ok(ret)
        // method 3: just do like this because of pointer offset TODO
        let mut count = 0;
        frames.iter().for_each(|frame| {
            let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frame, 1) };
            count += ret;
        });
        if count < len {
            log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, count);
        }
        Ok(count)
    }

    fn receive_canfd(&self, chl_hdl: ChannelHandler, size: u32, timeout: u32, resize: fn(&mut Vec<Self::FdFrame>, usize)) -> Result<Vec<Self::FdFrame>, ZCanError> {
        let mut frames = Vec::new();
        // frames.resize_with(size as usize, Default::default);
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_ReceiveFD)(chl_hdl, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive CANFD frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_canfd(&self, chl_hdl: ChannelHandler, frames: Vec<Self::FdFrame>) -> Result<u32, ZCanError> {
        let len = frames.len() as u32;
        // let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frames.as_ptr(), len) };
        // if ret < len {
        //     warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
        // }
        // Ok(ret)
        let mut count = 0;
        frames.iter().for_each(|frame| {
            let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frame, 1) };
            count += ret;
        });
        if count < len {
            log::warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, count);
        }
        Ok(count)
    }
}

impl ZLinApi for Api<'_> {
    type DeviceHandler = u32;
    type ChannelHandler = u32;
    fn init_lin_chl(&self, dev_hdl: DeviceHandler, channel: u8, cfg: &ZLinChlCfg) -> Result<u32, ZCanError> {
        unsafe {
            match (self.ZCAN_InitLIN)(dev_hdl, channel as u32, cfg) {
                Self::INVALID_CHANNEL_HANDLE => Err(ZCanError::MethodExecuteFailed("ZCAN_InitLIN".to_string(), Self::INVALID_CHANNEL_HANDLE)),
                handler => match (self.ZCAN_StartLIN)(dev_hdl) {
                    Self::STATUS_OK => Ok(handler),
                    code => Err(ZCanError::MethodExecuteFailed("ZCAN_StartLIN".to_string(), code)),
                }
            }
        }
    }
    fn reset_lin_chl(&self, chl_hdl: ChannelHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ResetLIN)(chl_hdl) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ResetLIN".to_string(), code)),
        }
    }
    fn get_lin_num(&self, chl_hdl: ChannelHandler) -> Result<u32, ZCanError> {
        let ret = unsafe { (self.ZCAN_GetLINReceiveNum)(chl_hdl) };
        log::debug!("ZLGCAN - get receive LIN number: {}.", ret);
        Ok(ret)
    }
    fn receive_lin(&self, chl_hdl: ChannelHandler, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZLinFrame>, usize)) -> Result<Vec<ZLinFrame>, ZCanError> {
        let mut frames = Vec::new();

        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_ReceiveLIN)(chl_hdl, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive LIN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }
    fn transmit_lin(&self, chl_hdl: ChannelHandler, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        let len = frames.len() as u32;
        let ret = unsafe { (self.ZCAN_TransmitLIN)(chl_hdl, frames.as_ptr(), len) };
        if ret < len {
            log::warn!("ZLGCAN - transmit LIN frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }
    fn set_lin_subscribe(&self, chl_hdl: ChannelHandler, cfg: Vec<ZLinSubscribe>) -> Result<(), ZCanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINSubscribe)(chl_hdl, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_SetLINSubscribe".to_string(), code)),
        }
    }
    fn set_lin_publish(&self, chl_hdl: ChannelHandler, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINPublish)(chl_hdl, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_SetLINPublish".to_string(), code)),
        }
    }
    fn wakeup_lin(&self, chl_hdl: ChannelHandler) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_WakeUpLIN)(chl_hdl) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_WakeUpLIN".to_string(), code)),
        }
    }
    fn set_lin_publish_ex(&self, chl_hdl: ChannelHandler, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.ZCAN_SetLINPublishEx)(chl_hdl, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_SetLINPublishEx".to_string(), code)),
        }
    }
    fn set_lin_slave_msg(&self, chl_hdl: ChannelHandler, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        let len = msg.len() as u32;
        match unsafe { (self.ZCAN_SetLINSlaveMsg)(chl_hdl, msg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_SetLINSlaveMsg".to_string(), code)),
        }
    }
    fn clear_lin_slave_msg(&self, chl_hdl: ChannelHandler, pids: Vec<u8>) -> Result<(), ZCanError> {
        let len = pids.len() as u32;
        match unsafe { (self.ZCAN_ClearLINSlaveMsg)(chl_hdl, pids.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("ZCAN_ClearLINSlaveMsg".to_string(), code)),
        }
    }
}

impl ZCloudApi<u32> for Api<'_> {
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        unsafe { (self.ZCLOUD_SetServerInfo)(server.http_url, server.http_port, server.mqtt_url, server.mqtt_port) }

        Ok(())
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError> {
        let username = CString::new(username).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
        let password = CString::new(password).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
        match unsafe { (self.ZCLOUD_ConnectServer)(username.as_ptr(), password.as_ptr()) } {
            Self::STATUS_OK => Ok(()),
            code=> Err(ZCanError::MethodExecuteFailed("ZCLOUD_ConnectServer".to_string(), code)),
        }
    }
    fn is_connected_server(&self) -> Result<bool, ZCanError> {
        unsafe { Ok((self.ZCLOUD_IsConnected)()) }
    }
    fn disconnect_server(&self) -> Result<(), ZCanError> {
        match unsafe { (self.ZCLOUD_DisconnectServer)() } {
            0 => Ok(()),
            code=> Err(ZCanError::MethodExecuteFailed("ZCLOUD_DisconnectServer".to_string(), code)),
        }
    }
    fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, ZCanError> {
        unsafe {
            let data = (self.ZCLOUD_GetUserData)(update);
            if data.is_null() {
                Err(ZCanError::MethodExecuteFailed("ZCLOUD_GetUserData".to_string(), 0))
            }
            else {
                Ok(*data)
            }
        }
    }
    fn receive_gps(&self, dev_hdl: DeviceHandler, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCloudGpsFrame>, usize)) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCLOUD_ReceiveGPS)(dev_hdl, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            log::warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }
}

#[cfg(test)]
mod tests {
    use dlopen2::symbor::{Library, SymBorApi};
    // use zlgcan_common::device::ZCanDeviceType;
    use super::Api;

    #[test]
    fn load_symbols() {
        // let dev_type = ZCanDeviceType::ZCAN_USBCAN1;

        let dll_path = "library/windows/x86_64/zlgcan.dll";
        let lib = Library::open(dll_path).expect("ZLGCAN - could not open library");

        let _ = unsafe { Api::load(&lib) }.expect("ZLGCAN - could not load symbols!");
    }
}

