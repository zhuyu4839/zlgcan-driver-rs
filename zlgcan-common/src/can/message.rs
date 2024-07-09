use std::fmt::{Display, Formatter};
use std::slice;
use can_type_rs::{Direct, constant::{CAN_FRAME_MAX_SIZE, CANFD_FRAME_MAX_SIZE}, frame::Frame, identifier::Id};
use can_type_rs::j1939::J1939Id;
use crate::utils::system_timestamp;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CanMessage {
    timestamp: u64,
    arbitration_id: u32,
    is_extended_id: bool,
    is_remote_frame: bool,
    is_error_frame: bool,
    channel: u8,
    length: usize,
    data: *const u8,
    is_fd: bool,
    direct: Direct,
    bitrate_switch: bool,
    error_state_indicator: bool,
    tx_mode: u8,
}

unsafe impl Send for CanMessage {}
unsafe impl Sync for CanMessage {}

impl Frame for CanMessage {
    type Channel = u8;
    #[inline]
    fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> where Self: Sized {
        let length = data.len();
        match match length {
            ..=CAN_FRAME_MAX_SIZE => Some(false),
            ..=CANFD_FRAME_MAX_SIZE => Some(true),
            _ => None,
        } {
            Some(v) => {
                let id: Id = id.into();
                Some(Self {
                    timestamp: 0,
                    arbitration_id: id.as_raw(),
                    is_extended_id: id.is_extended(),
                    is_remote_frame: false,
                    is_error_frame: false,
                    channel: Default::default(),
                    length,
                    data: Box::leak(data.to_vec().into_boxed_slice()).as_ptr(),
                    is_fd: v,
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                    tx_mode: 0,
                })
            },
            None => None,
        }
    }

    #[inline]
    fn new_remote(id: impl Into<Id>, len: usize) -> Option<Self> where Self: Sized {
        match match len {
            ..=CAN_FRAME_MAX_SIZE => Some(false),
            ..=CANFD_FRAME_MAX_SIZE => Some(true),
            _ => None,
        } {
            Some(v) => {
                let id = id.into();
                let mut data = Vec::new();
                data.resize(len, Default::default());
                Some(Self {
                    timestamp: 0,
                    arbitration_id: id.as_raw(),
                    is_extended_id: id.is_extended(),
                    is_remote_frame: true,
                    is_error_frame: false,
                    channel: Default::default(),
                    length: len,
                    data: Box::leak(data.into_boxed_slice()).as_ptr(),
                    is_fd: v,
                    direct: Default::default(),
                    bitrate_switch: false,
                    error_state_indicator: false,
                    tx_mode: 0,
                })
            },
            None => None,
        }
    }

    #[inline]
    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    #[inline]
    fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self where Self: Sized {
        self.timestamp = value.unwrap_or_else(system_timestamp);
        self
    }

    #[inline]
    fn id(&self, j1939: bool) -> Id {
        if self.is_extended_id && j1939 {
            Id::J1939(J1939Id::from_bits(self.arbitration_id))
        }
        else {
            Id::from_bits(self.arbitration_id, self.is_extended_id)
        }
    }

    #[inline]
    fn is_can_fd(&self) -> bool {
        self.is_fd
    }

    #[inline]
    fn set_can_fd(&mut self, value: bool) -> &mut Self where Self: Sized {
        if !value {
            match self.length {
                9.. => {
                    log::warn!("resize a fd-frame to: {}", CAN_FRAME_MAX_SIZE);
                    self.length = CAN_FRAME_MAX_SIZE;
                },
                _ => {},
            }
        }
        self.is_fd = value;
        self
    }

    #[inline]
    fn is_remote(&self) -> bool {
        self.is_remote_frame
    }

    #[inline]
    fn is_extended(&self) -> bool {
        self.is_extended_id
    }

    #[inline]
    fn direct(&self) -> Direct {
        self.direct.clone()
    }

    #[inline]
    fn set_direct(&mut self, direct: Direct) -> &mut Self where Self: Sized {
        self.direct = direct;
        self
    }

    #[inline]
    fn is_bitrate_switch(&self) -> bool {
        self.bitrate_switch
    }

    #[inline]
    fn set_bitrate_switch(&mut self, value: bool) -> &mut Self where Self: Sized {
        self.is_error_frame = value;
        self
    }

    #[inline]
    fn is_error_frame(&self) -> bool {
        self.is_error_frame
    }

    #[inline]
    fn set_error_frame(&mut self, value: bool) -> &mut Self where Self: Sized {
        self.is_error_frame = value;
        self
    }

    #[inline]
    fn is_esi(&self) -> bool {
        self.error_state_indicator
    }

    #[inline]
    fn set_esi(&mut self, value: bool) -> &mut Self where Self: Sized {
        self.error_state_indicator = value;
        self
    }

    #[inline]
    fn channel(&self) -> Self::Channel {
        self.channel + 1
    }

    #[inline]
    fn set_channel(&mut self, value: Self::Channel) -> &mut Self where Self: Sized {
        self.channel = value;
        self
    }

    #[inline]
    fn data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.data, self.length) }
    }

    #[inline]
    fn dlc(&self) -> usize {
        let len = self.length;
        match len {
            ..=CAN_FRAME_MAX_SIZE => len,
            ..=CANFD_FRAME_MAX_SIZE => {
                if !self.is_fd {
                    panic!("Invalid data length!");
                }
                match len {
                    9..=12 => 9,
                    13..=16 => 10,
                    17..=20 => 11,
                    21..=24 => 12,
                    25..=32 => 13,
                    33..=48 => 14,
                    49..=64 => 15,
                    _ => panic!("Invalid length!"),
                }
            },
            _ => panic!("Invalid length!"),
        }
    }

    #[inline]
    fn length(&self) -> usize {
        self.length
    }
}

impl PartialEq for CanMessage {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }

        if self.is_remote_frame {
            other.is_remote_frame && (self.arbitration_id == other.arbitration_id)
        }
        else {
            let len = self.length;

            let data = unsafe { slice::from_raw_parts(self.data, self.length) };
            let other_data = unsafe { slice::from_raw_parts(other.data, other.length) };

            (self.arbitration_id == other.arbitration_id) &&
                (self.is_extended_id == other.is_extended_id) &&
                (self.is_error_frame == other.is_error_frame) &&
                (self.error_state_indicator == other.error_state_indicator) &&
                (data[..len] == other_data[..len])
        }
    }
}

impl CanMessage {
    #[inline(always)]
    pub const fn tx_mode(&self) -> u8 { self.tx_mode }
    #[inline(always)]
    pub fn set_tx_mode(&mut self, tx_mode: u8) -> &mut Self {
        self.tx_mode = if tx_mode > 3 { Default::default() } else { tx_mode };
        self
    }
}

impl Display for CanMessage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <dyn Frame<Channel=u8> as Display>::fmt(self, f)
    }
}
