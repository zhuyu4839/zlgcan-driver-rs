// #![allow(non_upper_case_globals)]
// #![allow(non_camel_case_types)]
// #![allow(non_snake_case)]
// #![allow(unused)]
//
// #[cfg(not(debug_assertions))]
// include!(concat!(env!("OUT_DIR"), "/zlgcan.rs"));
//
// #[cfg(debug_assertions)]
// include!("bindings/zlgcan.rs");
use zlgcan_common as common;

use std::ffi::{c_char, c_int, c_uchar, c_uint, c_ushort, c_void, CString};
use log::warn;
use dlopen2::symbor::{Symbol, SymBorApi};
use common::can::channel::{ZCanChlCfgDetail, ZCanChlError, ZCanChlStatus};
use common::can::frame::{ZCanFdFrame, ZCanFrame};
use common::cloud::{ZCloudGpsFrame, ZCloudUserData};
use common::device::{CmdPath, IProperty, ZCanDeviceType, ZDeviceInfo};
use common::error::ZCanError;
use common::lin::channel::ZLinChlCfg;
use common::lin::frame::{ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};
use common::utils::c_str_to_string;

use crate::constant::{STATUS_OK, INVALID_DEVICE_HANDLE, STATUS_OFFLINE, STATUS_ONLINE};

mod can;
mod cloud;
mod lin;

#[allow(non_snake_case)]
#[derive(SymBorApi)]
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
    ZCAN_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_hdl: c_uint, channel: c_uint, cfg: *const ZCanChlCfgDetail) -> c_uint>,
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
    ZCAN_Transmit: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_Receive(CHANNEL_HANDLE channel_handle, ZCAN_Receive_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_Receive: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFrame, len: c_uint, timeout: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_TransmitFD(CHANNEL_HANDLE channel_handle, ZCAN_TransmitFD_Data* pTransmit, UINT len);
    ZCAN_TransmitFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *const ZCanFdFrame, len: c_uint) -> c_uint>,
    /// UINT FUNC_CALL ZCAN_ReceiveFD(CHANNEL_HANDLE channel_handle, ZCAN_ReceiveFD_Data* pReceive, UINT len, int wait_time DEF(-1));
    ZCAN_ReceiveFD: Symbol<'a, unsafe extern "C" fn(chl_hdl: c_uint, frames: *mut ZCanFdFrame, len: c_uint, timeout: c_uint) -> c_uint>,

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
    #[inline(always)]
    pub(crate) fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.ZCAN_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            INVALID_DEVICE_HANDLE => Err(ZCanError::new(0, format!("ZLGCAN - {} open failed", dev_type))),
            v => Ok(v),
        }
    }

    #[inline(always)]
    pub(crate) fn close(&self, dev_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_CloseDevice)(dev_hdl) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - device close failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn read_device_info(&self, dev_hdl: u32) -> Result<ZDeviceInfo, ZCanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.ZCAN_GetDeviceInf)(dev_hdl, &mut info) } {
            STATUS_OK => Ok(info),
            code => Err(ZCanError::new(code, "ZLGCAN - read device info failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn is_online(&self, dev_hdl: u32) -> Result<bool, ZCanError> {
        match unsafe { (self.ZCAN_IsDeviceOnLine)(dev_hdl) } {
            STATUS_ONLINE => Ok(true),
            STATUS_OFFLINE => Ok(false),
            code => Err(ZCanError::new(code, "ZLGCAN - unknown code device online check".to_string())),
        }
    }
}

#[allow(dead_code)]
impl Api<'_> {

    #[inline(always)]
    pub(self) unsafe fn get_property(&self, dev_hdl: u32) -> Option<IProperty> {
        let ret = (self.GetIProperty)(dev_hdl);
        if ret.is_null() {
            None
        }
        else {
            Some(*ret)
        }
    }

    #[inline(always)]
    pub(self) unsafe fn release_property(&self, p: IProperty) -> Result<(), ZCanError> {
        match (self.ReleaseIProperty)(&p) {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - release property failed".to_string())),
        }
    }

    pub fn set_check_value(&self, dev_hdl: u32, cmd_path: &CmdPath, value: &str, dev_type: ZCanDeviceType) -> Result<(), ZCanError> {
        unsafe {
            let path = cmd_path.get_path();
            let _path = CString::new(path).expect("ZLGCAN - convert path to C string failed!");
            let _value = CString::new(value).expect("ZLGCAN - convert value to C string failed!");
            match (self.ZCAN_SetValue)(dev_hdl, _path.as_ptr() as *const c_char, _value.as_ptr() as *const c_void) {
                STATUS_OK => {
                    if dev_type.get_value_support() {
                        let ret = (self.ZCAN_GetValue)(dev_hdl, _path.as_ptr() as *const c_char);
                        let ret = c_str_to_string(ret as *const c_char);
                        match ret {
                            Some(v) => {
                                if v.as_str() == value {
                                    Ok(())
                                }
                                else {
                                    Err(ZCanError::new(0, format!("ZLGCAN - set `{}` value: {} checked failed", path, value)))
                                }
                            },
                            None => Err(ZCanError::new(0, format!("ZLGCAN - set `{}` value: {} get returned value failed", path, value))),
                        }
                    }
                    else {
                        Ok(())
                    }
                },
                code=> Err(ZCanError::new(code, format!("ZLGCAN - set `{}` value: {} failed", path, value))),
            }
        }
    }

    #[allow(dead_code)]
    pub(self) unsafe fn set_values(&self, dev_hdl: u32, values: Vec<(CmdPath, &str)>) -> Result<(), ZCanError> {
        match self.get_property(dev_hdl) {
            Some(p) => {
                match p.SetValue {
                    Some(f) => {
                        values.iter().for_each(|(cmd, value)| {
                            let path = cmd.get_path();
                            // let _path = format!("{}/{}", path, channel);
                            let _path = CString::new(path).expect("ZLGCAN - couldn't convert to CString!");
                            let _value = value.to_string();
                            let _value = CString::new(_value).expect("ZLGCAN - couldn't convert to CString!");
                            match f(_path.as_ptr(), _value.as_ptr()) {
                                1 => (),
                                _ => warn!("ZLGCAN - set `{}` value: {} failed!", path, *value),
                            }
                        });

                        let _ = self.release_property(p).is_err_and(|e| -> bool {
                            warn!("{}", e);
                            true
                        });
                        Ok(())
                    },
                    None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not supported", "set_value"))),
                }
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not supported", "get_property"))),
        }
    }
    pub(self) unsafe fn get_values(&self, dev_hdl: u32, channel: u8, paths: Vec<CmdPath>) -> Result<Vec<String>, ZCanError> {
        match self.get_property(dev_hdl) {
            Some(p) => {
                match p.GetValue {
                    Some(f) => {
                        let mut result = Vec::new();
                        paths.iter().for_each(|s| {
                            let path = s.get_path();
                            let _path = CString::new(format!("{}/{}", path, channel)).expect("ZLGCAN - couldn't convert to CString!");
                            let ret = f(_path.as_ptr());
                            match c_str_to_string(ret) {
                                Some(v) => result.push(v),
                                None => warn!("ZLGCAN - get value failed!"),
                            }
                        });

                        let _ = self.release_property(p).is_err_and(|e| -> bool {
                            warn!("{}", e);
                            true
                        });

                        Ok(result)
                    },
                    None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not supported", "get_value"))),
                }
            }
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not supported", "get_property"))),
        }
    }
}

#[cfg(test)]
mod test {
    use dlopen2::symbor::{Library, SymBorApi};
    use common::device::ZCanDeviceType;
    use crate::windows::api::Api;

    #[test]
    fn load_symbols() {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN1;

        let dll_path = "library/windows/x86_64/zlgcan.dll";
        let lib = Library::open(dll_path).expect("ZLGCAN - could not open library");

        let _ = unsafe { Api::load(&lib) }.expect("ZLGCAN - could not load symbols!");
    }

}

