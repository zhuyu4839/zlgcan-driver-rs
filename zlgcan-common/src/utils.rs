use std::ffi::{c_char, CStr};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::error::ZCanError;

#[inline]
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

#[inline]
pub fn system_timestamp() -> u64 {
    match SystemTime::now()
        .duration_since(UNIX_EPOCH) {
        Ok(v) => v.as_millis() as u64,
        Err(e) => {
            log::warn!("ZLGCAN - SystemTimeError: {0} when conversion failed!", e);
            0
        }
    }
}

#[inline]
pub fn fix_system_time(frame_timestamp: u64, fix_timestamp: u64) -> u64 {
    frame_timestamp + fix_timestamp
}

