use std::collections::HashMap;
use std::fs::read_to_string;
use common::can::BitrateCfg;
use common::device::{Handler, ZCanDeviceType, ZlgDevice};
use common::error::ZCanError;
use self::constant::BITRATE_CFG_FILENAME;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::driver;

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::driver;

// lazy_static!(
//     static ref STATIC_DRIVER: driver::ZCanDriver<'static> = driver::ZCanDriver::new();
// );

impl driver::ZCanDriver<'_> {
    pub(crate) fn load_bitrate_cfg() -> HashMap<String, BitrateCfg> {
        let contents = read_to_string(BITRATE_CFG_FILENAME).unwrap_or_else(|e| { panic!("Unable to read `{}`: {:?}", BITRATE_CFG_FILENAME, e)});
        let config = serde_yaml::from_str(&contents).unwrap_or_else(|e| { panic!("Error parsing YAML: {:?}", e) });
        config
    }
    #[inline(always)]
    pub(crate) fn device_handler<C, T>(&self, dev_type: ZCanDeviceType, dev_idx: u32, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(&Handler) -> T {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match self.handlers.get(&dev_name) {
            Some(v) => Ok(callback(v)),
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(crate) fn device_handler1<C, T>(&self, dev_type: ZCanDeviceType, dev_idx: u32, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(ZCanDeviceType, u32) -> T {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match self.handlers.get(&dev_name) {
            Some(_) => Ok(callback(dev_type, dev_idx)),
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }
    #[inline(always)]
    pub(self) fn can_handler<C, T>(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u32) -> T {
        self.device_handler(dev_type, dev_idx, |hdl| -> Result<T, ZCanError> {
            let dev_name = Self::device_name(dev_type, dev_idx);
            match hdl.find_can(channel) {
                Some(chl) => Ok(callback(chl)),
                None => Err(ZCanError::new(0, format!("ZLGCAN - {} CAN channel: {} is not opened", dev_name, channel))),
            }
        }).unwrap()
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(self) fn can_handler1<C, T>(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(ZCanDeviceType, u32, u8) -> T {
        self.device_handler(dev_type, dev_idx, |hdl| -> Result<T, ZCanError> {
            let dev_name = Self::device_name(dev_type, dev_idx);
            match hdl.find_can(channel) {
                Some(_) => Ok(callback(dev_type, dev_idx, channel)),
                None => Err(ZCanError::new(0, format!("ZLGCAN - {} CAN channel: {} is not opened", dev_name, channel))),
            }
        }).unwrap()
    }

    #[inline(always)]
    pub(self) fn lin_handler<C, T>(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u32) -> T {
        self.device_handler(dev_type, dev_idx, |hdl| -> Result<T, ZCanError> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => Ok(callback(*chl)),
                None => Err(ZCanError::new(0, format!("ZLGCAN - CAN channel: {} is not opened", channel))),
            }
        }).unwrap()
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(self) fn lin_handler1<C, T>(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(ZCanDeviceType, u32, u8) -> T {
        self.device_handler(dev_type, dev_idx, |hdl| -> Result<T, ZCanError> {
            match hdl.lin_channels().get(&channel) {
                Some(_) => Ok(callback(dev_type, dev_idx, channel)),
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
    pub(crate) const BITRATE_CFG_FILENAME: &str = "bitrate.cfg.yaml";
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
