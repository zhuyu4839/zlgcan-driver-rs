use zlgcan_common as common;

// use common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use common::device::ZCloudDevice;
// use common::error::ZCanError;
use super::driver::ZCanDriver;

impl ZCloudDevice for ZCanDriver<'_> {
    // fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
    //     if self.dev_type.cloud_support() {
    //         self.usbcanfd_800u_api.set_server(server)
    //     }
    //     else {
    //         Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`set_server`")))
    //     }
    // }
    // fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError>{
    //     if self.dev_type.cloud_support() {
    //         self.usbcanfd_800u_api.connect_server(username, password)
    //     }
    //     else {
    //         Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`connect_server`")))
    //     }
    // }
    // fn is_connected_server(&self) -> Result<bool, ZCanError>{
    //     if self.dev_type.cloud_support() {
    //         Ok(self.usbcanfd_800u_api.is_connected_server())
    //     }
    //     else {
    //         Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`is_connected_server`")))
    //     }
    // }
    // fn disconnect_server(&self) -> Result<(), ZCanError> {
    //     if self.dev_type.cloud_support() {
    //         self.usbcanfd_800u_api.disconnect_server()
    //     }
    //     else {
    //         Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`disconnect_server`")))
    //     }
    // }
    // fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, ZCanError> {
    //     if self.dev_type.cloud_support() {
    //         self.usbcanfd_800u_api.get_userdata(update)
    //     }
    //     else {
    //         Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`get_userdata`")))
    //     }
    // }
    // fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
    //     let timeout = timeout.unwrap_or(50);
    //     if self.dev_type.cloud_support() {
    //         self.device_handler(|hdl| -> Vec<ZCloudGpsFrame> {
    //             self.usbcanfd_800u_api.receive_gps(hdl.device_handler(), size, timeout, |frames, size| {
    //                 frames.resize_with(size, Default::default)
    //             })
    //         })
    //     }
    //     else {
    //         Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`receive_gps`")))
    //     }
    // }
}
