use std::thread;
use std::time::{Duration, SystemTime};
use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;
use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, CAN_FRAME_LENGTH, CANFD_FRAME_LENGTH, ZCanChlMode, ZCanChlType, ZCanFrameType, CanMessage, ZCanTxMode};
use zlgcan_common::device::{DeriveInfo, ZCanDeviceType};
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

fn new_messages(size: u32, canfd: bool, extend: bool, brs: Option<bool>) -> Vec<CanMessage> {
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
        ).unwrap();
        frame.set_timestamp(None);
        frame.set_tx_mode(ZCanTxMode::SelfReception as u8);

        if canfd {
            frame.set_bitrate_switch(brs.unwrap_or(false));
        }

        frames.push(frame);
    }

    frames
}

pub fn can_device1(dev_type: ZCanDeviceType, derive_info: Option<DeriveInfo>) {
    let dev_idx = 0;
    let channels = 1;
    let trans_ch = 0;
    let recv_ch = 0;
    let comm_count = 5;
    let ext_count = 5;

    let mut driver = ZCanDriver::new(dev_type as u32, dev_idx, derive_info).unwrap();
    driver.open().unwrap();
    let dev_info = driver.device_info().unwrap();
    assert_eq!(dev_info.can_channels(), channels);
    assert_eq!(dev_info.canfd(), false);

    let factory = CanChlCfgFactory::new().unwrap();
    // reconfigure channels as CAN
    let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CAN as u8, ZCanChlMode::Normal as u8, 500_000, Default::default()).unwrap();
    let cfg = vec![ch1_cfg,];
    driver.init_can_chl(cfg).unwrap();

    let frames1 = new_messages(comm_count, false, false, None);
    let frames2 = new_messages(ext_count, false, true, None);
    // create CAN frames
    println!("source frames:");
    frames1.iter().for_each(|f| println!("{}", f));
    frames2.iter().for_each(|f| println!("{}", f));

    // transmit CAN frames
    let ret = driver.transmit_can(trans_ch, frames1).unwrap();
    assert_eq!(ret, comm_count);
    let ret = driver.transmit_can(trans_ch, frames2).unwrap();
    assert_eq!(ret, ext_count);

    loop {
        // waiting for receive message
        let cnt = driver.get_can_num(recv_ch, ZCanFrameType::CAN).unwrap();
        let cnt_fd = driver.get_can_num(recv_ch, ZCanFrameType::CANFD).unwrap();
        assert_eq!(cnt_fd, 0);

        if cnt > 0 {
            let frames = driver.receive_can(recv_ch, cnt, None).unwrap();
            assert_eq!(frames.len() as u32, cnt);
            println!("receive frames:");
            frames.iter().for_each(|f| println!("{}", f));

            driver.close();
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn can_device2(dev_type: ZCanDeviceType, derive_info: Option<DeriveInfo>) {
    let dev_idx = 0;
    let channels = 2;
    let trans_ch = 0;
    let recv_ch = 1;
    let comm_count = 5;
    let ext_count = 5;

    let mut driver = ZCanDriver::new(dev_type as u32, dev_idx, derive_info).unwrap();
    driver.open().unwrap();
    let dev_info = driver.device_info().unwrap();
    assert_eq!(dev_info.can_channels(), channels);
    assert_eq!(dev_info.canfd(), false);

    let factory = CanChlCfgFactory::new().unwrap();
    // reconfigure channels as CAN
    let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000, Default::default()).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000, Default::default()).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];
    driver.init_can_chl(cfg).unwrap();
    // create CAN frames
    let frames1 = new_messages(comm_count, false, false, None);
    let frames2 = new_messages(ext_count, false, true, None);
    println!("source frame:");
    frames1.iter().for_each(|f| println!("{}", f));
    frames1.iter().for_each(|f| println!("{}", f));

    // transmit CAN frames
    let ret = driver.transmit_can(trans_ch, frames1).unwrap();
    assert_eq!(ret, comm_count);
    let ret = driver.transmit_can(trans_ch, frames2).unwrap();
    assert_eq!(ret, ext_count);

    thread::sleep(Duration::from_millis(100));
    // get CAN receive count
    let cnt = driver.get_can_num(recv_ch, ZCanFrameType::CAN).unwrap();
    let cnt_fd = driver.get_can_num(recv_ch, ZCanFrameType::CANFD).unwrap();
    assert_eq!(cnt, comm_count + ext_count);
    assert_eq!(cnt_fd, 0);
    // receive CAN frames
    let frames = driver.receive_can(recv_ch, cnt, None).unwrap();
    assert_eq!(frames.len() as u32, cnt);

    println!("received frame:");
    frames.iter().for_each(|f| println!("{}", f));
}

pub fn canfd_device2(dev_type: ZCanDeviceType, channels: u8, available: u8, trans_ch: u8, recv_ch: u8) {
    let dev_idx = 0;
    let comm_count = 5;
    let ext_count = 5;
    let brs_count = 5;

    let mut driver = ZCanDriver::new(dev_type as u32, dev_idx, None).unwrap();
    driver.open().unwrap();
    let dev_info = driver.device_info().unwrap();
    assert_eq!(dev_info.can_channels(), channels);
    assert_eq!(dev_info.canfd(), true);

    let factory = CanChlCfgFactory::new().unwrap();
    // reconfigure channels as CAN
    let mut cfg = Vec::new();
    for _ in 0..available { // TODO USBCANFD-400U channel 3-4 is not supported
        cfg.push(factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000, Default::default()).unwrap());
    }
    driver.init_can_chl(cfg).unwrap();
    let frames1 = new_messages(comm_count, false, false, None);
    let frames2 = new_messages(ext_count, false, true, None);
    // create CAN frames
    // transmit CAN frames
    println!("source frames:");
    frames1.iter().for_each(|f| println!("{}", f));
    frames2.iter().for_each(|f| println!("{}", f));

    let ret = driver.transmit_can(trans_ch, frames1).unwrap();
    assert_eq!(ret, comm_count);
    let ret = driver.transmit_can(trans_ch, frames2).unwrap();
    assert_eq!(ret, ext_count);

    thread::sleep(Duration::from_millis(200));

    let timeout = Duration::from_secs(3);
    let start_time = SystemTime::now();
    loop {
        let cnt_tr = driver.get_can_num(trans_ch, ZCanFrameType::CAN).unwrap();
        let cnt_fd_tr = driver.get_can_num(trans_ch, ZCanFrameType::CANFD).unwrap();
        println!("self CAN Frames: {}, CANFD Frames: {}", cnt_tr, cnt_fd_tr);

        if cnt_tr > 0 || cnt_fd_tr > 0 {
            let frames = driver.receive_can(trans_ch, cnt_tr + cnt_fd_tr, None).unwrap();
            println!("self received frames:");
            frames.iter().for_each(|f| println!("{}", f));
        }

        // get CAN receive count
        let cnt = driver.get_can_num(recv_ch, ZCanFrameType::CAN).unwrap();
        let cnt_fd = driver.get_can_num(recv_ch, ZCanFrameType::CANFD).unwrap();
        println!("CAN Frames: {}, CANFD Frames: {}", cnt, cnt_fd);

        if cnt > 0 || cnt_fd > 0 {
            assert_eq!(cnt, comm_count + ext_count);
            assert_eq!(cnt_fd, 0);
            // receive CAN frames
            let frames = driver.receive_can(recv_ch, cnt + cnt_fd, None).unwrap();
            assert_eq!(frames.len() as u32, cnt + cnt_fd);
            println!("received frames:");
            frames.iter().for_each(|f| println!("{}", f));
            break
        }

        let elapsed_time = SystemTime::now().duration_since(start_time).unwrap();
        if elapsed_time > timeout {
            panic!("timeout.....");
        }
        thread::sleep(Duration::from_millis(100));
    }

    // reconfigure channels as CANFD
    let mut cfg = Vec::new();
    for _ in 0..available {
        let cfg_ext = CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None);
        cfg.push(factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000, cfg_ext).unwrap());
    }
    driver.init_can_chl(cfg).unwrap();
    // create CANFD frames
    let frames1 = new_messages(comm_count, true, false, None);
    let frames2 = new_messages(ext_count, true, true, None);
    let frames3 = new_messages(brs_count, true, false, Some(true));
    let frames4  = new_messages(comm_count, true, true, Some(true));

    println!("source frames:");
    frames1.iter().for_each(|f| println!("{}", f));
    frames2.iter().for_each(|f| println!("{}", f));
    frames3.iter().for_each(|f| println!("{}", f));
    frames4.iter().for_each(|f| println!("{}", f));

    // transmit CANFD frames
    driver.transmit_canfd(recv_ch, frames1).unwrap();
    driver.transmit_canfd(recv_ch, frames2).unwrap();
    driver.transmit_canfd(recv_ch, frames3).unwrap();
    driver.transmit_canfd(recv_ch, frames4).unwrap();

    thread::sleep(Duration::from_millis(100));

    let timeout = Duration::from_secs(3);
    let start_time = SystemTime::now();
    loop {
        // get CANFD receive count
        let cnt = driver.get_can_num(recv_ch, ZCanFrameType::CAN).unwrap();
        let cnt_fd = driver.get_can_num(trans_ch, ZCanFrameType::CANFD).unwrap();
        println!("CAN Frames: {}, CANFD Frames: {}", cnt, cnt_fd);

        if cnt > 0 || cnt_fd > 0 {
            assert_eq!(cnt_fd, comm_count + ext_count + 2 * brs_count);
            assert_eq!(cnt, 0);
            // receive CANFD frames
            let frames = driver.receive_canfd(trans_ch, cnt_fd, None).unwrap();
            assert_eq!(frames.len() as u32, cnt_fd);
            println!("received frame:");
            frames.iter().for_each(|f| println!("{}", f));
            break;
        }

        let elapsed_time = SystemTime::now().duration_since(start_time).unwrap();
        if elapsed_time > timeout {
            panic!("timeout.....");
        }
        thread::sleep(Duration::from_millis(100));
    }

    // close device
    driver.close();
}

#[cfg(test)]
mod tests {
    use super::new_messages;

    #[test]
    fn test_utils() {
        let size = 2;
        let messages = new_messages(size, false, false, None);
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let _ = new_messages(size, false, true, None);
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });

        let messages = new_messages(size, true, false, Some(false));
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let messages = new_messages(size, true, true, Some(false));
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let messages = new_messages(size, true, false, Some(true));
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
        let messages = new_messages(size, true, true, Some(true));
        messages.iter()
            .for_each(|msg| {
                println!("{}", msg);
            });
    }
}


