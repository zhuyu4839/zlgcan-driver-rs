use dlopen2::symbor::{Symbol, SymBorApi};
use std::ffi::{c_uint, c_void, CString};
use log::{debug, warn};
use zlgcan_common::can::{CanChlCfg, Reference, ZCanChlErrorV2, ZCanFrameType, ZCanChlError, ZCanChlStatus, ZCanFrameV2, ZCanFdFrameV1, ZCanChlCfgV2};
use zlgcan_common::device::{CmdPath, ZCanDeviceType, ZDeviceInfo};
use zlgcan_common::error::ZCanError;
use zlgcan_common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinSubscribe};
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};

#[allow(non_snake_case)]
#[derive(Debug, SymBorApi)]
pub(crate) struct USBCANFDApi<'a> {
    ///EXTERN_C U32 ZCAN_API VCI_OpenDevice(U32 Type, U32 Card, U32 Reserved);
    pub VCI_OpenDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, reserved: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_CloseDevice(U32 Type, U32 Card);
    pub VCI_CloseDevice: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_InitCAN(U32 Type, U32 Card, U32 Port, ZCAN_INIT *pInit);
    pub VCI_InitCAN: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, cfg: *const ZCanChlCfgV2) -> c_uint>,
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
    pub VCI_Transmit: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZCanFrameV2, len: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_TransmitFD(U32 Type, U32 Card, U32 Port, ZCAN_FD_MSG *pData, U32 Count);
    pub VCI_TransmitFD: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *const ZCanFdFrameV1, len: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_Receive(U32 Type, U32 Card, U32 Port, ZCAN_20_MSG *pData, U32 Count, U32 Time);
    pub VCI_Receive: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZCanFrameV2, size: c_uint, timeout: c_uint) -> c_uint>,
    /// EXTERN_C U32 ZCAN_API VCI_ReceiveFD(U32 Type, U32 Card, U32 Port, ZCAN_FD_MSG *pData, U32 Count, U32 Time);
    pub VCI_ReceiveFD: Symbol<'a, unsafe extern "C" fn(dev_type: c_uint, dev_idx: c_uint, channel: c_uint, frames: *mut ZCanFdFrameV1, size: c_uint, timeout: c_uint) -> c_uint>,
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
    // const INVALID_DEVICE_HANDLE: u32 = 0;
    // const INVALID_CHANNEL_HANDLE: u32 = 0;
    const STATUS_OK: u32 = 1;
}

impl ZDeviceApi<(ZCanDeviceType, u32), (ZCanDeviceType, u32, u8)> for USBCANFDApi<'_> {
    fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError> {
        match unsafe { (self.VCI_OpenDevice)(dev_type as u32, dev_idx, 0) } {
            Self::STATUS_OK => Ok(1),
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

    fn set_value(&self, chl_hdl: (ZCanDeviceType, u32, u8), cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        self.set_reference(chl_hdl, cmd_path, value)
    }

    fn get_value(&self, dev_type: ZCanDeviceType, chl_hdl: (ZCanDeviceType, u32, u8), cmd_path: &CmdPath) -> Result<*const c_void, ZCanError> {
        if dev_type.get_value_support() {
            let mut ret: Vec<u8> = Vec::new();
            ret.resize(16, 0);
            self.get_reference(chl_hdl, cmd_path, ret.as_mut_ptr() as *mut c_void)?;
            Ok(ret.as_ptr() as *const c_void)
        }
        else {
            Err(ZCanError::MethodNotSupported)
        }
    }

    fn debug(&self, level: u32) -> Result<(), ZCanError> {
        unsafe {
            match (self.VCI_Debug)(level) {
                Self::STATUS_OK => Ok(()),
                code => Err(ZCanError::MethodExecuteFailed("VCI_Debug".to_string(), code)),
            }
        }
    }
}

impl ZCanApi<(ZCanDeviceType, u32), (ZCanDeviceType, u32, u8), ZCanFrameV2, ZCanFdFrameV1> for USBCANFDApi<'_> {
    fn init_can_chl(&self, dev_hdl: (ZCanDeviceType, u32), channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx) = dev_hdl;
        unsafe {
            // set channel resistance status
            if dev_type.has_resistance() {
                let state = (cfg.extra().resistance() as u32).to_string();
                let resistance_path = CmdPath::new_reference(Reference::Resistance as u32);
                let _value = CString::new(state).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
                self.set_reference((dev_type, dev_idx, channel), &resistance_path, _value.as_ptr() as *mut c_void)?;
            }

            let cfg = ZCanChlCfgV2::try_from(cfg)?;
            match (self.VCI_InitCAN)(dev_type as u32, dev_idx, channel as u32, &cfg) {
                Self::STATUS_OK => {
                    match (self.VCI_StartCAN)(dev_type as u32, dev_idx, channel as u32) {
                        Self::STATUS_OK => Ok(0),
                        code => Err(ZCanError::MethodExecuteFailed("VCI_StartCAN".to_string(), code)),
                    }
                }
                code=> Err(ZCanError::MethodExecuteFailed("VCI_InitCAN".to_string(), code)),
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
            ZCanFrameType::ALL => return Err(ZCanError::ParamNotSupported),
        }
        let ret = unsafe { (self.VCI_GetReceiveNum)(dev_type as u32, dev_idx, _channel) };
        debug!("ZLGCAN - get receive {} number: {}.", can_type, ret);
        Ok(ret)
    }

    fn receive_can(&self, chl_hdl: (ZCanDeviceType, u32, u8), size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCanFrameV2>, usize)) -> Result<Vec<ZCanFrameV2>, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_Receive)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_can(&self, chl_hdl: (ZCanDeviceType, u32, u8), frames: Vec<ZCanFrameV2>) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_Transmit)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }

    fn receive_canfd(&self, chl_hdl: (ZCanDeviceType, u32, u8), size: u32, timeout: u32, resize: fn(&mut Vec<ZCanFdFrameV1>, usize)) -> Result<Vec<ZCanFdFrameV1>, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let mut frames = Vec::new();
        // frames.resize_with(size as usize, Default::default);
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_ReceiveFD)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            warn!("ZLGCAN - receive CANFD frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }

    fn transmit_canfd(&self, chl_hdl: (ZCanDeviceType, u32, u8), frames: Vec<ZCanFdFrameV1>) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_TransmitFD)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }
}

impl ZLinApi<(ZCanDeviceType, u32), (ZCanDeviceType, u32, u8)> for USBCANFDApi<'_> {
    fn init_lin_chl(&self, dev_hdl: (ZCanDeviceType, u32), channel: u8, cfg: &ZLinChlCfg) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx) = dev_hdl;
        unsafe {
            match (self.VCI_InitLIN)(dev_type as u32, dev_idx, channel as u32, cfg) {
                Self::STATUS_OK => match (self.VCI_StartLIN)(dev_type as u32, dev_idx, channel as u32) {
                    Self::STATUS_OK => Ok(Self::STATUS_OK),
                    code => Err(ZCanError::MethodExecuteFailed("VCI_StartLIN".to_string(), code)),
                },
                code => Err(ZCanError::MethodExecuteFailed("VCI_InitLIN".to_string(), code)),
            }
        }
    }
    fn reset_lin_chl(&self, chl_hdl: (ZCanDeviceType, u32, u8)) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        match unsafe { (self.VCI_ResetLIN)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_ResetLIN".to_string(), code)),
        }
    }
    fn clear_lin_buffer(&self, chl_hdl: (ZCanDeviceType, u32, u8)) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        match unsafe { (self.VCI_ClearLINBuffer)(dev_type as u32, dev_idx, channel as u32) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_ClearLINBuffer".to_string(), code)),
        }
    }
    fn get_lin_num(&self, chl_hdl: (ZCanDeviceType, u32, u8)) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let ret = unsafe { (self.VCI_GetLINReceiveNum)(dev_type as u32, dev_idx, channel as u32) };
        debug!("ZLGCAN - get receive LIN number: {}.", ret);
        Ok(ret)
    }
    fn receive_lin(&self, chl_hdl: (ZCanDeviceType, u32, u8), size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZLinFrame>, usize)) -> Result<Vec<ZLinFrame>, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_ReceiveLIN)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            warn!("ZLGCAN - receive LIN frame expect: {}, actual: {}!", size, ret);
        }
        Ok(frames)
    }
    fn transmit_lin(&self, chl_hdl: (ZCanDeviceType, u32, u8), frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_TransmitLIN)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            warn!("ZLGCAN - transmit LIN frame expect: {}, actual: {}!", len, ret);
        }
        Ok(ret)
    }
    fn set_lin_subscribe(&self, chl_hdl: (ZCanDeviceType, u32, u8), cfg: Vec<ZLinSubscribe>) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let len = cfg.len() as u32;
        match unsafe { (self.VCI_SetLINSubscribe)(dev_type as u32, dev_idx, channel as u32, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_SetLINSubscribe".to_string(), code)),
        }
    }
    fn set_lin_publish(&self, chl_hdl: (ZCanDeviceType, u32, u8), cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        let (dev_type, dev_idx, channel) = chl_hdl;
        let len = cfg.len() as u32;
        match unsafe { (self.VCI_SetLINPublish)(dev_type as u32, dev_idx, channel as u32, cfg.as_ptr(), len) } {
            Self::STATUS_OK => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("VCI_SetLINPublish".to_string(), code)),
        }
    }
}

impl<DH> ZCloudApi<DH> for USBCANFDApi<'_> {

}

#[cfg(test)]
mod tests {
    use dlopen2::symbor::{Library, SymBorApi};
    use zlgcan_common::can::{
        ZCanChlMode, ZCanChlType,
        ZCanFrameV2,
        CanMessage
    };
    use zlgcan_common::device::ZCanDeviceType;
    use zlgcan_common::can::CanChlCfgFactory;
    use crate::api::{ZCanApi, ZDeviceApi};
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
        assert_eq!(dev_info.can_channels(), channels);
        assert!(dev_info.canfd());

        api.init_can_chl((dev_type, dev_idx), 0, &cfg).unwrap();
        let frame = CanMessage::new(0x7E0, Some(0), [0x01, 0x02, 0x03], false, false, None).unwrap();
        let frame1 = CanMessage::new(0x1888FF00, Some(0), [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08], false, false, None).unwrap();
        let frames = vec![
            ZCanFrameV2::try_from(frame).unwrap(),
            ZCanFrameV2::try_from(frame1).unwrap()
        ];
        let ret = api.transmit_can((dev_type, dev_idx, channel), frames).unwrap();
        assert_eq!(ret, 2);

        api.reset_can_chl((dev_type, dev_idx, channel)).unwrap();

        api.close((dev_type, dev_idx)).unwrap();
    }
}

