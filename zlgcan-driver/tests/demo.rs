use isotp_rs::can::{frame::Frame, identifier::Id};
use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, ZCanChlMode, ZCanChlType, CanMessage};
use zlgcan_common::device::ZCanDeviceType;
use zlgcan_common::error::ZCanError;
use zlgcan_driver::driver::{ZCanDriver, ZDevice};

#[test]
fn main() -> anyhow::Result<()> {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    let dev_idx = 0;

    // Create driver instance
    let mut driver = ZCanDriver::new(dev_type as u32, dev_idx, None)?;

    // Open device
    driver.open()?;

    // Get device info and assert some information
    let dev_info = driver.device_info()?;
    assert_eq!(dev_info.can_channels(), 2);
    assert_eq!(dev_info.canfd(), true);

    // Create channel configuration factory
    let factory = CanChlCfgFactory::new()?;
    let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                          CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None))?;
    let ch2_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                          CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None))?;
    let cfg = vec![ch1_cfg, ch2_cfg];

    // intialize channels
    driver.init_can_chl(cfg)?;

    // Create CANFD frame
    let mut msg = CanMessage::new(
        Id::from_bits(0x7df, false), [0x01, 0x02, 0x03, 0x04, 0x05].as_slice()
    )
        .ok_or(ZCanError::Other("invalid data length".to_string()))?;
    msg.set_can_fd(true);
    let frames = vec![msg];

    // Transmit frame
    let ret = driver.transmit_canfd(0, frames)?;
    assert_eq!(ret, 1);

    driver.close();

    Ok(())
}
