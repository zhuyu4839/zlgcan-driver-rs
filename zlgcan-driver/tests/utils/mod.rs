use std::thread;
use std::time::{Duration, SystemTime};
use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;
use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, CAN_FRAME_LENGTH, CANFD_FRAME_LENGTH, ZCanChlMode, ZCanChlType, ZCanFrameType, CanMessage, ZCanTxMode};
use zlgcan_common::device::{DeriveInfo, ZCanDeviceType};
use zlgcan_common::error::ZCanError;
use zlgcan_driver::driver::{ZCanDriver, ZDevice};

fn generate_can_id(rng: &mut ThreadRng, extend: bool) -> u32 {
    if extend {
        rng.gen_range(0x800..0x1FFF_FFFF)
    }
    else {
        rng.gen_range(0..0x7FF)
    }
}

fn generate_data(rng: &mut ThreadRng, size: usize) -> Vec<u8> {
    let len = rng.gen_range(0..=size);
    (1..len).map(|i| (i + 1) as u8).collect()
}

fn new_messages(size: u32, canfd: bool, extend: bool, brs: Option<bool>) -> Result<Vec<CanMessage>, ZCanError> {
    let mut rng = thread_rng();
    let  mut frames = Vec::new();
    for _ in 0..size {
        let mut frame = CanMessage::new(
            generate_can_id(&mut rng, extend),
            None,
            generate_data(&mut rng, if canfd { CANFD_FRAME_LENGTH } else { CAN_FRAME_LENGTH}),
            canfd,
            false,
            Some(extend)
        )?;
        frame.set_timestamp(None);
        frame.set_tx_mode(ZCanTxMode::SelfReception as u8);

        if canfd {
            frame.set_bitrate_switch(brs.unwrap_or(false));
        }

        frames.push(frame);
    }

    Ok(frames)
}

fn device_open(dev_type: ZCanDeviceType, dev_idx: u32, derive_info: Option<DeriveInfo>, channels: u8, canfd: bool) -> Result<ZCanDriver<'static>, ZCanError> {
    let mut driver = ZCanDriver::new(dev_type as u32, dev_idx, derive_info)?;
    driver.open()?;
    let dev_info = driver.device_info()?;
    assert_eq!(dev_info.can_channels(), channels);
    assert_eq!(dev_info.canfd(), canfd);
    Ok(driver)
}

fn can_init(driver: &mut ZCanDriver, available: u8, cfg_ext: CanChlCfgExt) -> Result<(), ZCanError> {
    let factory = CanChlCfgFactory::new()?;
    let mut cfg = Vec::new();
    for _ in 0..available {
        cfg.push(factory.new_can_chl_cfg(driver.device_type() as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000, cfg_ext.clone())?);
    }
    driver.init_can_chl(cfg)?;
    Ok(())
}

fn transmit_can(driver: &ZCanDriver, comm_count: u32, ext_count: u32, trans_ch: u8, recv_ch: u8) -> Result<(), ZCanError> {
    let frames1 = new_messages(comm_count, false, false, None)?;
    let frames2 = new_messages(ext_count, false, true, None)?;
    // create CAN frames
    println!("source frames:");
    frames1.iter().for_each(|f| println!("{}", f));
    frames2.iter().for_each(|f| println!("{}", f));

    // transmit CAN frames
    let ret = driver.transmit_can(trans_ch, frames1)?;
    assert_eq!(ret, comm_count);
    let ret = driver.transmit_can(trans_ch, frames2)?;
    assert_eq!(ret, ext_count);

    thread::sleep(Duration::from_millis(100));

    loop {
        // waiting for receive message
        let cnt = driver.get_can_num(recv_ch, ZCanFrameType::CAN)?;
        let cnt_fd = driver.get_can_num(recv_ch, ZCanFrameType::CANFD)?;
        assert_eq!(cnt_fd, 0);

        if cnt > 0 {
            let frames = driver.receive_can(recv_ch, cnt, None)?;
            assert_eq!(frames.len() as u32, cnt);
            println!("receive frames: {cnt}");
            frames.iter().for_each(|f| println!("{}", f));

            break;
        }

        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}

fn transmit_canfd(driver: &ZCanDriver, comm_count: u32, ext_count: u32, brs_count: u32, recv_ch: u8, trans_ch: u8) -> Result<(), ZCanError> {
    let frames1 = new_messages(comm_count, true, false, None)?;
    let frames2 = new_messages(ext_count, true, true, None)?;
    let frames3 = new_messages(brs_count, true, false, Some(true))?;
    let frames4 = new_messages(comm_count, true, true, Some(true))?;

    println!("source frames:");
    frames1.iter().for_each(|f| println!("{}", f));
    frames2.iter().for_each(|f| println!("{}", f));
    frames3.iter().for_each(|f| println!("{}", f));
    frames4.iter().for_each(|f| println!("{}", f));

    // transmit CANFD frames
    let length = frames1.len();
    let ret = driver.transmit_canfd(recv_ch, frames1)? as usize;
    assert_eq!(ret, length);

    let length = frames2.len();
    let ret = driver.transmit_canfd(recv_ch, frames2)? as usize;
    assert_eq!(ret, length);

    let length = frames3.len();
    let ret = driver.transmit_canfd(recv_ch, frames3)? as usize;
    assert_eq!(ret, length);

    let length = frames4.len();
    let ret = driver.transmit_canfd(recv_ch, frames4)? as usize;
    assert_eq!(ret, length);

    thread::sleep(Duration::from_millis(100));

    let timeout = Duration::from_secs(3);
    let start_time = SystemTime::now();
    loop {
        // get CANFD receive count
        let cnt = driver.get_can_num(trans_ch, ZCanFrameType::CAN)?;
        let cnt_fd = driver.get_can_num(trans_ch, ZCanFrameType::CANFD)?;

        if cnt > 0 || cnt_fd > 0 {
            assert_eq!(cnt_fd, comm_count + ext_count + 2 * brs_count);
            assert_eq!(cnt, 0);
            // receive CANFD frames
            let frames = driver.receive_canfd(trans_ch, cnt_fd, None)?;
            assert_eq!(frames.len() as u32, cnt_fd);
            println!("received fd frame: {cnt_fd}");
            frames.iter().for_each(|f| println!("{}", f));
            break;
        }

        let elapsed_time = SystemTime::now().duration_since(start_time).unwrap();
        if elapsed_time > timeout {
            panic!("timeout.....");
        }
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}

pub fn can_device1(dev_type: ZCanDeviceType, derive_info: Option<DeriveInfo>) -> Result<(), ZCanError> {
    let dev_idx = 0;
    let channels = 1;
    let trans_ch = 0;
    let recv_ch = 0;
    let comm_count = 5;
    let ext_count = 5;

    let mut driver = device_open(dev_type, dev_idx, derive_info, channels, false)?;

    can_init(&mut driver, 1, CanChlCfgExt::default())?;
    transmit_can(&driver, comm_count, ext_count, trans_ch, recv_ch)?;

    driver.close();
    Ok(())
}

pub fn can_device2(dev_type: ZCanDeviceType, channels: u8, available: u8, trans_ch: u8, recv_ch: u8, derive_info: Option<DeriveInfo>) -> Result<(), ZCanError> {
    let dev_idx = 0;
    let comm_count = 5;
    let ext_count = 5;

    let mut driver = device_open(dev_type, dev_idx, derive_info, channels, false)?;

    can_init(&mut driver, available, CanChlCfgExt::default())?;
    transmit_can(&driver, comm_count, ext_count, trans_ch, recv_ch)?;

    driver.close();
    Ok(())
}

pub fn canfd_device2(dev_type: ZCanDeviceType, channels: u8, available: u8, trans_ch: u8, recv_ch: u8) -> Result<(), ZCanError> {
    let dev_idx = 0;
    let comm_count = 5;
    let ext_count = 5;
    let brs_count = 5;

    let mut driver = device_open(dev_type, dev_idx, None, channels, true)?;

    can_init(&mut driver, available, CanChlCfgExt::default())?;
    transmit_can(&driver, comm_count, ext_count, trans_ch, recv_ch)?;

    println!();

    can_init(&mut driver, available, CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None))?;
    transmit_canfd(&driver, comm_count, ext_count, brs_count, trans_ch, recv_ch)?;

    driver.close();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::new_messages;

    #[test]
    fn test_utils() {
        let size = 2;
        let messages = new_messages(size, false, false, None).unwrap();
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let messages = new_messages(size, false, true, None).unwrap();
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });

        let messages = new_messages(size, true, false, Some(false)).unwrap();
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let messages = new_messages(size, true, true, Some(false)).unwrap();
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let messages = new_messages(size, true, false, Some(true)).unwrap();
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let messages = new_messages(size, true, true, Some(true)).unwrap();
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
    }
}


