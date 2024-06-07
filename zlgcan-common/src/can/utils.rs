use crate::can::constant::{CAN_EFF_FLAG, CAN_ERR_FLAG, CAN_ID_FLAG, CAN_RTR_FLAG, CANFD_BRS, CANFD_ESI, ZCanFrameType};
use crate::can::frame::NewZCanFrame;
use crate::error::ZCanError;
use crate::{TryFrom, TryFromIterator};
use crate::utils::{fix_device_time, fix_system_time};
use super::{
    channel::{ZCanChlErrorV1, ZCanChlErrorV2},
    constant::ZCanHdrInfoField,
    frame::{ZCanHdrInfo, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3, ZCanFdFrameV1, ZCanFdFrameV2},
    message::CanMessage
};

fn frame_new<T: NewZCanFrame>(msg: CanMessage, canfd: bool, timestamp: u64) -> Result<T, ZCanError> {
    let mut info: ZCanHdrInfo = Default::default();

    if canfd {
        info.set_field(ZCanHdrInfoField::TxMode, msg.tx_mode());
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CANFD as u8);
        if msg.bitrate_switch() {
            info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 1);
        }
        if msg.error_state_indicator() {
            info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 1);
        }
    }
    else {
        info.set_field(ZCanHdrInfoField::TxMode, msg.tx_mode());
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CAN as u8);
    }

    if msg.is_extended_id() {
        info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
    }
    if msg.is_remote_frame() {
        info.set_field(ZCanHdrInfoField::IsRemoteFrame, 1);
    }
    if msg.is_error_frame() {
        info.set_field(ZCanHdrInfoField::IsErrorFrame, 1);
    }

    T::new(msg.arbitration_id(), msg.channel(), msg.data(), info, fix_device_time(timestamp))
}

impl TryFrom<CanMessage, u64> for ZCanFrameV1 {
    type Error = ZCanError;
    fn try_from(value: CanMessage, timestamp: u64) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, false, timestamp)
    }
}

impl TryFrom<ZCanFrameV1, u64> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFrameV1, timestamp: u64) -> Result<Self, Self::Error> {
        let mut message = CanMessage::new(
            value.can_id,
            None,
            value.data,
            false,
            false,
            Some(value.ext_flag > 0)
        )?;
        message.set_length(value.len);
        message.set_timestamp(Some(fix_system_time(value.timestamp as u64, timestamp)));
        message.set_is_remote_frame(value.rem_flag > 0);
        Ok(message)
    }
}

impl TryFromIterator<CanMessage, u64> for Vec<ZCanFrameV1> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=CanMessage>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <ZCanFrameV1 as TryFrom<CanMessage, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFromIterator<ZCanFrameV1, u64> for Vec<CanMessage> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=ZCanFrameV1>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <CanMessage as TryFrom<ZCanFrameV1, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFrom<CanMessage, u64> for ZCanFrameV2 {
    type Error = ZCanError;
    fn try_from(value: CanMessage, timestamp: u64) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, false, timestamp)
    }
}

impl TryFrom<ZCanFrameV2, u64> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFrameV2, timestamp: u64) -> Result<Self, Self::Error> {
        let hdr = value.hdr;
        let info = hdr.info;
        let mut message = CanMessage::new(
            hdr.can_id, Some(hdr.channel), value.data, false, false, Some(info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0)
        )?;
        message.set_length(hdr.len);
        message.set_timestamp(Some(fix_system_time(value.hdr.timestamp as u64, timestamp)));
        message.set_is_remote_frame(info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0)
            .set_is_error_frame(info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0);
        Ok(message)
    }
}

impl TryFromIterator<CanMessage, u64> for Vec<ZCanFrameV2> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=CanMessage>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <ZCanFrameV2 as TryFrom<CanMessage, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFromIterator<ZCanFrameV2, u64> for Vec<CanMessage> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=ZCanFrameV2>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <CanMessage as TryFrom<ZCanFrameV2, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFrom<CanMessage, u64> for ZCanFrameV3 {
    type Error = ZCanError;
    fn try_from(value: CanMessage, timestamp: u64) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, false, timestamp)
    }
}

impl TryFrom<ZCanFrameV3, u64> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFrameV3, timestamp: u64) -> Result<Self, Self::Error> {
        let hdr = value.hdr;

        let can_id = hdr.can_id;
        let mut message = CanMessage::new(
            can_id & CAN_ID_FLAG, Some(hdr.__res0), value.data, false, false, Some((can_id & CAN_EFF_FLAG) > 0)
        )?;
        message.set_length(hdr.can_len);
        message.set_timestamp(Some(fix_system_time(value.ts_or_mode as u64, timestamp)));
        message.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0);
        Ok(message)
    }
}

impl TryFromIterator<CanMessage, u64> for Vec<ZCanFrameV3> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=CanMessage>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <ZCanFrameV3 as TryFrom<CanMessage, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFromIterator<ZCanFrameV3, u64> for Vec<CanMessage> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=ZCanFrameV3>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <CanMessage as TryFrom<ZCanFrameV3, u64>>::try_from(v, timestamp))
            .collect()
    }
}


impl TryFrom<CanMessage, u64> for ZCanFdFrameV1 {
    type Error = ZCanError;
    fn try_from(value: CanMessage, timestamp: u64) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, true, timestamp)
    }
}

impl TryFrom<ZCanFdFrameV1, u64> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFdFrameV1, timestamp: u64) -> Result<Self, Self::Error> {
        let hdr = value.hdr;
        let info = hdr.info;

        let can_id = hdr.can_id;
        let mut message = CanMessage::new(
            can_id, Some(hdr.channel), value.data.data, true, false, Some( info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0)
        )?;
        message.set_length(hdr.len);
        message.set_timestamp(Some(fix_system_time(value.hdr.timestamp as u64, timestamp)));
        message.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0)
            .set_bitrate_switch(info.get_field(ZCanHdrInfoField::IsBitrateSwitch) > 0)
            .set_error_state_indicator(info.get_field(ZCanHdrInfoField::IsErrorStateIndicator) > 0);
        Ok(message)
    }
}

impl TryFromIterator<CanMessage, u64> for Vec<ZCanFdFrameV1> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=CanMessage>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <ZCanFdFrameV1 as TryFrom<CanMessage, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFromIterator<ZCanFdFrameV1, u64> for Vec<CanMessage> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=ZCanFdFrameV1>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <CanMessage as TryFrom<ZCanFdFrameV1, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFrom<CanMessage, u64> for ZCanFdFrameV2 {
    type Error = ZCanError;
    fn try_from(value: CanMessage, timestamp: u64) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, true, timestamp)
    }
}

impl TryFrom<ZCanFdFrameV2, u64> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFdFrameV2, timestamp: u64) -> Result<Self, Self::Error> {
        let hdr = value.hdr;

        let can_id = hdr.can_id;
        let mut message = CanMessage::new(
            can_id & CAN_ID_FLAG, Some(hdr.__res0), value.data.data, true, false, Some((can_id & CAN_EFF_FLAG) > 0)
        )?;
        message.set_length(hdr.can_len);
        message.set_timestamp(Some(fix_system_time(value.ts_or_mode as u64, timestamp)));
        let flag = hdr.flag;
        message.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0)
            .set_bitrate_switch(flag & CANFD_BRS > 0)
            .set_error_state_indicator(flag & CANFD_ESI > 0);
        Ok(message)
    }
}

impl TryFromIterator<CanMessage, u64> for Vec<ZCanFdFrameV2> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=CanMessage>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <ZCanFdFrameV2 as TryFrom<CanMessage, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFromIterator<ZCanFdFrameV2, u64> for Vec<CanMessage> {
    type Error = ZCanError;
    fn try_from_iter<T: IntoIterator<Item=ZCanFdFrameV2>>(iter: T, timestamp: u64) -> Result<Self, Self::Error> {
        iter.into_iter()
            .map(|v| <CanMessage as TryFrom<ZCanFdFrameV2, u64>>::try_from(v, timestamp))
            .collect()
    }
}

impl TryFrom<ZCanChlErrorV1, u64> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanChlErrorV1, timestamp: u64) -> Result<Self, Self::Error> {
        let hdr = value.hdr;
        let mut message = CanMessage::new(
            hdr.can_id, Some(hdr.channel), value.data, false, true, None
        )?;
        message.set_timestamp(Some(fix_system_time(value.hdr.timestamp as u64, timestamp)));
        Ok(message)
    }
}

#[allow(unused_variables)]
impl TryFrom<ZCanChlErrorV2, ()> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanChlErrorV2, _: ()) -> Result<Self, Self::Error> {
        todo!()
    }
}

