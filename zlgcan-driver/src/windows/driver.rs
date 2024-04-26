use zlgcan_common as common;

use std::collections::HashMap;
use dlopen2::symbor::{Library, SymBorApi};
use lazy_static::lazy_static;
use log::{info, warn};
use common::{
    error::*,
    device::*,
};
use zlgcan_common::can::CanMessage;

use crate::constant::{LOAD_LIB_FAILED, LOAD_SYMBOLS_FAILED};
use super::api::Api;

pub struct ZCanDriver<'a> {
    pub(crate) handlers: HashMap<String, Handler>,
    pub(crate) api:      Api<'a>,
    pub(crate) dev_type: ZCanDeviceType,
    pub(crate) dev_idx:  u32,
    pub(crate) derive:   Option<DeriveInfo>,
}

#[cfg(target_arch = "x86")]
const LIB_PATH: &str = "library/windows/x86/zlgcan.dll";
#[cfg(target_arch = "x86_64")]
const LIB_PATH: &str = "library/windows/x86_64/zlgcan.dll";

lazy_static!(
    static ref LIB: Library = Library::open(LIB_PATH).expect(LOAD_LIB_FAILED);
);

impl ZlgDevice for ZCanDriver<'_> {
    fn new(dev_type: ZCanDeviceType, dev_idx: u32, derive: Option<DeriveInfo>) -> Self {
        match unsafe { Api::load(&LIB) } {
            Ok(api) => {
                let handlers = Default::default();

                Self { handlers, api, dev_type, dev_idx, derive }
            },
            Err(e) => panic!("{} {}", LOAD_SYMBOLS_FAILED, e.to_string()),
        }
    }
    fn device_type(&self) -> ZCanDeviceType {
        self.dev_type
    }

    fn device_index(&self) -> u32 {
        self.dev_idx
    }

    /// Open a device.
    /// Specify the derive information when device is derivative.
    fn open(&mut self) -> Result<(), ZCanError> {
        let value = self.api.open(self.dev_type, self.dev_idx).unwrap();
        let dev_info = match &self.derive {
            Some(v) => ZDeviceInfo::from(v),
            None => self.api.read_device_info(value).unwrap(),
        };

        let handler = Handler::new(value, dev_info);
        self.handlers.insert(Self::device_name(self.dev_type, self.dev_idx), handler);
        Ok(())
    }

    /// Close the device. Do nothing if no device opened.
    fn close(&mut self) {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        if let Some(v) = self.handlers.get(&dev_name) {
            for (idx, hdl) in v.can_channels() {
                info!("ZLGCAN - closing CAN channel: {}", *idx);
                let hdl = *hdl;
                self.api.reset_can_chl(hdl).unwrap_or_else(|e| warn!("{}", e));
            }
            for (idx, hdl) in v.lin_channels() {
                info!("ZLGCAN - closing LIN channel: {}", *idx);
                let hdl = *hdl;
                self.api.reset_lin_chl(hdl).unwrap_or_else(|e| warn!("{}", e));
            }

            self.api.close(v.device_handler()).unwrap_or_else(|e| warn!("{}", e));

            self.handlers.remove(&dev_name);
        }
    }
    fn device_info(&self) -> Option<&ZDeviceInfo> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.handlers.get(&dev_name) {
            Some(v) => Some(&v.device_info()),
            None => None,
        }
    }

    fn is_derive_device(&self) -> bool{
        self.derive.is_some()
    }

    fn is_online(&self) -> Result<bool, ZCanError> {
        self.device_handler(|hdl| -> Result<bool, ZCanError> {
            self.api.is_online(hdl.device_handler())
        }).unwrap()
    }
}

#[cfg(test)]
mod test_driver {
    use zlgcan_common as common;

    use common::device::ZCanDeviceType;
    use super::*;
    #[test]
    fn usbcanfd_200u() {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let dev_idx = 0;

        let mut driver = ZCanDriver::new(dev_type, dev_idx, None);
        driver.open().unwrap();

        let info = driver.device_info().unwrap();
        println!("{}", info.sn());
        println!("{}", info.id());

        driver.close();
    }
}

