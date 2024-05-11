use std::ffi::{c_char, CStr};
use crate::error::ZCanError;

pub fn c_str_to_string(src: *const c_char) -> Result<String, ZCanError> {
    if src.is_null() {
        Err(ZCanError::CStringConvertFailed("null pointer".to_string()))
    } else {
        let c_str = unsafe { CStr::from_ptr(src) };
        let s_slice = c_str.to_str().map_err(|e| ZCanError::CStringConvertFailed(e.to_string()))?;
        let value = String::from(s_slice);

        Ok(value)
    }
}


