### How to use:
```rust
use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, ZCanChlMode, ZCanChlType, ZCanFdFrame, CanMessage};
use zlgcan_common::device::ZCanDeviceType;
use zlgcan_driver::driver::{ZCanDriver, ZDevice};
#[cfg(target_os = "windows")]
use zlgcan_common::can::ZCanFdFrameV2;
#[cfg(target_os = "linux")]
use zlgcan_common::can::ZCanFdFrameV1;


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
    let mut msg = CanMessage::new(0x7DF, None, [0x01, 0x02, 0x03, 0x04, 0x05], true, false, None).unwrap();
    msg.set_bitrate_switch(true);   // set canfd is bitrate switch
    #[cfg(target_os = "windows")]
        let frame = ZCanFdFrame::from(ZCanFdFrameV2::try_from(msg).unwrap());
    #[cfg(target_os = "linux")]
        let frame = ZCanFdFrame::from(ZCanFdFrameV1::try_from(msg).unwrap());

    let frames = vec![frame];

    // Transmit frame
    let ret = driver.transmit_canfd(0, frames).unwrap();
    assert_eq!(ret, 1);

    driver.close();
}
```

### How to test
  * Enter `driver/tests` folder. Select test file by your device type.
  * If the channels of device is less than 2:
    * Connect another CAN device with you device's channel for a monitor.
    * Then run the selected testcase.
    * When see receive frame from the monitor, then send frame by the monitor device.
    * It means pass when the testcase exited without any panic.
  * If the channels of device is rather than 2:
    * Connected the channels 0 and 1.
    * Then run the selected testcase.
    * It means pass when the testcase exited without any panic.
  * All testcase will output the send and received debug info.

### The tested device list(include `windows` and `linux`):
  * USBCAN1 (include office device and deriving device)
  * USBCAN2 (only deriving device)
  * USBCANFD-200U
  * USBCANFD-400U (without supporting channel 3 and 4)

### LICENSE
  * GNU LESSER GENERAL PUBLIC LICENSE V3
