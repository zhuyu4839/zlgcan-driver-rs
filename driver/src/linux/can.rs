use log::warn;
use common::can::CanChlCfg;
use common::can::channel::{ZCanChlError, ZCanChlStatus};
use common::can::constant::ZCanFrameType;
use common::can::frame::{ZCanFdFrame, ZCanFdFrameV1, ZCanFdFrameV2, ZCanFrame, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3};
use common::device::{ZCanDevice, ZCanDeviceType, ZlgDevice};
use common::error::ZCanError;

use super::driver::ZCanDriver;

impl ZCanDevice for ZCanDriver<'_> {
    /// Initialize a CAN channel.
    fn init_can_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, cfg: Vec<CanChlCfg>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match self.handlers.get_mut(&dev_name) {
            Some(dev_hdl) => {
                let dev_info = dev_hdl.device_info();
                let channels = dev_info.can_channels();

                for (idx, cfg) in cfg.iter().enumerate() {
                    let idx = idx as u8;
                    if idx >= channels {
                        warn!("ZLGCAN - the length of CAN channel configuration is out of channels!");
                        break;
                    }

                    let  chl_hdl: u32;
                    match dev_type {
                        ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                            if let Some(_) = dev_hdl.find_can(idx) {
                                self.usbcan_api.reset_can_chl(dev_type, dev_idx, idx).unwrap_or_else(|e| warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcan_api.init_can_chl(dev_type, dev_idx, idx, cfg).unwrap();
                        },
                        ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                            if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                                self.usbcan_4e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcan_4e_api.init_can_chl(dev_hdl.device_handler(), idx, cfg).unwrap();
                        },
                        ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                            if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                                self.usbcan_8e_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcan_8e_api.init_can_chl(dev_hdl.device_handler(), idx, cfg).unwrap();
                        },
                        ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                            if let Some(_) = dev_hdl.find_can(idx) {
                                self.usbcanfd_api.reset_can_chl(dev_type, dev_idx, idx).unwrap_or_else(|e| warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcanfd_api.init_can_chl(dev_type, dev_idx, idx, cfg).unwrap();
                        },
                        ZCanDeviceType::ZCAN_USBCANFD_800U => {
                            if let Some(chl_hdl) = dev_hdl.find_can(idx) {
                                self.usbcanfd_800u_api.reset_can_chl(chl_hdl).unwrap_or_else(|e| warn!("{}", e));
                                dev_hdl.remove_can(idx);
                            }
                            chl_hdl = self.usbcanfd_800u_api.init_can_chl(dev_hdl.device_handler(), idx, cfg).unwrap();
                        },
                        _ => return Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
                    }
                    dev_hdl.add_can(idx, chl_hdl);
                }
                Ok(())
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }

    fn reset_can_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match self.handlers.get_mut(&dev_name) {
            Some(dev_hdl) => {
                match dev_hdl.find_can(channel) {
                    Some(v) => {
                        match dev_type {
                            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                                self.usbcan_api.reset_can_chl(dev_type, dev_idx, channel).unwrap();
                            },
                            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                                self.usbcan_4e_api.reset_can_chl(v).unwrap();
                            },
                            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                                self.usbcan_8e_api.reset_can_chl(v).unwrap();
                            },
                            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                                self.usbcanfd_api.reset_can_chl(dev_type, dev_idx, channel).unwrap();
                            },
                            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                                self.usbcanfd_800u_api.reset_can_chl(v).unwrap();
                            },
                            _ => return Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
                        }
                        dev_hdl.remove_can(channel);
                        Ok(())
                    },
                    None => Err(ZCanError::new(0, format!("ZLGCAN - {} CAN channel: {} is not opened", dev_name, channel))),
                }
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }

    fn read_can_chl_status(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<ZCanChlStatus, ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Result<ZCanChlStatus, ZCanError> {
                    self.usbcan_api.read_can_chl_status(dev_type, dev_idx, channel)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<ZCanChlStatus, ZCanError> {
                    self.usbcan_4e_api.read_can_chl_status(chl_hdl)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<ZCanChlStatus, ZCanError> {
                    self.usbcan_8e_api.read_can_chl_status(chl_hdl)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Result<ZCanChlStatus, ZCanError> {
                    self.usbcanfd_api.read_can_chl_status(dev_type, dev_idx, channel)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<ZCanChlStatus, ZCanError> {
                    self.usbcanfd_800u_api.read_can_chl_status(chl_hdl)
                }).unwrap()
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

    fn read_can_chl_error(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<ZCanChlError, ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Result<ZCanChlError, ZCanError> {
                    self.usbcan_api.read_can_chl_error(dev_type, dev_idx, channel)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<ZCanChlError, ZCanError> {
                    self.usbcan_4e_api.read_can_chl_error(chl_hdl)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<ZCanChlError, ZCanError> {
                    self.usbcan_8e_api.read_can_chl_error(chl_hdl)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Result<ZCanChlError, ZCanError> {
                    self.usbcanfd_api.read_can_chl_error(dev_type, dev_idx, channel)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<ZCanChlError, ZCanError> {
                    self.usbcanfd_800u_api.read_can_chl_error(chl_hdl)
                }).unwrap()
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

    fn clear_can_buffer(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Result<(), ZCanError> {
                    self.usbcan_api.clear_can_buffer(dev_type, dev_idx, channel)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<(), ZCanError> {
                    self.usbcan_4e_api.clear_can_buffer(chl_hdl)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<(), ZCanError> {
                    self.usbcan_8e_api.clear_can_buffer(chl_hdl)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Result<(), ZCanError> {
                    self.usbcanfd_api.clear_can_buffer(dev_type, dev_idx, channel)
                }).unwrap()
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Result<(), ZCanError> {
                    self.usbcanfd_800u_api.clear_can_buffer(chl_hdl)
                }).unwrap()
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

    fn get_can_num(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> u32 {
                    self.usbcan_api.get_can_num(dev_type, dev_idx, channel, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| ->u32 {
                    self.usbcan_4e_api.get_can_num(chl_hdl, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
                    self.usbcan_8e_api.get_can_num(chl_hdl, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> u32 {
                    self.usbcanfd_api.get_can_num(dev_type, dev_idx, channel, can_type)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
                    self.usbcanfd_800u_api.get_can_num(chl_hdl, can_type)
                })
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

    fn receive_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFrame>, ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Vec<ZCanFrame> {
                    self.usbcan_api.receive_can(dev_type, dev_idx, channel, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v1(ZCanFrameV1::default()) });
                    })
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Vec<ZCanFrame> {
                    self.usbcan_4e_api.receive_can(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v3(ZCanFrameV3::default()) });
                    })
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Vec<ZCanFrame> {
                    self.usbcan_8e_api.receive_can(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v3(ZCanFrameV3::default()) });
                    })
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Vec<ZCanFrame> {
                    self.usbcanfd_api.receive_can(dev_type, dev_idx, channel, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v2(ZCanFrameV2::default()) });
                    })
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Vec<ZCanFrame> {
                    self.usbcanfd_800u_api.receive_can(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v3(ZCanFrameV3::default()) });
                    })
                })
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

    fn transmit_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFrame>) -> Result<u32, ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            ZCanDeviceType::ZCAN_USBCAN1 | ZCanDeviceType::ZCAN_USBCAN2 => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> u32 {
                    self.usbcan_api.transmit_can(dev_type, dev_idx, channel, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_4E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
                    self.usbcan_4e_api.transmit_can(chl_hdl, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCAN_8E_U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
                    self.usbcan_8e_api.transmit_can(chl_hdl, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> u32 {
                    self.usbcanfd_api.transmit_can(dev_type, dev_idx, channel, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
                    self.usbcanfd_800u_api.transmit_can(chl_hdl, frames)
                })
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

    fn receive_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFdFrame>, ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            // ZCanDeviceType::ZCAN_USBCAN_4E_U => {
            //     self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Vec<ZCanFdFrame> {
            //         self.usbcan_4e_api.receive_canfd(chl_hdl, size, timeout, |frames, size| {
            //             frames.resize_with(size, || -> ZCanFdFrame { ZCanFdFrame::from_v2(ZCanFdFrameV2::default()) });
            //         })
            //     })
            // },
            // ZCanDeviceType::ZCAN_USBCAN_8E_U => {
            //     self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Vec<ZCanFdFrame> {
            //         self.usbcan_8e_api.receive_canfd(chl_hdl, size, timeout, |frames, size| {
            //             frames.resize_with(size, || -> ZCanFdFrame { ZCanFdFrame::from_v2(ZCanFdFrameV2::default()) });
            //         })
            //     })
            // },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> Vec<ZCanFdFrame> {
                    self.usbcanfd_api.receive_canfd(dev_type, dev_idx, channel, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZCanFdFrame { ZCanFdFrame::from_v1(ZCanFdFrameV1::default()) });
                    })
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> Vec<ZCanFdFrame> {
                    self.usbcanfd_800u_api.receive_canfd(chl_hdl, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZCanFdFrame { ZCanFdFrame::from_v2(ZCanFdFrameV2::default()) });
                    })
                })
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

    fn transmit_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFdFrame>) -> Result<u32, ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match dev_type {
            // ZCanDeviceType::ZCAN_USBCAN_4E_U => {
            //     self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
            //         self.usbcan_4e_api.transmit_canfd(chl_hdl, frames)
            //     })
            // },
            // ZCanDeviceType::ZCAN_USBCAN_8E_U => {
            //     self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
            //         self.usbcan_8e_api.transmit_canfd(chl_hdl, frames)
            //     })
            // },
            ZCanDeviceType::ZCAN_USBCANFD_MINI | ZCanDeviceType::ZCAN_USBCANFD_100U | ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.can_handler1(dev_type, dev_idx, channel, |dev_type, dev_idx, channel| -> u32 {
                    self.usbcanfd_api.transmit_canfd(dev_type, dev_idx, channel, frames)
                })
            },
            ZCanDeviceType::ZCAN_USBCANFD_800U => {
                self.can_handler(dev_type, dev_idx, channel, |chl_hdl| -> u32 {
                    self.usbcanfd_800u_api.transmit_canfd(chl_hdl, frames)
                })
            },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
}

