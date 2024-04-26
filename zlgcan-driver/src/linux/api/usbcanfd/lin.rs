use zlgcan_common as common;

use log::{debug, warn};
use common::device::ZCanDeviceType;
use common::error::ZCanError;
use common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinSubscribe};
use crate::constant::{STATUS_OK, INVALID_CHANNEL_HANDLE};

use super::USBCANFDApi;

#[allow(unused_variables)]
impl USBCANFDApi<'_> {
    pub(crate) fn init_lin_chl(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: &ZLinChlCfg) -> Result<u32, ZCanError> {
        unsafe {
            let handler = (self.VCI_InitLIN)(dev_type as u32, dev_idx, channel as u32, cfg);
            if handler == INVALID_CHANNEL_HANDLE {
                return Err(ZCanError::new(INVALID_CHANNEL_HANDLE, format!("ZLGCAN - `InitLIN` channel: {} failed", channel)));
            }
            match (self.VCI_StartLIN)(dev_type as u32, dev_idx, channel as u32) {
                STATUS_OK => Ok(handler),
                code => Err(ZCanError::new(code, format!("ZLGCAN - `StartLIN` channel: {} failed", channel))),
            }
        }
    }

    #[inline(always)]
    pub(crate) fn reset_lin_chl(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        match unsafe { (self.VCI_ResetLIN)(dev_type as u32, dev_idx, channel as u32) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - LIN channel reset failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn clear_lin_buffer(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        match unsafe { (self.VCI_ClearLINBuffer)(dev_type as u32, dev_idx, channel as u32) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - LIN channel clear buffer failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn get_lin_num(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> u32 {
        let ret = unsafe { (self.VCI_GetLINReceiveNum)(dev_type as u32, dev_idx, channel as u32) };
        debug!("ZLGCAN - get receive LIN number: {}.", ret);
        ret
    }

    #[inline(always)]
    pub(crate) fn receive_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZLinFrame>, usize)) -> Vec<ZLinFrame> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.VCI_ReceiveLIN)(dev_type as u32, dev_idx, channel as u32, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            warn!("ZLGCAN - receive LIN frame expect: {}, actual: {}!", size, ret);
        }
        frames
    }

    #[inline(always)]
    pub(crate) fn transmit_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZLinFrame>) -> u32 {
        let len = frames.len() as u32;
        let ret = unsafe { (self.VCI_TransmitLIN)(dev_type as u32, dev_idx, channel as u32, frames.as_ptr(), len) };
        if ret < len {
            warn!("ZLGCAN - transmit LIN frame expect: {}, actual: {}!", len, ret);
        }
        ret
    }

    #[inline(always)]
    pub(crate) fn set_lin_subscribe(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinSubscribe>)-> Result<(), ZCanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.VCI_SetLINSubscribe)(dev_type as u32, dev_idx, channel as u32, cfg.as_ptr(), len) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`set_lin_subscribe`"))),
        }
    }

    #[inline(always)]
    pub(crate) fn set_lin_publish(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        let len = cfg.len() as u32;
        match unsafe { (self.VCI_SetLINPublish)(dev_type as u32, dev_idx, channel as u32, cfg.as_ptr(), len) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`set_lin_publish`"))),
        }
    }
    #[inline(always)]
    pub(crate) fn wakeup_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} failed", "`wakeup_lin`")))
    }

    // #[inline(always)]
    // pub(crate) fn set_lin_publish_ex(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
    //     Err(ZCanError::new(0xFF, format!("ZLGCAN - {} failed", "`set_lin_publish_ex`")))
    // }
    #[inline(always)]
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    pub(crate) fn set_lin_slave_msg(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} failed", "`set_lin_slave_msg`")))
    }
    #[inline(always)]
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    pub(crate) fn clear_lin_slave_msg(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} failed", "`clear_lin_slave_msg`")))
    }
}



