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
    canfd_device2(dev_type, 2, linux);
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
    canfd_device2(dev_type, 4, linux);
}
