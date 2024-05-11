mod dev;
mod property;
mod typedef;

pub use dev::*;
pub use property::*;
pub use typedef::*;
pub use crate::error::ZCanError;

use std::ffi::{c_char, CString};
use crate::utils::c_str_to_string;

/// The information about derive device.
#[repr(C)]
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
        let f = p.SetValue.ok_or(ZCanError::MethodNotSupported)?;
        let path = cmd_path.get_path();
        let path = CString::new(path).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
        match f(path.as_ptr(), value) {
            1 => Ok(()),
            code => Err(ZCanError::MethodExecuteFailed("SetValue".to_string(), code as u32)),
        }
    }
}

pub fn get_value(p: &IProperty, cmd_path: CmdPath) -> Result<String, ZCanError> {
    unsafe {
        let f = p.GetValue.ok_or(ZCanError::MethodNotSupported)?;
        let path = cmd_path.get_path();
        let path = CString::new(path).map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;

        let ret = f(path.as_ptr());
        c_str_to_string(ret)
    }
}

