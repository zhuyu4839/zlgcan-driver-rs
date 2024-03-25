pub mod channel;
pub mod constant;
pub mod frame;
pub mod message;
mod utils;

use std::collections::HashMap;
use serde::Deserialize;
use crate::device::ZCanDeviceType;
use self::{
    constant::{ZCanChlMode, ZCanChlType, ZCanFilterType, TIMING0, TIMING1},
    channel::{ZCanChlCfg, ZCanChlCfgDetail, ZCanChlCfgV1, ZCanChlCfgV1Union, ZCanChlCfgV2, ZCanFdChlCfgSet, ZCanFdChlCfgV1, ZCanFdChlCfgV2}
};

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
#[derive(Debug)]
pub struct CanChlCfg<'a> {
    dev_type: ZCanDeviceType,
    can_type: ZCanChlType,
    mode: ZCanChlMode,
    bitrate: u32,
    extra: CanChlCfgExt,
    cfg_ctx: &'a HashMap<String, BitrateCfg>,
    // cfg_ctx: HashMap<String, BitrateCfg>,
}

impl<'a> CanChlCfg<'a> {
    pub fn new(dev_type: ZCanDeviceType, can_type: ZCanChlType, mode: ZCanChlMode, bitrate: u32, extra: CanChlCfgExt, cfg_ctx: &'a HashMap<String, BitrateCfg>) -> Self {
        // let contents = fs::read_to_string(BITRATE_CFG_FILENAME).unwrap_or_else(|e| { panic!("Unable to read `{}`: {:?}", BITRATE_CFG_FILENAME, e)});
        // let config = serde_yaml::from_str(&contents).unwrap_or_else(|e| { panic!("Error parsing YAML: {:?}", e) });
        // println!("{:?}", config);
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
        match self.cfg_ctx.get(&(self.dev_type as u32).to_string()) {
            Some(v) => v.clock,
            None => None,
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

pub(self) fn to_chl_cfg(dev_type: ZCanDeviceType, mode: ZCanChlMode, bitrate: u32, cfg_ctx: &HashMap<String, BitrateCfg>, ext: &CanChlCfgExt) -> ZCanChlCfg {
    match cfg_ctx.get(&(dev_type as u32).to_string()) {
        Some(v) => {
            match v.bitrate.get(&bitrate.to_string()) {
                Some(v) => {
                    let timing0 = v.get(TIMING0).expect(format!("ZLGCAN - `{}` is not configured in file!", TIMING0).as_str());
                    let timing1 = v.get(TIMING1).expect(format!("ZLGCAN - `{}` is not configured in file!", TIMING1).as_str());
                    ZCanChlCfg::new(
                        mode, *timing0, *timing1, ext.filter, ext.acc_code, ext.acc_mask
                    )
                },
                None => panic!("ZLGCAN - the bitrate: `{}` is not configured in file!", bitrate),
            }
        },
        None => panic!("ZLGCAN - the device: `{}` is not configured in file!", dev_type),
    }
}

impl From<&CanChlCfg<'_>> for ZCanChlCfgV1 {
    fn from(value: &CanChlCfg) -> Self {
        let dev_type = value.dev_type;
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
                    to_chl_cfg(dev_type, value.mode, value.bitrate, &value.cfg_ctx, &value.extra)
                )
            )
        }
    }
}

impl From<&CanChlCfg<'_>> for ZCanChlCfgV2 {
    fn from(value: &CanChlCfg) -> Self {
        let dev_type = value.dev_type;
        if dev_type.canfd_support() {
            match value.cfg_ctx.get(&(dev_type as u32).to_string()) {
                Some(v) => {
                    let clock = v.clock.expect(format!("ZLGCAN - {} `clock` is not configured in file!", dev_type).as_str());
                    let ext = &value.extra;
                    let bitrate = value.bitrate;
                    let dbitrate = ext.dbitrate.unwrap_or(bitrate);
                    let bitrate_ctx = &v.bitrate;
                    let dbitrate_ctx = &v.data_bitrate;
                    let aset = bitrate_ctx
                        .get(&bitrate.to_string())
                        .expect(format!("ZLGCAN - the bitrate `{}` is not configured in file!", bitrate).as_str());
                    let dset;
                    match dbitrate_ctx {
                        Some(v) => {
                            match v.get(&dbitrate.to_string()) {
                                Some(v) => dset = v,
                                None => dset = aset,
                            }
                        },
                        None => dset = aset,
                    }
                    Self::from_canfd(
                        ZCanFdChlCfgV2::new(value.can_type, value.mode, clock, ZCanFdChlCfgSet::from(aset), ZCanFdChlCfgSet::from(dset))
                    )
                },
                None => panic!("ZLGCAN - the device: `{}` is not configured in file!", dev_type),
            }
        }
        else {
            Self::from_can(
                to_chl_cfg(dev_type, value.mode, value.bitrate, &value.cfg_ctx, &value.extra)
            )
        }
    }
}

impl From<&CanChlCfg<'_>> for ZCanChlCfgDetail {
    fn from(value: &CanChlCfg) -> Self {
        let dev_type = value.dev_type;
        if dev_type.is_can_chl_cfg_v1() {
            ZCanChlCfgDetail::from_v1(ZCanChlCfgV1::from(value))
        }
        else if dev_type.is_can_chl_cfg_v2() {
            ZCanChlCfgDetail::from_v2(ZCanChlCfgV2::from(value))
        }
        else {
            panic!("ZLGCAN - the device: `{}` is not supported!", dev_type)
        }
    }
}

