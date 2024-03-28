// use log::{debug, warn};
// use common::error::ZCanError;
// use common::lin::channel::ZLinChlCfg;
// use common::lin::frame::{ZLinFrame, ZLinPublish, ZLinSubscribe};
//
// use crate::constant::{STATUS_OK, INVALID_CHANNEL_HANDLE};
use super::USBCANFD800UApi;

impl USBCANFD800UApi<'_> {
    // pub(crate) fn init_lin_chl(&self, dev_hdl: u32, channel: u8, cfg: &ZLinChlCfg) -> Result<u32, ZCanError> {
    //     unsafe {
    //         let handler = (self.ZCAN_InitLIN)(dev_hdl, channel as u32, cfg);
    //         if handler == INVALID_CHANNEL_HANDLE {
    //             return Err(ZCanError::new(INVALID_CHANNEL_HANDLE, format!("ZLGCAN - `InitLIN` channel: {} failed", channel)));
    //         }
    //         match (self.ZCAN_StartLIN)(dev_hdl) {
    //             STATUS_OK => Ok(handler),
    //             code => Err(ZCanError::new(code, format!("ZLGCAN - `StartLIN` channel: {} failed", channel))),
    //         }
    //     }
    // }
    //
    // #[inline(always)]
    // pub(crate) fn reset_lin_chl(&self, chl_hdl: u32) -> Result<(), ZCanError> {
    //     match unsafe { (self.ZCAN_ResetLIN)(chl_hdl) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::new(code, "ZLGCAN - LIN channel reset failed".to_string())),
    //     }
    // }
    //
    // #[inline(always)]
    // pub(crate) fn clear_lin_buffer(&self, chl_hdl: u32) -> Result<(), ZCanError> {
    //     match unsafe { (self.ZCAN_ClearLINBuffer)(chl_hdl) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::new(code, "ZLGCAN - LIN channel clear buffer failed".to_string())),
    //     }
    // }
    //
    // #[inline(always)]
    // pub(crate) fn get_lin_num(&self, chl_hdl: u32) -> u32 {
    //     let ret = unsafe { (self.ZCAN_GetLINReceiveNum)(chl_hdl) };
    //     debug!("ZLGCAN - get receive LIN number: {}.", ret);
    //     ret
    // }
    //
    // #[inline(always)]
    // pub(crate) fn receive_lin(&self, chl_hdl: u32, size: u32, timeout: Option<u32>, resize: impl Fn(&mut Vec<ZLinFrame>, usize)) -> Vec<ZLinFrame> {
    //     let mut frames = Vec::new();
    //
    //     resize(&mut frames, size as usize);
    //
    //     let ret = unsafe { (self.ZCAN_ReceiveLIN)(chl_hdl, frames.as_mut_ptr(), size, timeout.unwrap_or(50)) };
    //     if ret < size {
    //         warn!("ZLGCAN - receive LIN frame expect: {}, actual: {}!", size, ret);
    //     }
    //     frames
    // }
    //
    // #[inline(always)]
    // pub(crate) fn transmit_lin(&self, chl_hdl: u32, frames: Vec<ZLinFrame>) -> u32 {
    //     let len = frames.len() as u32;
    //     let ret = unsafe { (self.ZCAN_TransmitLIN)(chl_hdl, frames.as_ptr(), len) };
    //     if ret < len {
    //         warn!("ZLGCAN - transmit LIN frame expect: {}, actual: {}!", len, ret);
    //     }
    //     ret
    // }
    //
    // #[inline(always)]
    // pub(crate) fn set_lin_subscribe(&self, chl_hdl: u32, cfg: Vec<ZLinSubscribe>)-> Result<(), ZCanError> {
    //     let len = cfg.len() as u32;
    //     match unsafe { (self.ZCAN_SetLINSubscribe)(chl_hdl, cfg.as_ptr(), len) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`set_lin_subscribe`"))),
    //     }
    // }
    //
    // #[inline(always)]
    // pub(crate) fn set_lin_publish(&self, chl_hdl: u32, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
    //     let len = cfg.len() as u32;
    //     match unsafe { (self.ZCAN_SetLINPublish)(chl_hdl, cfg.as_ptr(), len) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`set_lin_publish`"))),
    //     }
    // }
    // #[inline(always)]
    // pub(crate) fn wakeup_lin(&self, _chl_hdl: u32) -> Result<(), ZCanError> {
    //     Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`wakeup_lin`")))
    // }
    //
    // // #[inline(always)]
    // // pub(crate) fn set_lin_publish_ex(&self, chl_hdl: u32, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
    // //     let len = cfg.len() as u32;
    // //     match unsafe { (self.ZCAN_SetLINPublishEx)(chl_hdl, cfg.as_ptr(), len) } {
    // //         STATUS_OK => Ok(()),
    // //         code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`set_lin_publish_ex`"))),
    // //     }
    // // }
    // #[inline(always)]
    // #[deprecated(since="0.1.0", note="This method is deprecated!")]
    // pub(crate) fn set_lin_slave_msg(&self, chl_hdl: u32, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
    //     let len = msg.len() as u32;
    //     match unsafe { (self.ZCAN_SetLINSlaveMsg)(chl_hdl, msg.as_ptr(), len) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`set_lin_slave_msg`"))),
    //     }
    // }
    // #[inline(always)]
    // #[deprecated(since="0.1.0", note="This method is deprecated!")]
    // pub(crate) fn clear_lin_slave_msg(&self, chl_hdl: u32, pids: Vec<u8>) -> Result<(), ZCanError> {
    //     let len = pids.len() as u32;
    //     match unsafe { (self.ZCAN_ClearLINSlaveMsg)(chl_hdl, pids.as_ptr(), len) } {
    //         STATUS_OK => Ok(()),
    //         code => Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`clear_lin_slave_msg`"))),
    //     }
    // }
}



