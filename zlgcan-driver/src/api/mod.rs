#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "windows")]
pub(crate) mod windows;

use std::ffi::{c_char, c_void};
use zlgcan_common::can::{CanChlCfg, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use zlgcan_common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use zlgcan_common::device::{CmdPath, IProperty, ZCanDeviceType, ZDeviceInfo};
use zlgcan_common::error::ZCanError;
use zlgcan_common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[allow(unused_variables, dead_code)]
pub trait ZDeviceApi {
    type DeviceHandler;
    type ChannelHandler;

    fn open(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<u32, ZCanError>;
    fn close(&self, dev_hdl: Self::DeviceHandler) -> Result<(), ZCanError>;
    fn read_device_info(&self, dev_hdl: Self::DeviceHandler) -> Result<ZDeviceInfo, ZCanError>;
    fn is_online(&self, dev_hdl: Self::DeviceHandler) -> Result<bool, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_property(&self, dev_hdl: Self::DeviceHandler) -> Result<IProperty, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn release_property(&self, p: &IProperty) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_reference(&self, chl_hdl: Self::ChannelHandler, cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_reference(&self, chl_hdl: Self::ChannelHandler, cmd_path: &CmdPath, value: *mut c_void) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_value(&self, chl_hdl: Self::ChannelHandler, cmd_path: &CmdPath, value: *const c_void) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_value(&self, dev_type: ZCanDeviceType, chl_hdl: Self::ChannelHandler, cmd_path: &CmdPath) -> Result<*const c_void, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_values(&self, chl_hdl: Self::ChannelHandler, values: Vec<(CmdPath, *const c_char)>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_values(&self, chl_hdl: Self::ChannelHandler, channel: u8, paths: Vec<CmdPath>) -> Result<Vec<String>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn debug(&self, level: u32) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

#[allow(unused_variables)]
pub trait ZCanApi {
    type DeviceHandler;
    type ChannelHandler;
    type Frame;
    type FdFrame;
    fn init_can_chl(&self, dev_hdl: Self::DeviceHandler, channel: u8, cfg: &CanChlCfg) -> Result<u32, ZCanError>;
    fn reset_can_chl(&self, chl_hdl: Self::ChannelHandler) -> Result<(), ZCanError>;
    fn read_can_chl_status(&self, chl_hdl: Self::ChannelHandler) -> Result<ZCanChlStatus, ZCanError>;
    fn read_can_chl_error(&self, chl_hdl: Self::ChannelHandler) -> Result<ZCanChlError, ZCanError>;
    fn clear_can_buffer(&self, chl_hdl: Self::ChannelHandler) -> Result<(), ZCanError>;
    fn get_can_num(&self, chl_hdl: Self::ChannelHandler, can_type: ZCanFrameType) -> Result<u32, ZCanError>;
    fn receive_can(&self, chl_hdl: Self::ChannelHandler, size: u32, timeout: u32, resize: impl Fn(&mut Vec<Self::Frame>, usize)) -> Result<Vec<Self::Frame>, ZCanError>;
    fn transmit_can(&self, chl_hdl: Self::ChannelHandler, frames: Vec<Self::Frame>) -> Result<u32, ZCanError>;
    fn receive_canfd(&self, chl_hdl: Self::ChannelHandler, size: u32, timeout: u32, resize: fn(&mut Vec<Self::FdFrame>, usize)) -> Result<Vec<Self::FdFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_canfd(&self, chl_hdl: Self::ChannelHandler, frames: Vec<Self::FdFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZLinApi {
    type DeviceHandler;
    type ChannelHandler;
    fn init_lin_chl(&self, dev_hdl: Self::DeviceHandler, channel: u8, cfg: &ZLinChlCfg) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn reset_lin_chl(&self, chl_hdl: Self::ChannelHandler) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn clear_lin_buffer(&self, chl_hdl: Self::ChannelHandler) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_lin_num(&self, chl_hdl: Self::ChannelHandler) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_lin(
        &self,
        chl_hdl: Self::ChannelHandler,
        size: u32,
        timeout: u32,
        resize: impl Fn(&mut Vec<ZLinFrame>, usize)
    ) -> Result<Vec<ZLinFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_lin(&self, chl_hdl: Self::ChannelHandler, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_subscribe(&self, chl_hdl: Self::ChannelHandler, cfg: Vec<ZLinSubscribe>)-> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish(&self, chl_hdl: Self::ChannelHandler, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn wakeup_lin(&self, chl_hdl: Self::ChannelHandler) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish_ex(&self, chl_hdl: Self::ChannelHandler, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn set_lin_slave_msg(&self, chl_hdl: Self::ChannelHandler, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn clear_lin_slave_msg(&self, chl_hdl: Self::ChannelHandler, pids: Vec<u8>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZCloudApi {
    type DeviceHandler;
    type ChannelHandler;
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
    fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_gps(
        &self,
        dev_hdl: Self::DeviceHandler,
        size: u32,
        timeout: u32,
        resize: impl Fn(&mut Vec<ZCloudGpsFrame>, usize)
    ) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
}

