mod utils;

use zlgcan_common::device::ZCanDeviceType;
use self::utils::canfd_device2;

#[test]
fn usbcanfd_800u() {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_800U;
    #[cfg(target_os = "linux")]
    let version = "v3";
    #[cfg(target_os = "windows")]
    let version = "v3";
    canfd_device2(dev_type, 8, 8, 0, 1, version);
}
