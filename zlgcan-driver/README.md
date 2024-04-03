### How to use:
```rust
use zlgcan_common as common;
use zlgcan_driver as driver;

use common::can::{
    CanChlCfgExt, CanChlCfgFactory,
    ZCanChlMode, ZCanChlType,
    ZCanFdFrame, ZCanFdFrameV2,
    CanMessage
};
use common::device::{ZCanDevice, ZCanDeviceType, ZlgDevice};
use driver::ZCanDriver;

fn main() {
    let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
    let dev_idx = 0;

    // Create driver instance
    let mut driver = ZCanDriver::new();

    // Open device
    driver.open(dev_type, dev_idx, None).unwrap();

    // Get device info and assert some information
    let dev_info = driver.device_info(dev_type, dev_idx).unwrap();
    assert_eq!(dev_info.can_channels(), 2);
    assert_eq!(dev_info.canfd(), true);

    // Create channel configuration factory
    let factory = CanChlCfgFactory::new();
    let ch1_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000,
                                          CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None)).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000,
                                          CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None)).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];

    // intialize channels
    driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();

    // Create CANFD frame
    let mut msg = CanMessage::new(0x7DF, None, [0x01, 0x02, 0x03, 0x04, 0x05], true, false, None).unwrap();
    msg.set_bitrate_switch(true);   // set canfd is bitrate switch
    #[cfg(target_os = "windows")]
    let frame = ZCanFdFrame::from(ZCanFdFrameV2::from(msg));
    #[cfg(target_os = "linux")]
    let frame = ZCanFdFrame::from(ZCanFdFrameV1::from(frame));

    let frames = vec![frame];

    // Transmit frame
    let ret = driver.transmit_canfd(dev_type, dev_idx, 0, frames).unwrap();
    assert_eq!(ret, 1);

    driver.close(dev_type, dev_idx);
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
