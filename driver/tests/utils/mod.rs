use std::thread;
use std::time::Duration;
use rand::{Rng, thread_rng};
use rand::prelude::ThreadRng;
use common::can::{CanChlCfgExt, CanChlCfgFactory};
use common::can::constant::{CAN_FRAME_LENGTH, CANFD_FRAME_LENGTH, ZCanChlMode, ZCanChlType, ZCanFrameType};
use common::can::frame::{ZCanFdFrame, ZCanFdFrameV1, ZCanFdFrameV2, ZCanFrame, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3};
use common::can::message::CanMessage;
use common::device::{ZCanDevice, ZCanDeviceType, ZlgDevice};
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
        let frame = CanMessage::new(
            generate_can_id(&mut rng, extend),
            None,
            generate_data(&mut rng, CANFD_FRAME_LENGTH),
            true,
            false,
            None
        ).unwrap();

        frames.push(ZCanFdFrame::from_v2(ZCanFdFrameV2::from(frame)));
    }
    frames
}

///
pub fn canfd_device2(dev_type: ZCanDeviceType) {
    let dev_idx = 0;
    let channels = 2;
    let trans_ch = 0;
    let recv_ch = 1;
    let comm_count = 5;
    let ext_count = 5;
    let brs_count = 5;

    let mut driver = ZCanDriver::new();
    driver.open(dev_type, dev_idx, None).unwrap();
    let dev_info = driver.device_info(dev_type, dev_idx).unwrap();
    assert_eq!(dev_info.can_channels(), channels);

    let factory = CanChlCfgFactory::new();
    // reconfigure channels as CAN
    let ch1_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, Default::default()).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];
    driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();
    // create CAN frames
    let frames1 = new_v2_frames(comm_count, false);
    let frames2 = new_v2_frames(ext_count, true);

    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames1).unwrap();
    assert_eq!(ret, comm_count);
    // transmit CAN frames
    let ret = driver.transmit_can(dev_type, dev_idx, trans_ch, frames2).unwrap();
    assert_eq!(ret, ext_count);

    thread::sleep(Duration::from_millis(50));
    // get CAN receive count
    let count = driver.get_can_num(dev_type, dev_idx, recv_ch, ZCanFrameType::CAN).unwrap();
    assert_eq!(count, comm_count + ext_count);
    // receive CAN frames
    let frames = driver.receive_can(dev_type, dev_idx, recv_ch, count, None).unwrap();
    assert_eq!(frames.len() as u32, count);
    // reconfigure channels as CANFD
    let ch1_ext = CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None);
    let ch2_ext = CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None);
    let ch1_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, ch1_ext).unwrap();
    let ch2_cfg = factory.new_can_chl_cfg(dev_type, ZCanChlType::CANFD_ISO, ZCanChlMode::Normal, 500_000, ch2_ext).unwrap();
    let cfg = vec![ch1_cfg, ch2_cfg];
    driver.init_can_chl(dev_type, dev_idx, cfg).unwrap();
    // create CANFD frames
    let frames1 = new_v1_fdframes(comm_count, false, false);
    let frames2 = new_v1_fdframes(ext_count, true, false);
    let frames3 = new_v1_fdframes(brs_count, false, true);
    let frames4 = new_v1_fdframes(brs_count, true, true);
    // transmit CANFD frames
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames1).unwrap();
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames2).unwrap();
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames3).unwrap();
    driver.transmit_canfd(dev_type, dev_idx, recv_ch, frames4).unwrap();

    thread::sleep(Duration::from_millis(100));
    // get CANFD receive count
    let count = driver.get_can_num(dev_type, dev_idx, trans_ch, ZCanFrameType::CANFD).unwrap();
    assert_eq!(count, comm_count + ext_count + 2 * brs_count);
    // receive CANFD frames
    let frames = driver.receive_canfd(dev_type, dev_idx, trans_ch, count, None).unwrap();
    assert_eq!(frames.len() as u32, count);

    // close device
    driver.close(dev_type, dev_idx);
}

#[cfg(test)]
mod test {
    use crate::utils::{new_v1_fdframes, new_v1_frames};

    #[test]
    fn test_utils() {
        let size = 2;
        let frames = new_v1_frames(size, false);
        let frames = new_v1_frames(size, true);


        let frames = new_v1_fdframes(size, false, false);
        let frames = new_v1_fdframes(size, true, false);
        let frames = new_v1_fdframes(size, false, true);
        let frames = new_v1_fdframes(size, false, true);
    }
}


