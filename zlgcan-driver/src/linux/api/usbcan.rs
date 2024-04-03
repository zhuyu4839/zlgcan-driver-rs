// #![allow(non_upper_case_globals)]
// #![allow(non_camel_case_types)]
// #![allow(non_snake_case)]
// #![allow(unused)]
//
// #[cfg(not(debug_assertions))]
// include!(concat!(env!("OUT_DIR"), "/usbcan.rs"));
//
// #[cfg(debug_assertions)]
// include!("bindings/usbcan.rs");
use zlgcan_common as common;

use std::ffi::{c_void, CString};
use dlopen2::symbor::{Symbol, SymBorApi};
use log::{debug, warn};
use common::can::{
    CanChlCfg,
    ZCanChlCfgDetail, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus,
    ZCanFrameType,
    ZCanFrame
};
use common::device::{CmdPath, ZCanDeviceType, ZDeviceInfo};
use common::error::ZCanError;
use crate::constant::STATUS_OK;

#[allow(non_snake_case)]
#[derive(SymBorApi)]
pub(crate) struct USBCANApi<'a> {
    /// EXTERN_C DWORD VCI_OpenDevice(DWORD DeviceType,DWORD DeviceInd,DWORD Reserved);
    VCI_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, reserved: u32) -> u32>,
    ///EXTERN_C DWORD VCI_CloseDevice(DWORD DeviceType,DWORD DeviceInd);
    VCI_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32) -> u32>,
    /// EXTERN_C DWORD VCI_InitCAN(DWORD DeviceType, DWORD DeviceInd, DWORD CANInd, PVCI_INIT_CONFIG pInitConfig);
    VCI_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cfg: *const ZCanChlCfgDetail) -> u32>,

    /// EXTERN_C DWORD VCI_ReadBoardInfo(DWORD DeviceType,DWORD DeviceInd,PVCI_BOARD_INFO pInfo);
    VCI_ReadBoardInfo: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, info: *mut ZDeviceInfo) -> u32>,
    /// EXTERN_C DWORD VCI_ReadErrInfo(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_ERR_INFO pErrInfo);
    VCI_ReadErrInfo: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, err: *mut ZCanChlError) -> u32>,
    /// EXTERN_C DWORD VCI_ReadCANStatus(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_STATUS pCANStatus);
    VCI_ReadCANStatus: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, status: *mut ZCanChlStatus) -> u32>,
    /// EXTERN_C DWORD VCI_GetReference(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,DWORD RefType,PVOID pData);
    VCI_GetReference: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cmd: u32, value: *mut c_void) -> u32>,
    /// EXTERN_C DWORD VCI_SetReference(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,DWORD RefType,PVOID pData);
    VCI_SetReference: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cmd: u32, value: *const c_void) -> u32>,
    /// EXTERN_C ULONG VCI_GetReceiveNum(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_GetReceiveNum: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_ClearBuffer(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_ClearBuffer: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_StartCAN(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_StartCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C DWORD VCI_ResetCAN(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd);
    VCI_ResetCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32) -> u32>,
    /// EXTERN_C ULONG VCI_Transmit(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_OBJ pSend,UINT Len);
    VCI_Transmit: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *const ZCanFrame, len: u32) -> u32>,
    /// EXTERN_C ULONG VCI_Receive(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_OBJ pReceive,UINT Len,INT WaitTime);
    VCI_Receive: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *mut ZCanFrame, size: u32, timeout: u32) -> u32>,
}

impl USBCANApi<'_> {
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
    pub(crate) fn init_can_chl(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        unsafe {
            let dev_type = dev_type as u32;
            let channel = channel as u32;
            let cfg = ZCanChlCfgDetail::from(cfg);
            match (self.VCI_InitCAN)(dev_type, dev_idx, channel, &cfg) {
                STATUS_OK => {
                    match (self.VCI_StartCAN)(dev_type, dev_idx, channel) {
                        STATUS_OK => Ok(0),
                        code => Err(ZCanError::new(code, format!("ZLGCAN - `StartCAN` channel: {} failed", channel))),
                    }
                },
                code => Err(ZCanError::new(code, format!("ZLGCAN - `InitCAN` channel: {} failed", channel))),
            }
        }
    }
    #[inline(always)]
    pub(crate) fn reset_can_chl(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        match unsafe { (self.VCI_ResetCAN)(dev_type as u32, dev_idx, channel as u32) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - CAN channel reset failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn read_can_chl_status(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<ZCanChlStatus, ZCanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.VCI_ReadCANStatus)(dev_type as u32, dev_idx, channel as u32, &mut status) } {
            STATUS_OK => Ok(status),
            code =>Err(ZCanError::new(code, "ZLGCAN - read CAN channel status failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn read_can_chl_error(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<ZCanChlError, ZCanError> {
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.VCI_ReadErrInfo)(dev_type as u32, dev_idx, channel as u32, &mut info) } {
            STATUS_OK => Ok(info),
            code =>Err(ZCanError::new(code, "ZLGCAN - read CAN channel error info failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn clear_can_buffer(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        match unsafe { (self.VCI_ClearBuffer)(dev_type as u32, dev_idx, channel as u32) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - clear CAN channel's buffer failed".to_string())),
        }
    }
    #[inline(always)]
    pub(crate) fn get_can_num(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, msg: ZCanFrameType) -> u32 {
        let mut _channel = channel as u32;
        match msg {
            ZCanFrameType::CAN => {},
            ZCanFrameType::CANFD => _channel |= 0x8000_0000,
            ZCanFrameType::ALL => panic!("ZLGCAN - CAN receive numbers is not supported!"),
        }
        let ret = unsafe { (self.VCI_GetReceiveNum)(dev_type as u32, dev_idx, _channel) };
        debug!("ZLGCAN - get receive {} number: {}.", msg, ret);
        ret
    }
    #[inline(always)]
    pub(crate) fn receive_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>, resize: impl Fn(&mut Vec<ZCanFrame>, usize)) -> Vec<ZCanFrame> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_Receive)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout.unwrap_or(50)) };
        if ret < size {
            warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        frames
    }
    #[inline(always)]
    pub(crate) fn transmit_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFrame>) -> u32 {
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_Transmit)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        ret
    }
}

#[allow(dead_code)]
impl USBCANApi<'_> {
    #[inline(always)]
    pub(self) fn set_reference(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cmd_path: CmdPath, value: &str) -> Result<(), ZCanError> {
        let cmd = cmd_path.get_reference();
        let _value = CString::new(value).expect("ZLGCAN - couldn't convert to CString!");
        match unsafe { (self.VCI_SetReference)(dev_type as u32, dev_idx, channel as u32, cmd, _value.as_ptr() as *const c_void) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, format!("ZLGCAN - set reference for channel: {} failed", channel))),
        }
    }
    #[inline(always)]
    pub(self) fn get_reference<T>(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cmd_path: CmdPath, value: *mut c_void) -> Result<(), ZCanError> {
        let cmd = cmd_path.get_reference();
        match unsafe { (self.VCI_GetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, format!("ZLGCAN - get reference for channel: {} failed", channel))),
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
    use super::USBCANApi;

    #[test]
    fn test_init_channel() {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
        let dev_idx = 0;
        let channel = 0;

        let so_path = "library/linux/x86_64/libusbcan.so";
        let lib = Library::open(so_path).expect("ZLGCAN - could not open library");

        let api = unsafe { USBCANApi::load(&lib) }.expect("ZLGCAN - could not load symbols!");

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
        assert_eq!(dev_info.can_channels(), 1);
        assert!(!dev_info.canfd());

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

