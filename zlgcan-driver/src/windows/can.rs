use zlgcan_common as common;

use log::warn;
use common::can::{
    CanChlCfg, ZCanFrameType, {ZCanChlError, ZCanChlStatus},
    ZCanFdFrame, ZCanFdFrameV1, ZCanFrame, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3
};
use common::device::{ZCanDevice, ZlgDevice};
use common::error::ZCanError;
use super::driver::ZCanDriver;

impl ZCanDevice for ZCanDriver<'_> {
    fn init_can_chl(&mut self, cfg: Vec<CanChlCfg>) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
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

                    let chl_hdl = self.api.init_can_chl(dev_hdl.device_handler(), idx, cfg)?;

                    dev_hdl.add_can(idx, chl_hdl);
                }
                Ok(())
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }

    fn reset_can_chl(&mut self, channel: u8) -> Result<(), ZCanError> {
        let dev_name = Self::device_name(self.dev_type, self.dev_idx);
        match self.handlers.get_mut(&dev_name) {
            Some(dev_hdl) => {
                match dev_hdl.find_can(channel) {
                    Some(v) => {
                        self.api.reset_can_chl(v)?;
                        dev_hdl.remove_can(channel);
                        Ok(())
                    },
                    None => Err(ZCanError::new(0, format!("ZLGCAN - {} CAN channel: {} is not opened", dev_name, channel))),
                }
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not opened", dev_name))),
        }
    }

    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, ZCanError> {
        self.can_handler(channel, |hdl| -> Result<ZCanChlStatus, ZCanError> {
            self.api.read_can_chl_status(hdl)
        })?
    }

    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, ZCanError> {
        self.can_handler(channel, |hdl| -> Result<ZCanChlError, ZCanError> {
            self.api.read_can_chl_error(hdl)
        })?
    }

    fn clear_can_buffer(&self, channel: u8) -> Result<(), ZCanError> {
        self.can_handler(channel, |hdl| -> Result<(), ZCanError> {
            self.api.clear_can_buffer(hdl)
        })?
    }

    fn get_can_num(&self, channel: u8, msg: ZCanFrameType) -> Result<u32, ZCanError> {
        self.can_handler(channel, |hdl| -> u32 {
            self.api.get_can_num(hdl, msg)
        })
    }

    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFrame>, ZCanError> {
        let timeout = timeout.unwrap_or(50);
        self.can_handler(channel, |hdl| -> Vec<ZCanFrame> {
            self.api.receive_can(hdl, size, timeout, |frames, size| {
                if self.dev_type.is_frame_v1() {
                    frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from(ZCanFrameV1::default()) });
                }
                else if self.dev_type.is_frame_v2() {
                    frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from(ZCanFrameV2::default()) });
                }
                else if self.dev_type.is_frame_v3() {
                    frames.resize_with(size, || -> ZCanFrame { ZCanFrame::from(ZCanFrameV3::default()) });
                }
                else {
                    panic!("ZLGCAN - receive CAN frame is not supported!");
                }
            })
        })
    }

    fn transmit_can(&self, channel: u8, frames: Vec<ZCanFrame>) -> Result<u32, ZCanError> {
        self.can_handler(channel, |hdl| -> u32 {
            self.api.transmit_can(hdl, frames)
        })
    }

    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFdFrame>, ZCanError> {
        let timeout = timeout.unwrap_or(50);
        self.can_handler(channel, |hdl| -> Vec<ZCanFdFrame> {
            self.api.receive_canfd(hdl, size, timeout, |frames, size| {
                frames.resize_with(size, || -> ZCanFdFrame { ZCanFdFrame::from(ZCanFdFrameV1::default()) });
            })
        })
    }

    fn transmit_canfd(&self, channel: u8, frames: Vec<ZCanFdFrame>) -> Result<u32, ZCanError> {
        self.can_handler(channel, |hdl| -> u32 {
            self.api.transmit_canfd(hdl, frames)
        })
    }
}

#[cfg(test)]
mod test_can {
    use zlgcan_common as common;

    use common::can::{CanChlCfgFactory, ZCanChlMode, ZCanChlType};
    use common::device::{ZCanDevice, ZCanDeviceType, ZlgDevice};
    use crate::ZCanDriver;

    #[test]
    fn usbcanfd_200u() {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let dev_idx = 0;

        let mut driver = ZCanDriver::new(dev_type, dev_idx, None);
        driver.open().unwrap();

        let factory = CanChlCfgFactory::new();

        let cfg1 = factory.new_can_chl_cfg(ZCanDeviceType::ZCAN_USBCANFD_200U, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
        let cfg2 = factory.new_can_chl_cfg(ZCanDeviceType::ZCAN_USBCANFD_200U, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
        let cfg = vec![cfg1, cfg2];

        driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();

        driver.close(dev_type, dev_idx);
    }
}


