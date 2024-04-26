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
    static ref USBCAN_LIB:       Library = Library::open("library/linux/x86_64/libusbcan.so"      ).expect(LOAD_LIB_FAILED);
    static ref USBCAN4E_LIB:     Library = Library::open("library/linux/x86_64/libusbcan-4e.so"   ).expect(LOAD_LIB_FAILED);
    static ref USBCAN8E_LIB:     Library = Library::open("library/linux/x86_64/libusbcan-8e.so"   ).expect(LOAD_LIB_FAILED);
    static ref USBCANFD_LIB:     Library = Library::open("library/linux/x86_64/libusbcanfd.so"    ).expect(LOAD_LIB_FAILED);
    static ref USBCANFD800U_LIB: Library = Library::open("library/linux/x86_64/libusbcanfd800u.so").expect(LOAD_LIB_FAILED);
);

pub struct ZCanDriver<'a> {
    pub(crate) handlers:          HashMap<String, Handler>,
    pub(crate) usbcan_api:        USBCANApi<'a>,
    pub(crate) usbcan_4e_api:     USBCANEApi<'a>,
    pub(crate) usbcan_8e_api:     USBCANEApi<'a>,
    pub(crate) usbcanfd_api:      USBCANFDApi<'a>,
    pub(crate) usbcanfd_800u_api: USBCANFD800UApi<'a>,
    pub(crate) dev_type:          ZCanDeviceType,
    pub(crate) dev_idx:           u32,
    pub(crate) derive:            Option<DeriveInfo>,
}

impl ZlgDevice for ZCanDriver<'_> {
    fn new(dev_type: ZCanDeviceType, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, ZCanError> {
        unsafe {
            let usbcan_api     = USBCANApi::load(&USBCAN_LIB).map_err(|e| {
                warn!("{:?}", e);
                ZCanError::new(0x01, LOAD_SYMBOLS_FAILED.to_string())
            })?;
            let usbcan_4e_api = USBCANEApi::load(&USBCAN4E_LIB).map_err(|e| {
                warn!("{:?}", e);
                ZCanError::new(0x01, LOAD_SYMBOLS_FAILED.to_string())
            })?;
            let usbcan_8e_api = USBCANEApi::load(&USBCAN8E_LIB).map_err(|e| {
                warn!("{:?}", e);
                ZCanError::new(0x01, LOAD_SYMBOLS_FAILED.to_string())
            })?;
            let usbcanfd_api = USBCANFDApi::load(&USBCANFD_LIB).map_err(|e| {
                warn!("{:?}", e);
                ZCanError::new(0x01, LOAD_SYMBOLS_FAILED.to_string())
            })?;
            let usbcanfd_800u_api = USBCANFD800UApi::load(&USBCANFD800U_LIB).map_err(|e| {
                warn!("{:?}", e);
                ZCanError::new(0x01, LOAD_SYMBOLS_FAILED.to_string())
            })?;
            let handlers = Default::default();
            Ok(Self {
                handlers,
                usbcan_api,
                usbcan_4e_api,
                usbcan_8e_api,
                usbcanfd_api,
                usbcanfd_800u_api,
                dev_type,
                dev_idx,
                derive
            })
        }
    }

    fn device_type(&self) -> ZCanDeviceType {
        self.dev_type
    }

    fn device_index(&self) -> u32 {
        self.dev_idx
    }

    fn open(&mut self) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        let dev_hdl: u32;
        let dev_info: ZDeviceInfo;
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                dev_hdl = self.usbcan_api.open(self.dev_type, self.dev_idx)?;
                match self.derive {
                    Some(v) => {
                        dev_info = ZDeviceInfo::from(&v);
                    },
                    None => dev_info = self.usbcan_api.read_device_info(self.dev_type, self.dev_idx)?,
                }
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                dev_hdl = self.usbcan_4e_api.open(self.dev_type, self.dev_idx)?;
                dev_info = self.usbcan_4e_api.read_device_info(dev_hdl)?;
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                dev_hdl = self.usbcan_8e_api.open(self.dev_type, self.dev_idx)?;
                dev_info = self.usbcan_8e_api.read_device_info(dev_hdl)?;
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                dev_hdl = self.usbcanfd_api.open(self.dev_type, self.dev_idx)?;
                dev_info = self.usbcanfd_api.read_device_info(self.dev_type, self.dev_idx)?;
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                dev_hdl = self.usbcanfd_800u_api.open(self.dev_type, self.dev_idx)?;
                dev_info = self.usbcanfd_800u_api.read_device_info(dev_hdl)?;
            },
            _ => return Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
        self.handlers.insert(dev_name, Handler::new(dev_hdl, dev_info));
        Ok(())
    }

    fn close(&mut self) {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        if let Some(dev_hdl) = self.handlers.get(&dev_name) {
            let cans = dev_hdl.can_channels();
            let lins = dev_hdl.lin_channels();

            match self.dev_type {
                ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                    for (idx, _hdl) in cans {
                        info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_api.reset_can_chl(self.dev_type, self.dev_idx, *idx).unwrap_or_else(|e| warn!("{}", e));
                    }

                    self.usbcan_api.close(self.dev_type, self.dev_idx).unwrap_or_else(|e| warn!("{}", e));
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
                        self.usbcanfd_api.reset_can_chl(self.dev_type, self.dev_idx, *idx).unwrap_or_else(|e| warn!("{}", e));
                    }

                    for (idx, _hdl) in lins {
                        info!("ZLGCAN - closing LIN channel: {}", *idx);
                        self.usbcanfd_api.reset_lin_chl(self.dev_type, self.dev_idx, *idx).unwrap_or_else(|e| warn!("{}", e));
                    }

                    self.usbcanfd_api.close(self.dev_type, self.dev_idx).unwrap_or_else(|e| warn!("{}", e))
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

    fn device_info(&self) -> Option<&ZDeviceInfo> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.handlers.get(&dev_name) {
            Some(hdr) => Some(hdr.device_info()),
            None => None,
        }
    }

    fn is_derive_device(&self) -> bool{
        self.derive.is_some()
    }

    // fn is_online(&self) -> Result<bool, ZCanError> {
    //     self.device_handler(|dev_hdl| -> Result<bool, ZCanError> {
    //         match dev_type {
    //             ZCanDeviceType::ZCAN_USBCANFD_800U => self.usbcanfd_800u_api.is_online(dev_hdl.device_handler()),
    //             _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} is not supported", "`is_online`"))),
    //         }
    //     })?
    // }
}

