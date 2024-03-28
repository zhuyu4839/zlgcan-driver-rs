use crate::can::constant::{CAN_EFF_FLAG, CAN_ERR_FLAG, CAN_ID_FLAG, CAN_RTR_FLAG, CANFD_BRS, CANFD_ESI, ZCanFrameType};
use crate::can::frame::NewZCanFrame;
use super::{
    channel::{ZCanChlErrorV1, ZCanChlErrorV2},
    constant::ZCanHdrInfoField,
    frame::{ZCanHdrInfo, ZCanFrameV1, ZCanFrameV2, ZCanFrameV3, ZCanFdFrameV1, ZCanFdFrameV2},
    message::CanMessage
};

pub(self) fn frame_new<T: NewZCanFrame>(msg: CanMessage, canfd: bool) -> T {
    let mut info: ZCanHdrInfo = Default::default();

    if canfd {
        info.set_field(ZCanHdrInfoField::FrameType, ZCanFrameType::CANFD as u8);
        if msg.bitrate_switch() {
            info.set_field(ZCanHdrInfoField::IsBitrateSwitch, 1);
        }
        if msg.error_state_indicator() {
            info.set_field(ZCanHdrInfoField::IsErrorStateIndicator, 1);
        }
    }
    else {
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

    let frame = T::new(msg.arbitration_id(), msg.channel().unwrap_or_default(), msg.data(), info).unwrap();
    assert_eq!(msg.is_extended_id(), info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0);

    frame
}

impl From<CanMessage> for ZCanFrameV1 {
    fn from(value: CanMessage) -> Self {
        frame_new::<Self>(value, false)
    }
}

impl From<ZCanFrameV1> for CanMessage {
    fn from(value: ZCanFrameV1) -> Self {
        let mut msg = CanMessage::new(
            value.can_id, None, value.data, false, false, Some(value.ext_flag > 0)
        ).unwrap();

        msg.set_is_remote_frame(value.rem_flag > 0);
        msg
    }
}

impl FromIterator<CanMessage> for Vec<ZCanFrameV1> {
    fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
        let mut results = Vec::new();
        for msg in iter {
            results.push(ZCanFrameV1::from(msg));
        }
        results
    }
}

impl FromIterator<ZCanFrameV1> for Vec<CanMessage> {
    fn from_iter<T: IntoIterator<Item=ZCanFrameV1>>(iter: T) -> Self {
        let mut results = Vec::new();
        for frame in iter {
            results.push(CanMessage::from(frame));
        }
        results
    }
}

impl From<CanMessage> for ZCanFrameV2 {
    fn from(value: CanMessage) -> Self {
        frame_new::<Self>(value, false)
    }
}

impl From<ZCanFrameV2> for CanMessage {
    fn from(value: ZCanFrameV2) -> Self {
        let hdr = value.hdr;
        let info = hdr.info;
        let mut msg = CanMessage::new(
            hdr.can_id, Some(hdr.channel), value.data, false, false, Some(info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0)
        ).unwrap();

        msg.set_is_remote_frame(info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0)
            .set_is_error_frame(info.get_field(ZCanHdrInfoField::IsRemoteFrame) > 0);

        msg
    }
}

impl FromIterator<CanMessage> for Vec<ZCanFrameV2> {
    fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
        let mut results = Vec::new();
        for msg in iter {
            results.push(ZCanFrameV2::from(msg));
        }
        results
    }
}

impl FromIterator<ZCanFrameV2> for Vec<CanMessage> {
    fn from_iter<T: IntoIterator<Item=ZCanFrameV2>>(iter: T) -> Self {
        let mut results = Vec::new();
        for frame in iter {
            results.push(CanMessage::from(frame));
        }
        results
    }
}

impl From<CanMessage> for ZCanFrameV3 {
    fn from(value: CanMessage) -> Self {
        frame_new::<Self>(value, false)
    }
}

impl From<ZCanFrameV3> for CanMessage {
    fn from(value: ZCanFrameV3) -> Self {
        let hdr = value.hdr;

        let can_id = hdr.can_id;
        let mut msg = CanMessage::new(
            can_id & CAN_ID_FLAG, None, value.data, false, false, Some((can_id & CAN_EFF_FLAG) > 0)
        ).unwrap();

        msg.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
           .set_is_error_frame(can_id & CAN_ERR_FLAG > 0);
        msg
    }
}

impl FromIterator<CanMessage> for Vec<ZCanFrameV3> {
    fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
        let mut results = Vec::new();
        for msg in iter {
            results.push(ZCanFrameV3::from(msg));
        }
        results
    }
}

impl FromIterator<ZCanFrameV3> for Vec<CanMessage> {
    fn from_iter<T: IntoIterator<Item=ZCanFrameV3>>(iter: T) -> Self {
        let mut results = Vec::new();
        for frame in iter {
            results.push(CanMessage::from(frame));
        }
        results
    }
}


impl From<CanMessage> for ZCanFdFrameV1 {
    fn from(value: CanMessage) -> Self {
        frame_new::<Self>(value, true)
    }
}

impl From<ZCanFdFrameV1> for CanMessage {
    fn from(value: ZCanFdFrameV1) -> Self {
        let hdr = value.hdr;
        let info = hdr.info;

        let can_id = hdr.can_id;
        let mut msg = CanMessage::new(
            can_id, None, value.data.data, true, false, Some( info.get_field(ZCanHdrInfoField::IsExtendFrame) > 0)
        ).unwrap();

        msg.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0)
            .set_bitrate_switch(info.get_field(ZCanHdrInfoField::IsBitrateSwitch) > 0)
            .set_error_state_indicator(info.get_field(ZCanHdrInfoField::IsErrorStateIndicator) > 0);
        msg
    }
}

impl FromIterator<CanMessage> for Vec<ZCanFdFrameV1> {
    fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
        let mut results = Vec::new();
        for msg in iter {
            results.push(ZCanFdFrameV1::from(msg));
        }
        results
    }
}

impl FromIterator<ZCanFdFrameV1> for Vec<CanMessage> {
    fn from_iter<T: IntoIterator<Item=ZCanFdFrameV1>>(iter: T) -> Self {
        let mut results = Vec::new();
        for frame in iter {
            results.push(CanMessage::from(frame));
        }
        results
    }
}

impl From<CanMessage> for ZCanFdFrameV2 {
    fn from(value: CanMessage) -> Self {
        frame_new::<Self>(value, true)
    }
}

impl From<ZCanFdFrameV2> for CanMessage {
    fn from(value: ZCanFdFrameV2) -> Self {
        let hdr = value.hdr;

        let can_id = hdr.can_id;
        let mut msg = CanMessage::new(
            can_id & CAN_ID_FLAG, None, value.data.data, true, false, Some((can_id & CAN_EFF_FLAG) > 0)
        ).unwrap();

        let flag = hdr.flag;
        msg.set_is_remote_frame(can_id & CAN_RTR_FLAG > 0)
            .set_is_error_frame(can_id & CAN_ERR_FLAG > 0)
            .set_bitrate_switch(flag & CANFD_BRS > 0)
            .set_error_state_indicator(flag & CANFD_ESI > 0);
        msg
    }
}

impl FromIterator<CanMessage> for Vec<ZCanFdFrameV2> {
    fn from_iter<T: IntoIterator<Item=CanMessage>>(iter: T) -> Self {
        let mut results = Vec::new();
        for msg in iter {
            results.push(ZCanFdFrameV2::from(msg));
        }
        results
    }
}

impl FromIterator<ZCanFdFrameV2> for Vec<CanMessage> {
    fn from_iter<T: IntoIterator<Item=ZCanFdFrameV2>>(iter: T) -> Self {
        let mut results = Vec::new();
        for frame in iter {
            results.push(CanMessage::from(frame));
        }
        results
    }
}

impl From<ZCanChlErrorV1> for CanMessage {
    fn from(value: ZCanChlErrorV1) -> Self {
        let hdr = value.hdr;
        let msg = CanMessage::new(
            hdr.can_id, Some(hdr.channel), value.data, false, true, None
        ).unwrap();

        msg
    }
}

#[warn(unused_variables)]
impl From<ZCanChlErrorV2> for CanMessage {
    fn from(value: ZCanChlErrorV2) -> Self {
        todo!()
    }
}

