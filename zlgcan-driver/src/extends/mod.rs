mod asynchronous;
pub use asynchronous::*;
mod synchronous;
pub use synchronous::*;


use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use can_type_rs::device::CanListener;
use can_type_rs::frame::Frame;
use can_type_rs::identifier::Id;
use zlgcan_common::can::{CanMessage, ZCanFrameType};
use zlgcan_common::device::Handler;
use crate::driver::{ZCanDriver, ZDevice};

#[inline]
pub(crate) fn register_listener<Frame, Channel>(
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = Frame, Channel = Channel>>>>>,
    name: String,
    listener: Box<dyn CanListener<Frame = Frame, Channel = Channel>>,
) -> bool {
    match listeners.lock() {
        Ok(mut v) => {
            v.insert(name, listener);
            true
        },
        Err(e) => {
            log::warn!("ZLGCAN - mutex error: {:?} when inserting listener", e);
            false
        },
    }
}

#[inline]
pub(crate) fn unregister_listener(
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
    name: String,
) -> bool {
    match listeners.lock() {
        Ok(mut v) => {
            v.remove(&name).is_some()
        },
        Err(e) => {
            log::warn!("ZLGCAN - mutex error: {:?} when removing listener", e);
            false
        },
    }
}

#[inline]
pub(crate) fn unregister_all(
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
) -> bool {
    match listeners.lock() {
        Ok(mut v) => {
            v.clear();
            true
        },
        Err(e) => {
            log::warn!("ZLGCAN - mutex error: {:?} when removing all listeners", e);
            false
        },
    }
}

#[inline]
pub(crate) fn listener_names(
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
) -> Vec<String> {
    match listeners.lock() {
        Ok(v) => {
            v.keys()
                .into_iter()
                .map(|f| f.clone())
                .collect()
        },
        Err(e) => {
            log::warn!("ZLGCAN - mutex error: {:?} when removing all listeners", e);
            vec![]
        },
    }
}

#[inline]
fn on_messages_util(
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
    messages: &Vec<CanMessage>,
    channel: u8
) {
    match listeners.lock() {
        Ok(mut v) => v.values_mut()
            .for_each(|o| {
                o.on_frame_received(messages, channel);
            }),
        Err(e) =>
            log::error!("ZLGCAN - mutex error: {e:?} `on_messages`"),
    }
}

#[inline]
fn on_transmit_util(
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
    id: Id,
    size: u32,
    channel: u8
) {
    if size > 0 {
        match listeners.lock() {
            Ok(mut v) => v.values_mut()
                .for_each(|o| {
                    o.on_frame_transmitted(id, channel);
                }),
            Err(e) =>
                log::error!("ZLGCAN - mutex error: {e:?} `on_transmit`"),
        }
    }
}

#[inline]
pub(crate) fn transmit_callback(
    receiver: &Arc<Mutex<Receiver<CanMessage>>>,
    device: &ZCanDriver,
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
) {
    if let Ok(receiver) = receiver.lock() {
        if let Ok(msg) = receiver.try_recv() {
            log::debug!("ZLGCAN - transmit: {}", msg);
            let channel = msg.channel();
            let fd = msg.is_can_fd();
            let id = msg.id(false);
            if fd {
                if let Ok(v) = device.transmit_canfd(channel, vec![msg, ]) {
                    on_transmit_util(listeners, id, v, channel);
                }
            }
            else {
                if let Ok(v) = device.transmit_can(channel, vec![msg, ]) {
                    on_transmit_util(listeners, id, v, channel);
                }
            }
        }
    }
}

#[inline]
pub(crate) fn receive_callback(
    device: &ZCanDriver,
    handler: Handler,
    listeners: &Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
) {
    let can_chs = handler.can_channels().len() as u8;
    for channel in 0..can_chs {
        if let Ok(count) = device.get_can_num(channel, ZCanFrameType::CAN) {
            if count > 0 {
                if let Ok(messages) = device.receive_can(channel, count, None) {
                    on_messages_util(listeners, &messages, channel);
                }
            }
        }

        if device.dev_type.canfd_support() {
            if let Ok(count) = device.get_can_num(channel, ZCanFrameType::CANFD) {
                if count > 0 {
                    if let Ok(messages) = device.receive_canfd(channel, count, None) {
                        on_messages_util(listeners, &messages, channel);
                    }
                }
            }
        }
    }
}

