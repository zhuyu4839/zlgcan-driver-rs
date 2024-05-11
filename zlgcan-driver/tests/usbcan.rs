mod utils;

use zlgcan_common::device::{DeriveInfo, ZCanDeviceType};
use self::utils::{can_device1, can_device2};

#[test]
fn usbcan_official1() {
    #[cfg(target_os = "linux")]
    let linux = true;
    #[cfg(target_os = "windows")]
    let linux = false;
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    can_device1(dev_type, linux, None);
}

#[test]
fn usbcan_derive1() {
    #[cfg(target_os = "linux")]
    let linux = true;
    #[cfg(target_os = "windows")]
    let linux = false;
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    let derive_info = DeriveInfo::new(false, 1);
    can_device1(dev_type, linux, Some(derive_info));
}

#[test]
fn usbcan_official2() {
    // TODO has no this device
}

#[test]
fn usbcan_derive2() {
    #[cfg(target_os = "linux")]
    let linux = true;
    #[cfg(target_os = "windows")]
    let linux = false;
    let dev_type = ZCanDeviceType::ZCAN_USBCAN2;
    let derive_info = DeriveInfo::new(false, 2);

    can_device2(dev_type, linux, Some(derive_info));
}

