use zlgcan_common::can::{CanChlCfg, CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use zlgcan_common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use zlgcan_common::device::{DeriveInfo, Handler, ZCanDeviceType, ZCanError, ZDeviceInfo};
use zlgcan_common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::ZCanDriver;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::ZCanDriver;

#[allow(dead_code)]
impl ZCanDriver<'_> {
    #[inline(always)]
    pub(crate) fn device_handler<C, T>(&self, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(&Handler) -> Result<T, ZCanError> {
        match &self.handler {
            Some(v) => callback(v),
            None => Err(ZCanError::DeviceNotOpened),
        }
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(crate) fn device_handler1<C, T>(&self, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce() -> Result<T, ZCanError> {
        match self.handler {
            Some(_) => callback(),
            None => Err(ZCanError::DeviceNotOpened),
        }
    }
    #[inline(always)]
    pub(self) fn can_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u32) -> Result<T, ZCanError> {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.find_can(channel) {
                Some(chl) => callback(chl),
                None => Err(ZCanError::ChannelNotOpened),
            }
        })
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(self) fn can_handler1<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u8) -> Result<T, ZCanError> {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.find_can(channel) {
                Some(_) => callback(channel),
                None => Err(ZCanError::ChannelNotOpened),
            }
        })
    }

    #[inline(always)]
    pub(self) fn lin_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u32) -> Result<T, ZCanError> {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => callback(*chl),
                None => Err(ZCanError::ChannelNotOpened),
            }
        })
    }
    #[cfg(target_os = "linux")]
    #[inline(always)]
    pub(self) fn lin_handler1<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(u8) -> Result<T, ZCanError> {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.lin_channels().get(&channel) {
                Some(_) => callback(channel),
                None => Err(ZCanError::ChannelNotOpened),
            }
        })
    }
}

#[allow(unused_variables)]
pub trait ZDevice {
    fn new(dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, ZCanError>
        where Self: Sized;
    fn device_type(&self) -> ZCanDeviceType;
    fn device_index(&self) -> u32;
    fn open(&mut self) -> Result<(), ZCanError>;
    fn close(&mut self);
    fn device_info(&self) -> Result<&ZDeviceInfo, ZCanError>;
    fn is_derive_device(&self) -> bool;
    fn is_online(&self) -> Result<bool, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn device_name(dev_type: ZCanDeviceType, dev_idx: u32) -> String {
        format!("{}_{}", dev_type, dev_idx)
    }
    fn init_can_chl(&mut self, cfg: Vec<CanChlCfg>) -> Result<(), ZCanError>;
    fn reset_can_chl(&mut self, channel: u8) -> Result<(), ZCanError>;
    // fn resistance_state(&self, dev_idx: u32, channel: u8) -> Result<(), ZCanError>;
    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, ZCanError>;
    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, ZCanError>;
    fn clear_can_buffer(&self, channel: u8) -> Result<(), ZCanError>;
    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, ZCanError>;
    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError>;
    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, ZCanError>;
    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn init_lin_chl(&mut self, cfg: Vec<ZLinChlCfg>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn clear_lin_buffer(&self, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_lin_num(&self, channel: u8) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn wakeup_lin(&self, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn is_connected_server(&self) -> Result<bool, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn disconnect_server(&self) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn timestamp(&self, channel: u8) -> u64;
}

