use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn};
use std::time::Duration;
use can_type_rs::device::{CanListener, SyncCanDevice};
use zlgcan_common::can::CanMessage;
use zlgcan_common::device::Handler;
use crate::driver::{ZCanDriver, ZDevice};
use crate::extends::{listener_names, receive_callback, register_listener, transmit_callback, unregister_all, unregister_listener};

#[derive(Clone)]
pub struct ZCanSync {
    device: ZCanDriver,
    sender: Sender<CanMessage>,
    receiver: Arc<Mutex<Receiver<CanMessage>>>,
    listeners: Arc<Mutex<HashMap<String, Box<dyn CanListener<Frame = CanMessage, Channel = u8>>>>>,
    stop_tx: Sender<()>,
    stop_rx: Arc<Mutex<Receiver<()>>>,
    send_task: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    receive_task: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
}

impl From<ZCanDriver> for ZCanSync {
    fn from(value: ZCanDriver) -> Self {
        Self::new(value)
    }
}

impl SyncCanDevice for ZCanSync {
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

    fn sync_transmit(device: MutexGuard<Self>, interval_ms: u64, stopper: Arc<Mutex<Receiver<()>>>) {
        sync_util(device, interval_ms, stopper, |_, device| {
            transmit_callback(&device.receiver, &device.device, &device.listeners)
        });
    }

    fn sync_receive(device: MutexGuard<Self>, interval_ms: u64, stopper: Arc<Mutex<Receiver<()>>>) {
        sync_util(device, interval_ms, stopper, |handler, device| {
            receive_callback(&device.device, handler, &device.listeners)
        });
    }

    fn sync_start(&mut self, interval_ms: u64) {
        let self_arc = Arc::new(Mutex::new(self.clone()));
        let stop_rx = Arc::clone(&self.stop_rx);
        let tx_task = spawn(move || {
            if let Ok(self_clone) = self_arc.lock() {
                Self::sync_transmit(self_clone, interval_ms, Arc::clone(&stop_rx));
            }
        });

        let self_arc = Arc::new(Mutex::new(self.clone()));
        let stop_rx = Arc::clone(&self.stop_rx);
        let rx_task = spawn(move || {
            if let Ok(self_clone) = self_arc.lock() {
                Self::sync_receive(self_clone, interval_ms, Arc::clone(&stop_rx));
            }
        });

        if let Ok(mut task) = self.send_task.lock() {
            task.replace(tx_task);
        }
        if let Ok(mut task) = self.receive_task.lock() {
            task.replace(rx_task);
        }
    }

    fn close(&mut self) {
        log::info!("ZLGCAN - closing(sync)");

        if let Err(e) = self.stop_tx.send(()) {
            log::warn!("ZLGCAN - error: {} when sending stop signal", e);
        }

        if let Ok(mut task) = self.send_task.lock() {
            if let Some(task) = task.take() {
                if !task.is_finished() {

                }
            }
        }

        if let Ok(mut task) = self.receive_task.lock() {
            if let Some(task) = task.take() {
                if !task.is_finished() {

                }
            }
        }

        self.device.close();
    }
}

#[inline]
fn sync_util(device: MutexGuard<ZCanSync>, interval: u64, stopper: Arc<Mutex<Receiver<()>>>, callback: fn(Handler, &MutexGuard<ZCanSync>)) {
    loop {
        if let Some(handler) = device.device.handler.clone() {
            callback(handler, &device);
        }
        else {
            log::info!("ZLGCAN - exit sync receive.");
            break;
        }

        if let Ok(stopper) = stopper.lock() {
            if let Ok(()) = stopper.recv() {
                break
            }
        }

        sleep(Duration::from_millis(interval));
    }
}
