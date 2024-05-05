use zlgcan_driver::{unify_recv, unify_send, ZCanDriver};
use std::ffi::{c_char, c_void, CString};
use zlgcan_common::can::{CanChlCfg, CanChlCfgExt, ZCanChlMode, ZCanChlType, ZCanFilterType};
use zlgcan_common::device::{ZCanDevice, ZCanDeviceType, ZlgDevice};

pub use zlgcan_common::can::CanMessage;
pub use zlgcan_common::can::CanChlCfgFactory;
pub use zlgcan_common::device::DeriveInfo;

#[repr(C)]
pub struct ZCanChlCfgApi {
    dev_type: u32,
    chl_type: u8,
    chl_mode: u8,
    bitrate: u32,
    filter: *const u8,
    dbitrate: *const u32,
    resistance: *const u8,
    acc_code: *const u32,
    acc_mask: *const u32,
    brp: *const u32,
}

#[allow(unused_assignments, unused_variables)]
#[inline]
pub(self) fn set_error(msg: String, error: &mut *const c_char) {
    let err = CString::new(msg).unwrap();
    let err_ptr = err.into_raw();
    *error = err_ptr;
}

#[inline]
pub(self) fn convert<'a, T>(ptr: *const T, mut error: &mut *const c_char) -> Option<&'a mut T> {
    if ptr.is_null() {
        set_error(String::from("The parameter is error!"), &mut error);
        return None;
    }

    Some(unsafe { ptr.cast_mut().as_mut() }.unwrap())
}

#[no_mangle]
pub extern "C" fn zlgcan_cfg_factory_can(mut error: &mut *const c_char) -> *const CanChlCfgFactory {
    match CanChlCfgFactory::new() {
        Ok(v) => Box::into_raw(Box::new(v)),
        Err(e) => {
            set_error(format!("{}", e), &mut error);
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn zlgcan_chl_cfg_can(
    factory: *const CanChlCfgFactory,
    cfg: ZCanChlCfgApi,
    mut error: &mut *const c_char
) -> *const c_void {
    match convert(factory, error) {
        Some(v) => unsafe {
            let bitrate = cfg.bitrate;
            let filter = if cfg.filter.is_null() { None } else { Some(ZCanFilterType::from(*cfg.filter)) };
            let dbitrate = if cfg.dbitrate.is_null() { None } else { Some(*cfg.dbitrate) };
            let resistance = if cfg.resistance.is_null() { None } else { Some(*cfg.resistance == 0) };
            let acc_code = if cfg.acc_code.is_null() { None } else { Some(*cfg.acc_code) };
            let acc_mask = if cfg.acc_mask.is_null() { None } else { Some(*cfg.acc_mask) };
            let brp = if cfg.brp.is_null() { None } else { Some(*cfg.brp) };
            match v.new_can_chl_cfg(
                ZCanDeviceType::from(cfg.dev_type),
                ZCanChlType::from(cfg.chl_type),
                ZCanChlMode::from(cfg.chl_mode),
                bitrate,
                CanChlCfgExt::new(filter, dbitrate, resistance, acc_code, acc_mask, brp, )) {
                Some(v) => Box::into_raw(Box::new(v)) as *const c_void,
                None => {
                    match dbitrate {
                        Some(v) => set_error(format!("Can't create configuration for bitrate: {}, dbitrate: {}", bitrate, v), &mut error),
                        None => set_error(format!("Can't create configuration for bitrate: {}", bitrate), &mut error),
                    }
                    std::ptr::null()
                }
            }
        },
        None => std::ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn zlgcan_open(
    dev_type: u32,
    dev_idx: u32,
    derive: *const DeriveInfo,
    mut error: &mut *const c_char
) -> *const c_void {
    let derive = if !derive.is_null() {
        let derive = unsafe { derive.as_ref().unwrap() };
        Some(derive.clone())
    }
    else {
        None
    };

    match ZCanDriver::new(ZCanDeviceType::from(dev_type), dev_idx, derive) {
        Ok(mut device) => {
            if let Err(e) = device.open() {
                set_error(e.to_string(), &mut error);
                return std::ptr::null();
            }
            Box::into_raw(Box::new(device)) as *const c_void
        },
        Err(e) => {
            set_error(e.to_string(), &mut error);
            std::ptr::null()
        }
    }
}

#[no_mangle]
pub extern "C" fn zlgcan_init_can(
    device: *const c_void,
    cfg: *const *const c_void,
    len: usize,
    mut error: &mut *const c_char
) -> bool {
    if cfg.is_null() {
        set_error(String::from("The configuration array is error!"), &mut error);
        return false;
    }

    match convert(device as *const ZCanDriver, error) {
        Some(v) => {
            let slice: &[*const CanChlCfg] = unsafe { std::slice::from_raw_parts(cfg as *const *const CanChlCfg, len) };
            let cfg = slice.to_vec();
            let mut _cfg = Vec::new();
            for (idx, ptr) in cfg.into_iter().enumerate() {
                if ptr.is_null() {
                    set_error(format!("The configuration index at: {} is error!", idx), &mut error);
                    return false;
                }
                _cfg.push(unsafe { ptr.as_ref().unwrap().clone() })
            }

            if let Err(e) = v.init_can_chl(_cfg) {
                set_error(e.to_string(), &mut error);
                return false;
            }
            true
        },
        None => false
    }
}

#[no_mangle]
pub extern "C" fn zlgcan_device_info(
    device: *const c_void,
    error: &mut *const c_char
) -> *const c_char {
    match convert(device as *const ZCanDriver, error) {
        Some(v) => {
            match v.device_info() {
                Some(v) => {
                    let val = CString::new(format!("{}", v)).unwrap();
                    val.into_raw()
                },
                None => std::ptr::null()
            }
        },
        None => std::ptr::null()
    }
}

#[no_mangle]
pub extern "C" fn zlgcan_clear_can_buffer(
    device: *const c_void,
    channel: u8,
    mut error: &mut *const c_char
) -> bool {
    match convert(device as *const ZCanDriver, error) {
        Some(v) => {
            if let Err(e) = v.clear_can_buffer(channel) {
                set_error(e.to_string(), &mut error);
                return false;
            }
            true
        },
        None => false
    }

}

#[no_mangle]
pub extern "C" fn zlgcan_send(
    device: *const c_void,
    msg: CanMessage,
    error: &mut *const c_char
) -> bool {
    match convert(device as *const ZCanDriver, error) {
        Some(v) => unify_send(v, msg),
        None => false,
    }
}

#[allow(unused_assignments, unused_variables)]
#[no_mangle]
pub extern "C" fn zlgcan_recv(
    device: *const c_void,
    channel: u8,
    timeout: u32,
    buffer: &mut *const CanMessage,
    mut error: &mut *const c_char
) -> u32 {
    match convert(device as *const ZCanDriver, error) {
        Some(v) => {
            let timeout = if timeout == 0 {
                None
            }
            else {
                Some(timeout)
            };

            match unify_recv(v, channel, timeout) {
                Ok(v) => {
                    let size = v.len();
                    let messages = Box::into_raw(Box::new(v.as_ptr()));
                    unsafe { *buffer = *messages };
                    std::mem::forget(v);

                    size as u32
                },
                Err(e) => {
                    set_error(e.to_string(), &mut error);
                    0
                }
            }
        },
        None => 0,
    }
}

#[no_mangle]
pub extern "C" fn zlgcan_close(
    device: *const c_void,
) {
    let mut error: *const c_char = std::ptr::null_mut();
    if let Some(v) = convert(device as *const ZCanDriver, &mut error) {
        v.close();
    }
}

#[cfg(test)]
mod tests {
    use std::ffi::CStr;
    use std::time::Duration;
    use zlgcan_common::can::CanMessage;
    use zlgcan_common::device::ZCanDeviceType;
    use super::{ZCanChlCfgApi, zlgcan_cfg_factory_can, zlgcan_chl_cfg_can, zlgcan_close, zlgcan_device_info, zlgcan_init_can, zlgcan_open, zlgcan_recv, zlgcan_send};

    // RUSTFLAGS=-Zsanitizer=leak cargo test --package zlgcan-driver-rs-api --lib tests::test_usbcanfd_200u --release -- --show-output
    #[test]
    fn test_usbcanfd_200u() {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let mut error = std::ptr::null();
        let derive = std::ptr::null();
        let device = zlgcan_open(dev_type as u32, 0, derive, &mut error);

        if device.is_null() {
            assert!(!error.is_null());

            let c_str = unsafe { CStr::from_ptr(error) };
            println!("Error: {}", c_str.to_string_lossy().to_string());
            return;
        }

        let mut error = std::ptr::null();
        let dev_info = zlgcan_device_info(device, &mut error);
        assert!(!dev_info.is_null());
        let dev_info = unsafe { CStr::from_ptr(dev_info) };
        println!("{}", dev_info.to_string_lossy().to_string());

        let factory = zlgcan_cfg_factory_can(&mut error);
        if factory.is_null() {
            assert!(!error.is_null());

            let c_str = unsafe { CStr::from_ptr(error) };
            println!("Error: {}", c_str.to_string_lossy().to_string());
            return;
        }

        let mut cfg = Vec::new();
        for _ in 0..2 {
            let mut error = std::ptr::null();
            let chl_cfg = ZCanChlCfgApi {
                dev_type: dev_type as u32,
                chl_type: 0,
                chl_mode: 0,
                bitrate: 500_000,
                filter: std::ptr::null(),
                dbitrate: std::ptr::null(),
                resistance: std::ptr::null(),
                acc_code: std::ptr::null(),
                acc_mask: std::ptr::null(),
                brp: std::ptr::null(),
            };
            let cfg1 = zlgcan_chl_cfg_can(factory, chl_cfg, &mut error);
            if cfg1.is_null() {
                assert!(!error.is_null());

                let c_str = unsafe { CStr::from_ptr(error) };
                println!("Error: {}", c_str.to_string_lossy().to_string());
                return;
            }
            cfg.push(cfg1);
        }

        let cfg_len = cfg.len();
        let mut error = std::ptr::null();
        let ret = zlgcan_init_can(device, cfg.as_ptr(), cfg_len, &mut error);
        if !ret {
            assert!(!error.is_null());

            let c_str = unsafe { CStr::from_ptr(error) };
            println!("Error: {}", c_str.to_string_lossy().to_string());
            return;
        }

        for _ in 0..=2 {
            let data = vec![0x01, 0x02, 0x03];
            let send = CanMessage::new(0xEF, Some(0), data, false, false, None).unwrap();
            let mut error = std::ptr::null();
            assert!(zlgcan_send(device, send, &mut error));
        }

        std::thread::sleep(Duration::from_micros(100));

        let mut error = std::ptr::null();
        let mut recv = std::ptr::null();
        let count = zlgcan_recv(device, 1, 0, &mut recv, &mut error);
        assert!(!recv.is_null());
        let slice = unsafe { std::slice::from_raw_parts(recv, count as usize) };
        let recv = slice.to_vec();
        println!("{:?}", recv);
        for msg in recv {
            println!("length: {} ({:?})", msg.length(), msg.data())
        }

        println!("received: {}", count);

        zlgcan_close(device);
    }
}
