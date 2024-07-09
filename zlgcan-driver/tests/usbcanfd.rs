mod utils;

use zlgcan_common::device::ZCanDeviceType;
use zlgcan_common::error::ZCanError;
use self::utils::canfd_device2;

#[test]
fn usbcanfd_200u() -> Result<(), ZCanError> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    canfd_device2(dev_type, 2, 2, 0, 1)?;
    Ok(())
}

/// `Attention:`
/// The USBCANFD-400U only supported channel0 and channel1 on Linux
#[test]
fn usbcanfd_400u() -> Result<(), ZCanError> {
    // TODO USBCANFD-400U channel 3-4 is not supported
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    canfd_device2(dev_type, 4, 2, 0, 1)?;
    Ok(())
}
