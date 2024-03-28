use zlgcan_common as common;

mod utils;

use common::device::ZCanDeviceType;
use self::utils::canfd_device2;

#[test]
fn usbcanfd_200u() {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    #[cfg(target_os = "linux")]
    let linux = true;
    #[cfg(target_os = "windows")]
    let linux = false;
    canfd_device2(dev_type, 2, 0, 1, linux);
}

/// `Attention:`
/// The USBCANFD-400U only supported channel0 and channel1 on Linux
#[test]
fn usbcanfd_400u() {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    #[cfg(target_os = "linux")]
    let linux = true;
    #[cfg(target_os = "windows")]
    let linux = false;
    canfd_device2(dev_type, 4, 0, 1, linux);
}

#[cfg(target_os = "windows")]
#[test]
fn usbcanfd_400u_other() {
    // TODO USBCANFD-400U channel 3-4 is not supported
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    canfd_device2(dev_type, 4, 2, 3, false);
}
