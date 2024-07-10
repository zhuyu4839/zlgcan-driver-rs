use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, mpsc::{channel, Receiver, Sender}, Mutex, MutexGuard};
use can_type_rs::{AsyncCanDevice, CanListener};
use can_type_rs::frame::Frame;
use can_type_rs::identifier::Id;
use zlgcan_common::can::{CanMessage, ZCanFrameType};
use zlgcan_common::device::Handler;
use zlgcan_common::error::ZCanError;

use crate::driver::{ZCanDriver, ZDevice};

#[derive(Clone)]
pub struct ZCanExtend {
    device: ZCanDriver,
    sender: Sender<CanMessage>,
    receiver: Arc<Mutex<Receiver<CanMessage>>>,
    listeners: Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage>>>>>,
}

impl From<ZCanDriver> for ZCanExtend {
    fn from(value: ZCanDriver) -> Self {
        Self::new(value)
    }
}

impl AsyncCanDevice for ZCanExtend {
    type Frame = CanMessage;
    type Device = ZCanDriver;
    fn new(device: Self::Device) -> Self {
        let (tx, rx) = channel();
        Self {
            device,
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
            listeners: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[inline]
    fn sender(&self) -> Sender<Self::Frame> {
        self.sender.clone()
    }

    #[inline]
    fn register_listener(
        &mut self,
        name: String,
        listener: Box<dyn CanListener<Frame = Self::Frame>>
    ) -> anyhow::Result<()> {
        self.listeners.lock()
            .map_err(|e|
                ZCanError::Other(format!("mutex error: {e:?} when inserting listener")))?
            .insert(name, listener)
            .ok_or(ZCanError::Other("failed to insert listener".to_string()))?;

        Ok(())
    }

    #[inline]
    fn unregister_listener(&mut self, name: String) -> anyhow::Result<bool> {
        Ok(self.listeners.lock()
            .map_err(|e|
                ZCanError::Other(format!("mutex error: {e:?} when inserting listener")))?
            .remove(&name)
            .is_some())
    }

    #[inline]
    fn unregister_all(&mut self) -> anyhow::Result<()> {
        Ok(self.listeners.lock()
            .map_err(|e|
                ZCanError::Other(format!("mutex error: {e:?} when inserting listener")))?
            .clear())
    }

    #[inline]
    fn listener_names(&self) -> anyhow::Result<Vec<String>> {
        Ok(self.listeners.lock()
            .map_err(|e|
                ZCanError::Other(format!("mutex error: {e:?} when inserting listener")))?
            .keys()
            .into_iter()
            .map(|f| f.clone())
            .collect())
    }

    fn async_transmit(device: Arc<Mutex<Self>>, interval_ms: u64) -> impl Future<Output=()> + Send {
        async move {
            async_util(device, interval_ms, |_, device| {
                if let Ok(msg) = device.receiver.lock().unwrap().try_recv() {
                    log::debug!("ZLGCAN - transmit: {}", msg);
                    let channel = msg.channel();
                    let fd = msg.is_can_fd();
                    let id = msg.id(false);
                    if fd {
                        if let Ok(v) = device.device.transmit_canfd(channel, vec![msg, ]) {
                            on_transmit_util(&device, id, v);
                        }
                    }
                    else {
                        if let Ok(v) = device.device.transmit_can(channel, vec![msg, ]) {
                            on_transmit_util(&device, id, v);
                        }
                    }
                }
            }).await;
        }
    }

    fn async_receive(device: Arc<Mutex<Self>>, interval_ms: u64) -> impl Future<Output=()> + Send {
        async move {
            async_util(device, interval_ms, |handler, device| {
                let dev = &device.device;
                let can_chs = handler.can_channels().len() as u8;
                for channel in 0..can_chs {
                    if let Ok(count) = dev.get_can_num(channel, ZCanFrameType::CAN) {
                        if count > 0 {
                            if let Ok(messages) = dev.receive_can(channel, count, None) {
                                on_messages_util(&device, &messages);
                            }
                        }
                    }

                    if dev.dev_type.canfd_support() {
                        if let Ok(count) = dev.get_can_num(channel, ZCanFrameType::CANFD) {
                            if count > 0 {
                                if let Ok(messages) = dev.receive_canfd(channel, count, None) {
                                    on_messages_util(&device, &messages);
                                }
                            }
                        }
                    }
                }
            }).await;
        }
    }

    #[inline]
    fn async_start(&self, interval_ms: u64) {
        tokio::spawn(ZCanExtend::async_transmit(Arc::new(Mutex::new(self.clone())), interval_ms));
        tokio::spawn(ZCanExtend::async_receive(Arc::new(Mutex::new(self.clone())), interval_ms));
    }

    #[inline]
    fn close(&mut self) {
        self.device.close();
    }
}

#[inline]
async fn async_util(device: Arc<Mutex<ZCanExtend>>, interval: u64, callback: fn(Handler, MutexGuard<ZCanExtend>)) {
    loop {
        if let Ok(device) = device.lock() {
            if let Some(handler) = device.device.handler.clone() {
                callback(handler, device);
            }
            else {
                log::info!("ZLGCAN - exit async receive.");
                break;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(interval)).await;
    }
}

#[inline]
fn on_messages_util(device: &MutexGuard<ZCanExtend>, messages: &Vec<CanMessage>) {
    match device.listeners.lock() {
        Ok(v) => v.values()
            .for_each(|o| if let Err(e) = o.on_frame_received(messages) {
                println!("{e:?}");
            }),
        Err(e) =>
            log::error!("ZLGCAN - mutex error: {e:?} `on_messages`"),
    }
}

#[inline]
fn on_transmit_util(device: &MutexGuard<ZCanExtend>, id: Id, size: u32) {
    if size > 0 {
        match device.listeners.lock() {
            Ok(v) => v.values()
                .for_each(|o| if let Err(e) = o.on_frame_transmitted(id) {
                    println!("{e:?}");
                }),
            Err(e) =>
                log::error!("ZLGCAN - mutex error: {e:?} `on_transmit`"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use can_type_rs::AsyncCanDevice;
    use can_type_rs::frame::Frame;
    use can_type_rs::identifier::Id;
    use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, CanMessage, ZCanChlMode, ZCanChlType};
    use zlgcan_common::device::ZCanDeviceType;
    use zlgcan_common::error::ZCanError;
    use crate::driver::{ZCanDriver, ZDevice};
    use crate::extends::ZCanExtend;

    #[tokio::test]
    async fn can_trait() -> anyhow::Result<()> {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let mut device = ZCanDriver::new(dev_type as u32, 0, None)?;
        device.open()?;

        let factory = CanChlCfgFactory::new()?;
        let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                              CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None))?;
        let ch2_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CANFD_ISO as u8, ZCanChlMode::Normal as u8, 500_000,
                                              CanChlCfgExt::new(None, Some(1_000_000), None, None, None, None))?;
        let cfg = vec![ch1_cfg, ch2_cfg];
        device.init_can_chl(cfg)?;

        let mut device = ZCanExtend::from(device);
        device.async_start(10);

        let tmp_send = device.sender();

        let data = vec![0x02, 0x10, 0x01];
        let message = CanMessage::new(
            Id::from_bits(0x7DF, false),
            data.as_slice()
        )?;
        tmp_send.send(message).unwrap();

        let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut message = CanMessage::new(
            Id::from_bits(0x7DF, false),
            data.as_slice()
        )?;
        message.set_can_fd(true);
        message.set_bitrate_switch(true);
        tmp_send.send(message).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;
        device.close();

        Ok(())
    }
}

