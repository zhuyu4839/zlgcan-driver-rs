use zlgcan_common as common;

use std::ffi::{c_void, CString};
use log::{debug, warn};
use common::can::{
    CanChlCfg,
    ZCanChlCfgDetail, ZCanChlError, ZCanChlErrorV2, ZCanChlStatus,
    Reference, ZCanFrameType,
    ZCanFdFrame, ZCanFrame,
};
use common::device::{CmdPath, ZCanDeviceType};
use common::error::ZCanError;
use crate::constant::STATUS_OK;

use super::USBCANFDApi;

impl USBCANFDApi<'_> {
    pub(crate) fn init_can_chl(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        // let dev_type = cfg.device_type();
        unsafe {
            // set channel resistance status
            if dev_type.has_resistance() {
                let state = (cfg.extra().resistance() as u32).to_string();
                let resistance_path = CmdPath::new_reference(Reference::Resistance);
                let _value = CString::new(state).expect("ZLGCAN - convert value to C string failed!");
                self.set_reference(dev_type, dev_idx, channel, &resistance_path, _value.as_ptr() as *mut c_void).unwrap();
            }

            let cfg = ZCanChlCfgDetail::from(cfg);
            match (self.VCI_InitCAN)(dev_type as u32, dev_idx, channel as u32, &cfg) {
                STATUS_OK => {
                    match (self.VCI_StartCAN)(dev_type as u32, dev_idx, channel as u32) {
                        STATUS_OK => Ok(0),
                        code => Err(ZCanError::new(code, format!("ZLGCAN - `StartCAN` channel: {} failed", channel))),
                    }
                }
                code=> Err(ZCanError::new(code, format!("ZLGCAN - `InitCAN` channel: {} failed", channel))),
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
            ZCanFrameType::ALL => panic!("ZLGCAN - device is not supported!"),
        }
        let ret = unsafe { (self.VCI_GetReceiveNum)(dev_type as u32, dev_idx, _channel) };
        debug!("ZLGCAN - get receive {} number: {}.", msg, ret);
        ret
    }

    #[inline(always)]
    pub(crate) fn receive_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCanFrame>, usize)) -> Vec<ZCanFrame> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_Receive)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
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

    #[inline(always)]
    pub(crate) fn receive_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: u32, resize: fn(&mut Vec<ZCanFdFrame>, usize)) -> Vec<ZCanFdFrame> {
        let mut frames = Vec::new();
        // frames.resize_with(size as usize, Default::default);
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_ReceiveFD)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            warn!("ZLGCAN - receive CANFD frame expect: {}, actual: {}!", size, ret);
        }
        frames
    }

    #[inline(always)]
    pub(crate) fn transmit_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFdFrame>) -> u32 {
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_TransmitFD)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
        }
        ret
    }
}
