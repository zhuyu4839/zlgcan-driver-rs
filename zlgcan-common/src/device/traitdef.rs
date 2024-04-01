use crate::can::{CanChlCfg, ZCanFdFrame, ZCanFrame, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use crate::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use crate::error::ZCanError;
use crate::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinSubscribe};
use super::{DeriveInfo, ZCanDeviceType, ZDeviceInfo};

#[allow(unused_variables)]
pub trait ZlgDevice {
    fn new() -> Self;
    fn open(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<(), ZCanError>;
    fn close(&mut self, dev_type: ZCanDeviceType, dev_idx: u32);
    fn device_info(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Option<&ZDeviceInfo>;
    fn is_online(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<bool, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`is_online`")))
    }
    fn device_name(dev_type: ZCanDeviceType, dev_idx: u32) -> String {
        format!("{}_{}", dev_type, dev_idx)
    }
}

#[allow(unused_variables)]
pub trait ZCanDevice {
    fn init_can_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, cfg: Vec<CanChlCfg>) -> Result<(), ZCanError>;
    fn reset_can_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError>;
    // fn resistance_state(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError>;
    fn read_can_chl_status(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<ZCanChlStatus, ZCanError>;
    fn read_can_chl_error(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<ZCanChlError, ZCanError>;
    fn clear_can_buffer(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError>;
    fn get_can_num(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, msg: ZCanFrameType) -> Result<u32, ZCanError>;
    fn receive_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFrame>, ZCanError>;
    fn transmit_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFrame>) -> Result<u32, ZCanError>;
    fn receive_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFdFrame>, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`receive_canfd`")))
    }
    fn transmit_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFdFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`transmit_canfd`")))
    }
}

#[allow(unused_variables)]
pub trait ZLinDevice {
    fn init_lin_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, cfg: Vec<ZLinChlCfg>) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`init_lin_chl`")))
    }
    fn reset_lin_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`reset_lin_chl`")))
    }
    fn clear_lin_buffer(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`clear_lin_buffer`")))
    }
    fn get_lin_num(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<u32, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`get_lin_num`")))
    }
    fn receive_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`receive_lin`")))
    }
    fn transmit_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`transmit_lin`")))
    }
    fn set_lin_subscribe(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinSubscribe>)-> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`set_lin_subscribe`")))
    }
    fn set_lin_publish(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`set_lin_publish`")))
    }
    fn wakeup_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`wakeup_lin`")))
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn set_lin_slave_msg(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`set_lin_slave_msg`")))
    }
    #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn clear_lin_slave_msg(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`set_lin_slave_msg`")))
    }
}

#[allow(unused_variables)]
pub trait ZCloudDevice {
    fn set_server(&self, dev_type: ZCanDeviceType, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`set_server`")))
    }
    fn connect_server(&self, username: &str, password: &str, dev_type: ZCanDeviceType) -> Result<(), ZCanError>{
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`connect_server`")))
    }
    fn is_connected_server(&self, dev_type: ZCanDeviceType) -> Result<bool, ZCanError>{
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`is_connected_server`")))
    }
    fn disconnect_server(&self, dev_type: ZCanDeviceType) -> Result<(), ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`disconnect_server`")))
    }
    fn get_userdata(&self, update: Option<i32>, dev_type: ZCanDeviceType) -> Result<ZCloudUserData, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`get_userdata`")))
    }
    fn receive_gps(&self, dev_type: ZCanDeviceType, dev_idx: u32, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`receive_gps`")))
    }
}
