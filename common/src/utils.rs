use std::ffi::{c_char, CStr};

pub fn c_str_to_string(src: *const c_char) -> Option<String> {
    if !src.is_null() {
        let c_str = unsafe { CStr::from_ptr(src) };
        let s_slice = c_str.to_str().expect("couldn't convert to string slice!");
        let value = String::from(s_slice);

        Some(value)
    } else {
        None
    }
}


