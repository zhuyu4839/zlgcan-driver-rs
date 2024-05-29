use zlgcan_common::can::{CanMessage, ZCanFdFrame, ZCanFdFrameV1, ZCanFdFrameV2, ZCanFrame, ZCanFrameType, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3};
use zlgcan_common::error::ZCanError;
use crate::driver::{ZDevice, ZCanDriver};

pub fn unify_send(device: &ZCanDriver, msg: CanMessage) -> Result<u32, ZCanError> {
    let channel = msg.channel();
    if msg.is_fd() {
        let frames =
            if device.device_type().is_fdframe_v1() {
                vec![ZCanFdFrame::from(ZCanFdFrameV1::try_from(msg)?)]
            }
            else if device.device_type().is_fdframe_v2() {
                vec![ZCanFdFrame::from(ZCanFdFrameV2::try_from(msg)?)]
            }
            else {
                return Err(ZCanError::InvalidDeviceType);
            };

        device.transmit_canfd(channel, frames)
    }
    else {
        let frames =
            if device.device_type().is_frame_v1() {
                vec![ZCanFrame::from(ZCanFrameV1::try_from(msg)?)]
            }
            else if device.device_type().is_frame_v2() {
                vec![ZCanFrame::from(ZCanFrameV2::try_from(msg)?)]
            }
            else if device.device_type().is_frame_v3() {
                vec![ZCanFrame::from(ZCanFrameV3::try_from(msg)?)]
            }
            else {
                return Err(ZCanError::InvalidDeviceType);
            };

        device.transmit_can(channel, frames)
    }
}
pub fn unify_recv(device: &ZCanDriver, channel: u8, timeout: Option<u32>) -> Result<Vec<CanMessage>, ZCanError> {
    let count_can = device.get_can_num(channel, ZCanFrameType::CAN)?;
    let mut results: Vec<CanMessage> = Vec::new();

    let frames = device.receive_can(channel, count_can, timeout)?;
    for frame in frames {
        if device.device_type().is_frame_v1() {
            results.push(CanMessage::try_from(ZCanFrameV1::from(frame))?);
        }
        else if device.device_type().is_frame_v2() {
            results.push(CanMessage::try_from(ZCanFrameV2::from(frame))?);
        }
        else if device.device_type().is_frame_v3() {
            results.push(CanMessage::try_from(ZCanFrameV3::from(frame))?);
        }
        else {
            return Err(ZCanError::InvalidDeviceType);
        }
    }

    if device.device_type().canfd_support() {
        let count_fd = device.get_can_num(channel, ZCanFrameType::CANFD)?;
        let frames = device.receive_canfd(channel, count_fd, timeout)?;
        for frame in frames {
            if device.device_type().is_fdframe_v1() {
                results.push(CanMessage::try_from(ZCanFdFrameV1::from(&frame))?);
            }
            else if device.device_type().is_fdframe_v2() {
                results.push(CanMessage::try_from(ZCanFdFrameV2::from(&frame))?);
            }
            else {
                return Err(ZCanError::InvalidDeviceType);
            }
        }
    }

    Ok(results)
}


