use zlgcan_common as common;

use log::{debug, warn};
use common::can::CanChlCfg;
use common::can::{ZCanChlCfgDetail, ZCanChlError, ZCanChlStatus, ZCanFdFrame, ZCanFrame, ZCanChlType, ZCanFrameType};
use common::device::{CmdPath, ZCanDeviceType};
use common::error::ZCanError;

use crate::constant::{BAUD_RATE, CANFD_ABIT_BAUD_RATE, CANFD_DBIT_BAUD_RATE, INTERNAL_RESISTANCE, INVALID_CHANNEL_HANDLE, PROTOCOL, STATUS_OK};
use super::Api;

impl Api<'_> {
    pub(crate) fn init_can_chl(&self, dev_hdl: u32, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError> {
        let dev_type = cfg.device_type();
        unsafe {
            if !matches!(cfg.device_type(), ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2) {
                // configure the clock
                if let Some(clock) = cfg.clock() {
                    let clock_path = CmdPath::new_path("clock");
                    self.set_check_value(dev_hdl, &clock_path, clock.to_string().as_str(), dev_type).unwrap();
                }
                // set channel resistance status
                if dev_type.has_resistance() {
                    let state = (cfg.extra().resistance() as u32).to_string();
                    let resistance_path = format!("{}/{}", channel, INTERNAL_RESISTANCE);
                    let resistance_path = CmdPath::new_path(resistance_path.as_str());
                    self.set_check_value(dev_hdl, &resistance_path, state.as_str(), dev_type).unwrap();
                }
                // set channel protocol
                let can_type = cfg.can_type();
                let protocol = (can_type as u32).to_string();
                let protocol_path = format!("{}/{}", channel, PROTOCOL);
                let protocol_path = CmdPath::new_path(protocol_path.as_str());
                self.set_check_value(dev_hdl, &protocol_path, protocol.as_str(), dev_type).unwrap();

                // set channel bitrate
                let bitrate = cfg.bitrate();
                if dev_type.canfd_support() {
                    let abitrate_path = format!("{}/{}", channel, CANFD_ABIT_BAUD_RATE);
                    let abitrate_path = CmdPath::new_path(abitrate_path.as_str());
                    self.set_check_value(dev_hdl, &abitrate_path, bitrate.to_string().as_str(), dev_type).unwrap();
                    match can_type {
                        ZCanChlType::CANFD_ISO | ZCanChlType::CANFD_NON_ISO => {
                            let dbitrate = cfg.extra().dbitrate().unwrap_or(bitrate).to_string();
                            let dbitrate_path = format!("{}/{}", channel, CANFD_DBIT_BAUD_RATE);
                            let dbitrate_path = CmdPath::new_path(dbitrate_path.as_str());
                            self.set_check_value(dev_hdl, &dbitrate_path, dbitrate.as_str(), dev_type).unwrap();
                        },
                        _ => {},
                    }
                }
                else {
                    let bitrate_path = format!("{}/{}", channel, BAUD_RATE);
                    let bitrate_path = CmdPath::new_path(bitrate_path.as_str());
                    self.set_check_value(dev_hdl, &bitrate_path, bitrate.to_string().as_str(), dev_type).unwrap();
                }
            }

            let cfg = ZCanChlCfgDetail::from(cfg);
            let handler = (self.ZCAN_InitCAN)(dev_hdl, channel as u32, &cfg);
            if handler == INVALID_CHANNEL_HANDLE {
                return Err(ZCanError::new(INVALID_CHANNEL_HANDLE, format!("ZLGCAN - `InitCAN` channel: {} failed", channel)));
            }

            match (self.ZCAN_StartCAN)(handler) {
                STATUS_OK => Ok(handler),
                code => Err(ZCanError::new(code, format!("ZLGCAN - `StartCAN` channel: {} failed", channel))),
            }
        }
    }

    #[inline(always)]
    pub(crate) fn reset_can_chl(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ResetCAN)(chl_hdl) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - CAN channel reset failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn read_can_chl_status(&self, chl_hdl: u32) -> Result<ZCanChlStatus, ZCanError> {
        let mut status: ZCanChlStatus = Default::default();
        match unsafe { (self.ZCAN_ReadChannelStatus)(chl_hdl, &mut status) } {
            STATUS_OK => Ok(status),
            code =>Err(ZCanError::new(code, "ZLGCAN - read CAN channel status failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn read_can_chl_error(&self, chl_hdl: u32) -> Result<ZCanChlError, ZCanError> {
        let mut info: ZCanChlError = ZCanChlError::from_v2(Default::default());
        match unsafe { (self.ZCAN_ReadChannelErrInfo)(chl_hdl, &mut info) } {
            STATUS_OK => Ok(info),
            code =>Err(ZCanError::new(code, "ZLGCAN - read CAN channel error info failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn clear_can_buffer(&self, chl_hdl: u32) -> Result<(), ZCanError> {
        match unsafe { (self.ZCAN_ClearBuffer)(chl_hdl) } {
            STATUS_OK => Ok(()),
            code => Err(ZCanError::new(code, "ZLGCAN - clear CAN channel's buffer failed".to_string())),
        }
    }

    #[inline(always)]
    pub(crate) fn get_can_num(&self, chl_hdl: u32, msg: ZCanFrameType) -> u32 {
        let ret = unsafe { (self.ZCAN_GetReceiveNum)(chl_hdl, msg as u8) };
        debug!("ZLGCAN - get receive {} number: {}.", msg, ret);
        ret
    }

    #[inline(always)]
    pub(crate) fn receive_can(&self, chl_hdl: u32, size: u32, timeout: Option<u32>, resize: impl Fn(&mut Vec<ZCanFrame>, usize)) -> Vec<ZCanFrame> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_Receive)(chl_hdl, frames.as_mut_ptr(), size, timeout.unwrap_or(50)) };
        if ret < size {
            warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        frames
    }

    #[inline(always)]
    pub(crate) fn transmit_can(&self, chl_hdl: u32, frames: Vec<ZCanFrame>) -> u32 {
        let len = frames.len() as u32;
        // method 1
        // let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frames.as_ptr(), len) };
        // if ret < len {
        //     warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        // }
        // ret
        // method 2
        // let mut boxed_slice: Box<[ZCanFrame]> = frames.into_boxed_slice();
        // let array: *mut ZCanFrame = boxed_slice.as_mut_ptr();
        // // let ptr = frames.as_ptr();
        // let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, array, len) };
        // mem::forget(boxed_slice);
        // if ret < len {
        //     warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, ret);
        // }
        // ret
        // method 3: just do like this because of pointer offset TODO
        let mut count = 0;
        frames.iter().for_each(|frame| {
            let ret = unsafe { (self.ZCAN_Transmit)(chl_hdl, frame, 1) };
            count += ret;
        });
        if count < len {
            warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, count);
        }
        count
    }

    #[inline(always)]
    pub(crate) fn receive_canfd(&self, chl_hdl: u32, size: u32, timeout: Option<u32>, resize: fn(&mut Vec<ZCanFdFrame>, usize)) -> Vec<ZCanFdFrame> {
        let mut frames = Vec::new();
        // frames.resize_with(size as usize, Default::default);
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCAN_ReceiveFD)(chl_hdl, frames.as_mut_ptr(), size, timeout.unwrap_or(50)) };
        if ret < size {
            warn!("ZLGCAN - receive CANFD frame expect: {}, actual: {}!", size, ret);
        }
        frames
    }

    #[inline(always)]
    pub(crate) fn transmit_canfd(&self, chl_hdl: u32, frames: Vec<ZCanFdFrame>) -> u32 {
        let len = frames.len() as u32;
        // let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frames.as_ptr(), len) };
        // if ret < len {
        //     warn!("ZLGCAN - transmit CANFD frame expect: {}, actual: {}!", len, ret);
        // }
        // ret
        let mut count = 0;
        frames.iter().for_each(|frame| {
            let ret = unsafe { (self.ZCAN_TransmitFD)(chl_hdl, frame, 1) };
            count += ret;
        });
        if count < len {
            warn!("ZLGCAN - transmit CAN frame expect: {}, actual: {}!", len, count);
        }
        count
    }
}

