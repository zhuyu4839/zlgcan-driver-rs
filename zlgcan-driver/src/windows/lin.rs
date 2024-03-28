use zlgcan_common as common;

use log::warn;
use common::device::{ZCanDeviceType, ZlgDevice, ZLinDevice};
use common::error::ZCanError;
use common::lin::channel::ZLinChlCfg;
use common::lin::constant::ZLinDataType;
use common::lin::frame::{ZLinFrame, ZLinFrameData, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

use super::driver::ZCanDriver;

impl ZLinDevice for ZCanDriver<'_> {
    fn init_lin_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, cfg: Vec<ZLinChlCfg>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match self.handlers.get_mut(&dev_name) {
            Some(dev_hdl) => {
                // let dev_info = dev_hdl.device_info();
                let channels = 2;   //dev_info.lin_channels();  // TODO
                for (idx, cfg) in cfg.iter().enumerate() {
                    let idx = idx as u8;
                    if idx >= channels {
                        warn!("ZLGCAN - the length of LIN channel configuration is out of channels!");
                        break;
                    }

                    if let Some(v) = dev_hdl.find_lin(idx) {
                        self.api.reset_lin_chl(v).unwrap_or_else(|e| warn!("{}", e));
                        dev_hdl.remove_lin(idx);
                    }

                    let chl_hdl = self.api.init_lin_chl(dev_hdl.device_handler(), idx, cfg).unwrap();
                    dev_hdl.add_lin(idx, chl_hdl);
                }

                Ok(())
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }
    fn reset_lin_chl(&mut self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(dev_type, dev_idx);
        match self.handlers.get_mut(&dev_name) {
            Some(dev_hdl) => {
                match dev_hdl.find_lin(channel) {
                    Some(v) => {
                        self.api.reset_lin_chl(v).unwrap();
                        dev_hdl.remove_lin(channel);
                        Ok(())
                    },
                    None => Err(ZCanError::new(0, format!("ZLGCAN - {} LIN channel: {} is not opened", dev_name, channel))),
                }
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }

    // fn clear_lin_buffer(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
    //     self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
    //         self.api.clear_lin_buffer(hdl)
    //     }).unwrap()
    // }
    
    fn get_lin_num(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<u32, ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> u32 {
            self.api.get_lin_num(hdl)
        })
    }
    fn receive_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Vec<ZLinFrame> {
            self.api.receive_lin(hdl, size, timeout, |frames, size| {
                frames.resize_with(size, || -> ZLinFrame { ZLinFrame::new(channel, ZLinDataType::Data, ZLinFrameData::from_data(Default::default())) })
            })
        })
    }
    fn transmit_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> u32 {
            self.api.transmit_lin(hdl, frames)
        })
    }
    fn set_lin_subscribe(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinSubscribe>)-> Result<(), ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
            self.api.set_lin_subscribe(hdl, cfg)
        }).unwrap()
    }
    fn set_lin_publish(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
            self.api.set_lin_publish(hdl, cfg)
        }).unwrap()
    }
    fn wakeup_lin(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
            self.api.wakeup_lin(hdl)
        }).unwrap()
    }
    #[allow(deprecated)]
    fn set_lin_slave_msg(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
            self.api.set_lin_slave_msg(hdl, msg)
        }).unwrap()
    }
    #[allow(deprecated)]
    fn clear_lin_slave_msg(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
            self.api.clear_lin_slave_msg(hdl, pids)
        }).unwrap()
    }
}

impl ZCanDriver<'_> {
    pub fn set_lin_publish_ext(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
        self.lin_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
            self.api.set_lin_publish_ex(hdl, cfg)
        }).unwrap()
    }
}

