// use std::ffi::CString;
// use log::warn;
// use common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
// use common::error::ZCanError;
// use crate::constant::STATUS_OK;
use super::USBCANFD800UApi;

impl USBCANFD800UApi<'_> {
    // #[inline(always)]
    // pub(crate) fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
    //     let http = CString::new(server.http_url).expect("");
    //     let mqtt = CString::new(server.mqtt_url).expect("");
    //     unsafe { (self.ZCLOUD_SetServerInfo)(http.as_ptr(), server.http_port, mqtt.as_ptr(), server.mqtt_port) };
    //
    //     Ok(())
    // }

    // #[inline(always)]
    // pub(crate) fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError>{
    //     let username = CString::new(username).expect("");
    //     let password = CString::new(password).expect("");
    //     match unsafe { (self.ZCLOUD_ConnectServer)(username.as_ptr(), password.as_ptr()) } {
    //         STATUS_OK => Ok(()),
    //         code=> Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`connect_server`"))),
    //     }
    // }

    // #[inline(always)]
    // pub(crate) fn is_connected_server(&self) -> bool {
    //     match unsafe { (self.ZCLOUD_IsConnected)() } {
    //         0 => true,
    //         1 => false,
    //         code => {
    //             warn!("ZLGCAN - `is_connected_server` unknown code: {}", code);
    //             false
    //         }
    //     }
    // }

    // #[inline(always)]
    // pub(crate) fn disconnect_server(&self) -> Result<(), ZCanError> {
    //     match unsafe { (self.ZCLOUD_DisconnectServer)() } {
    //         0 => Ok(()),
    //         code=> Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`disconnect_server`"))),
    //     }
    // }
    //
    // #[inline(always)]
    // pub(crate) fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, ZCanError> {
    //     unsafe {
    //         let data = (self.ZCLOUD_GetUserData)(update.unwrap_or(0));
    //         if data.is_null() {
    //             Err(ZCanError::new(0, format!("ZLGCAN - {} failed", "`get_userdata`")))
    //         }
    //         else {
    //             Ok(*data)
    //         }
    //     }
    // }
    //
    // #[inline(always)]
    // pub(crate) fn receive_gps(&self, dev_hdl: u32, size: u32, timeout: Option<u32>, resize: impl Fn(&mut Vec<ZCloudGpsFrame>, usize)) -> Vec<ZCloudGpsFrame> {
    //     let mut frames = Vec::new();
    //     resize(&mut frames, size as usize);
    //
    //     let ret = unsafe { (self.ZCLOUD_ReceiveGPS)(dev_hdl, frames.as_mut_ptr(), size, timeout.unwrap_or(50)) };
    //     if ret < size {
    //         warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
    //     }
    //     frames
    // }
}

