use dlopen2::symbor::{Library, SymBorApi};
use lazy_static::lazy_static;
use zlgcan_common::can::{CanChlCfg, ZCanChlError, ZCanChlStatus, ZCanFdFrame, ZCanFdFrameV1, ZCanFrame, ZCanFrameType, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3};
use zlgcan_common::cloud::{ZCloudGpsFrame, ZCloudServerInfo, ZCloudUserData};
use zlgcan_common::device::{DeriveInfo, Handler, ZCanDeviceType, ZCanError, ZDeviceInfo};
use zlgcan_common::lin::{ZLinChlCfg, ZLinDataType, ZLinFrame, ZLinFrameData, ZLinPublish, ZLinPublishEx, ZLinSubscribe};
use crate::api::{ZCanApi, ZCloudApi, ZDeviceApi, ZLinApi};
use crate::api::windows::Api;
use crate::constant::LOAD_LIB_FAILED;
use crate::driver::ZDevice;

#[cfg(target_arch = "x86")]
const LIB_PATH: &str = "library/windows/x86/zlgcan.dll";
#[cfg(target_arch = "x86_64")]
const LIB_PATH: &str = "library/windows/x86_64/zlgcan.dll";

lazy_static!(
    static ref LIB: Library = Library::open(LIB_PATH).expect(LOAD_LIB_FAILED);
);

pub struct ZCanDriver<'a> {
    pub(crate) handler:  Option<Handler>,
    pub(crate) api:      Api<'a>,
    pub(crate) dev_type: ZCanDeviceType,
    pub(crate) dev_idx:  u32,
    pub(crate) derive:   Option<DeriveInfo>,
}

impl ZDevice for ZCanDriver<'_> {
    fn new(dev_type: u32, dev_idx: u32, derive: Option<DeriveInfo>) -> Result<Self, ZCanError> where Self: Sized {
        let api =  unsafe {
            Api::load(&LIB).map_err(|e| ZCanError::LibraryLoadFailed(e.to_string()))
        }?;
        let dev_type = ZCanDeviceType::try_from(dev_type)?;
        Ok(Self { handler: Default::default(), api, dev_type, dev_idx, derive, })
    }

    fn device_type(&self) -> ZCanDeviceType {
        self.dev_type
    }

    fn device_index(&self) -> u32 {
        self.dev_idx
    }

    fn open(&mut self) -> Result<(), ZCanError> {
        let value = self.api.open(self.dev_type, self.dev_idx)?;
        let dev_info = match &self.derive {
            Some(v) => ZDeviceInfo::try_from(v)?,
            None => self.api.read_device_info(value)?,
        };

        self.handler = Some(Handler::new(value, dev_info));
        Ok(())
    }

    fn close(&mut self) {
        if let Some(handler) = &mut self.handler {
            for (idx, hdl) in handler.can_channels() {
                log::info!("ZLGCAN - closing CAN channel: {}", *idx);
                let hdl = *hdl;
                self.api.reset_can_chl(hdl).unwrap_or_else(|e| log::warn!("{}", e));
            }
            for (idx, hdl) in handler.lin_channels() {
                log::info!("ZLGCAN - closing LIN channel: {}", *idx);
                let hdl = *hdl;
                self.api.reset_lin_chl(hdl).unwrap_or_else(|e| log::warn!("{}", e));
            }

            self.api.close(handler.device_handler()).unwrap_or_else(|e| log::warn!("{}", e));
            self.handler = None
        }

    }

    fn device_info(&self) -> Result<&ZDeviceInfo, ZCanError> {
        match &self.handler {
            Some(handler) => Ok(&handler.device_info()),
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn is_derive_device(&self) -> bool {
        self.derive.is_some()
    }

    fn is_online(&self) -> Result<bool, ZCanError> {
        self.device_handler(|hdl| -> Result<bool, ZCanError> {
            self.api.is_online(hdl.device_handler())
        })
    }

    fn init_can_chl(&mut self, cfg: Vec<CanChlCfg>) -> Result<(), ZCanError> {
        match &mut self.handler {
            Some(dev_hdl) => {
                let dev_info = dev_hdl.device_info();
                let channels = dev_info.can_channels();
                for (idx, cfg) in cfg.iter().enumerate() {
                    let idx = idx as u8;
                    if idx >= channels {
                        log::warn!("ZLGCAN - the length of CAN channel configuration is out of channels!");
                        break;
                    }

                    if let Some(v) = dev_hdl.find_can(idx) {
                        self.api.reset_can_chl(v).unwrap_or_else(|e| log::warn!("{}", e));
                        dev_hdl.remove_can(idx);
                    }

                    let chl_hdl = self.api.init_can_chl(dev_hdl.device_handler(), idx, cfg)?;

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
                        self.api.reset_can_chl(v)?;
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
        self.can_handler(channel, |hdl| {
            self.api.read_can_chl_status(hdl)
        })
    }

    fn read_can_chl_error(&self, channel: u8) -> Result<ZCanChlError, ZCanError> {
        self.can_handler(channel, |hdl| {
            self.api.read_can_chl_error(hdl)
        })
    }

    fn clear_can_buffer(&self, channel: u8) -> Result<(), ZCanError> {
        self.can_handler(channel, |hdl| {
            self.api.clear_can_buffer(hdl)
        })
    }

    fn get_can_num(&self, channel: u8, can_type: ZCanFrameType) -> Result<u32, ZCanError> {
        self.can_handler(channel, |hdl| {
            self.api.get_can_num(hdl, can_type)
        })
    }

    fn receive_can(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFrame>, ZCanError> {
        let timeout = timeout.unwrap_or(0xFFFFFFFF);
        let frames = self.can_handler(channel, |hdl| {
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
        })?;

        let frames = frames.into_iter().map(|f| -> ZCanFrame {
            let mut frame = f.get_v3();
            frame.update_channel(channel);
            ZCanFrame::from(frame)
        })
            .collect::<Vec<_>>();
        Ok(frames)
    }

    fn transmit_can(&self, channel: u8, frames: Vec<ZCanFrame>) -> Result<u32, ZCanError> {
        self.can_handler(channel, |hdl| {
            self.api.transmit_can(hdl, frames)
        })
    }

    fn receive_canfd(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZCanFdFrame>, ZCanError> {
        let timeout = timeout.unwrap_or(0xFFFFFFFF);
        let frames = self.can_handler(channel, |hdl| {
            self.api.receive_canfd(hdl, size, timeout, |frames, size| {
                frames.resize_with(size, || -> ZCanFdFrame { ZCanFdFrame::from(ZCanFdFrameV1::default()) });
            })
        })?;
        let frames = frames.into_iter().map(|f| -> ZCanFdFrame {
            let mut frame = f.get_v2();
            frame.update_channel(channel);
            ZCanFdFrame::from(frame)
        })
            .collect::<Vec<_>>();
        Ok(frames)
    }

    fn transmit_canfd(&self, channel: u8, frames: Vec<ZCanFdFrame>) -> Result<u32, ZCanError> {
        self.can_handler(channel, |hdl| {
            self.api.transmit_canfd(hdl, frames)
        })
    }

    fn init_lin_chl(&mut self, cfg: Vec<ZLinChlCfg>) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
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

                    if let Some(v) = dev_hdl.find_lin(idx) {
                        self.api.reset_lin_chl(v).unwrap_or_else(|e| log::warn!("{}", e));
                        dev_hdl.remove_lin(idx);
                    }

                    let chl_hdl = self.api.init_lin_chl(dev_hdl.device_handler(), idx, cfg)?;
                    dev_hdl.add_lin(idx, chl_hdl);
                }

                Ok(())
            },
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn reset_lin_chl(&mut self, channel: u8) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        match &mut self.handler {
            Some(dev_hdl) => {
                match dev_hdl.find_lin(channel) {
                    Some(v) => {
                        self.api.reset_lin_chl(v)?;
                        dev_hdl.remove_lin(channel);
                        Ok(())
                    },
                    None => Err(ZCanError::ChannelNotOpened),
                }
            },
            None => Err(ZCanError::DeviceNotOpened),
        }
    }

    fn get_lin_num(&self, channel: u8) -> Result<u32, ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.get_lin_num(hdl)
        })
    }

    fn receive_lin(&self, channel: u8, size: u32, timeout: Option<u32>) -> Result<Vec<ZLinFrame>, ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        let timeout = timeout.unwrap_or(0xFFFFFFFF);
        self.lin_handler(channel, |hdl| {
            self.api.receive_lin(hdl, size, timeout, |frames, size| {
                frames.resize_with(size, || -> ZLinFrame { ZLinFrame::new(channel, ZLinDataType::TypeData, ZLinFrameData::from_data(Default::default())) })
            })
        })
    }

    fn transmit_lin(&self, channel: u8, frames: Vec<ZLinFrame>) -> Result<u32, ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.transmit_lin(hdl, frames)
        })
    }

    fn set_lin_subscribe(&self, channel: u8, cfg: Vec<ZLinSubscribe>) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.set_lin_subscribe(hdl, cfg)
        })
    }

    fn set_lin_publish(&self, channel: u8, cfg: Vec<ZLinPublish>) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.set_lin_publish(hdl, cfg)
        })
    }

    fn set_lin_publish_ext(&self, channel: u8, cfg: Vec<ZLinPublishEx>) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.set_lin_publish_ex(hdl, cfg)
        })
    }

    fn wakeup_lin(&self, channel: u8) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.wakeup_lin(hdl)
        })
    }

    #[allow(deprecated)]
    fn set_lin_slave_msg(&self, channel: u8, msg: Vec<ZLinFrame>) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.set_lin_slave_msg(hdl, msg)
        })
    }

    #[allow(deprecated)]
    fn clear_lin_slave_msg(&self, channel: u8, pids: Vec<u8>) -> Result<(), ZCanError> {
        if !self.dev_type.lin_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.lin_handler(channel, |hdl| {
            self.api.clear_lin_slave_msg(hdl, pids)
        })
    }

    fn set_server(&self, server: ZCloudServerInfo) -> Result<(), ZCanError> {
        if !self.dev_type.cloud_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.api.set_server(server)
    }

    fn connect_server(&self, username: &str, password: &str) -> Result<(), ZCanError> {
        if !self.dev_type.cloud_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.api.connect_server(username, password)
    }

    fn is_connected_server(&self) -> Result<bool, ZCanError> {
        if !self.dev_type.cloud_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.api.is_connected_server()
    }

    fn disconnect_server(&self) -> Result<(), ZCanError> {
        if !self.dev_type.cloud_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.api.disconnect_server()
    }

    fn get_userdata(&self, update: Option<i32>) -> Result<ZCloudUserData, ZCanError> {
        if !self.dev_type.cloud_support() {
            return Err(ZCanError::MethodNotSupported);
        }
        self.api.get_userdata(update.unwrap_or(0))
    }

    fn receive_gps(&self, size: u32, timeout: Option<u32>) -> Result<Vec<ZCloudGpsFrame>, ZCanError> {
        if !self.dev_type.cloud_support() {
            return Err(ZCanError::MethodNotSupported);
        }

        let timeout = timeout.unwrap_or(0xFFFFFFFF);
        self.device_handler(|hdl| {
            self.api.receive_gps(hdl.device_handler(), size, timeout, |frames, size| {
                frames.resize_with(size, Default::default)
            })
        })
    }
}

