use zlgcan_common::can::{CanChlCfg, CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use zlgcan_common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use zlgcan_common::device::{DeriveInfo, Handler, ZCanDeviceType, ZCanError, ZChannelContext, ZDeviceInfo};
use zlgcan_common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::ZCanDriver;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::ZCanDriver;

#[allow(unused_variables)]
pub trait ZDevice {
    fn new(dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> anyhow::Result<Self>
        where Self: Sized;
    fn device_type(&self) -> ZCanDeviceType;
    fn device_index(&self) -> u32;
    fn open(&mut self) -> anyhow::Result<()>;
    fn close(&mut self);
    fn device_info(&self) -> anyhow::Result<&ZDeviceInfo>;
    fn is_derive_device(&self) -> bool;
    fn is_online(&self) -> anyhow::Result<bool> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn init_can_chl(&mut self, cfg: Vec<CanChlCfg>) -> anyhow::Result<()>;
    fn reset_can_chl(&mut self, channel: u8) -> anyhow::Result<()>;
    // fn resistance_state(&self, dev_idx: u32, channel: u8) -> anyhow::Result<()>;
    fn read_can_chl_status(&self, channel: u8) -> anyhow::Result<ZCanChlStatus>;
    fn read_can_chl_error(&self, channel: u8) -> anyhow::Result<ZCanChlError>;
    fn clear_can_buffer(&self, channel: u8) -> anyhow::Result<()>;
    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> anyhow::Result<u32>;
    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> anyhow::Result<Vec<CanMessage>>;
    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> anyhow::Result<u32>;
    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> anyhow::Result<Vec<CanMessage>> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> anyhow::Result<u32> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn init_lin_chl(&mut self, cfg: Vec<ZLinChlCfg>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn reset_lin_chl(&mut self, channel: u8) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn clear_lin_buffer(&self, channel: u8) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn get_lin_num(&self, channel: u8) -> anyhow::Result<u32> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> anyhow::Result<Vec<ZLinFrame>> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> anyhow::Result<u32> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn wakeup_lin(&self, channel: u8) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_server(&self, server: ZCloudServerInfo) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn connect_server(&self, username: &str, password: &str) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn is_connected_server(&self) -> anyhow::Result<bool> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn disconnect_server(&self) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn get_userdata(&self, update: Option<i32>) -> anyhow::Result<ZCloudUserData> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> anyhow::Result<Vec<ZCloudGpsFrame>> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn timestamp(&self, channel: u8) -> anyhow::Result<u64>;
    fn device_handler<C, T>(&self, callback: C) -> anyhow::Result<T>
        where
            C: FnOnce(&Handler) -> anyhow::Result<T>;
    #[inline(always)]
    fn can_handler<C, T>(&self, channel: u8, callback: C) -> anyhow::Result<T>
        where
            C: FnOnce(&ZChannelContext) -> anyhow::Result<T> {
        self.device_handler(|hdl| -> anyhow::Result<T> {
            match hdl.find_can(channel) {
                Some(context) => callback(context),
                None => Err(anyhow::anyhow!(ZCanError::ChannelNotOpened)),
            }
        })
    }

    #[inline(always)]
    fn lin_handler<C, T>(&self, channel: u8, callback: C) -> anyhow::Result<T>
        where
            C: FnOnce(&ZChannelContext) -> anyhow::Result<T> {
        self.device_handler(|hdl| -> anyhow::Result<T> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => callback(chl),
                None => Err(anyhow::anyhow!(ZCanError::ChannelNotOpened)),
            }
        })
    }
}
