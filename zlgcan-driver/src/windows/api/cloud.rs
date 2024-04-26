use zlgcan_common as common;

use std::ffi::CString;
use log::warn;
use common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use common::error::ZCanError;

use crate::constant::STATUS_OK;
use super::Api;

impl Api<'_> {
    #[inline(always)]
    pub(crate) fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        unsafe { (self.ZCLOUD_SetServerInfo)(server.http_url, server.http_port, server.mqtt_url, server.mqtt_port) }

        Ok(())
    }

    #[inline(always)]
    pub(crate) fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError>{
        let username = CString::new(username).expect("");
        let password = CString::new(password).expect("");
        match unsafe { (self.ZCLOUD_ConnectServer)(username.as_ptr(), password.as_ptr()) } {
            STATUS_OK => Ok(()),
            code=> Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`connect_server`"))),
        }
    }

    #[inline(always)]
    pub(crate) fn is_connected_server(&self) -> bool {
        unsafe { (self.ZCLOUD_IsConnected)() }
    }

    #[inline(always)]
    pub(crate) fn disconnect_server(&self) -> Result<(), ZCanError> {
        match unsafe { (self.ZCLOUD_DisconnectServer)() } {
            0 => Ok(()),
            code=> Err(ZCanError::new(code, format!("ZLGCAN - {} failed", "`disconnect_server`"))),
        }
    }

    #[inline(always)]
    pub(crate) fn get_userdata(&self, update: i32) -> Result<ZCloudUserData, ZCanError> {
        unsafe {
            let data = (self.ZCLOUD_GetUserData)(update);
            if data.is_null() {
                Err(ZCanError::new(0, format!("ZLGCAN - {} failed", "`get_userdata`")))
            }
            else {
                Ok(*data)
            }
        }
    }

    #[inline(always)]
    pub(crate) fn receive_gps(&self, dev_hdl: u32, size: u32, timeout: u32, resize: impl Fn(&mut Vec<ZCloudGpsFrame>, usize)) -> Vec<ZCloudGpsFrame> {
        let mut frames = Vec::new();
        resize(&mut frames, size as usize);

        let ret = unsafe { (self.ZCLOUD_ReceiveGPS)(dev_hdl, frames.as_mut_ptr(), size, timeout) };
        if ret < size {
            warn!("ZLGCAN - receive CAN frame expect: {}, actual: {}!", size, ret);
        }
        frames
    }
}

