use std::collections::HashMap;
use log::warn;
use common::can::{CanChlCfg, constant::ZCanFrameType, channel::{ZCanChlError, ZCanChlStatus}, frame::{ZCanFdFrame, ZCanFdFrameV1, ZCanFrame}, BitrateCfg};
use common::device::{ZCanDevice, ZCanDeviceType, ZlgDevice};
use common::error::ZCanError;
use super::driver::ZCanDriver;

impl ZCanDevice for ZCanDriver<'_> {
    fn can_bitrate_cfg(&self) -> &HashMap<String, BitrateCfg> {
        &self.bitrate_cfg
    }
    
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

                    if let Some(v) = dev_hdl.find_can(idx) {
                        self.api.reset_can_chl(v).unwrap_or_else(|e| warn!("{}", e));
                        dev_hdl.remove_can(idx);
                    }

                    let chl_hdl = self.api.init_can_chl(dev_hdl.device_handler(), idx, cfg).unwrap();

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
                        self.api.reset_can_chl(v).unwrap();
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
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> Result<ZCanChlStatus, ZCanError> {
            self.api.read_can_chl_status(hdl)
        }).unwrap()
    }

    fn read_can_chl_error(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<ZCanChlError, ZCanError> {
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> Result<ZCanChlError, ZCanError> {
            self.api.read_can_chl_error(hdl)
        }).unwrap()
    }

    fn clear_can_buffer(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8) -> Result<(), ZCanError> {
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> Result<(), ZCanError> {
            self.api.clear_can_buffer(hdl)
        }).unwrap()
    }

    fn get_can_num(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, msg: ZCanFrameType) -> Result<u32, ZCanError> {
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> u32 {
            self.api.get_can_num(hdl, msg)
        })
    }

    fn receive_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFrame>, ZCanError> {
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> Vec<ZCanFrame> {
            self.api.receive_can(hdl, size, timeout, |frames, size| {
                if dev_type.is_frame_v1() {
                    frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v1(Default::default()) });
                }
                else if dev_type.is_frame_v2() {
                    frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v3(Default::default()) });
                }
                else if dev_type.is_frame_v3() {
                    frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from_v3(Default::default()) });
                }
                else {
                    panic!("ZLGCAN - receive CAN frame is not supported!");
                }
            })
        })
    }

    fn transmit_can(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFrame>) -> Result<u32, ZCanError> {
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> u32 {
            self.api.transmit_can(hdl, frames)
        })
    }

    fn receive_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFdFrame>, ZCanError> {
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> Vec<ZCanFdFrame> {
            self.api.receive_canfd(hdl, size, timeout, |frames, size| {
                frames.resize_with(size, || -> ZCanFdFrame { ZCanFdFrame::from_v1(ZCanFdFrameV1::default()) });
            })
        })
    }

    fn transmit_canfd(&self, dev_type: ZCanDeviceType, dev_idx: u32, channel: u8, frames: Vec<ZCanFdFrame>) -> Result<u32, ZCanError> {
        self.can_handler(dev_type, dev_idx, channel, |hdl| -> u32 {
            self.api.transmit_canfd(hdl, frames)
        })
    }
}

#[cfg(test)]
mod test_can {
    use common::can::CanChlCfg;
    use common::can::constant::{ZCanChlMode, ZCanChlType};
    use common::device::{ZCanDevice, ZCanDeviceType, ZlgDevice};
    use crate::driver::ZCanDriver;

    #[test]
    fn usbcanfd_200u() {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let dev_idx = 0;

        let mut driver = ZCanDriver::new();
        driver.open(dev_type, dev_idx, None).unwrap();

        let cfg1 = driver.new_can_chl_cfg(ZCanDeviceType::ZCAN_USBCANFD_200U, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default());
        let cfg2 = driver.new_can_chl_cfg(ZCanDeviceType::ZCAN_USBCANFD_200U, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default());
        let cfg = vec![cfg1, cfg2];

        driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();

        driver.close(dev_type, dev_idx);
    }
}


