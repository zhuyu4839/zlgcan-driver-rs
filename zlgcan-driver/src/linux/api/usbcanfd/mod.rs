// #![allow(non_upper_case_globals)]
// #![allow(non_camel_case_types)]
// #![allow(non_snake_case)]
// #![allow(unused)]
//
// #[cfg(not(debug_assertions))]
// include!(concat!(env!("OUT_DIR"), "/usbcanfd.rs"));
//
// #[cfg(debug_assertions)]
// include!("bindings/usbcanfd.rs");

// pub(crate) const STATUS_OK: u32 = 1;
use zlgcan_common as common;

use dlopen2::symbor::{Symbol, SymBorApi};
use std::ffi::{c_uint, c_void, CString};
use common::can::{
    ZCanChlCfgDetail, ZCanChlError, ZCanChlStatus,
    ZCanFdFrame, ZCanFrame
};
use common::device::{CmdPath, ZCanDeviceType, ZDeviceInfo};
use common::error::ZCanError;
use common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinSubscribe};
use crate::constant::STATUS_OK;

mod can;
mod lin;

#[allow(non_snake_case)]
#[derive(SymBorApi)]
pub(crate) struct USBCANFDApi<'a> {
    ///EXTERN_C U32 ZCAN_API VCI_OpenDevice(U32 Type, U32 Card, U32 Reserved);
    pub VCI_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, reserved: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_CloseDevice(U32 Type, U32 Card);
    pub VCI_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_InitCAN(U32 Type, U32 Card, U32 Port, ZCAN_INIT *pInit);
    pub VCI_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZCanChlCfgDetail) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReadBoardInfo(U32 Type, U32 Card, ZCAN_DEV_INF *pInfo);
    pub VCI_ReadBoardInfo: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, info: *mut ZDeviceInfo) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReadErrInfo(U32 Type, U32 Card, U32 Port, ZCAN_ERR_MSG *pErr);
    pub VCI_ReadErrInfo: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, err: *mut ZCanChlError) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReadCANStatus(U32 Type, U32 Card, U32 Port, ZCAN_STAT *pStat);
    pub VCI_ReadCANStatus: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, status: *mut ZCanChlStatus) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_GetReference(U32 Type, U32 Card, U32 Port, U32 Ref, void *pData);
    pub VCI_GetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cmd: c_uint, value: *mut c_void) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_SetReference(U32 Type, U32 Card, U32 Port, U32 Ref, void *pData);
    pub VCI_SetReference: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cmd: c_uint, value: *const c_void) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_GetReceiveNum(U32 Type, U32 Card, U32 Port);
    pub VCI_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ClearBuffer(U32 Type, U32 Card, U32 Port);
    pub VCI_ClearBuffer: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_StartCAN(U32 Type, U32 Card, U32 Port);
    pub VCI_StartCAN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ResetCAN(U32 Type, U32 Card, U32 Port);
    pub VCI_ResetCAN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_Transmit(U32 Type, U32 Card, U32 Port, ZCAN_20_MSG *pData, U32 Count);
    pub VCI_Transmit: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZCanFrame, len: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_TransmitFD(U32 Type, U32 Card, U32 Port, ZCAN_FD_MSG *pData, U32 Count);
    pub VCI_TransmitFD: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZCanFdFrame, len: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_Receive(U32 Type, U32 Card, U32 Port, ZCAN_20_MSG *pData, U32 Count, U32 Time);
    pub VCI_Receive: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZCanFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReceiveFD(U32 Type, U32 Card, U32 Port, ZCAN_FD_MSG *pData, U32 Count, U32 Time);
    pub VCI_ReceiveFD: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZCanFdFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_Debug(U32 Debug);
    pub VCI_Debug: Symbol<'a, unsafe extern "C" fn(debug: c_uint) -> c_uint>,

    /// UINT VCI_InitLIN(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_INIT_CONFIG pLINInitConfig);
    pub VCI_InitLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZLinChlCfg) -> c_uint>,
    /// UINT VCI_StartLIN(U32 Type, U32 Card, U32 LinChn);
    pub VCI_StartLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// UINT VCI_ResetLIN(U32 Type, U32 Card, U32 LinChn);
    pub VCI_ResetLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// UINT VCI_TransmitLIN(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_MSG pSend, U32 Len);
    pub VCI_TransmitLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZLinFrame, len: c_uint) -> c_uint>,
    /// UINT VCI_GetLINReceiveNum(U32 Type, U32 Card, U32 LinChn);
    pub VCI_GetLINReceiveNum: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// EXTERN_C U32 VCI_ClearLINBuffer(U32 Type, U32 Card, U32 LinChn);
    pub VCI_ClearLINBuffer: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint) -> c_uint>,
    /// UINT VCI_ReceiveLIN(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_MSG pReceive, U32 Len,int WaitTime);
    pub VCI_ReceiveLIN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZLinFrame, size: c_uint, timeout: c_uint) -> c_uint>,
    /// UINT  VCI_SetLINSubscribe(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_SUBSCIBE_CFG pSend, U32 nSubscribeCount);
    pub VCI_SetLINSubscribe: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZLinSubscribe, len: c_uint) -> c_uint>,
    /// UINT  VCI_SetLINPublish(U32 Type, U32 Card, U32 LinChn, PZCAN_LIN_PUBLISH_CFG pSend, U32 nPublishCount);
    pub VCI_SetLINPublish: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZLinPublish, len: c_uint) -> c_uint>,

    // EXTERN_C U32 VCI_TransmitData(unsigned Type, unsigned Card, unsigned Port, ZCANDataObj *pData, unsigned Count);
    // pub VCI_TransmitData: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, data: *const ZCANDataObj, len: c_uint) -> c_uint>,
    // EXTERN_C U32 VCI_ReceiveData(unsigned Type, unsigned Card, unsigned Port, ZCANDataObj *pData, unsigned Count, unsigned Time);
    // pub VCI_ReceiveData: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, data: *mut ZCANDataObj, size: c_uint, timeout: c_uint) -> c_uint>,

    // EXTERN_C U32 VCI_UDS_Request(unsigned Type, unsigned Card, const ZCAN_UDS_REQUEST *req, ZCAN_UDS_RESPONSE *resp, U8 *dataBuf, U32 dataBufSize);
    // pub VCI_UDS_Request: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, req: *const ZCAN_UDS_REQUEST, resp: *mut ZCAN_UDS_RESPONSE, buff: *mut c_uchar, buff_size: c_uint) -> c_uint>,
    // EXTERN_C U32 VCI_UDS_Control(unsigned Type, unsigned Card, const ZCAN_UDS_CTRL_REQ *ctrl, ZCAN_UDS_CTRL_RESP *resp);
    // pub VCI_UDS_Control: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, req: *const ZCAN_UDS_REQUEST, resp: *mut ZCAN_UDS_RESPONSE) -> c_uint>,
}

impl USBCANFDApi<'_> {
    #[inline(always)]
    pub(crate) fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.VCI_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            STATUS_OK => Ok(1),
            code => Err(ZCanError::new(code, format!("ZLGCAN - {} open failed", dev_type))),
        }
    }
    #[inline(always)]
    pub(crate) fn close(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<(), ZCanError> {
        match unsafe { (self.VCI_CloseDevice)(dev_type as u32, dev_idx) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - {} close failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn read_device_info(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<ZDeviceInfo, ZCanError> {
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.VCI_ReadBoardInfo)(dev_type as u32, dev_idx, &mut info) } {
            STATUS_OK => Ok(info),
            code => Err(ZCanError::new(code, "ZLGCAN - read device info failed".to_string())),
        }
    }
    // #[inline(always)]
    // pub(crate) fn is_online(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<bool, ZCanError> {
    //     Err(ZCanError::new(0, format!("ZLGCAN - method not supported by device: {}_{}", dev_type, dev_idx)))
    // }
    #[inline(always)]
    pub(self) fn set_reference(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        let cmd = cmd_path.get_reference();
        // let _value = CString::new(value).expect("ZLGCAN - couldn't convert to CString!");
        match unsafe { (self.VCI_SetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, format!("ZLGCAN - set reference for channel: {} failed", channel))),
        }
    }
    #[inline(always)]
    pub(self) fn get_reference(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cmd_path: &CmdPath, value: *mut c_void) -> Result<(), ZCanError> {
        let cmd = cmd_path.get_reference();
        match unsafe { (self.VCI_GetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, format!("ZLGCAN - get reference for channel: {} failed", channel))),
        }
    }
    #[allow(dead_code)]
    pub(self) fn set_check_value(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cmd_path: &CmdPath, value: &str) -> Result<(), ZCanError> {
        let _value = CString::new(value).expect("ZLGCAN - convert value to C string failed!");

        self.set_reference(dev_type, dev_idx, channel, cmd_path, _value.as_ptr() as *const c_void)?;
        if dev_type.get_value_support() {
            let mut ret: Vec<u8> = Vec::new();
            ret.resize(16, 0);
            match self.get_reference(dev_type, dev_idx, channel, cmd_path, ret.as_mut_ptr() as *mut c_void)  {
                Ok(()) => {
                    let ret = String::from_iter(ret.iter().take_while(|c| **c != 0).map(|c| *c as char));
                    if value == ret.as_str() {
                        Ok(())
                    }
                    else {
                        Err(ZCanError::new(0, format!("ZLGCAN - set value: {} checked failed", value)))
                    }
                },
                Err(e) => Err(e),
            }
        }
        else {
            Ok(())
        }
    }
    #[allow(dead_code)]
    pub(crate) fn debug(&self, level: u32) -> Result<(), ZCanError> {
        unsafe {
            match (self.VCI_Debug)(level) {
                STATUS_OK => Ok(()),
                code => Err(ZCanError::new(code, format!("ZLGCAN - set debug level: {} failed", level))),
            }
        }
    }

}

#[cfg(test)]
mod test {
    use zlgcan_common as common;

    use dlopen2::symbor::{Library, SymBorApi};
    use common::can::{
        CanChlCfg,
        ZCanChlMode, ZCanChlType,
        ZCanFrame, ZCanFrameV1,
        CanMessage
    };
    use common::device::ZCanDeviceType;
    use zlgcan_common::can::CanChlCfgFactory;
    use crate::ZCanDriver;
    use super::USBCANFDApi;

    #[test]
    fn test_init_channel() {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let dev_idx = 0;
        let channel = 0;
        let channels = 2;

        let so_path = "library/linux/x86_64/libusbcanfd.so";
        let lib = Library::open(so_path).expect("ZLGCAN - could not open library");

        let api = unsafe { USBCANFDApi::load(&lib) }.expect("ZLGCAN - could not load symbols!");
        let factory = CanChlCfgFactory::new();

        let cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CAN, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
        api.open(dev_type, dev_idx).unwrap();

        let dev_info = api.read_device_info(dev_type, dev_idx).unwrap();
        println!("{:?}", dev_info);
        println!("{}", dev_info.id());
        println!("{}", dev_info.sn());
        println!("{}", dev_info.hardware_version());
        println!("{}", dev_info.firmware_version());
        println!("{}", dev_info.driver_version());
        println!("{}", dev_info.api_version());
        assert_eq!(dev_info.can_channels(), channels);
        assert!(dev_info.canfd());

        api.init_can_chl(dev_type, dev_idx, 0, &cfg).unwrap();
        let frame = CanMessage::new(0x7E0, Some(0), [0x01, 0x02, 0x03], false, false, None).unwrap();
        let frame1 = CanMessage::new(0x1888FF00, Some(0), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08], false, false, None).unwrap();
        let frames = vec![ZCanFrame::from(ZCanFrameV1::from(frame)), ZCanFrame::from(ZCanFrameV1::from(frame1))];
        let ret = api.transmit_can(dev_type, dev_idx, channel, frames);
        assert_eq!(ret, 2);

        api.reset_can_chl(dev_type, dev_idx, channel).unwrap();

        api.close(dev_type, dev_idx).unwrap();
    }
}

