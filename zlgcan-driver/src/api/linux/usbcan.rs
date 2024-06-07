use std::ffi::c_void;
use dlopen2::symbor::{Symbol, SymBorApi};
use log::{debug, warn};
use zlgcan_common::can::{CanChlCfg, ZCanChlCfgV2, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus, ZCanFrameType, ZCanFrameV1};
use zlgcan_common::device::{CmdPath, ZCanDeviceType, ZDeviceInfo};
use zlgcan_common::error::ZCanError;
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};

#[allow(non_snake_case)]
#[derive(Debug, SymBorApi)]
pub(crate) struct USBCANApi<'a> {
    /// EXTERN_C DWORD VCI_OpenDevice(DWORD DeviceType,DWORD DeviceInd,DWORD Reserved);
    VCI_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, reserved: u32) -> u32>,
    ///EXTERN_C DWORD VCI_CloseDevice(DWORD DeviceType,DWORD DeviceInd);
    VCI_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32) -> u32>,
    /// EXTERN_C DWORD VCI_InitCAN(DWORD DeviceType, DWORD DeviceInd, DWORD CANInd, PVCI_INIT_CONFIG pInitConfig);
    VCI_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, cfg: *const ZCanChlCfgV2) -> u32>,

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
    VCI_Transmit: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *const ZCanFrameV1, len: u32) -> u32>,
    /// EXTERN_C ULONG VCI_Receive(DWORD DeviceType,DWORD DeviceInd,DWORD CANInd,PVCI_CAN_OBJ pReceive,UINT Len,INT WaitTime);
    VCI_Receive: Symbol<'a, unsafe extern "C" fn(dev_type: u32, dev_index: u32, channel: u32, frames: *mut ZCanFrameV1, size: u32, timeout: u32) -> u32>,
}

impl USBCANApi<'_> {
    // const INVALID_DEVICE_HANDLE: u32 = 0;
    // const INVALID_CHANNEL_HANDLE: u32 = 0;
    const STATUS_OK: u32 = 1;
}

impl ZDeviceApi<(ZCanDeviceType, u32), (ZCanDeviceType, u32, u8)> for USBCANApi<'_> {
    fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.VCI_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            Self::STATUS_OK => Ok(Self::STATUS_OK),
            code => Err(ZCanError::MethodExecuteFailed("VCI_OpenDevice".to_string(), code)),
        }
    }

    fn close(&self, dev_hdl: (ZCanDeviceType, u32)) -> Result<(), ZCanError> {
        let (dev_type, dev_idx) = dev_hdl;
        match unsafe { (self.VCI_CloseDevice)(dev_type as u32, dev_idx) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_CloseDevice".to_string(), code)),
        }
    }

    fn read_device_info(&self, dev_hdl: (ZCanDeviceType, u32)) -> Result<ZDeviceInfo, ZCanError> {
        let (dev_type, dev_idx) = dev_hdl;
        let mut info = ZDeviceInfo::default();
        match unsafe { (self.VCI_ReadBoardInfo)(dev_type as u32, dev_idx, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("VCI_ReadBoardInfo".to_string(), code)),
        }
    }

    fn set_reference(&self, chl_hdl: (ZCanDeviceType, u32, u8), cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let cmd = cmd_path.get_reference();
        // let _value = CString::new(value).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
        match unsafe { (self.VCI_SetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_SetReference".to_string(), code)),
        }
    }

    fn get_reference(&self, chl_hdl: (ZCanDeviceType, u32, u8), cmd_path: &CmdPath, value: *mut c_void) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let cmd = cmd_path.get_reference();
        match unsafe { (self.VCI_GetReference)(dev_type as u32, dev_idx, channel as u32, cmd, value) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_GetReference".to_string(), code)),
        }
    }
}

impl ZCanApi<(ZCanDeviceType, u32), (ZCanDeviceType, u32, u8), ZCanFrameV1, ()> for USBCANApi<'_> {
    fn init_can_chl(&self, dev_hdl: (ZCanDeviceType, u32), channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx) = dev_hdl;
        unsafe {
            let dev_type = dev_type as u32;
            let channel = channel as u32;
            let cfg = ZCanChlCfgV2::try_from(cfg)?;
            match (self.VCI_InitCAN)(dev_type, dev_idx, channel, &cfg) {
                Self::STATUS_OK => {
                    match (self.VCI_StartCAN)(dev_type, dev_idx, channel) {
                        Self::STATUS_OK => Ok(0),
                        code => Err(ZCanError::MethodExecuteFailed("VCI_StartCAN".to_string(), code)),
                    }
                },
                code => Err(ZCanError::MethodExecuteFailed("VCI_InitCAN".to_string(), code)),
            }
        }
    }

    fn reset_can_chl(&self, chl_hdl: (ZCanDeviceType, u32, u8)) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        match unsafe { (self.VCI_ResetCAN)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_ResetCAN".to_string(), code)),
        }
    }

    fn read_can_chl_status(&self, chl_hdl: (ZCanDeviceType, u32, u8)) -> Result<ZCanChlStatus, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.VCI_ReadCANStatus)(dev_type as u32, dev_idx, channel as u32, &mut status) } {
            Self::STATUS_OK => Ok(status),
            code => Err(ZCanError::MethodExecuteFailed("VCI_ReadCANStatus".to_string(), code)),
        }
    }

    fn read_can_chl_error(&self, chl_hdl: (ZCanDeviceType, u32, u8)) -> Result<ZCanChlError, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let mut info: ZCanChlError = ZCanChlError::from(ZCanChlErrorV2::default());
        match unsafe { (self.VCI_ReadErrInfo)(dev_type as u32, dev_idx, channel as u32, &mut info) } {
            Self::STATUS_OK => Ok(info),
            code => Err(ZCanError::MethodExecuteFailed("VCI_ReadErrInfo".to_string(), code)),
        }
    }

    fn clear_can_buffer(&self, chl_hdl: (ZCanDeviceType, u32, u8)) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        match unsafe { (self.VCI_ClearBuffer)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_ClearBuffer".to_string(), code)),
        }
    }

    fn get_can_num(&self, chl_hdl: (ZCanDeviceType, u32, u8), can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let mut _channel = channel as u32;
        match can_type {
            ZCanFrameType::CAN => {},
            ZCanFrameType::CANFD => _channel |= 0x8000_0000,
            ZCanFrameType::ALL => return Err(ZCanError::MethodNotSupported),
        }
        let ret = unsafe { (self.VCI_GetReceiveNum)(dev_type as u32, dev_idx, _channel) };
        debug!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        Ok(ret)
    }

    fn receive_can(&self, chl_hdl: (ZCanDeviceType, u32, u8), size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCanFrameV1>, usize)) -> Result<Vec<ZCanFrameV1>, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_Receive)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_can(&self, chl_hdl: (ZCanDeviceType, u32, u8), frames: Vec<ZCanFrameV1>) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_Transmit)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }
}

impl<DH, CH> ZLinApi<DH, CH> for USBCANApi<'_> {}
impl<DH> ZCloudApi<DH> for USBCANApi<'_> {}

#[cfg(test)]
mod tests {
    use dlopen2::symbor::{Library, SymBorApi};
    use zlgcan_common::can::{
        ZCanChlMode, ZCanChlType,
        ZCanFrameV1,
        CanMessage, CanChlCfgFactory
    };
    use zlgcan_common::device::ZCanDeviceType;
    use super::USBCANApi;
    use crate::api::{ZCanApi, ZDeviceApi};

    #[test]
    fn test_init_channel() {
        let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
        let dev_idx = 0;
        let channel = 0;

        let so_path = "library/linux/x86_64/libusbcan.so";
        let lib = Library::open(so_path).expect("ZLGCAN - could not open library");

        let api = unsafe { USBCANApi::load(&lib) }.expect("ZLGCAN - could not load symbols!");

        let factory = CanChlCfgFactory::new().unwrap();
        let cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CAN as u8, ZCanChlMode::Normal as u8, 500_000, Default::default()).unwrap();
        api.open(dev_type, dev_idx).unwrap();

        let dev_info = api.read_device_info((dev_type, dev_idx)).unwrap();
        println!("{:?}", dev_info);
        println!("{}", dev_info.id());
        println!("{}", dev_info.sn());
        println!("{}", dev_info.hardware_version());
        println!("{}", dev_info.firmware_version());
        println!("{}", dev_info.driver_version());
        println!("{}", dev_info.api_version());
        assert_eq!(dev_info.can_channels(), 1);
        assert!(!dev_info.canfd());

        api.init_can_chl((dev_type, dev_idx), 0, &cfg).unwrap();
        let frame = CanMessage::new(0x7E0, Some(0), [0x01, 0x02, 0x03], false, false, None).unwrap();
        let frame1 = CanMessage::new(0x1888FF00, Some(0), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08], false, false, None).unwrap();
        let frames = vec![
            ZCanFrameV1::try_from(frame).unwrap(),
            ZCanFrameV1::try_from(frame1).unwrap()
        ];
        let ret = api.transmit_can((dev_type, dev_idx, channel), frames).unwrap();
        assert_eq!(ret, 2);

        api.reset_can_chl((dev_type, dev_idx, channel)).unwrap();

        api.close((dev_type, dev_idx)).unwrap();
    }
}

