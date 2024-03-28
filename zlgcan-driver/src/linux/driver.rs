use zlgcan_common as common;

use std::collections::HashMap;
use dlopen2::symbor::{Library, SymBorApi};
use lazy_static::lazy_static;
use log::{info, warn};
use common::device::{DeriveInfo, Handler, ZCanDeviceType, ZDeviceInfo, ZlgDevice};
use common::error::ZCanError;

use crate::constant::{LOAD_LIB_FAILED, LOAD_SYMBOLS_FAILED};
use super::api::{usbcan::*, usbcan_e::*, usbcanfd_800u::*, usbcanfd::*};

#[cfg(target_arch = "x86_64")]
lazy_static!(
    static ref USBCAN_LIB: Library = Library::open("library/linux/x86_64/libusbcan.so").expect(LOAD_LIB_FAILED);
    static ref USBCAN4E_LIB: Library = Library::open("library/linux/x86_64/libusbcan-4e.so").expect(LOAD_LIB_FAILED);
    static ref USBCAN8E_LIB: Library = Library::open("library/linux/x86_64/libusbcan-8e.so").expect(LOAD_LIB_FAILED);
    static ref USBCANFD_LIB: Library = Library::open("library/linux/x86_64/libusbcanfd.so").expect(LOAD_LIB_FAILED);
    static ref USBCANFD800U_LIB: Library = Library::open("library/linux/x86_64/libusbcanfd800u.so").expect(LOAD_LIB_FAILED);
);

pub struct ZCanDriver<'a> {
    pub(crate) handlers: HashMap<String, Handler>,
    pub(crate) usbcan_api: USBCANApi<'a>,
    pub(crate) usbcan_4e_api: USBCANEApi<'a>,
    pub(crate) usbcan_8e_api: USBCANEApi<'a>,
    pub(crate) usbcanfd_api: USBCANFDApi<'a>,
    pub(crate) usbcanfd_800u_api: USBCANFD800UApi<'a>,
}

impl ZlgDevice for ZCanDriver<'_> {
    fn new() -> Self {
        unsafe {
            let usbcan_api = USBCANApi::load(&USBCAN_LIB).expect(LOAD_SYMBOLS_FAILED);
            let usbcan_4e_api = USBCANEApi::load(&USBCAN4E_LIB).expect(LOAD_SYMBOLS_FAILED);
            let usbcan_8e_api = USBCANEApi::load(&USBCAN4E_LIB).expect(LOAD_SYMBOLS_FAILED);
            let usbcanfd_api = USBCANFDApi::load(&USBCANFD_LIB).expect(LOAD_SYMBOLS_FAILED);
            let usbcanfd_800u_api = USBCANFD800UApi::load(&USBCANFD800U_LIB).expect(LOAD_SYMBOLS_FAILED);
            let handlers = Default::default();
            Self {
                handlers,
                usbcan_api,
                usbcan_4e_api,
                usbcan_8e_api,
                usbcanfd_api,
                usbcanfd_800u_api,
            }
        }
    }

    fn open(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        let dev_hdl: u32;
        let dev_info: ZDeviceInfo;
        match dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                dev_hdl = self.usbcan_api.open(dev_type, dev_idx).unwrap();
                match derive {
                    Some(v) => {
                        dev_info = ZDeviceInfo::from(v);
                    },
                    None => dev_info = self.usbcan_api.read_device_info(dev_type, dev_idx).unwrap(),
                }
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                dev_hdl = self.usbcan_4e_api.open(dev_type, dev_idx).unwrap();
                dev_info = self.usbcan_4e_api.read_device_info(dev_hdl).unwrap();
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                dev_hdl = self.usbcan_8e_api.open(dev_type, dev_idx).unwrap();
                dev_info = self.usbcan_8e_api.read_device_info(dev_hdl).unwrap();
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                dev_hdl = self.usbcanfd_api.open(dev_type, dev_idx).unwrap();
                dev_info = self.usbcanfd_api.read_device_info(dev_type, dev_idx).unwrap();
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                dev_hdl = self.usbcanfd_800u_api.open(dev_type, dev_idx).unwrap();
                dev_info = self.usbcanfd_800u_api.read_device_info(dev_hdl).unwrap();
            },
            _ => return Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
        self.handlers.insert(dev_name, Handler::new(dev_hdl, dev_info));
        Ok(())
    }

    fn close(&mut self, dev_type: ZCanDeviceType, dev_idx: u32) {
        let dev_name = Self::device_name(dev_type, dev_idx);
        if let Some(dev_hdl) = self.handlers.get(&dev_name) {
            let cans = dev_hdl.can_channels();
            let lins = dev_hdl.lin_channels();

            match dev_type {
                ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                    for (idx, _hdl) in cans {
                        info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_api.reset_can_chl(dev_type, dev_idx, *idx).unwrap_or_else(|e| warn!("{}", e));
                    }

                    self.usbcan_api.close(dev_type, dev_idx).unwrap_or_else(|e| warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                    for (idx, hdl) in cans {
                        info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_4e_api.reset_can_chl(*hdl).unwrap_or_else(|e| warn!("{}", e));
                    }

                    self.usbcan_4e_api.close(dev_hdl.device_handler()).unwrap_or_else(|e| warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                    for (idx, hdl) in cans {
                        info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_8e_api.reset_can_chl(*hdl).unwrap_or_else(|e| warn!("{}", e));
                    }
                    self.usbcan_8e_api.close(dev_hdl.device_handler()).unwrap_or_else(|e| warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                    for (idx, _hdl) in cans {
                        info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcanfd_api.reset_can_chl(dev_type, dev_idx, *idx).unwrap_or_else(|e| warn!("{}", e));
                    }

                    for (idx, _hdl) in lins {
                        info!("ZLGCAN - closing LIN channel: {}", *idx);
                        self.usbcanfd_api.reset_lin_chl(dev_type, dev_idx, *idx).unwrap_or_else(|e| warn!("{}", e));
                    }

                    self.usbcanfd_api.close(dev_type, dev_idx).unwrap_or_else(|e| warn!("{}", e))
                },
                ZCanDeviceType::ZCAN_USBCANFD_800U => {
                    for (idx, hdl) in cans {
                        info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcanfd_800u_api.reset_can_chl(*hdl).unwrap_or_else(|e| warn!("{}", e));
                    }

                    // for (idx, hdl) in lins {
                    //     info!("ZLGCAN - closing LIN channel: {}", *idx);
                    //     self.usbcanfd_800u_api.reset_lin_chl(*hdl).unwrap_or_else(|e| warn!("{}", e));
                    // }

                    self.usbcanfd_800u_api.close(dev_hdl.device_handler()).unwrap_or_else(|e| warn!("{}", e));
                },
                _ => warn!("ZLGCAN - {} not supported!", dev_name),
            }

            self.handlers.remove(&dev_name);
        }
    }

    fn device_info(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Option<&ZDeviceInfo> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match self.handlers.get(&dev_name) {
            Some(hdr) => Some(hdr.device_info()),
            None => None,
        }
    }

    // fn is_online(&self, dev_type: ZCanDeviceType, dev_idx: u32) -> Result<bool, ZCanError> {
    //     self.device_handler(dev_type, dev_idx, |dev_hdl| -> Result<bool, ZCanError> {
    //         match dev_type {
    //             ZCanDeviceType::ZCAN_USBCANFD_800U => self.usbcanfd_800u_api.is_online(dev_hdl.device_handler()),
    //             _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`is_online`"))),
    //         }
    //     }).unwrap()
    // }
}

