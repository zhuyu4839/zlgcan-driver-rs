use std::thread;
use std::time::{Duration, SystemTime};
use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;
use common::can::{CanChlCfgExt, CanChlCfgFactory};
use common::can::constant::{CAN_FRAME_LENGTH, CANFD_FRAME_LENGTH, ZCanChlMode, ZCanChlType, ZCanFrameType};
use common::can::frame::{ZCanFdFrame, ZCanFdFrameV1, ZCanFdFrameV2, ZCanFrame, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3};
use common::can::message::CanMessage;
use common::device::{DeriveInfo, ZCanDevice, ZCanDeviceType, ZlgDevice};
use driver::ZCanDriver;

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
    (0..len).map(|_| rng.gen_range(0..=0xFF)).collect()
}

fn new_v1_frames(size: u32, extend: bool) -> Vec<ZCanFrame> {
    let mut rng = thread_rng();
    let  mut frames = Vec::new();
    for _ in 0..size {
        let frame = CanMessage::new(
            generate_can_id(&mut rng, extend),
            None,
            generate_data(&mut rng, CAN_FRAME_LENGTH),
            false,
            false,
            None
        ).unwrap();

        frames.push(ZCanFrame::from_v1(ZCanFrameV1::from(frame)));
    }
    frames
}

fn new_v2_frames(size: u32, extend: bool) -> Vec<ZCanFrame> {
    let mut rng = thread_rng();
    let  mut frames = Vec::new();
    for _ in 0..size {
        let frame = CanMessage::new(
            generate_can_id(&mut rng, extend),
            None,
            generate_data(&mut rng, CAN_FRAME_LENGTH),
            false,
            false,
            None
        ).unwrap();

        frames.push(ZCanFrame::from_v2(ZCanFrameV2::from(frame)));
    }
    frames
}

fn new_v3_frames(size: u32, extend: bool) -> Vec<ZCanFrame> {
    let mut rng = thread_rng();
    let  mut frames = Vec::new();
    for _ in 0..size {
        let frame = CanMessage::new(
            generate_can_id(&mut rng, extend),
            None,
            generate_data(&mut rng, CAN_FRAME_LENGTH),
            false,
            false,
            None
        ).unwrap();

        frames.push(ZCanFrame::from_v3(ZCanFrameV3::from(frame)));
    }
    frames
}

fn new_v1_fdframes(size: u32, extend: bool, brs: bool) -> Vec<ZCanFdFrame> {
    let mut rng = thread_rng();
    let  mut frames = Vec::new();
    for _ in 0..size {
        let mut frame = CanMessage::new(
            generate_can_id(&mut rng, extend),
            None,
            generate_data(&mut rng, CANFD_FRAME_LENGTH),
            true,
            false,
            None
        ).unwrap();

        if brs {
            frame.set_bitrate_switch(true);
        }

        frames.push(ZCanFdFrame::from_v1(ZCanFdFrameV1::from(frame)));
    }
    frames
}

fn new_v2_fdframes(size: u32, extend: bool, brs: bool) -> Vec<ZCanFdFrame> {
    let mut rng = thread_rng();
    let  mut frames = Vec::new();
    for _ in 0..size {
        let mut frame = CanMessage::new(
            generate_can_id(&mut rng, extend),
            None,
            generate_data(&mut rng, CANFD_FRAME_LENGTH),
            true,
            false,
            None
        ).unwrap();

        if brs {
            frame.set_bitrate_switch(true);
        }

        frames.push(ZCanFdFrame::from_v2(ZCanFdFrameV2::from(frame)));
    }
    frames
}

pub fn can_device1(dev_type: ZCanDeviceType, linux: bool, derive_info: Option<DeriveInfo>) {
    let dev_idx = 0;
    let channels = 1;
    let trans_ch = 0;
    let recv_ch = 0;
    let comm_count = 1;
    let ext_count = 1;

    let mut driver = ZCanDriver::new();
    driver.open(dev_type, dev_idx, derive_info).unwrap();
    let dev_info = driver.device_info(dev_type, dev_idx).unwrap();
    assert_eq!(dev_info.can_channels(), channels);
    assert_eq!(dev_info.canfd(), false);

    let factory = CanChlCfgFactory::new();
    // reconfigure channels as CAN
    let ch1_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CAN, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
    let cfg = vec![ch1_cfg,];
    driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();

    let frames1;
    let frames2;
    // create CAN frames
    if linux {
        frames1 = new_v1_frames(comm_count, false);
        frames2 = new_v1_frames(ext_count, true);
    }
    else {
        frames1 = new_v3_frames(comm_count, false);
        frames2 = new_v3_frames(ext_count, true);
    }
    println!("source frame:");
    frames1.iter().for_each(|frame| {
        println!("{:?}", frame.get_v1());
    });
    frames2.iter().for_each(|frame| {
        println!("{:?}", frame.get_v1());
    });

    // transmit CAN frames
    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames1).unwrap();
    assert_eq!(ret, comm_count);
    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames2).unwrap();
    assert_eq!(ret, ext_count);

    loop {
        // waiting for receive message
        let cnt = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CAN).unwrap();
        let cnt_fd = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CANFD).unwrap();
        assert_eq!(cnt_fd, 0);

        if cnt > 0 {
            let frames = driver.receive_can(dev_type, dev_idx, recv_ch, cnt, None).unwrap();
            assert_eq!(frames.len() as u32, cnt);
            frames.iter().for_each(|frame| {
                println!("{:?}", frame.get_v1());
            });

            driver.close(dev_type, dev_idx);
            break;
        }

        thread::sleep(Duration::from_millis(100));
    }
}

pub fn can_device2(dev_type: ZCanDeviceType, linux: bool, derive_info: Option<DeriveInfo>) {
    let dev_idx = 0;
    let channels = 2;
    let trans_ch = 0;
    let recv_ch = 1;
    let comm_count = 5;
    let ext_count = 5;

    let mut driver = ZCanDriver::new();
    driver.open(dev_type, dev_idx, derive_info).unwrap();
    let dev_info = driver.device_info(dev_type, dev_idx).unwrap();
    assert_eq!(dev_info.can_channels(), channels);
    assert_eq!(dev_info.canfd(), false);

    let factory = CanChlCfgFactory::new();
    // reconfigure channels as CAN
    let ch1_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];
    driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();
    // create CAN frames
    let frames1;
    let frames2;
    if linux {
        frames1 = new_v1_frames(comm_count, false);
        frames2 = new_v1_frames(ext_count, true);
    }
    else {
        frames1 = new_v3_frames(comm_count, false);
        frames2 = new_v3_frames(ext_count, true);
    }
    println!("source frame:");
    frames1.iter().for_each(|frame| {
        println!("{:?}", frame.get_v1());
    });
    frames2.iter().for_each(|frame| {
        println!("{:?}", frame.get_v1());
    });

    // transmit CAN frames
    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames1).unwrap();
    assert_eq!(ret, comm_count);
    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames2).unwrap();
    assert_eq!(ret, ext_count);

    thread::sleep(Duration::from_millis(100));
    // get CAN receive count
    let cnt = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CAN).unwrap();
    let cnt_fd = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CANFD).unwrap();
    assert_eq!(cnt, comm_count + ext_count);
    assert_eq!(cnt_fd, 0);
    // receive CAN frames
    let frames = driver.receive_can(dev_type, dev_idx, recv_ch, cnt, None).unwrap();
    assert_eq!(frames.len() as u32, cnt);
    println!("received frame:");
    frames.iter().for_each(|frame| {
        println!("{:?}", frame.get_v1());
    });
}

pub fn canfd_device2(dev_type: ZCanDeviceType, channels: u8, linux: bool) {
    let dev_idx = 0;
    let trans_ch = 0;
    let recv_ch = 1;
    let comm_count = 5;
    let ext_count = 5;
    let brs_count = 5;

    let mut driver = ZCanDriver::new();
    driver.open(dev_type, dev_idx, None).unwrap();
    let dev_info = driver.device_info(dev_type, dev_idx).unwrap();
    assert_eq!(dev_info.can_channels(), channels);
    assert_eq!(dev_info.canfd(), true);

    let factory = CanChlCfgFactory::new();
    // reconfigure channels as CAN
    let ch1_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];
    driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();
    let frames1;
    let frames2;
    // create CAN frames
    if linux {
        frames1 = new_v2_frames(comm_count, false);
        frames2 = new_v2_frames(ext_count, true);
    }
    else {
        frames1 = new_v3_frames(comm_count, false);
        frames2 = new_v3_frames(ext_count, true);
    }

    // transmit CAN frames
    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames1).unwrap();
    assert_eq!(ret, comm_count);
    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames2).unwrap();
    assert_eq!(ret, ext_count);

    thread::sleep(Duration::from_millis(100));

    let timeout = Duration::from_secs(3);
    let start_time = SystemTime::now();
    loop {
        // get CAN receive count
        let cnt = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CAN).unwrap();
        let cnt_fd = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CANFD).unwrap();
        if cnt > 0 || cnt_fd > 0 {
            assert_eq!(cnt, comm_count + ext_count);
            assert_eq!(cnt_fd, 0);
            // receive CAN frames
            let frames = driver.receive_can(dev_type, dev_idx, recv_ch, cnt + cnt_fd, None).unwrap();
            assert_eq!(frames.len() as u32, cnt + cnt_fd);
            frames.iter().for_each(|frame| {
                if linux {
                    println!("{:?}", frame.get_v2());
                }
                else {
                    println!("{:?}", frame.get_v3());
                }
            });
            break
        }

        let elapsed_time = SystemTime::now().duration_since(start_time).unwrap();
        if elapsed_time > timeout {
            panic!("timeout.....");
        }
        thread::sleep(Duration::from_millis(100));
    }

    // reconfigure channels as CANFD
    let ch1_ext = CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None);
    let ch2_ext = CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None);
    let ch1_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, ch1_ext).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, ch2_ext).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];
    driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();
    // create CANFD frames
    let frames1;
    let frames2;
    let frames3;
    let frames4;
    if linux {
        frames1 = new_v1_fdframes(comm_count, false, false);
        frames2 = new_v1_fdframes(ext_count, true, false);
        frames3 = new_v1_fdframes(brs_count, false, true);
        frames4 = new_v1_fdframes(brs_count, true, true);
    }
    else {
        frames1 = new_v2_fdframes(comm_count, false, false);
        frames2 = new_v2_fdframes(ext_count, true, false);
        frames3 = new_v2_fdframes(brs_count, false, true);
        frames4 = new_v2_fdframes(brs_count, true, true);
    }
    // transmit CANFD frames
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames1).unwrap();
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames2).unwrap();
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames3).unwrap();
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames4).unwrap();

    thread::sleep(Duration::from_millis(100));

    let timeout = Duration::from_secs(3);
    let start_time = SystemTime::now();
    loop {
        // get CANFD receive count
        let cnt = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CAN).unwrap();
        let cnt_fd = driver.get_can_num(dev_type, dev_idx, trans_ch, ZCanFrameType::CANFD).unwrap();

        if cnt > 0 || cnt_fd > 0 {
            assert_eq!(cnt_fd, comm_count + ext_count + 2 * brs_count);
            assert_eq!(cnt, 0);
            // receive CANFD frames
            let frames = driver.receive_canfd(dev_type, dev_idx, trans_ch, cnt_fd, None).unwrap();
            assert_eq!(frames.len() as u32, cnt_fd);
            frames.iter().for_each(|frame| {
                if linux {
                    println!("{:?}", frame.get_v1());
                }
                else {
                    println!("{:?}", frame.get_v2());
                }
            });

            break;
        }

        let elapsed_time = SystemTime::now().duration_since(start_time).unwrap();
        if elapsed_time > timeout {
            panic!("timeout.....");
        }
        thread::sleep(Duration::from_millis(100));
    }

    // close device
    driver.close(dev_type, dev_idx);
}

#[cfg(test)]
mod test {
    use super::{new_v1_fdframes, new_v1_frames, new_v2_frames, new_v3_frames, new_v2_fdframes};

    #[test]
    fn test_utils() {
        let size = 2;
        let _ = new_v1_frames(size, false);
        let _ = new_v1_frames(size, true);

        let _ = new_v2_frames(size, false);
        let _ = new_v2_frames(size, true);

        let _ = new_v3_frames(size, false);
        let _ = new_v3_frames(size, true);

        let _ = new_v1_fdframes(size, false, false);
        let _ = new_v1_fdframes(size, true, false);
        let _ = new_v1_fdframes(size, false, true);
        let _ = new_v1_fdframes(size, false, true);

        let _ = new_v2_fdframes(size, false, false);
        let _ = new_v2_fdframes(size, true, false);
        let _ = new_v2_fdframes(size, false, true);
        let _ = new_v2_fdframes(size, false, true);
    }
}


