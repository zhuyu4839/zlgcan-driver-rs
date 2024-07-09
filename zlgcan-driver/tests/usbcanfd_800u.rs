mod utils;

use zlgcan_common::device::ZCanDeviceType;
use zlgcan_common::error::ZCanError;
use self::utils::canfd_device2;

#[test]
fn usbcanfd_800u() -> Result<(), ZCanError> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_800U;
    canfd_device2(dev_type, 8, 8, 0, 1)?;
    Ok(())
}
