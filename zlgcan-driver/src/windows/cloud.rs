use zlgcan_common as common;

use common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use common::device::ZCloudDevice;
use common::error::ZCanError;
use super::driver::ZCanDriver;

impl ZCloudDevice for ZCanDriver<'_> {
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        if self.dev_type.cloud_support() {
            self.api.set_server(server)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", self.dev_type)))
        }
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError>{
        if self.dev_type.cloud_support() {
            self.api.connect_server(username, password)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", self.dev_type)))
        }
    }
    fn is_connected_server(&self) -> Result<bool, ZCanError>{
        if self.dev_type.cloud_support() {
            Ok(self.api.is_connected_server())
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", self.dev_type)))
        }
    }
    fn disconnect_server(&self) -> Result<(), ZCanError> {
        if self.dev_type.cloud_support() {
            self.api.disconnect_server()
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", self.dev_type)))
        }
    }
    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, ZCanError> {
        if self.dev_type.cloud_support() {
            self.api.get_userdata(update.unwrap_or(0))
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", self.dev_type)))
        }
    }
    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        let timeout = timeout.unwrap_or(50);
        if self.dev_type.cloud_support() {
            self.device_handler(|hdl| -> Vec<ZCloudGpsFrame> {
                self.api.receive_gps(hdl.device_handler(), size, timeout, |frames, size| {
                    frames.resize_with(size, Default::default)
                })
            })
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", self.dev_type)))
        }
    }
}
