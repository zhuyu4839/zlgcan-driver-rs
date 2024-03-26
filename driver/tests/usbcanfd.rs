use common::device::ZCanDeviceType;
use crate::utils::canfd_device2;

mod utils;

#[test]
fn usbcanfd_200u() {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    canfd_device2(dev_type)
}

