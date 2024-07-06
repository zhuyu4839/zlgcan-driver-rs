use std::future::Future;
use can_type_rs::{CanDeviceAsync, CanDeviceSync};
use zlgcan_common::can::{CanChlCfg, CanMessage, ZCanChlError, ZCanChlStatus, ZCanFrameType};
use zlgcan_common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use zlgcan_common::device::{DeriveInfo, Handler, ZCanDeviceType, ZCanError, ZChannelContext, ZDeviceInfo};
use zlgcan_common::lin::{ZLinChlCfg, ZLinFrame, ZLinPublish, ZLinPublishEx, ZLinSubscribe};

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use windows::ZCanDriver;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::ZCanDriver;

impl CanDeviceSync for ZCanDriver<'_> {
    type Error = ZCanError;
    type Frame = Vec<CanMessage>;
    type Channel = u8;

    fn transmit_sync(&self, channel: Self::Channel, frames: Self::Frame, canfd: bool, _: Option<usize>) -> Result<usize, Self::Error> {
        let size = if canfd {
            self.transmit_canfd(channel, frames)
        }
        else {
            self.transmit_can(channel, frames)
        }?;

        log::info!("Can device transmit frame: {}", size);
        Ok(size as usize)
    }

    fn receive_sync(&self, channel: Self::Channel, canfd: bool, timeout: Option<usize>) -> Result<Self::Frame, Self::Error> {
        let timeout = match timeout {
            Some(v) => Some(v as u32),
            None => None
        };

        if canfd {
            if self.dev_type.canfd_support() {
                let count_fd = self.get_can_num(channel, ZCanFrameType::CANFD)?;
                if count_fd > 0 {
                    return self.receive_canfd(channel, count_fd, timeout);
                }
            }
        }

        let count_can = self.get_can_num(channel, ZCanFrameType::CAN)?;
        if count_can > 0 {
            return self.receive_can(channel, count_can, timeout)
        }

        Err(Self::Error::NoMessageReceived)
    }
}

impl CanDeviceAsync for ZCanDriver<'_> {
    type Error = ZCanError;
    type Frame = Vec<CanMessage>;
    type Channel = u8;

    #[inline]
    fn transmit_async(&self, channel: Self::Channel, frame: Self::Frame, canfd: bool, timeout: Option<usize>)
        -> impl Future<Output=Result<usize, Self::Error>> {
        async move {
            self.transmit_sync(channel, frame, canfd, timeout)
        }
    }

    #[inline]
    fn receive_async(&self, channel: Self::Channel, canfd: bool, timeout: Option<usize>)
        -> impl Future<Output=Result<Self::Frame, Self::Error>> {
        async move {
            self.receive_sync(channel, canfd, timeout)
        }
    }
}

#[allow(unused_variables)]
pub trait ZDevice {
    fn new(dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, ZCanError>
        where Self: Sized;
    fn device_type(&self) -> ZCanDeviceType;
    fn device_index(&self) -> u32;
    fn open(&mut self) -> Result<(), ZCanError>;
    fn close(&mut self);
    fn device_info(&self) -> Result<&ZDeviceInfo, ZCanError>;
    fn is_derive_device(&self) -> bool;
    fn is_online(&self) -> Result<bool, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn init_can_chl(&mut self, cfg: Vec<CanChlCfg>) -> Result<(), ZCanError>;
    fn reset_can_chl(&mut self, channel: u8) -> Result<(), ZCanError>;
    // fn resistance_state(&self, dev_idx: u32, channel: u8) -> Result<(), ZCanError>;
    fn read_can_chl_status(&self, channel: u8) -> Result<ZCanChlStatus, ZCanError>;
    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, ZCanError>;
    fn clear_can_buffer(&self, channel: u8) -> Result<(), ZCanError>;
    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, ZCanError>;
    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError>;
    fn transmit_can(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, ZCanError>;
    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_canfd(&self, channel: u8, frames: Vec<CanMessage>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn init_lin_chl(&mut self, cfg: Vec<ZLinChlCfg>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn clear_lin_buffer(&self, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_lin_num(&self, channel: u8) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn wakeup_lin(&self, channel: u8) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    #[deprecated(since = "0.1.0", note = "This method is deprecated!")]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn is_connected_server(&self) -> Result<bool, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn disconnect_server(&self) -> Result<(), ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        Err(ZCanError::MethodNotSupported)
    }
    fn timestamp(&self, channel: u8) -> Result<u64, ZCanError>;
    fn device_handler<C, T>(&self, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(&Handler) -> Result<T, ZCanError>;
    #[inline(always)]
    fn can_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(&ZChannelContext) -> Result<T, ZCanError> {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.find_can(channel) {
                Some(context) => callback(context),
                None => Err(ZCanError::ChannelNotOpened),
            }
        })
    }

    #[inline(always)]
    fn lin_handler<C, T>(&self, channel: u8, callback: C) -> Result<T, ZCanError>
        where
            C: FnOnce(&ZChannelContext) -> Result<T, ZCanError> {
        self.device_handler(|hdl| -> Result<T, ZCanError> {
            match hdl.lin_channels().get(&channel) {
                Some(chl) => callback(chl),
                None => Err(ZCanError::ChannelNotOpened),
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;
    use can_type_rs::CanDeviceSync;
    use can_type_rs::frame::Frame;
    use can_type_rs::identifier::Id;
    use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, CanMessage, ZCanChlMode, ZCanChlType};
    use zlgcan_common::device::ZCanDeviceType;
    use zlgcan_common::error::ZCanError;
    use crate::driver::{ZCanDriver, ZDevice};

    #[test]
    fn can_trait() -> Result<(), ZCanError> {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let mut device = ZCanDriver::new(dev_type as u32, 0, None)?;
        device.open()?;

        let factory = CanChlCfgFactory::new().unwrap();
        let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                              CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None)).unwrap();
        let ch2_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                              CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None)).unwrap();
        let cfg = vec![ch1_cfg, ch2_cfg];
        device.init_can_chl(cfg).unwrap();

        let data = vec![0x02, 0x10, 0x01];
        let message = CanMessage::new(
            Id::new(0x7DF, None).unwrap(),
            data.as_slice()
        ).unwrap();
        device.transmit_sync(0, vec![message, ], false, None)?;

        let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut message = CanMessage::new(
            Id::new(0x7DF, None).unwrap(),
            data.as_slice()
        ).unwrap();
        message.set_can_fd(true);
        message.set_bitrate_switch(true);
        device.transmit_sync(0, vec![message, ], true, None)?;

        thread::sleep(Duration::from_millis(100));

        let messages = device.receive_sync(1, false, None)?;
        messages.into_iter()
            .for_each(|message| println!("{:?}", message));

        let messages = device.receive_sync(1, true, None)?;
        messages.into_iter()
            .for_each(|message| println!("{:?}", message));

        Ok(())
    }
}

