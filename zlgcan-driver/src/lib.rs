use zlgcan_common as common;

use common::device::{Handler, ZlgDevice};
use common::error::ZCanError;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::driver::ZCanDriver;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::driver::ZCanDriver;
use zlgcan_common::can::{CanMessage, ZCanFdFrame, ZCanFdFrameV1, ZCanFdFrameV2, ZCanFrame, ZCanFrameType, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3};
use zlgcan_common::device::ZCanDevice;

pub fn unify_send(device: &ZCanDriver, msg: CanMessage) -> bool {
    let channel = msg.channel();
    if msg.is_fd() {
        let frames =
            if device.device_type().is_fdframe_v1() {
                vec![ZCanFdFrame::from(ZCanFdFrameV1::from(msg))]
            }
            else if device.device_type().is_fdframe_v2() {
                vec![ZCanFdFrame::from(ZCanFdFrameV2::from(msg))]
            }
            else {
                panic!("")
            };

        device.transmit_canfd(channel, frames).is_ok()
    }
    else {
        let frames =
            if device.device_type().is_frame_v1() {
                vec![ZCanFrame::from(ZCanFrameV1::from(msg))]
            }
            else if device.device_type().is_frame_v2() {
                vec![ZCanFrame::from(ZCanFrameV2::from(msg))]
            }
            else if device.device_type().is_frame_v3() {
                vec![ZCanFrame::from(ZCanFrameV3::from(msg))]
            }
            else {
                panic!("")
            };

        device.transmit_can(channel, frames).is_ok()
    }
}
pub fn unify_recv(device: &ZCanDriver, channel: u8, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError> {
    let count_can = device.get_can_num(channel, ZCanFrameType::CAN)?;
    let mut results: Vec<CanMessage> = Vec::new();

    let frames = device.receive_can(channel, count_can, timeout)?;
    let mut frames = frames.iter().map(|f| -> CanMessage {
        if device.device_type().is_frame_v1() {
            CanMessage::from(ZCanFrameV1::from(f))
        }
        else if device.device_type().is_frame_v2() {
            CanMessage::from(ZCanFrameV2::from(f))
        }
        else if device.device_type().is_frame_v3() {
            CanMessage::from(ZCanFrameV3::from(f))
        }
        else {
            panic!("")
        }
    });

    results.extend(&mut frames);
    if device.device_type().canfd_support() {
        let count_fd = device.get_can_num(channel, ZCanFrameType::CANFD)?;
        let frames = device.receive_canfd(channel, count_fd, timeout)?;
        let mut frames = frames.iter().map(|f| -> CanMessage {
            if device.device_type().is_fdframe_v1() {
                CanMessage::from(ZCanFdFrameV1::from(f))
            }
            else if device.device_type().is_fdframe_v2() {
                CanMessage::from(ZCanFdFrameV2::from(f))
            }
            else {
                panic!("")
            }
        });

        results.extend(&mut frames);
    }

    Ok(results)
}

#[allow(dead_code)]
impl ZCanDriver<'_> {
    #[inline(always)]
    pub(crate) fn device_handler<C, T>(&self, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(&Handler) -> T {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.handlers.get(&dev_name) {
            Some(v) => Ok(callback(v)),
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(crate) fn device_handler1<C, T>(&self, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce() -> T {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.handlers.get(&dev_name) {
            Some(_) => Ok(callback()),
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }
    #[inline(always)]
    pub(self) fn can_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u32) -> T {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            let dev_name = Self::device_name(self.dev_type, self.dev_idx);
            match hdl.find_can(channel) {
                Some(chl) => Ok(callback(chl)),
                None => Err(ZCanError::new(0, format!("ZLGCAN - {} CAN channel: {} is not opened", dev_name, channel))),
            }
        }).unwrap()
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(self) fn can_handler1<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u8) -> T {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            let dev_name = Self::device_name(self.dev_type, self.dev_idx);
            match hdl.find_can(channel) {
                Some(_) => Ok(callback(channel)),
                None => Err(ZCanError::new(0, format!("ZLGCAN - {} CAN channel: {} is not opened", dev_name, channel))),
            }
        }).unwrap()
    }

    #[inline(always)]
    pub(self) fn lin_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u32) -> T {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => Ok(callback(*chl)),
                None => Err(ZCanError::new(0, format!("ZLGCAN - CAN channel: {} is not opened", channel))),
            }
        }).unwrap()
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(self) fn lin_handler1<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u8) -> T {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.lin_channels().get(&channel) {
                Some(_) => Ok(callback(channel)),
                None => Err(ZCanError::new(0, format!("ZLGCAN - CAN channel: {} is not opened", channel))),
            }
        }).unwrap()
    }
}

#[allow(dead_code)]
pub(crate) mod constant {
    pub(crate) const LOAD_LIB_FAILED: &str = "ZLGCAN - could not open library!";
    pub(crate) const LOAD_SYMBOLS_FAILED: &str = "ZLGCAN - could not load symbols!";
    pub(crate) const INVALID_DEVICE_HANDLE: u32 = 0;
    pub(crate) const INVALID_CHANNEL_HANDLE: u32 = 0;
    pub(crate) const STATUS_OK: u32 = 1;
    pub(crate) const STATUS_ONLINE: u32 = 2;
    pub(crate) const STATUS_OFFLINE: u32 = 3;
    pub(crate) const INTERNAL_RESISTANCE: &str = "initenal_resistance";
    pub(crate) const PROTOCOL: &str = "protocol";
    pub(crate) const BAUD_RATE: &str = "baud_rate";
    pub(crate) const CANFD_ABIT_BAUD_RATE: &str = "canfd_abit_baud_rate";
    pub(crate) const CANFD_DBIT_BAUD_RATE: &str = "canfd_dbit_baud_rate";
    pub(crate) const BAUD_RATE_CUSTOM: &str = "baud_rate_custom";
    pub(crate) const CANFD_STANDARD: &str = "canfd_standard";
    pub(crate) const TX_TIMEOUT: &str = "tx_timeout";
    pub(crate) const AUTO_SEND: &str = "auto_send";
    pub(crate) const AUTO_SEND_CANFD: &str = "auto_send_canfd";
    pub(crate) const AUTO_SEND_PARAM: &str = "auto_send_param";
    pub(crate) const CLEAR_AUTO_SEND: &str = "clear_auto_send";
    pub(crate) const APPLY_AUTO_SEND: &str = "apply_auto_send";
    pub(crate) const SET_SEND_MODE: &str = "set_send_mode";
    pub(crate) const GET_DEVICE_AVAILABLE_TX_COUNT: &str = "get_device_available_tx_count/1";
    pub(crate) const CLEAR_DELAY_SEND_QUEUE: &str = "clear_delay_send_queue";
    pub(crate) const SET_DEVICE_RECV_MERGE: &str = "set_device_recv_merge";
    pub(crate) const GET_DEVICE_RECV_MERGE: &str = "get_device_recv_merge/1";
    pub(crate) const SET_CN: &str = "set_cn";
    pub(crate) const SET_NAME: &str = "set_name";
    pub(crate) const GET_CN: &str = "get_cn/1";
    pub(crate) const GET_NAME : &str = "get_name/1 ";
    pub(crate) const FILTER_MODE: &str = "filter_mode";
    pub(crate) const FILTER_START: &str = "filter_start";
    pub(crate) const FILTER_END: &str = "filter_end";
    pub(crate) const FILTER_ACK: &str = "filter_ack";
    pub(crate) const FILTER_CLEAR: &str = "filter_clear";
    pub(crate) const SET_BUS_USAGE_ENABLE: &str = "set_bus_usage_enable";
    pub(crate) const SET_BUS_USAGE_PERIOD: &str = "set_bus_usage_period";
    pub(crate) const GET_BUS_USAGE: &str = "get_bus_usage/1";
    pub(crate) const SET_TX_RETRY_POLICY: &str = "set_tx_retry_policy";
// USBCAN-8E-U 支持属性列表
// "info/channel/channel_x/redirect"
// USBCAN-4E-U 支持属性列表
// "info/channel/channel_x/baud_rate"
// "info/channel/channel_x/work_mode"
// "info/channel/channel_x/redirect"
// "info/channel/channel_x/whitelisting"
// "info/channel/channel_x/autotxobj"
}
