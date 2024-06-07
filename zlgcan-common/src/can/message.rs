use std::fmt::Write;
use std::slice;
use crate::error::ZCanError;
use crate::utils::system_timestamp;
use super::constant::{CAN_EFF_MASK, CAN_FRAME_LENGTH, CAN_ID_FLAG, CANFD_FRAME_LENGTH};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct CanMessage {
    timestamp: u64,
    arbitration_id: u32,
    is_extended_id: bool,
    is_remote_frame: bool,
    is_error_frame: bool,
    channel: u8,
    len: u8,
    data: *const u8,
    is_fd: bool,
    is_rx: bool,
    bitrate_switch: bool,
    error_state_indicator: bool,
    tx_mode: u8,
}

impl PartialEq for CanMessage {
    fn eq(&self, other: &Self) -> bool {
        if self.len != other.len {
            return false;
        }

        if self.is_remote_frame {
            other.is_remote_frame && (self.arbitration_id == other.arbitration_id)
        }
        else {
            let len = self.len as usize;

            let data = unsafe { slice::from_raw_parts(self.data, self.len as usize) };
            let other_data = unsafe { slice::from_raw_parts(other.data, other.len as usize) };

            (self.arbitration_id == other.arbitration_id) &&
                (self.is_extended_id == other.is_extended_id) &&
                (self.is_error_frame == other.is_error_frame) &&
                (self.error_state_indicator == other.error_state_indicator) &&
                (data[..len] == other_data[..len])
        }
    }
}

impl std::fmt::Display for CanMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data_str = if self.is_remote_frame {
            " ".to_owned()
        } else {
            self.data().iter()
                .fold(String::new(), |mut out, &b| {
                    let _ = write!(out, "{b:02x} ");
                    out
                })
        };

        if self.is_fd {
            let mut flags = 1 << 12;
            write!(f, "CANFD {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                self.channel + 1,
                if self.is_rx { "Rx" } else { "Tx" },
                format!("{: >8x}", self.arbitration_id),
                if self.bitrate_switch {
                    flags |= 1 << 13;
                    1
                } else { 0 },
                if self.error_state_indicator {
                    flags |= 1 << 14;
                    1
                } else { 0 },
                format!("{: >2}", self.dlc().unwrap()),
                format!("{: >2}", self.len),
                data_str,
                format!("{: >8}", 0),       // message_duration
                format!("{: <4}", 0),       // message_length
                format!("{: >8x}", flags),
                format!("{: >8}", 0),       // crc
                format!("{: >8}", 0),       // bit_timing_conf_arb
                format!("{: >8}", 0),       // bit_timing_conf_data
                format!("{: >8}", 0),       // bit_timing_conf_ext_arb
                format!("{: >8}", 0),       // bit_timing_conf_ext_data
            )
        }
        else {
            write!(f, "{} {} {}{: <4} {} {} {} {}",
                self.timestamp as f64 / 1000.,
                self.channel + 1,
                format!("{: >8x}", self.arbitration_id),
                if self.is_extended_id { "x" } else { "" },
                if self.is_rx { "Rx" } else { "Tx" },
                if self.is_remote_frame { "r" } else { "d" },
                format!("{: >2}", self.len),
                data_str,
            )
        }
    }
}

impl CanMessage {
    pub fn new<T>(
        arbitration_id: u32,
        channel: Option<u8>,
        data: T,
        is_fd: bool,
        is_error_frame: bool,
        is_extended_id: Option<bool>
    ) -> Result<Self, ZCanError>
        where
            T: AsRef<[u8]>  {
        match arbitration_id {
            0..=CAN_ID_FLAG => {
                let data = Vec::from(data.as_ref());
                let len = data.len();

                if (is_fd && len > CANFD_FRAME_LENGTH) ||
                    (!is_fd && len > CAN_FRAME_LENGTH) {
                    Err(ZCanError::ParamNotSupported)
                }
                else {
                    Ok(Self {
                        timestamp: 0,
                        arbitration_id,
                        is_extended_id: is_extended_id.unwrap_or_default() | (arbitration_id & CAN_EFF_MASK > 0),
                        is_remote_frame: false,
                        is_error_frame,
                        channel: channel.unwrap_or(0),
                        len: len as u8,
                        data: Box::leak(data.into_boxed_slice()).as_ptr(),
                        is_fd,
                        is_rx: true,
                        bitrate_switch: false,
                        error_state_indicator: false,
                        tx_mode: Default::default(),
                    })
                }
            },
            _ => Err(ZCanError::ParamNotSupported),
        }
    }
    #[inline(always)]
    pub const fn timestamp(&self) -> u64 { self.timestamp }
    #[inline(always)]
    pub fn set_timestamp(&mut self, value: Option<u64>) -> &mut Self {
        self.timestamp = value.unwrap_or_else(system_timestamp);
        self
    }
    #[inline(always)]
    pub const fn arbitration_id(&self) -> u32 { self.arbitration_id  }
    #[inline(always)]
    pub fn set_arbitration_id(&mut self, value: u32) -> &mut Self {
        self.arbitration_id = value;
        self
    }
    #[inline(always)]
    pub const fn is_extended_id(&self) -> bool { self.is_extended_id  }
    #[inline(always)]
    pub fn set_is_extended_id(&mut self, value: bool) -> &mut Self {
        match self.arbitration_id & 0xFFFF800 {
            0..=0x7FF => self.is_extended_id = value,
            _ => self.is_extended_id = true,
        }
        self
    }
    #[inline(always)]
    pub const fn is_remote_frame(&self) -> bool { self.is_remote_frame  }
    #[inline(always)]
    pub fn set_is_remote_frame(&mut self, value: bool) -> &mut Self {
        self.is_remote_frame = value;
        self
    }
    #[inline(always)]
    pub const fn is_error_frame(&self) -> bool { self.is_error_frame  }
    #[inline(always)]
    pub fn set_is_error_frame(&mut self, value: bool) -> &mut Self {
        self.is_remote_frame = value;
        self
    }
    #[inline(always)]
    pub const fn channel(&self) -> u8 { self.channel  }
    #[inline(always)]
    pub fn set_channel(&mut self, value: u8) -> &mut Self {
        self.channel = value;
        self
    }
    #[inline(always)]
    pub const fn length(&self) -> u8 { self.len  }
    #[inline(always)]
    pub(crate) fn set_length(&mut self, len: u8) {
        self.len = len;
    }
    #[inline(always)]
    pub fn data(&self) -> &[u8] { unsafe { slice::from_raw_parts(self.data, self.len as usize) }  }
    #[inline(always)]
    pub const fn is_fd(&self) -> bool { self.is_fd  }
    #[inline(always)]
    pub fn set_is_fd(&mut self, value: bool) -> &mut Self {
        self.is_fd = value;
        self
    }
    #[inline(always)]
    pub const fn is_rx(&self) -> bool { self.is_rx  }
    #[inline(always)]
    pub fn set_is_rx(&mut self, value: bool) -> &mut Self {
        self.is_rx = value;
        self
    }
    #[inline(always)]
    pub const fn bitrate_switch(&self) -> bool { self.bitrate_switch  }
    #[inline(always)]
    pub fn set_bitrate_switch(&mut self, value: bool) -> &mut Self {
        self.bitrate_switch = value;
        self
    }
    #[inline(always)]
    pub const fn error_state_indicator(&self) -> bool { self.error_state_indicator  }
    #[inline(always)]
    pub fn set_error_state_indicator(&mut self, value: bool) -> &mut Self {
        self.error_state_indicator = value;
        self
    }
    #[inline(always)]
    pub const fn tx_mode(&self) -> u8 { self.tx_mode }
    #[inline(always)]
    pub fn set_tx_mode(&mut self, tx_mode: u8) -> &mut Self {
        self.tx_mode = if tx_mode > 3 { Default::default() } else { tx_mode };
        self
    }

    pub const fn dlc(&self) -> Option<u8> {
        if self.is_fd {
            match self.len as usize {
                0..=CAN_FRAME_LENGTH => Some(self.len),
                9..=12 => Some(9),
                13..=16 => Some(10),
                17..=20 => Some(11),
                21..=24 => Some(12),
                25..=32 => Some(13),
                33..=48 => Some(14),
                49..=64 => Some(15),
                _ => None,
            }
        }
        else {
            match self.len as usize {
                0..=CAN_FRAME_LENGTH => Some(self.len),
                _ => None,
            }
        }
    }
}

