use common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use common::device::{ZCanDeviceType, ZCloudDevice};
use common::error::ZCanError;
use super::driver::ZCanDriver;

impl ZCloudDevice for ZCanDriver<'_> {
    fn set_server(&self, dev_type: ZCanDeviceType, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        if dev_type.cloud_support() {
            self.usbcanfd_800u_api.set_server(server)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported!", "`set_server`")))
        }
    }
    fn connect_server(&self, username: &str, password: &str, dev_type: ZCanDeviceType) -> Result<(), ZCanError>{
        if dev_type.cloud_support() {
            self.usbcanfd_800u_api.connect_server(username, password)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported!", "`connect_server`")))
        }
    }
    fn is_connected_server(&self, dev_type: ZCanDeviceType) -> Result<bool, ZCanError>{
        if dev_type.cloud_support() {
            Ok(self.usbcanfd_800u_api.is_connected_server())
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported!", "`is_connected_server`")))
        }
    }
    fn disconnect_server(&self, dev_type: ZCanDeviceType) -> Result<(), ZCanError> {
        if dev_type.cloud_support() {
            self.usbcanfd_800u_api.disconnect_server()
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported!", "`disconnect_server`")))
        }
    }
    fn get_userdata(&self, update: Option<i32>, dev_type: ZCanDeviceType) -> Result<ZCloudUserData, ZCanError> {
        if dev_type.cloud_support() {
            self.usbcanfd_800u_api.get_userdata(update)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported!", "`get_userdata`")))
        }
    }
    fn receive_gps(&self, dev_type: ZCanDeviceType, dev_idx: u32, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        if dev_type.cloud_support() {
            self.device_handler(dev_type, dev_idx, |hdl| -> Vec<ZCloudGpsFrame> {
                self.usbcanfd_800u_api.receive_gps(hdl.device_handler(), size, timeout, |frames, size| {
                    frames.resize_with(size, Default::default)
                })
            })
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported!", "`receive_gps`")))
        }
    }
}
