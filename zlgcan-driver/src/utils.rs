use isotp_rs::can::frame::Frame;
use zlgcan_common::can::{CanMessage, ZCanFrameType};
use zlgcan_common::device::ZCanError;
use crate::driver::{ZDevice, ZCanDriver};

pub fn unify_send(device: &ZCanDriver, msg: *const CanMessage) -> Result<u32, ZCanError> {
    let msg = unsafe {
        if msg.is_null() {
            return Err(ZCanError::MessageConvertFailed);
        }
        std::ptr::read(msg)
    };
    let channel = msg.channel();

    if msg.is_can_fd() {
        let frames = vec![msg];
        device.transmit_canfd(channel, frames)
    }
    else {
        let frames = vec![msg];
        device.transmit_can(channel, frames)
    }
}

pub fn unify_recv(device: &ZCanDriver, channel: u8, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError> {
    let count_can = device.get_can_num(channel, ZCanFrameType::CAN)?;
    let mut results: Vec<CanMessage> = Vec::new();

    let mut frames = device.receive_can(channel, count_can, timeout)?;
    results.append(&mut frames);

    if device.device_type().canfd_support() {
        let count_fd = device.get_can_num(channel, ZCanFrameType::CANFD)?;
        let mut frames = device.receive_canfd(channel, count_fd, timeout)?;
        results.append(&mut frames);
    }

    Ok(results)
}


