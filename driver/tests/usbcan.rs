mod utils;

use common::device::{DeriveInfo, ZCanDeviceType};
use self::utils::can_device1;

#[test]
fn usbcan_official1() {
    // #[cfg(target_os = "linux")]
    // let linux = true;
    // #[cfg(target_os = "windows")]
    // let linux = false;
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    can_device1(dev_type, None);
}

#[test]
fn usbcan_derive1() {
    // #[cfg(target_os = "linux")]
    // let linux = true;
    // #[cfg(target_os = "windows")]
    // let linux = false;
    let dev_type = ZCanDeviceType::ZCAN_USBCAN1;
    let derive_info = DeriveInfo::new(false, 1);
    can_device1(dev_type, Some(derive_info));
}

#[test]
fn usbcan_official2() {
    // TODO has no this device
}

#[test]
fn usbcan_derive2() {

}

