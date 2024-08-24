use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, mpsc::{channel, Receiver, Sender}, Mutex, MutexGuard, Weak};
use std::time::Duration;
use isotp_rs::device::{AsyncDevice, Listener};
use zlgcan_common::can::CanMessage;
use zlgcan_common::device::Handler;
use tokio::{spawn, time::sleep, task::JoinHandle};

use crate::driver::{ZCanDriver, ZDevice};
use crate::extends::{listener_names, ListenerType, receive_callback, register_listener, transmit_callback, unregister_all, unregister_listener};

#[derive(Clone)]
pub struct ZCanAsync {
    device: ZCanDriver,
    sender: Sender<CanMessage>,
    receiver: Arc<Mutex<Receiver<CanMessage>>>,
    listeners: Arc<Mutex<HashMap<String, ListenerType>>>,
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
    type Device = ZCanDriver;
    type Channel = u8;
    type Id = u32;
    type Frame = CanMessage;

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
    fn sender(&self) -> Sender<Self::Frame> {
        self.sender.clone()
    }

    #[inline]
    fn register_listener(
        &mut self,
        name: String,
        listener: Box<dyn Listener<Self::Channel, Self::Id, Self::Frame>>,
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

    fn async_transmit(device: Arc<Mutex<Self>>, interval_us: u64, stopper: Arc<Mutex<Receiver<()>>>) -> impl Future<Output=()> + Send {
        async move {
            async_util(device, interval_us, stopper, |_, device| {
                transmit_callback(&device.receiver, &device.device, &device.listeners)
            }).await;
        }
    }

    fn async_receive(device: Arc<Mutex<Self>>, interval_us: u64, stopper: Arc<Mutex<Receiver<()>>>) -> impl Future<Output=()> + Send {
        async move {
            async_util(device, interval_us, stopper, |handler, device| {
                receive_callback(&device.device, handler, &device.listeners)
            }).await;
        }
    }

    #[inline]
    fn async_start(&mut self, interval_us: u64) {
        let tx_task = spawn(Self::async_transmit(Arc::new(Mutex::new(self.clone())), interval_us, Arc::clone(&self.stop_rx)));
        let rx_task = spawn(Self::async_receive(Arc::new(Mutex::new(self.clone())), interval_us, Arc::clone(&self.stop_rx)));

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
        sleep(Duration::from_micros(interval)).await;
    }
}
