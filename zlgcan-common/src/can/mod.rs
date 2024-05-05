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
    filter: ZCanFilterType,
    dbitrate: Option<u32>,
    resistance: Option<bool>,
    acc_code: Option<u32>,
    acc_mask: Option<u32>,
    brp: Option<u32>,
}
#[allow(dead_code)]
impl CanChlCfgExt {
    pub fn new(filter: Option<ZCanFilterType>,
               dbitrate: Option<u32>, resistance: Option<bool>,
               acc_code: Option<u32>, acc_mask: Option<u32>,
               brp: Option<u32>) -> Self {
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
    pub fn filter(&self) -> ZCanFilterType {
        self.filter
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
    dev_type: ZCanDeviceType,
    can_type: ZCanChlType,
    mode: ZCanChlMode,
    bitrate: u32,
    extra: CanChlCfgExt,
    cfg_ctx: Weak<HashMap<String, BitrateCfg>>,
    // cfg_ctx: HashMap<String, BitrateCfg>,
}

impl CanChlCfg {
    pub fn new(dev_type: ZCanDeviceType,
               can_type: ZCanChlType,
               mode: ZCanChlMode,
               bitrate: u32,
               extra: CanChlCfgExt,
               cfg_ctx: Weak<HashMap<String, BitrateCfg>>) -> Self {
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
    pub fn device_type(&self) -> ZCanDeviceType {
        self.dev_type
    }
    #[inline(always)]
    pub fn can_type(&self) -> ZCanChlType {
        self.can_type
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

pub(self) fn to_chl_cfg(mode: ZCanChlMode, bitrate: u32, cfg_ctx: &BitrateCfg, ext: &CanChlCfgExt) -> ZCanChlCfg {
    match cfg_ctx.bitrate.get(&bitrate.to_string()) {
        Some(v) => {
            let timing0 = v.get(TIMING0).expect(format!("ZLGCAN - `{}` is not configured in file!", TIMING0).as_str());
            let timing1 = v.get(TIMING1).expect(format!("ZLGCAN - `{}` is not configured in file!", TIMING1).as_str());
            ZCanChlCfg::new(
                mode, *timing0, *timing1, ext.filter, ext.acc_code, ext.acc_mask
            )
        },
        None => panic!("ZLGCAN - the bitrate: `{}` is not configured in file!", bitrate),
    }
}

impl From<&CanChlCfg> for ZCanChlCfgV1 {
    fn from(value: &CanChlCfg) -> Self {
        let dev_type = value.dev_type;
        let binding = value.cfg_ctx.upgrade().unwrap();
        let cfg = binding.get(&(dev_type as u32).to_string()).unwrap();
        if dev_type.canfd_support() {      // the device supported canfd can't set CAN type to CAN
            let ext = &value.extra;
            let can_type = match value.can_type {
                ZCanChlType::CAN | ZCanChlType::CANFD_ISO => ZCanChlType::CANFD_ISO,
                v => v,
            };
            ZCanChlCfgV1::new(
                can_type,
                ZCanChlCfgV1Union::from_canfd(
                    ZCanFdChlCfgV1::new(
                        value.mode, 0, 0, ext.filter, ext.acc_code, ext.acc_mask, ext.brp  // TODO timing0 and timing1 ignored
                    )
                )
            )
        }
        else {
            ZCanChlCfgV1::new(
                ZCanChlType::CAN,
                ZCanChlCfgV1Union::from_can(
                    to_chl_cfg(value.mode, value.bitrate, cfg, &value.extra)
                )
            )
        }
    }
}

impl From<&CanChlCfg> for ZCanChlCfgV2 {
    fn from(value: &CanChlCfg) -> Self {
        let dev_type = value.dev_type;
        let binding = value.cfg_ctx.upgrade().unwrap();
        let cfg = binding.get(&(dev_type as u32).to_string()).unwrap();
        if dev_type.canfd_support() {
            let clock = cfg.clock.expect(format!("ZLGCAN - {} `clock` is not configured in file!", dev_type).as_str());
            let ext = &value.extra;
            let bitrate = value.bitrate;
            let dbitrate = ext.dbitrate;
            let bitrate_ctx = &cfg.bitrate;
            let dbitrate_ctx = &cfg.data_bitrate;
            let aset = bitrate_ctx
                .get(&bitrate.to_string())
                .expect(format!("ZLGCAN - the bitrate `{}` is not configured in file!", bitrate).as_str());
            let dset;
            match dbitrate {
                Some(v) => {    // dbitrate is not None
                    match dbitrate_ctx {
                        Some(ctx) => {  // dbitrate context is not None
                            match ctx.get(&v.to_string()) {
                                Some(value) => dset = value,
                                None => panic!("ZLGCAN - the data bitrate `{}` is not configured in file!", v),
                            }
                        },
                        None => {   // dbitrate context is None
                            match bitrate_ctx.get(&v.to_string()) {
                                Some(value) => dset = value,
                                None => panic!("ZLGCAN - the data bitrate `{}` is not configured in file!", v),
                            }
                        }
                    }
                },
                None => {   // dbitrate is None
                    match dbitrate_ctx {
                        Some(ctx) => {
                            match ctx.get(&bitrate.to_string()) {
                                Some(value) => dset = value,
                                None => dset = aset,
                            }
                        },
                        None => dset = aset,
                    }
                }
            }
            Self::from_canfd(
                ZCanFdChlCfgV2::new(value.can_type, value.mode, clock, ZCanFdChlCfgSet::from(aset), ZCanFdChlCfgSet::from(dset))
            )
        }
        else {
            Self::from_can(
                to_chl_cfg(value.mode, value.bitrate, cfg, &value.extra)
            )
        }
    }
}

impl From<&CanChlCfg> for ZCanChlCfgDetail {
    fn from(value: &CanChlCfg) -> Self {
        let dev_type = value.dev_type;
        if dev_type.is_can_chl_cfg_v1() {
            ZCanChlCfgDetail::from(ZCanChlCfgV1::from(value))
        }
        else if dev_type.is_can_chl_cfg_v2() {
            ZCanChlCfgDetail::from(ZCanChlCfgV2::from(value))
        }
        else {
            panic!("ZLGCAN - the device: `{}` is not supported!", dev_type)
        }
    }
}

#[repr(C)]
pub struct CanChlCfgFactory(Rc<HashMap<String, BitrateCfg>>);


impl CanChlCfgFactory {
    pub fn new() -> Result<Self, ZCanError> {
        match read_to_string(BITRATE_CFG_FILENAME) {
            Ok(v) => match serde_yaml::from_str(&v) {
                Ok(v) => Ok(Self(Rc::new(v))),
                Err(e) => Err(ZCanError::new(0x02, format!("Error parsing YAML: {:?}", e))),
            },
            Err(e) => Err(ZCanError::new(0x02, format!("Unable to read `{}`: {:?}", BITRATE_CFG_FILENAME, e))),
        }
    }

    pub fn new_can_chl_cfg(&self, dev_type: ZCanDeviceType, can_type: ZCanChlType, mode: ZCanChlMode, bitrate: u32, extra: CanChlCfgExt) -> Option<CanChlCfg> {
        if self.0.contains_key(&(dev_type as u32).to_string()) {
            Some(CanChlCfg::new(dev_type, can_type, mode, bitrate, extra, Rc::downgrade(&self.0)))
        }
        else {
            None
        }
    }
}


