use can_type_rs::{constant::{IdentifierFlags, SFF_MASK}, frame::{Frame, Direct}, identifier::Id};
use can_type_rs::constant::EFF_MASK;
use crate::can::constant::{CANFD_BRS, CANFD_ESI, ZCanFrameType};
use crate::can::frame::NewZCanFrame;
use crate::{TryFrom, TryFromIterator};
use crate::error::ZCanError;
use crate::utils::{fix_device_time, fix_system_time};
use super::{
    channel::{ZCanChlErrorV1, ZCanChlErrorV2},
    constant::ZCanHdrInfoField,
    frame::{ZCanHdrInfo, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3, ZCanFdFrameV1, ZCanFdFrameV2},
    message::CanMessage
};

fn frame_new<T: NewZCanFrame<Error = ZCanError>>(
    msg: CanMessage,
    canfd: bool,
    timestamp: u64
) -> Result<T, ZCanError> {
    let mut info: ZCanHdrInfo = Default::default();

    if canfd {
        info.set_field(ZCanHdrInfoField::TxMode, msg.tx_mode());
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CANFD as u8);
        if msg.is_bitrate_switch() {
            info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 1);
        }
        if msg.is_esi() {
            info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 1);
        }
    }
    else {
        info.set_field(ZCanHdrInfoField::TxMode, msg.tx_mode());
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CAN as u8);
    }

    if msg.is_extended() {
        info.set_field(ZCanHdrInfoField::IsExtendFrame, 1);
    }
    if msg.is_remote() {
        info.set_field(ZCanHdrInfoField::IsRemoteFrame, 1);
    }
    if msg.is_error_frame() {
        info.set_field(ZCanHdrInfoField::IsErrorFrame, 1);
    }

    T::new(match msg.id(false) {
        Id::Standard(v) => v as u32,
        Id::Extended(v) => v,
        Id::J1939(v) => v.into_bits(),
    },
           msg.channel(),
           msg.data(),
           info,
           fix_device_time(timestamp)
    )
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
        let id = if value.ext_flag > 0 {
            Id::Extended(value.can_id)
        }
        else {
            Id::Standard(value.can_id as u16)
        };
        let mut message = if value.rem_flag > 0 {
            CanMessage::new_remote(id, value.len as usize)
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }
        else {
            let mut data = value.data.to_vec();
            data.resize(value.len as usize, Default::default());
            CanMessage::new(id, data.as_slice())
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }?;

        message.set_direct(Direct::Receive)
            .set_timestamp(Some(fix_system_time(value.timestamp as u64, timestamp)))
            .set_channel(value.channel);

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

        let id = if info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0 {
            Id::Extended(hdr.can_id)
        }
        else {
            Id::Standard(hdr.can_id as u16)
        };
        let mut message = if info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0 {
            CanMessage::new_remote(id, hdr.len as usize)
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }
        else {
            let mut data = value.data.to_vec();
            data.resize(hdr.len as usize, Default::default());
            CanMessage::new(id, data.as_slice())
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }?;

        message.set_direct(Direct::Receive)
            .set_timestamp(Some(fix_system_time(value.hdr.timestamp as u64, timestamp)))
            .set_channel(hdr.channel)
            .set_error_frame(info.get_field(ZCanHdrInfoField::IsErrorFrame) > 0);

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

        let id = if (can_id & IdentifierFlags::EXTENDED.bits()) > 0 {
            Id::Extended(hdr.can_id)
        }
        else {
            Id::Standard(hdr.can_id as u16)
        };
        let mut message = if can_id & IdentifierFlags::REMOTE.bits() > 0 {
            CanMessage::new_remote(id, hdr.can_len as usize)
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }
        else {
            let mut data = value.data.to_vec();
            data.resize(hdr.can_len as usize, Default::default());
            CanMessage::new(id, data.as_slice())
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }?;

        message.set_direct(Direct::Receive)
            .set_timestamp(Some(fix_system_time(value.ts_or_mode as u64, timestamp)))
            .set_channel(hdr.__res0)
            .set_error_frame((can_id & IdentifierFlags::ERROR.bits()) > 0);

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

        let id = if info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0 {
            Id::Extended(can_id)
        }
        else {
            Id::Standard(can_id as u16)
        };
        let mut message = if can_id & IdentifierFlags::REMOTE.bits() > 0 {
            CanMessage::new_remote(id, hdr.len as usize)
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }
        else {
            let mut data = value.data.data.to_vec();
            data.resize(hdr.len as usize, Default::default());
            CanMessage::new(id, data.as_slice())
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }?;

        message.set_direct(Direct::Receive)
            .set_can_fd(true)
            .set_timestamp(Some(fix_system_time(hdr.timestamp as u64, timestamp)))
            .set_channel(hdr.channel)
            .set_error_frame((can_id & IdentifierFlags::ERROR.bits()) > 0)
            .set_bitrate_switch(info.get_field(ZCanHdrInfoField::IsBitrateSwitch) > 0)
            .set_esi(info.get_field(ZCanHdrInfoField::IsErrorStateIndicator) > 0);

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
        let flag = hdr.flag;

        let id = if (can_id & IdentifierFlags::EXTENDED.bits()) > 0 {
            Id::Extended(can_id & EFF_MASK)
        }
        else {
            Id::Standard((can_id & SFF_MASK) as u16)
        };
        let mut message = if can_id & IdentifierFlags::REMOTE.bits() > 0 {
            CanMessage::new_remote(id, hdr.can_len as usize)
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }
        else {
            let mut data = value.data.data.to_vec();
            data.resize(hdr.can_len as usize, Default::default());
            CanMessage::new(id, data.as_slice())
                .ok_or(ZCanError::Other("invalid data length".to_string()))
        }?;

        message.set_direct(Direct::Receive)
            .set_can_fd(true)
            .set_timestamp(Some(fix_system_time(value.ts_or_mode as u64, timestamp)))
            .set_channel(hdr.__res0)
            .set_error_frame(can_id & IdentifierFlags::ERROR.bits() > 0)
            .set_bitrate_switch(flag & CANFD_BRS > 0)
            .set_esi(flag & CANFD_ESI > 0);

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
        let info = hdr.info;

        let id = if info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0 {
            Id::Extended(hdr.can_id)
        }
        else {
            Id::Standard(hdr.can_id  as u16)
        };
        let mut data = value.data.to_vec();
        data.resize(hdr.len as usize, Default::default());
        let mut message = CanMessage::new(id, data.as_slice())
            .ok_or(ZCanError::Other("invalid data length".to_string()))?;

        message.set_direct(Direct::Receive)
            .set_timestamp(Some(fix_system_time(hdr.timestamp as u64, timestamp)))
            .set_channel(hdr.channel)
            .set_error_frame(true);

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

