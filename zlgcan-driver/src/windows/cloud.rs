use zlgcan_common as common;

use common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use common::device::{ZCanDeviceType, ZCloudDevice};
use common::error::ZCanError;
use super::driver::ZCanDriver;

impl ZCloudDevice for ZCanDriver<'_> {
    fn set_server(&self, dev_type: ZCanDeviceType, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        if dev_type.cloud_support() {
            self.api.set_server(server)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", dev_type)))
        }
    }
    fn connect_server(&self, username: &str, password: &str, dev_type: ZCanDeviceType) -> Result<(), ZCanError>{
        if dev_type.cloud_support() {
            self.api.connect_server(username, password)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", dev_type)))
        }
    }
    fn is_connected_server(&self, dev_type: ZCanDeviceType) -> Result<bool, ZCanError>{
        if dev_type.cloud_support() {
            Ok(self.api.is_connected_server())
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", dev_type)))
        }
    }
    fn disconnect_server(&self, dev_type: ZCanDeviceType) -> Result<(), ZCanError> {
        if dev_type.cloud_support() {
            self.api.disconnect_server()
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", dev_type)))
        }
    }
    fn get_userdata(&self, update: Option<i32>, dev_type: ZCanDeviceType) -> Result<ZCloudUserData, ZCanError> {
        if dev_type.cloud_support() {
            self.api.get_userdata(update)
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", dev_type)))
        }
    }
    fn receive_gps(&self, dev_type: ZCanDeviceType, dev_idx: u32, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        if dev_type.cloud_support() {
            self.device_handler(dev_type, dev_idx, |hdl| -> Vec<ZCloudGpsFrame> {
                self.api.receive_gps(hdl.device_handler(), size, timeout, |frames, size| {
                    frames.resize_with(size, Default::default)
                })
            })
        }
        else {
            Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", dev_type)))
        }
    }
}
