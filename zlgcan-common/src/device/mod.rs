mod dev;
mod property;
mod traitdef;
mod typedef;

use std::ffi::{c_char, CString};
pub use dev::*;
pub use property::*;
pub use traitdef::*;
pub use typedef::*;
use crate::error::ZCanError;
use crate::utils::c_str_to_string;

/// The information about derive device.
#[derive(Debug, Default, Copy, Clone)]
pub struct DeriveInfo {
    pub(crate) canfd: bool,
    pub(crate) channels: u8,
    // pub(crate) resistance: bool,
}

impl DeriveInfo {
    pub fn new(canfd: bool, channels: u8) -> Self {
        Self { canfd, channels }
    }
}

pub fn set_value(p: &IProperty, cmd_path: CmdPath, value: *const c_char) -> Result<(), ZCanError> {
    unsafe {
        match p.SetValue {
            Some(f) => {
                let path = cmd_path.get_path();
                let _path = CString::new(path).expect("ZLGCAN - couldn't convert to CString!");

                match f(_path.as_ptr(), value) {
                    1 => Ok(()),
                    code => Err(ZCanError::new(code as u32, format!("ZLGCAN - set `{}` value failed", path))),
                }
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not supported", "set_value"))),
        }
    }
}

pub fn get_value(p: &IProperty, cmd_path: CmdPath) -> Result<Option<String>, ZCanError> {
    unsafe {
        match p.GetValue {
            Some(f) => {
                let path = cmd_path.get_path();
                let _path = CString::new(path).expect("ZLGCAN - couldn't convert to CString!");

                let ret = f(_path.as_ptr());
                Ok(c_str_to_string(ret))
            },
            None => Err(ZCanError::new(0, format!("ZLGCAN - {} is not supported", "set_value"))),
        }
    }
}

