use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, mpsc::{channel, Receiver, Sender}, Mutex, MutexGuard, Weak};
use std::time::Duration;
use can_type_rs::identifier::Id;
use isotp_rs::device::{AsyncDevice, Listener};
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
    listeners: Arc<Mutex<HashMap<String, Box<dyn Listener<u8, Id, CanMessage>>>>>,
    stop_tx: Sender<()>,
    stop_rx: Arc<Mutex<Receiver<()>>>,
    send_task: Weak<JoinHandle<()>>,
    receive_task: Weak<JoinHandle<()>>,
}

impl From<ZCanDriver> for ZCanAsync {
    fn from(value: ZCanDriver) -> Self {
        Self::new(value)
    }
}

impl AsyncDevice for ZCanAsync {
    type Channel = u8;
    type Tx = Id;
    type Rx = CanMessage;
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
            send_task: Default::default(),
            receive_task: Default::default(),
        }
    }

    #[inline]
    fn sender(&self) -> Sender<Self::Rx> {
        self.sender.clone()
    }

    #[inline]
    fn register_listener(
        &mut self,
        name: String,
        listener: Box<dyn Listener<Self::Channel, Self::Tx, Self::Rx>>,
    ) -> bool {
        register_listener(&self.listeners, name, listener)
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

        let tx_task = Arc::new(tx_task);
        let rx_task = Arc::new(rx_task);

        self.send_task = Arc::downgrade(&tx_task);
        self.receive_task = Arc::downgrade(&rx_task);
    }

    #[inline]
    fn close(&mut self) -> impl Future<Output = ()> + Send {
        async {
            log::info!("ZLGCAN - closing(async)");

            if let Err(e) = self.stop_tx.send(()) {
                log::warn!("ZLGCAN - error: {} when sending stop signal", e);
            }

            if let Some(task) = self.send_task.upgrade() {
                if !task.is_finished() {
                    task.abort()
                }
            }

            if let Some(task) = self.receive_task.upgrade() {
                if !task.is_finished() {
                    task.abort()
                }
            }

            self.device.close();
        }
    }
}

#[inline]
async fn async_util(device: Arc<Mutex<ZCanAsync>>,
                    interval: u64,
                    stopper: Arc<Mutex<Receiver<()>>>,
                    callback: fn(Handler, MutexGuard<ZCanAsync>)
) {
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
            if let Ok(()) = stopper.try_recv() {
                break
            }
        }
        sleep(Duration::from_millis(interval)).await;
    }
}

#[cfg(test)]
mod tests {
    use isotp_rs::device::AsyncDevice;
    use can_type_rs::isotp::Address;
    use can_type_rs::isotp::{AsyncCanIsoTp, CanIsoTp};
    use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, ZCanChlMode, ZCanChlType};
    use zlgcan_common::device::ZCanDeviceType;
    use crate::driver::{ZCanDriver, ZDevice};
    use crate::extends::ZCanAsync;

    #[tokio::test]
    async fn test_device() -> anyhow::Result<()> {
        let dev_type = ZCanDeviceType::ZCAN_USBCANFD_200U;
        let mut device = ZCanDriver::new(dev_type as u32, 0, None)?;
        device.open()?;

        let factory = CanChlCfgFactory::new()?;
        let ch1_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CAN as u8, ZCanChlMode::Normal as u8, 500_000,
                                              CanChlCfgExt::default())?;
        let ch2_cfg = factory.new_can_chl_cfg(dev_type as u32, ZCanChlType::CAN as u8, ZCanChlMode::Normal as u8, 500_000,
                                              CanChlCfgExt::default())?;
        let cfg = vec![ch1_cfg, ch2_cfg];
        device.init_can_chl(cfg)?;

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let mut device = ZCanAsync::from(device);

        let mut isotp = AsyncCanIsoTp::new(0, Address {
            tx_id: 0x7E4,
            rx_id: 0x7EC,
            fid: 0x7DF,
        });
        device.register_listener("UdsClient".to_string(), isotp.get_frame_listener());

        device.async_start(10);

        isotp.write(device.sender(), true, vec![0x10, 0x01]).await?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // println!("Press Ctrl+C to stop...");
        // if let Err(e) = tokio::signal::ctrl_c().await {
        //     eprintln!("Failed to listen for Ctrl+C: {}", e);
        //     return Ok(());
        // }

        device.close().await;

        Ok(())
    }
}
