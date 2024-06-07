use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, ZCanChlMode, ZCanChlType, CanMessage};
use zlgcan_common::device::ZCanDeviceType;
use zlgcan_driver::driver::{ZCanDriver, ZDevice};

#[test]
fn main() {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    let dev_idx = 0;

    // Create driver instance
    let mut driver = ZCanDriver::new(dev_type as u32, dev_idx, None).unwrap();

    // Open device
    driver.open().unwrap();

    // Get device info and assert some information
    let dev_info = driver.device_info().unwrap();
    assert_eq!(dev_info.can_channels(), 2);
    assert_eq!(dev_info.canfd(), true);

    // Create channel configuration factory
    let factory = CanChlCfgFactory::new().unwrap();
    let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                          CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None)).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                          CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None)).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];

    // intialize channels
    driver.init_can_chl(cfg).unwrap();

    // Create CANFD frame
    let msg = CanMessage::new(0x7DF, None, [0x01, 0x02, 0x03, 0x04, 0x05], true, false, None).unwrap();
    let frames = vec![msg];

    // Transmit frame
    let ret = driver.transmit_canfd(0, frames).unwrap();
    assert_eq!(ret, 1);

    driver.close();
}
