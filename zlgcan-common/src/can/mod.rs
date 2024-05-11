mod channel;
mod constant;
mod frame;
mod message;
mod utils;

pub use channel::*;
pub use constant::*;
pub use frame::*;
pub use message::*;

use std::collections::HashMap;
use std::fs::read_to_string;
use std::rc::{Rc, Weak};
use serde::Deserialize;
use crate::device::ZCanDeviceType;
use crate::error::ZCanError;

/// The deserialize object mapped to configuration file context.
#[derive(Debug, Deserialize)]
pub struct BitrateCfg {
    pub(crate) bitrate: HashMap<String, HashMap<String, u32>>,
    pub(crate) clock: Option<u32>,
    pub(crate) data_bitrate: Option<HashMap<String, HashMap<String, u32>>>
}

/// The extra info for common CAN channel configuration.
#[derive(Debug, Default, Copy, Clone)]
pub struct CanChlCfgExt {
    filter: u8,
    dbitrate: Option<u32>,
    resistance: Option<bool>,
    acc_code: Option<u32>,
    acc_mask: Option<u32>,
    brp: Option<u32>,
}
#[allow(dead_code)]
impl CanChlCfgExt {
    pub fn new(
        filter: Option<u8>,
        dbitrate: Option<u32>,
        resistance: Option<bool>,
        acc_code: Option<u32>,
        acc_mask: Option<u32>,
        brp: Option<u32>
    ) -> Self {
        Self {
            filter: filter.unwrap_or_default(),
            // canfd,
            dbitrate,
            resistance,
            acc_code,
            acc_mask,
            brp,
        }
    }
    #[inline(always)]
    pub fn filter(&self) -> Result<ZCanFilterType, ZCanError> {
        ZCanFilterType::try_from(self.filter)
    }
    #[inline(always)]
    pub fn dbitrate(&self) -> Option<u32> {
        self.dbitrate
    }
    #[inline(always)]
    pub fn resistance(&self) -> bool {
        self.resistance.unwrap_or(true)
    }
    #[inline(always)]
    pub fn acc_code(&self) -> u32 {
        self.acc_code.unwrap_or_default()
    }
    #[inline(always)]
    pub fn acc_mask(&self) -> u32 {
        self.acc_mask.unwrap_or(0xFFFFFFFF)
    }
}

/// The common CAN channel configuration.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CanChlCfg {
    dev_type: u32,
    can_type: u8,
    mode: u8,
    bitrate: u32,
    extra: CanChlCfgExt,
    cfg_ctx: Weak<HashMap<String, BitrateCfg>>,
    // cfg_ctx: HashMap<String, BitrateCfg>,
}

impl CanChlCfg {
    pub fn new(
        dev_type: u32,
        can_type: u8,
        mode: u8,
        bitrate: u32,
        extra: CanChlCfgExt,
        cfg_ctx: Weak<HashMap<String, BitrateCfg>>
    ) -> Self {
        Self {
            dev_type,
            can_type,
            mode,
            bitrate,
            extra,
            cfg_ctx,
        }
    }
    #[inline(always)]
    pub fn device_type(&self) -> Result<ZCanDeviceType, ZCanError> {
        ZCanDeviceType::try_from(self.dev_type)
    }
    #[inline(always)]
    pub fn can_type(&self) -> Result<ZCanChlType, ZCanError> {
        ZCanChlType::try_from(self.can_type)
    }
    #[inline(always)]
    pub fn clock(&self) -> Option<u32> {
        match self.cfg_ctx.upgrade().unwrap().get(&self.bitrate.to_string()) {
            Some(v) => v.clock,
            None => None
        }
    }
    #[inline(always)]
    pub fn bitrate(&self) -> u32 {
        self.bitrate
    }
    #[inline(always)]
    pub fn extra(&self) -> &CanChlCfgExt {
        &self.extra
    }
}

pub(self) fn to_chl_cfg(mode: u8, bitrate: u32, cfg_ctx: &BitrateCfg, ext: &CanChlCfgExt) -> Result<ZCanChlCfg, ZCanError> {
    match cfg_ctx.bitrate.get(&bitrate.to_string()) {
        Some(v) => {
            let timing0 = v.get(TIMING0)
                .ok_or(ZCanError::ConfigurationError(format!("`{}` is not configured in file!", TIMING0)))?;
            let timing1 = v.get(TIMING1)
                .ok_or(ZCanError::ConfigurationError(format!("`{}` is not configured in file!", TIMING1)))?;
            ZCanChlCfg::new(
                mode, *timing0, *timing1, ext.filter, ext.acc_code, ext.acc_mask
            )
        },
        None => Err(ZCanError::ConfigurationError(format!("the bitrate: `{}` is not configured", bitrate))),
    }
}

impl TryFrom<&CanChlCfg> for ZCanChlCfgV1 {
    type Error = ZCanError;

    fn try_from(value: &CanChlCfg) -> Result<Self, Self::Error> {
        let dev_type = value.dev_type;
        let binding = value.cfg_ctx.upgrade().unwrap();
        let cfg = binding.get(&dev_type.to_string())
            .ok_or(ZCanError::ConfigurationError(format!("device: {:?} is not configured in file!", dev_type)))?;
        if value.device_type()?
            .canfd_support() {      // the device supported canfd can't set CAN type to CAN
            let ext = &value.extra;
            ZCanChlCfgV1::new(
                value.can_type,
                ZCanChlCfgV1Union::from_canfd(
                    ZCanFdChlCfgV1::new(
                        value.mode, 0, 0, ext.filter, ext.acc_code, ext.acc_mask, ext.brp  // TODO timing0 and timing1 ignored
                    )?
                )
            )
        }
        else {
            ZCanChlCfgV1::new(
                ZCanChlType::CAN as u8,
                ZCanChlCfgV1Union::from_can(
                    to_chl_cfg(value.mode, value.bitrate, cfg, &value.extra)?
                )
            )
        }
    }
}

impl TryFrom<&CanChlCfg> for ZCanChlCfgV2 {

    type Error = ZCanError;
    fn try_from(value: &CanChlCfg) -> Result<Self, Self::Error> {
        let dev_type = value.dev_type;
        let binding = value.cfg_ctx.upgrade().unwrap();
        let cfg = binding.get(&dev_type.to_string())
            .ok_or(ZCanError::ConfigurationError(format!("device: {:?} is not configured in file!", dev_type)))?;
        if value.device_type()?
            .canfd_support() {
            let clock = cfg.clock
                .ok_or(ZCanError::ConfigurationError("`clock` is not configured in file!".to_string()))?;
            let ext = &value.extra;
            let bitrate = value.bitrate;
            let dbitrate = ext.dbitrate;
            let bitrate_ctx = &cfg.bitrate;
            let dbitrate_ctx = &cfg.data_bitrate;
            let aset = bitrate_ctx
                .get(&bitrate.to_string())
                .ok_or(ZCanError::ConfigurationError(format!("bitrate `{}` is not configured in file!", bitrate)))?;
            let dset=
            match dbitrate {
                Some(v) => {    // dbitrate is not None
                    match dbitrate_ctx {
                        Some(ctx) => {  // dbitrate context is not None
                            match ctx.get(&v.to_string()) {
                                Some(value) => Ok(value),
                                None => Err(ZCanError::ConfigurationError(format!("data bitrate `{}` is not configured in file!", v))),
                            }
                        },
                        None => {   // dbitrate context is None
                            match bitrate_ctx.get(&v.to_string()) {
                                Some(value) => Ok(value),
                                None => Err(ZCanError::ConfigurationError(format!("data bitrate `{}` is not configured in file!", v))),
                            }
                        }
                    }
                },
                None => {   // dbitrate is None
                    match dbitrate_ctx {
                        Some(ctx) => {
                            match ctx.get(&bitrate.to_string()) {
                                Some(value) => Ok(value),
                                None => Ok(aset),
                            }
                        },
                        None => Ok(aset),
                    }
                }
            }?;
            Ok(Self::from_canfd(
                ZCanFdChlCfgV2::new(
                    value.can_type,
                    value.mode,
                    clock,
                    ZCanFdChlCfgSet::try_from(aset)?,
                    ZCanFdChlCfgSet::try_from(dset)?
                )?
            ))
        }
        else {
            Ok(Self::from_can(
                to_chl_cfg(value.mode, value.bitrate, cfg, &value.extra)?
            ))
        }
    }
}

impl TryFrom<&CanChlCfg> for ZCanChlCfgDetail {

    type Error = ZCanError;

    fn try_from(value: &CanChlCfg) -> Result<Self, Self::Error> {
        let dev_type = value.device_type()?;
        if dev_type.is_can_chl_cfg_v1() {
            Ok(ZCanChlCfgDetail::from(ZCanChlCfgV1::try_from(value)?))
        }
        else if dev_type.is_can_chl_cfg_v2() {
            Ok(ZCanChlCfgDetail::from(ZCanChlCfgV2::try_from(value)?))
        }
        else {
            Err(ZCanError::DeviceNotSupported)
        }
    }
}

#[repr(C)]
pub struct CanChlCfgFactory(Rc<HashMap<String, BitrateCfg>>);


impl CanChlCfgFactory {
    pub fn new() -> Result<Self, ZCanError> {
        let data = read_to_string(BITRATE_CFG_FILENAME)
            .map_err(|e| ZCanError::ConfigurationError(format!("Unable to read `{}`: {:?}", BITRATE_CFG_FILENAME, e)))?;
        let result = serde_yaml::from_str(&data)
            .map_err(|e| ZCanError::ConfigurationError(format!("Error parsing YAML: {:?}", e)))?;
        Ok(Self(Rc::new(result)))
    }

    pub fn new_can_chl_cfg(
        &self,
        dev_type: u32,
        can_type: u8,
        mode: u8,
        bitrate: u32,
        extra: CanChlCfgExt
    ) -> Result<CanChlCfg, ZCanError> {
        if self.0.contains_key(&dev_type.to_string()) {
            Ok(CanChlCfg::new(dev_type, can_type, mode, bitrate, extra, Rc::downgrade(&self.0)))
        }
        else {
            Err(ZCanError::ConfigurationError(format!("device: {:?} is not configured in file!", dev_type)))
        }
    }
}


