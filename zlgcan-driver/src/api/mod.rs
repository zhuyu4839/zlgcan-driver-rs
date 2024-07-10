#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "windows")]
pub(crate) mod windows;

use std::ffi::{c_char, c_void};
use zlgcan_common::can::{CanChlCfg, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use zlgcan_common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use zlgcan_common::device::{CmdPath, IProperty, ZChannelContext, ZDeviceContext, ZDeviceInfo};
use zlgcan_common::error::ZCanError;
use zlgcan_common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[allow(unused_variables, dead_code)]
pub trait ZDeviceApi {
    fn open(&self, context: &mut ZDeviceContext) -> anyhow::Result<()>;
    fn close(&self, context: &ZDeviceContext) -> anyhow::Result<()>;
    fn read_device_info(&self, context: &ZDeviceContext) -> anyhow::Result<ZDeviceInfo>;
    fn is_online(&self, context: &ZDeviceContext) -> anyhow::Result<bool> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn get_property(&self, context: &ZChannelContext) -> anyhow::Result<IProperty> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn release_property(&self, p: &IProperty) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn get_reference(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *mut c_void) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_value(&self, context: &ZChannelContext, cmd_path: &CmdPath, value: *const c_void) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn get_value(&self, context: &ZChannelContext, cmd_path: &CmdPath) -> anyhow::Result<*const c_void> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_values(&self, context: &ZChannelContext, values: Vec<(CmdPath, *const c_char)>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn get_values(&self, context: &ZChannelContext, paths: Vec<CmdPath>) -> anyhow::Result<Vec<String>> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn debug(&self, level: u32) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
}

#[allow(unused_variables)]
pub trait ZCanApi {
    type Frame;
    type FdFrame;
    fn init_can_chl(&self, context: &mut ZChannelContext, cfg: &CanChlCfg) -> anyhow::Result<()>;
    fn reset_can_chl(&self, context: &ZChannelContext) -> anyhow::Result<()>;
    fn read_can_chl_status(&self, context: &ZChannelContext) -> anyhow::Result<ZCanChlStatus>;
    fn read_can_chl_error(&self, context: &ZChannelContext) -> anyhow::Result<ZCanChlError>;
    fn clear_can_buffer(&self, context: &ZChannelContext) -> anyhow::Result<()>;
    fn get_can_num(&self, context: &ZChannelContext, can_type: ZCanFrameType) -> anyhow::Result<u32>;
    fn receive_can(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: impl Fn(&mut Vec<Self::Frame>, usize)) -> anyhow::Result<Vec<Self::Frame>>;
    fn transmit_can(&self, context: &ZChannelContext, frames: Vec<Self::Frame>) -> anyhow::Result<u32>;
    fn receive_canfd(&self, context: &ZChannelContext, size: u32, timeout: u32, resize: fn(&mut Vec<Self::FdFrame>, usize)) -> anyhow::Result<Vec<Self::FdFrame>> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn transmit_canfd(&self, context: &ZChannelContext, frames: Vec<Self::FdFrame>) -> anyhow::Result<u32> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZLinApi {
    fn init_lin_chl(&self, context: &mut ZChannelContext, cfg: &ZLinChlCfg) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn reset_lin_chl(&self, context: &ZChannelContext) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn clear_lin_buffer(&self, context: &ZChannelContext) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn get_lin_num(&self, context: &ZChannelContext) -> anyhow::Result<u32> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn receive_lin(
        &self,
        context: &ZChannelContext,
        size: u32,
        timeout: u32,
        resize: impl Fn(&mut Vec<ZLinFrame>, usize)
    ) -> anyhow::Result<Vec<ZLinFrame>> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn transmit_lin(&self, context: &ZChannelContext, frames: Vec<ZLinFrame>) -> anyhow::Result<u32> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_lin_subscribe(&self, context: &ZChannelContext, cfg: Vec<ZLinSubscribe>)-> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_lin_publish(&self, context: &ZChannelContext, cfg: Vec<ZLinPublish>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn wakeup_lin(&self, context: &ZChannelContext) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn set_lin_publish_ex(&self, context: &ZChannelContext, cfg: Vec<ZLinPublishEx>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn set_lin_slave_msg(&self, context: &ZChannelContext, msg: Vec<ZLinFrame>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn clear_lin_slave_msg(&self, context: &ZChannelContext, pids: Vec<u8>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
}

#[allow(unused_variables, dead_code)]
pub trait ZCloudApi {
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
    fn get_userdata(&self, update: i32) -> anyhow::Result<ZCloudUserData> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
    fn receive_gps(
        &self,
        context: &ZDeviceContext,
        size: u32,
        timeout: u32,
        resize: impl Fn(&mut Vec<ZCloudGpsFrame>, usize)
    ) -> anyhow::Result<Vec<ZCloudGpsFrame>> {
        Err(anyhow::anyhow!(ZCanError::MethodNotSupported))
    }
}

