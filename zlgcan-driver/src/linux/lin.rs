use zlgcan_common as common;

use log::warn;
use common::device::{ZCanDeviceType, ZlgDevice, ZLinDevice};
use common::error::ZCanError;
use common::lin::{ZLinFrame, ZLinFrameData, ZLinPublish, ZLinSubscribe, ZLinChlCfg, ZLinDataType};

use super::driver::ZCanDriver;

#[allow(deprecated)]
impl ZLinDevice for ZCanDriver<'_> {
    fn init_lin_chl(&mut self, cfg: Vec<ZLinChlCfg>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.handlers.get_mut(&dev_name) {
            Some(dev_hdl) => {
                let channels = 2;   //dev_info.lin_channels();  // TODO
                for (idx, cfg) in cfg.iter().enumerate() {
                    let idx = idx as u8;
                    if idx >= channels {
                        warn!("ZLGCAN - the length of LIN channel configuration is out of channels!");
                        break;
                    }

                    let chl_hdl: u32;

                    match self.dev_type {
                        ZCanDeviceType::ZCAN_USBCANFD_200U => {
                            if let Some(_) = dev_hdl.find_lin(idx) {
                                self.usbcanfd_api.reset_lin_chl(self.dev_type, self.dev_idx, idx).unwrap_or_else(|e| warn!("{}", e));
                                dev_hdl.remove_lin(idx);
                            }

                            chl_hdl = self.usbcanfd_api.init_lin_chl(self.dev_type, self.dev_idx, idx, cfg)?;
                        },
                        // ZCanDeviceType::ZCAN_USBCANFD_800U => {
                        //     if let Some(v) = dev_hdl.find_lin(idx) {
                        //         self.usbcanfd_800u_api.reset_lin_chl(v).unwrap_or_else(|e| warn!("{}", e));
                        //         dev_hdl.remove_lin(idx);
                        //     }
                        //
                        //     chl_hdl = self.usbcanfd_800u_api.init_lin_chl(dev_hdl.device_handler(), idx, cfg)?;
                        // },
                        _ => return Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
                    }

                    dev_hdl.add_lin(idx, chl_hdl);
                }

                Ok(())
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }
    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.handlers.get_mut(&dev_name) {
            Some(dev_hdl) => {
                match dev_hdl.find_lin(channel) {
                    Some(_v) => {
                        match self.dev_type {
                            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                                self.usbcanfd_api.reset_lin_chl(self.dev_type, self.dev_idx, channel)
                            },
                            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
                            //     self.usbcanfd_800u_api.reset_lin_chl(v)
                            // },
                            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
                        }
                    },
                    None => Err(ZCanError::new(0, format!("ZLGCAN - {} LIN channel: {} is not opened", dev_name, channel))),
                }
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),

        }
    }
    fn clear_lin_buffer(&self, channel: u8) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> Result<(), ZCanError> {
                    self.usbcanfd_api.clear_lin_buffer(self.dev_type, self.dev_idx, channel)
                })?
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> Result<(), ZCanError> {
            //         self.usbcanfd_800u_api.clear_lin_buffer(chl_hdl)
            //     })?
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    fn get_lin_num(&self, channel: u8) -> Result<u32, ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> u32 {
                    self.usbcanfd_api.get_lin_num(self.dev_type, self.dev_idx, channel)
                })
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> u32 {
            //         self.usbcanfd_800u_api.get_lin_num(chl_hdl)
            //     })
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        let timeout = timeout.unwrap_or(50);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> Vec<ZLinFrame> {
                    self.usbcanfd_api.receive_lin(self.dev_type, self.dev_idx, channel, size, timeout, |frames, size| {
                        frames.resize_with(size, || -> ZLinFrame { ZLinFrame::new(channel, ZLinDataType::Data, ZLinFrameData::from_data(Default::default())) })
                    })
                })
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> Vec<ZLinFrame> {
            //         self.usbcanfd_800u_api.receive_lin(chl_hdl, size, timeout, |frames, size| {
            //             frames.resize_with(size, || -> ZLinFrame { ZLinFrame::new(channel, ZLinDataType::Data, ZLinFrameData::from_data(Default::default())) })
            //         })
            //     })
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> u32 {
                    self.usbcanfd_api.transmit_lin(self.dev_type, self.dev_idx, channel, frames)
                })
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> u32 {
            //         self.usbcanfd_800u_api.transmit_lin(chl_hdl, frames)
            //     })
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> Result<(), ZCanError> {
                    self.usbcanfd_api.set_lin_subscribe(self.dev_type, self.dev_idx, channel, cfg)
                })?
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> Result<(), ZCanError> {
            //         self.usbcanfd_800u_api.set_lin_subscribe(chl_hdl, cfg)
            //     })?
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> Result<(), ZCanError> {
                    self.usbcanfd_api.set_lin_publish(self.dev_type, self.dev_idx, channel, cfg)
                })?
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> Result<(), ZCanError> {
            //         self.usbcanfd_800u_api.set_lin_publish(chl_hdl, cfg)
            //     })?
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    fn wakeup_lin(&self, channel: u8) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> Result<(), ZCanError> {
                    self.usbcanfd_api.wakeup_lin(self.dev_type, self.dev_idx, channel)
                })?
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> Result<(), ZCanError> {
            //         self.usbcanfd_800u_api.wakeup_lin(chl_hdl)
            //     })?
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    // #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> Result<(), ZCanError> {
                    self.usbcanfd_api.set_lin_slave_msg(self.dev_type, self.dev_idx, channel, msg)
                })?
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> Result<(), ZCanError> {
            //         self.usbcanfd_800u_api.set_lin_slave_msg(chl_hdl, msg)
            //     })?
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }
    // #[deprecated(since="0.1.0", note="This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.dev_type {
            ZCanDeviceType::ZCAN_USBCANFD_200U => {
                self.lin_handler1(channel, |channel| -> Result<(), ZCanError> {
                    self.usbcanfd_api.clear_lin_slave_msg(self.dev_type, self.dev_idx, channel, pids)
                })?
            },
            // ZCanDeviceType::ZCAN_USBCANFD_800U => {
            //     self.lin_handler(channel, |chl_hdl| -> Result<(), ZCanError> {
            //         self.usbcanfd_800u_api.clear_lin_slave_msg(chl_hdl, pids)
            //     })?
            // },
            _ => Err(ZCanError::new(0xFF, format!("ZLGCAN - {} not supported", dev_name))),
        }
    }

}

