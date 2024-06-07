use std::collections::HashMap;
use dlopen2::symbor::{Library, SymBorApi};
use lazy_static::lazy_static;
use zlgcan_common::can::{CanChlCfg, CanMessage, ZCanChlError, ZCanChlStatus, ZCanFdFrameV1, ZCanFdFrameV2, ZCanFrameType, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3};
use zlgcan_common::device::{DeriveInfo, Handler, ZCanDeviceType, ZCanError, ZDeviceInfo};
use zlgcan_common::lin::{ZLinChlCfg, ZLinDataType, ZLinFrame, ZLinFrameData, ZLinPublish, ZLinSubscribe};
use zlgcan_common::TryFromIterator;
use zlgcan_common::utils::system_timestamp;
use crate::api::linux::usbcan::USBCANApi;
use crate::api::linux::usbcan_e::USBCANEApi;
use crate::api::linux::usbcanfd::USBCANFDApi;
use crate::api::linux::usbcanfd_800u::USBCANFD800UApi;
use crate::api::{ZCanApi, ZDeviceApi, ZLinApi};
use crate::constant::LOAD_LIB_FAILED;
use crate::driver::ZDevice;

#[cfg(target_arch = "x86_64")]
lazy_static!(
    static ref USBCAN_LIB:       Library = Library::open("library/linux/x86_64/libusbcan.so"      ).expect(LOAD_LIB_FAILED);
    static ref USBCAN4E_LIB:     Library = Library::open("library/linux/x86_64/libusbcan-4e.so"   ).expect(LOAD_LIB_FAILED);
    static ref USBCAN8E_LIB:     Library = Library::open("library/linux/x86_64/libusbcan-8e.so"   ).expect(LOAD_LIB_FAILED);
    static ref USBCANFD_LIB:     Library = Library::open("library/linux/x86_64/libusbcanfd.so"    ).expect(LOAD_LIB_FAILED);
    static ref USBCANFD800U_LIB: Library = Library::open("library/linux/x86_64/libusbcanfd800u.so").expect(LOAD_LIB_FAILED);
);

pub struct ZCanDriver<'a> {
    pub(crate) handler:           Option<Handler>,
    pub(crate) usbcan_api:        USBCANApi<'a>,
    pub(crate) usbcan_4e_api:     USBCANEApi<'a>,
    pub(crate) usbcan_8e_api:     USBCANEApi<'a>,
    pub(crate) usbcanfd_api:      USBCANFDApi<'a>,
    pub(crate) usbcanfd_800u_api: USBCANFD800UApi<'a>,
    pub(crate) dev_type:          ZCanDeviceType,
    pub(crate) dev_idx:           u32,
    pub(crate) derive:            Option<DeriveInfo>,
    pub(crate) timestamps:        HashMap<u8, u64>,
}

impl ZDevice for ZCanDriver<'_> {
    fn new(dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, ZCanError> where Self: Sized {
        unsafe {
            let usbcan_api     = USBCANApi::load(&USBCAN_LIB).map_err(|e| ZCanError::LibraryLoadFailed(e.to_string()))?;
            let usbcan_4e_api = USBCANEApi::load(&USBCAN4E_LIB).map_err(|e| ZCanError::LibraryLoadFailed(e.to_string()))?;
            let usbcan_8e_api = USBCANEApi::load(&USBCAN8E_LIB).map_err(|e| ZCanError::LibraryLoadFailed(e.to_string()))?;
            let usbcanfd_api = USBCANFDApi::load(&USBCANFD_LIB).map_err(|e| ZCanError::LibraryLoadFailed(e.to_string()))?;
            let usbcanfd_800u_api = USBCANFD800UApi::load(&USBCANFD800U_LIB).map_err(|e| ZCanError::LibraryLoadFailed(e.to_string()))?;
            let dev_type = ZCanDeviceType::try_from(dev_type)?;
            Ok(Self {
                handler: Default::default(),
                usbcan_api, usbcan_4e_api, usbcan_8e_api, usbcanfd_api, usbcanfd_800u_api,
                dev_type, dev_idx, derive,
                timestamps: Default::default(),
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
        let dev_hdl: u32;
        let dev_info: ZDeviceInfo;
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                dev_hdl = self.usbcan_api.open(self.dev_type, self.dev_idx)?;
                match self.derive {
                    Some(v) => {
                        dev_info = ZDeviceInfo::try_from(&v)?;
                    },
                    None => dev_info = self.usbcan_api.read_device_info((self.dev_type, self.dev_idx))?,
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
                dev_info = self.usbcanfd_api.read_device_info((self.dev_type, self.dev_idx))?;
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                dev_hdl = self.usbcanfd_800u_api.open(self.dev_type, self.dev_idx)?;
                dev_info = self.usbcanfd_800u_api.read_device_info(dev_hdl)?;
            },
            _ => return Err(ZCanError::DeviceNotSupported),
        };
        self.handler = Some(Handler::new(dev_hdl, dev_info));
        Ok(())
    }

    fn close(&mut self) {
        if let Some(dev_hdl) = &mut self.handler {
            let cans = dev_hdl.can_channels();
            let lins = dev_hdl.lin_channels();

            match self.dev_type {
                ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                    for (idx, _hdl) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_api.reset_can_chl((self.dev_type, self.dev_idx, *idx)).unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcan_api.close((self.dev_type, self.dev_idx)).unwrap_or_else(|e| log::warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                    for (idx, hdl) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_4e_api.reset_can_chl(*hdl).unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcan_4e_api.close(dev_hdl.device_handler()).unwrap_or_else(|e| log::warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                    for (idx, hdl) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcan_8e_api.reset_can_chl(*hdl).unwrap_or_else(|e| log::warn!("{}", e));
                    }
                    self.usbcan_8e_api.close(dev_hdl.device_handler()).unwrap_or_else(|e| log::warn!("{}", e));
                },
                ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                    for (idx, _hdl) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcanfd_api.reset_can_chl((self.dev_type, self.dev_idx, *idx)).unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    for (idx, _hdl) in lins {
                        log::info!("ZLGCAN - closing LIN channel: {}", *idx);
                        self.usbcanfd_api.reset_lin_chl((self.dev_type, self.dev_idx, *idx)).unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcanfd_api.close((self.dev_type, self.dev_idx)).unwrap_or_else(|e| log::warn!("{}", e))
                },
                ZCanDeviceType::ZCAN_USBCANFD_800U => {
                    for (idx, hdl) in cans {
                        log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                        self.usbcanfd_800u_api.reset_can_chl(*hdl).unwrap_or_else(|e| log::warn!("{}", e));
                    }

                    self.usbcanfd_800u_api.close(dev_hdl.device_handler()).unwrap_or_else(|e| log::warn!("{}", e));
                },
                _ => log::warn!("{:?}", ZCanError::DeviceNotSupported),
            }
            self.handler = None;
        }
    }

    fn device_info(&self) -> Result<&ZDeviceInfo, ZCanError> {
        match &self.handler {
            Some(v) => Ok(v.device_info()),
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn is_derive_device(&self) -> bool {
        self.derive.is_some()
    }

    fn init_can_chl(&mut self, cfg: Vec<CanChlCfg>) -> Result<(), ZCanError> {
        match &mut self.handler {
            Some(dev_hdl) => {
                let dev_info = dev_hdl.device_info();
                let channels = dev_info.can_channels();

                if self.dev_type == ZCanDeviceType::ZCAN_USBCAN_4E_U {
                    return self.usbcan_4e_api.init_can_chl_ex(dev_hdl, channels, &cfg, &mut self.timestamps);
                }

                for (idx, cfg) in cfg.iter().enumerate() {
                    let idx = idx as u8;
                    if idx >= channels {
                        log::warn!("ZLGCAN - the length of CAN channel configuration is out of channels!");
                        break;
                    }

                    let  chl_hdl: u32;
                    match self.dev_type {
                        ZCanDeviceType::ZCAN_USBCAN1
                        | ZCanDeviceType::ZCAN_USBCAN2 => {
                            if let Some(_) = dev_hdl.find_can(idx) {
                                self.usbcan_api.reset_can_chl((self.dev_type, self.dev_idx, idx)).unwrap_or_else(|e| log::warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcan_api.init_can_chl((self.dev_type, self.dev_idx), idx, cfg)?;
                        },
                        // ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                        //     if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                        //         self.usbcan_4e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                        //         dev_hdl.remove_can(idx);
                        //     }
                        //     chl_hdl = self.usbcan_4e_api.init_can_chl(dev_hdl.device_handler(), idx, cfg)?;
                        // },
                        ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                            if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                                self.usbcan_8e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcan_8e_api.init_can_chl(dev_hdl.device_handler(), idx, cfg)?;
                        },
                        ZCanDeviceType::ZCAN_USBCANFD_MINI
                        | ZCanDeviceType::ZCAN_USBCANFD_100U
                        | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                            if let Some(_) = dev_hdl.find_can(idx) {
                                self.usbcanfd_api.reset_can_chl((self.dev_type, self.dev_idx, idx))?;
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcanfd_api.init_can_chl((self.dev_type, self.dev_idx), idx, cfg)?;
                        },
                        ZCanDeviceType::ZCAN_USBCANFD_800U => {
                            if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                                self.usbcanfd_800u_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| log::warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            self.usbcanfd_800u_api.init_can_chl_ex(self.dev_type, self.dev_idx, idx, cfg)?;
                            chl_hdl = self.usbcanfd_800u_api.init_can_chl(dev_hdl.device_handler(), idx, cfg)?;
                        },
                        _ => return Err(ZCanError::DeviceNotSupported),
                    }

                    self.timestamps.insert(idx, system_timestamp());
                    dev_hdl.add_can(idx, chl_hdl);
                }
                Ok(())
            },
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn reset_can_chl(&mut self, channel: u8) -> Result<(), ZCanError> {
        match &mut self.handler {
            Some(dev_hdl) => {
                match dev_hdl.find_can(channel) {
                    Some(v) => {
                        match self.dev_type {
                            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                                self.usbcan_api.reset_can_chl((self.dev_type, self.dev_idx, channel))?;
                            },
                            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                                self.usbcan_4e_api.reset_can_chl(v)?;
                            },
                            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                                self.usbcan_8e_api.reset_can_chl(v)?;
                            },
                            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                                self.usbcanfd_api.reset_can_chl((self.dev_type, self.dev_idx, channel))?;
                            },
                            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                                self.usbcanfd_800u_api.reset_can_chl(v)?;
                            },
                            _ => return Err(ZCanError::DeviceNotSupported),
                        }
                        dev_hdl.remove_can(channel);
                        Ok(())
                    },
                    None => Err(ZCanError::ChannelNotOpened),
                }
            },
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(channel, |channel| {
                    self.usbcan_api.read_can_chl_status((self.dev_type, self.dev_idx, channel))
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_4e_api.read_can_chl_status(chl_hdl)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_8e_api.read_can_chl_status(chl_hdl)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.read_can_chl_status((self.dev_type, self.dev_idx, channel))
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.read_can_chl_status(chl_hdl)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(channel, |channel| {
                    self.usbcan_api.read_can_chl_error((self.dev_type, self.dev_idx, channel))
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_4e_api.read_can_chl_error(chl_hdl)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_8e_api.read_can_chl_error(chl_hdl)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.read_can_chl_error((self.dev_type, self.dev_idx, channel))
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.read_can_chl_error(chl_hdl)
                })
            },
            _ => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn clear_can_buffer(&self, channel: u8) -> Result<(), ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(channel, |channel| {
                    self.usbcan_api.clear_can_buffer((self.dev_type, self.dev_idx, channel))
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_4e_api.clear_can_buffer(chl_hdl)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_8e_api.clear_can_buffer(chl_hdl)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.clear_can_buffer((self.dev_type, self.dev_idx, channel))
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.clear_can_buffer(chl_hdl)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(channel, |channel| {
                    self.usbcan_api.get_can_num((self.dev_type, self.dev_idx, channel), can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_4e_api.get_can_num(chl_hdl, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_8e_api.get_can_num(chl_hdl, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.get_can_num((self.dev_type, self.dev_idx, channel), can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.get_can_num(chl_hdl, can_type)
                })
            },
            _ => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError> {
        let timeout = timeout.unwrap_or(0xFFFFFFFF);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                let results = self.can_handler1(channel, |channel| {
                    self.usbcan_api.receive_can((self.dev_type, self.dev_idx, channel), size, timeout, |frames, size| {
                        frames.resize_with(size, ZCanFrameV1::default);
                    })
                })?;

                Vec::try_from_iter(results, self.timestamp(channel))
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                let results = self.can_handler(channel, |chl_hdl| {
                    self.usbcan_4e_api.receive_can(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, ZCanFrameV3::default);
                    })
                })?;

                Vec::try_from_iter(results, self.timestamp(channel))
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                let results = self.can_handler(channel, |chl_hdl| {
                    self.usbcan_8e_api.receive_can(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, ZCanFrameV3::default);
                    })
                })?;

                Vec::try_from_iter(results, self.timestamp(channel))
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                let results = self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.receive_can((self.dev_type, self.dev_idx, channel), size, timeout, |frames, size| {
                        frames.resize_with(size, ZCanFrameV2::default);
                    })
                })?;

                Vec::try_from_iter(results, self.timestamp(channel))
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                let results = self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.receive_can(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, ZCanFrameV3::default);
                    })
                })?;

                Vec::try_from_iter(results, self.timestamp(channel))
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                let frames = frames.into_iter()
                    .map(ZCanFrameV1::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                self.can_handler1(channel, |channel| {
                    self.usbcan_api.transmit_can((self.dev_type, self.dev_idx, channel), frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                let frames = frames.into_iter()
                    .map(ZCanFrameV3::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_4e_api.transmit_can(chl_hdl, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                let frames = frames.into_iter()
                    .map(ZCanFrameV3::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                self.can_handler(channel, |chl_hdl| {
                    self.usbcan_8e_api.transmit_can(chl_hdl, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                let frames = frames.into_iter()
                    .map(ZCanFrameV2::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.transmit_can((self.dev_type, self.dev_idx, channel), frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                let frames = frames.into_iter()
                    .map(ZCanFrameV3::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.transmit_can(chl_hdl, frames)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError> {
        let timeout = timeout.unwrap_or(0xFFFFFFFF);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                let results = self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.receive_canfd((self.dev_type, self.dev_idx, channel), size, timeout, |frames, size| {
                        frames.resize_with(size, ZCanFdFrameV1::default);
                    })
                })?;

                Vec::try_from_iter(results, self.timestamp(channel))
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                let results = self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.receive_canfd(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, ZCanFdFrameV2::default);
                    })
                })?;

                Vec::try_from_iter(results, self.timestamp(channel))
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                let frames = frames.into_iter()
                    .map(ZCanFdFrameV1::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                self.can_handler1(channel, |channel| {
                    self.usbcanfd_api.transmit_canfd((self.dev_type, self.dev_idx, channel), frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                let frames = frames.into_iter()
                    .map(ZCanFdFrameV2::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                self.can_handler(channel, |chl_hdl| {
                    self.usbcanfd_800u_api.transmit_canfd(chl_hdl, frames)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn init_lin_chl(&mut self, cfg: Vec<ZLinChlCfg>) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::DeviceNotSupported)
        }
        match &mut self.handler {
            Some(dev_hdl) => {
                let channels = 2;   //dev_info.lin_channels();  // TODO
                for (idx, cfg) in cfg.iter().enumerate() {
                    let idx = idx as u8;
                    if idx >= channels {
                        log::warn!("ZLGCAN - the length of LIN channel configuration is out of channels!");
                        break;
                    }

                    let chl_hdl: u32;

                    match self.dev_type {
                        ZCanDeviceType::ZCAN_USBCANFD_200U => {
                            if let Some(_) = dev_hdl.find_lin(idx) {
                                self.usbcanfd_api.reset_lin_chl((self.dev_type, self.dev_idx, idx))?;
                                dev_hdl.remove_lin(idx);
                            }

                            chl_hdl = self.usbcanfd_api.init_lin_chl((self.dev_type, self.dev_idx), idx, cfg)?;
                        },
                        _ => return Err(ZCanError::DeviceNotSupported),
                    }

                    dev_hdl.add_lin(idx, chl_hdl);
                }

                Ok(())
            },
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::DeviceNotSupported)
        }
        match &mut self.handler {
            Some(dev_hdl) => {
                match dev_hdl.find_lin(channel) {
                    Some(_v) => {
                        match self.dev_type {
                            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                                self.usbcanfd_api.reset_lin_chl((self.dev_type, self.dev_idx, channel))
                            },
                            _ => Err(ZCanError::DeviceNotSupported),
                        }
                    },
                    None => Err(ZCanError::ChannelNotOpened),
                }
            },
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn clear_lin_buffer(&self, channel: u8) -> Result<(), ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.clear_lin_buffer((self.dev_type, self.dev_idx, channel))
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn get_lin_num(&self, channel: u8) -> Result<u32, ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.get_lin_num((self.dev_type, self.dev_idx, channel))
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, ZCanError> {
        let timeout = timeout.unwrap_or(0xFFFFFFFF);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.receive_lin(
                        (self.dev_type, self.dev_idx, channel),
                        size,
                        timeout,
                        |frames, size| {
                            frames.resize_with(size, || ZLinFrame::new(channel, ZLinDataType::TypeData, ZLinFrameData::from_data(Default::default())))
                        })
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.transmit_lin((self.dev_type, self.dev_idx, channel), frames)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.set_lin_subscribe((self.dev_type, self.dev_idx, channel), cfg)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.set_lin_publish((self.dev_type, self.dev_idx, channel), cfg)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    fn wakeup_lin(&self, channel: u8) -> Result<(), ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.wakeup_lin((self.dev_type, self.dev_idx, channel))
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    #[allow(deprecated)]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.set_lin_slave_msg((self.dev_type, self.dev_idx, channel), msg)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    #[allow(deprecated)]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| {
                    self.usbcanfd_api.clear_lin_slave_msg((self.dev_type, self.dev_idx, channel), pids)
                })
            },
            _ => Err(ZCanError::DeviceNotSupported),
        }
    }

    #[inline]
    fn timestamp(&self, channel: u8) -> u64 {
        match self.timestamps.get(&channel) {
            Some(v) => *v,
            None => 0,
        }
    }
}

