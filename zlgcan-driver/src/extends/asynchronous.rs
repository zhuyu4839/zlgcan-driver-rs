use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, mpsc::{channel, Receiver, Sender}, Mutex, MutexGuard};
use std::time::Duration;
use can_type_rs::device::{AsyncCanDevice, CanListener};
use zlgcan_common::can::CanMessage;
use zlgcan_common::device::Handler;
use tokio::{spawn, time::sleep, task::JoinHandle};

use crate::driver::{ZCanDriver, ZDevice};
use crate::extends::{listener_names, receive_callback, register_listener, transmit_callback, unregister_all, unregister_listener};

#[derive(Clone)]
pub struct ZCanAsync {
    device: ZCanDriver,
    sender: Sender<CanMessage>,
    receiver: Arc<Mutex<Receiver<CanMessage>>>,
    listeners: Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
    stop_tx: Sender<()>,
    stop_rx: Arc<Mutex<Receiver<()>>>,
    send_task: Arc<Mutex<Option<JoinHandle<()>>>>,
    receive_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl From<ZCanDriver> for ZCanAsync {
    fn from(value: ZCanDriver) -> Self {
        Self::new(value)
    }
}

impl AsyncCanDevice for ZCanAsync {
    type Channel = u8;
    type Frame = CanMessage;
    type Device = ZCanDriver;
    fn new(device: Self::Device) -> Self {
        let (tx, rx) = channel();
        let (stop_tx, stop_rx) = channel();
        Self {
            device,
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
            listeners: Arc::new(Mutex::new(HashMap::new())),
            stop_tx,
            stop_rx: Arc::new(Mutex::new(stop_rx)),
            send_task: Arc::new(Mutex::new(None)),
            receive_task: Arc::new(Mutex::new(None)),
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
        listener: Box<dyn CanListener<Frame = Self::Frame, Channel = Self::Channel>>,
    ) -> bool {
        register_listener::<Self::Frame, Self::Channel>(&self.listeners, name, listener)
    }

    #[inline]
    fn unregister_listener(&mut self, name: String) -> bool {
        unregister_listener(&self.listeners, name)
    }

    #[inline]
    fn unregister_all(&mut self) -> bool {
        unregister_all(&self.listeners)
    }

    #[inline]
    fn listener_names(&self) -> Vec<String> {
        listener_names(&self.listeners)
    }

    fn async_transmit(device: Arc<Mutex<Self>>, interval_ms: u64, stopper: Arc<Mutex<Receiver<()>>>) -> impl Future<Output=()> + Send {
        async move {
            async_util(device, interval_ms, stopper, |_, device| {
                transmit_callback(&device.receiver, &device.device, &device.listeners)
            }).await;
        }
    }

    fn async_receive(device: Arc<Mutex<Self>>, interval_ms: u64, stopper: Arc<Mutex<Receiver<()>>>) -> impl Future<Output=()> + Send {
        async move {
            async_util(device, interval_ms, stopper, |handler, device| {
                receive_callback(&device.device, handler, &device.listeners)
            }).await;
        }
    }

    #[inline]
    fn async_start(&mut self, interval_ms: u64) {
        let tx_task = spawn(Self::async_transmit(Arc::new(Mutex::new(self.clone())), interval_ms, Arc::clone(&self.stop_rx)));
        let rx_task = spawn(Self::async_receive(Arc::new(Mutex::new(self.clone())), interval_ms, Arc::clone(&self.stop_rx)));
        if let Ok(mut task) = self.send_task.lock() {
            task.replace(tx_task);
        }
        if let Ok(mut task) = self.receive_task.lock() {
            task.replace(rx_task);
        }
    }

    #[inline]
    fn close(&mut self) -> impl Future<Output = ()> + Send {
        async {
            log::info!("ZLGCAN - closing(async)");

            if let Err(e) = self.stop_tx.send(()) {
                log::warn!("ZLGCAN - error: {} when sending stop signal", e);
            }

            if let Ok(mut task) = self.send_task.lock() {
                if let Some(task) = task.take() {
                    if !task.is_finished() {
                        task.abort();
                    }
                }
            }

            if let Ok(mut task) = self.receive_task.lock() {
                if let Some(task) = task.take() {
                    if !task.is_finished() {
                        task.abort();
                    }
                }
            }

            self.device.close();
        }
    }
}

#[inline]
async fn async_util(device: Arc<Mutex<ZCanAsync>>, interval: u64, stopper: Arc<Mutex<Receiver<()>>>, callback: fn(Handler, MutexGuard<ZCanAsync>)) {
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

        if let Ok(stopper) = stopper.lock() {
            if let Ok(()) = stopper.recv() {
                break
            }
        }
        sleep(Duration::from_millis(interval)).await;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use can_type_rs::device::AsyncCanDevice;
    use can_type_rs::frame::Frame;
    use can_type_rs::identifier::Id;
    use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, CanMessage, ZCanChlMode, ZCanChlType};
    use zlgcan_common::device::{ZCanDeviceType, ZCanError};
    use crate::driver::{ZCanDriver, ZDevice};
    use super::ZCanAsync;

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

        let mut device = ZCanAsync::from(device);
        device.async_start(10);

        let tmp_send = device.sender();

        let data = vec![0x02, 0x10, 0x01];
        let message = CanMessage::new(
            Id::from_bits(0x7DF, false),
            data.as_slice()
        )
            .ok_or(ZCanError::Other("invalid data length".to_string()))?;
        tmp_send.send(message).unwrap();

        let data = vec![0x02, 0x10, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut message = CanMessage::new(
            Id::from_bits(0x7DF, false),
            data.as_slice()
        )
            .ok_or(ZCanError::Other("invalid data length".to_string()))?;
        message.set_can_fd(true);
        message.set_bitrate_switch(true);
        tmp_send.send(message).unwrap();

        tokio::time::sleep(Duration::from_secs(2)).await;
        device.close().await;

        Ok(())
    }
}
