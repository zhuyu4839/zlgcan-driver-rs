use std::ffi::{c_uchar, c_uint};
use super::constant::{ZLinCheckSumMode, ZLinMode};

#[allow(non_snake_case)]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ZLinChlCfg {
    linMode: c_uchar,   // 是否作为主机，0-从机，1-主机
    chkSumMode: c_uchar,// 校验方式，1-经典校验 2-增强校验 3-自动(对应eZLINChkSumMode的模式)
    maxLength: c_uchar, // 最大数据长度，8~64
    reserved: c_uchar,  // 保留
    linBaud: c_uint,    // 波特率，取值1000~20000
}

impl ZLinChlCfg {
    /// Create LIN channel configuration.
    /// max_len is required only windows.
    pub fn new(mode: ZLinMode, cs_mode: ZLinCheckSumMode, bitrate: u32, max_len: Option<u8>) -> Option<Self> {
        match max_len {
            Some(v) => {
                match v {
                    8..=64 => {
                        match bitrate {
                            1000..=20000 => {
                                Some(Self {
                                    linMode: mode as c_uchar,
                                    chkSumMode: cs_mode as c_uchar,
                                    maxLength: v,
                                    reserved: Default::default(),
                                    linBaud: bitrate
                                })
                            },
                            _ => None,
                        }
                    },
                    _ => None,
                }
            },
            None => {
                Some(Self {
                    linMode: mode as c_uchar,
                    chkSumMode: cs_mode as c_uchar,
                    maxLength: Default::default(),
                    reserved: Default::default(),
                    linBaud: bitrate
                })
            },
        }
    }
}


