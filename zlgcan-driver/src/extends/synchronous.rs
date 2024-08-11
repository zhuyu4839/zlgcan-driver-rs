use std::collections::HashMap;
use std::sync::{Arc, mpsc::{channel, Receiver, Sender}, Mutex, MutexGuard, Weak};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;
use can_type_rs::identifier::Id;
use isotp_rs::device::{Listener, SyncDevice};
use zlgcan_common::can::CanMessage;
use zlgcan_common::device::Handler;
use crate::driver::{ZCanDriver, ZDevice};
use crate::extends::{listener_names, receive_callback, register_listener, transmit_callback, unregister_all, unregister_listener};

#[derive(Clone)]
pub struct ZCanSync {
    device: ZCanDriver,
    sender: Sender<CanMessage>,
    receiver: Arc<Mutex<Receiver<CanMessage>>>,
    listeners: Arc<Mutex<HashMap<String, Box<dyn Listener<u8, Id, CanMessage>>>>>,
    stop_tx: Sender<()>,
    stop_rx: Arc<Mutex<Receiver<()>>>,
    send_task: Weak<JoinHandle<()>>,
    receive_task: Weak<JoinHandle<()>>,
    interval: Option<u64>,
}

impl From<ZCanDriver> for ZCanSync {
    fn from(value: ZCanDriver) -> Self {
        Self::new(value)
    }
}

impl SyncDevice for ZCanSync {
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
            interval: Default::default(),
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
        self.interval = Some(interval_ms);

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

        self.send_task = Arc::downgrade(&Arc::new(tx_task));
        self.receive_task = Arc::downgrade(&Arc::new(rx_task));
    }

    fn close(&mut self) {
        log::info!("ZLGCAN - closing(sync)");

        if let Err(e) = self.stop_tx.send(()) {
            log::warn!("ZLGCAN - error: {} when sending stop signal", e);
        }

        sleep(Duration::from_millis(2 * self.interval.unwrap_or(50)));

        if let Some(task) = self.send_task.upgrade() {
            if !task.is_finished() {
                log::warn!("ZLGCAN - send task is running after stop signal");
            }
        }

        if let Some(task) = self.receive_task.upgrade() {
            if !task.is_finished() {
                log::warn!("ZLGCAN - receive task is running after stop signal");
            }
        }

        self.device.close();
    }
}

#[inline]
fn sync_util(device: MutexGuard<ZCanSync>,
             interval: u64,
             stopper: Arc<Mutex<Receiver<()>>>,
             callback: fn(Handler, &MutexGuard<ZCanSync>)
) {
    loop {
        if let Some(handler) = device.device.handler.clone() {
            callback(handler, &device);
        }
        else {
            log::info!("ZLGCAN - exit sync receive.");
            break;
        }

        if let Ok(stopper) = stopper.lock() {
            if let Ok(()) = stopper.try_recv() {
                break
            }
        }

        sleep(Duration::from_millis(interval));
    }
}

#[cfg(test)]
mod tests {
    use isotp_rs::device::SyncDevice;
    use can_type_rs::isotp::Address;
    use can_type_rs::isotp::{CanIsoTp, SyncCanIsoTp};
    use zlgcan_common::can::{CanChlCfgExt, CanChlCfgFactory, ZCanChlMode, ZCanChlType};
    use zlgcan_common::device::ZCanDeviceType;
    use crate::driver::{ZCanDriver, ZDevice};
    use crate::extends::ZCanSync;

    #[test]
    fn test_device() -> anyhow::Result<()> {
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

        std::thread::sleep(std::time::Duration::from_millis(100));

        let mut device = ZCanSync::from(device);

        let mut isotp = SyncCanIsoTp::new(0, Address {
            tx_id: 0x7E4,
            rx_id: 0x7EC,
            fid: 0x7DF,
        });
        device.register_listener("UdsClient".to_string(), isotp.get_frame_listener());

        device.sync_start(10);

        isotp.write(device.sender(), true, vec![0x10, 0x01])?;

        std::thread::sleep(std::time::Duration::from_millis(500));

        // println!("Press Ctrl+C to stop...");
        // if let Err(e) = tokio::signal::ctrl_c().await {
        //     eprintln!("Failed to listen for Ctrl+C: {}", e);
        //     return Ok(());
        // }

        device.close();

        Ok(())
    }
}
