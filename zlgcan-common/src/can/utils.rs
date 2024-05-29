use crate::can::constant::{CAN_EFF_FLAG, CAN_ERR_FLAG, CAN_ID_FLAG, CAN_RTR_FLAG, CANFD_BRS, CANFD_ESI, ZCanFrameType};
use crate::can::frame::NewZCanFrame;
use crate::error::ZCanError;
use super::{
    channel::{ZCanChlErrorV1, ZCanChlErrorV2},
    constant::ZCanHdrInfoField,
    frame::{ZCanHdrInfo, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3, ZCanFdFrameV1, ZCanFdFrameV2},
    message::CanMessage
};

fn frame_new<T: NewZCanFrame>(msg: CanMessage, canfd: bool) -> Result<T, ZCanError> {
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

    T::new(msg.arbitration_id(), msg.channel(), msg.data(), info)
}

impl TryFrom<CanMessage> for ZCanFrameV1 {
    type Error = ZCanError;
    fn try_from(value: CanMessage) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, false)
    }
}

impl TryFrom<ZCanFrameV1> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFrameV1) -> Result<Self, Self::Error> {
        let mut message = CanMessage::new(
            value.can_id,
            None,
            value.data,
            false,
            false,
            Some(value.ext_flag > 0)
        )?;
        message.set_length(value.len);
        message.set_timestamp(None);
        message.set_is_remote_frame(value.rem_flag > 0);
        Ok(message)
    }
}

// impl FromIterator<CanMessage> for Vec<ZCanFrameV1> {
//     fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for msg in iter {
//             results.push(ZCanFrameV1::try_from(msg).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }
//
// impl FromIterator<ZCanFrameV1> for Vec<CanMessage> {
//     fn from_iter<T: IntoIterator<Item=ZCanFrameV1>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for frame in iter {
//             results.push(CanMessage::try_from(frame).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }

impl TryFrom<CanMessage> for ZCanFrameV2 {
    type Error = ZCanError;
    fn try_from(value: CanMessage) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, false)
    }
}

impl TryFrom<ZCanFrameV2> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFrameV2) -> Result<Self, Self::Error> {
        let hdr = value.hdr;
        let info = hdr.info;
        let mut message = CanMessage::new(
            hdr.can_id, Some(hdr.channel), value.data, false, false, Some(info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0)
        )?;
        message.set_length(hdr.len);
        message.set_timestamp(None);
        message.set_is_remote_frame(info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0)
            .set_is_error_frame(info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0);
        Ok(message)
    }
}

// impl FromIterator<CanMessage> for Vec<ZCanFrameV2> {
//     fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for msg in iter {
//             results.push(ZCanFrameV2::try_from(msg).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }
//
// impl FromIterator<ZCanFrameV2> for Vec<CanMessage> {
//     fn from_iter<T: IntoIterator<Item=ZCanFrameV2>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for frame in iter {
//             results.push(CanMessage::try_from(frame).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }

impl TryFrom<CanMessage> for ZCanFrameV3 {
    type Error = ZCanError;
    fn try_from(value: CanMessage) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, false)
    }
}

impl TryFrom<ZCanFrameV3> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFrameV3) -> Result<Self, Self::Error> {
        let hdr = value.hdr;

        let can_id = hdr.can_id;
        let mut message = CanMessage::new(
            can_id & CAN_ID_FLAG, Some(hdr.__res0), value.data, false, false, Some((can_id & CAN_EFF_FLAG) > 0)
        )?;
        message.set_length(hdr.can_len);
        message.set_timestamp(None);
        message.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0);
        Ok(message)
    }
}

// impl FromIterator<CanMessage> for Vec<ZCanFrameV3> {
//     fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for msg in iter {
//             results.push(ZCanFrameV3::try_from(msg).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }
//
// impl FromIterator<ZCanFrameV3> for Vec<CanMessage> {
//     fn from_iter<T: IntoIterator<Item=ZCanFrameV3>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for frame in iter {
//             results.push(CanMessage::try_from(frame).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }


impl TryFrom<CanMessage> for ZCanFdFrameV1 {
    type Error = ZCanError;
    fn try_from(value: CanMessage) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, true)
    }
}

impl TryFrom<ZCanFdFrameV1> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFdFrameV1) -> Result<Self, Self::Error> {
        let hdr = value.hdr;
        let info = hdr.info;

        let can_id = hdr.can_id;
        let mut message = CanMessage::new(
            can_id, Some(hdr.channel), value.data.data, true, false, Some( info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0)
        )?;
        message.set_length(hdr.len);
        message.set_timestamp(None);
        message.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0)
            .set_bitrate_switch(info.get_field(ZCanHdrInfoField::IsBitrateSwitch) > 0)
            .set_error_state_indicator(info.get_field(ZCanHdrInfoField::IsErrorStateIndicator) > 0);
        Ok(message)
    }
}

// impl FromIterator<CanMessage> for Vec<ZCanFdFrameV1> {
//     fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for msg in iter {
//             results.push(ZCanFdFrameV1::try_from(msg).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }
//
// impl FromIterator<ZCanFdFrameV1> for Vec<CanMessage> {
//     fn from_iter<T: IntoIterator<Item=ZCanFdFrameV1>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for frame in iter {
//             results.push(CanMessage::try_from(frame).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }

impl TryFrom<CanMessage> for ZCanFdFrameV2 {
    type Error = ZCanError;
    fn try_from(value: CanMessage) -> Result<Self, Self::Error> {
        frame_new::<Self>(value, true)
    }
}

impl TryFrom<ZCanFdFrameV2> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanFdFrameV2) -> Result<Self, Self::Error> {
        let hdr = value.hdr;

        let can_id = hdr.can_id;
        let mut message = CanMessage::new(
            can_id & CAN_ID_FLAG, Some(hdr.__res0), value.data.data, true, false, Some((can_id & CAN_EFF_FLAG) > 0)
        )?;
        message.set_length(hdr.can_len);
        message.set_timestamp(None);
        let flag = hdr.flag;
        message.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0)
            .set_bitrate_switch(flag & CANFD_BRS > 0)
            .set_error_state_indicator(flag & CANFD_ESI > 0);
        Ok(message)
    }
}

// impl FromIterator<CanMessage> for Vec<ZCanFdFrameV2> {
//     fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for msg in iter {
//             results.push(ZCanFdFrameV2::try_from(msg).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }
//
// impl FromIterator<ZCanFdFrameV2> for Vec<CanMessage> {
//     fn from_iter<T: IntoIterator<Item=ZCanFdFrameV2>>(iter: T) -> Self {
//         let mut results = Vec::new();
//         for frame in iter {
//             results.push(CanMessage::try_from(frame).expect("ZLGCAN - Can't convert message!"));
//         }
//         results
//     }
// }

impl TryFrom<ZCanChlErrorV1> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanChlErrorV1) -> Result<Self, Self::Error> {
        let hdr = value.hdr;
        let mut message = CanMessage::new(
            hdr.can_id, Some(hdr.channel), value.data, false, true, None
        )?;
        message.set_timestamp(None);
        Ok(message)
    }
}

#[allow(unused_variables)]
impl TryFrom<ZCanChlErrorV2> for CanMessage {
    type Error = ZCanError;
    fn try_from(value: ZCanChlErrorV2) -> Result<Self, Self::Error> {
        todo!()
    }
}

