use crate::perphierals::{CAN_RX, CAN_TX};
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{Context, Poll};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    Can, Fifo, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId,
    TxInterruptHandler,
};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN1;
use embassy_sync::blocking_mutex::NoopMutex;
use embassy_sync::blocking_mutex::raw::{NoopRawMutex, ThreadModeRawMutex};
use embassy_sync::lazy_lock::LazyLock;
use embassy_sync::mutex::{Mutex, MutexGuard};
use embassy_sync::once_lock::OnceLock;
use embassy_time::Instant;
use futures::task::UnsafeFutureObj;
use shared::config::config::Config;
use shared::controllers::mcu::McuController;
use shared::controllers::shared::Lockable;
use shared::messages::messages::Message;
use shared::utils::time::Timestamp;

use embassy_time::{Delay, Duration, Timer};
use embedded_can::Id;
use {defmt_rtt as _, panic_probe as _};

fn get_timestamp() -> Timestamp {
    let now = Instant::now();
    Timestamp::from_micros(now.as_micros())
}

async fn local_sleep(dur: shared::utils::time::Duration) {
    trace!("Trying to sleep: {}", dur.as_millis());
    Timer::after_millis(dur.as_millis()).await;
    trace!("Done sleeping");
}

async fn get_can_message() -> Message {
    loop {
        // Wait for a new one
        let envelope = {
            let mut can_lock = CAN_RX.lock().await;
            trace!("listening for requests");
            can_lock.as_mut().unwrap().read().await.unwrap()
        };
        trace!("got for request");

        if let Id::Standard(id) = envelope.frame.id() {
            if let Some(msg) = Message::from_bytes(*id, envelope.frame.data()) {
                debug!("{}", msg);
                return msg;
            }
        }
    }
}
async fn broadcast_can_message(msg: Message) {
    debug!("{}", msg);
    let frame = Frame::new_data(msg.to_id(), &msg.to_bytes()).unwrap();
    let mut can_lock = CAN_TX.lock().await;
    can_lock.as_mut().unwrap().write(&frame).await;
    trace!("Wrote msg: {}", msg);
}

pub struct DeviceMcuRunner {
    controller: Mutex<ThreadModeRawMutex, McuController>,
}

impl DeviceMcuRunner {
    fn new(config: Config) -> Self {
        let controler = McuController::new(config);
        let controller = Mutex::new(controler);
        DeviceMcuRunner { controller }
    }

    pub async fn broadcast_ecu(&self) {
        loop {
            let (sleep_time, msg) = {
                let controller = self.controller.lock().await;
                let msg = controller.broadcast_ecu();
                (controller.config.mcu.ecu_poll, msg)
            };
            //eprintln!("{}", Into::<String>::into(msg));
            (broadcast_can_message(msg)).await;
            local_sleep(sleep_time).await
        }
    }
    pub async fn run_engine_subsystem(&self) {
        loop {
            let sleep_time = {
                let mut controller = self.controller.lock().await;
                controller.run_engine_subsystem((get_timestamp()));
                controller.config.mcu.engine_poll
            };
            local_sleep(sleep_time).await
        }
    }

    pub async fn process_messages(&self) {
        loop {
            let msg = get_can_message().await;
            {
                let mut controller = self.controller.lock().await;
                controller.process_message(msg);
            }
        }
    }
}

#[embassy_executor::task]
pub async fn broadcast_ecu(runner: &'static DeviceMcuRunner) {
    runner.broadcast_ecu().await;
}

#[embassy_executor::task]
pub async fn run_engine_subsystem(runner: &'static DeviceMcuRunner) {
    runner.run_engine_subsystem().await;
}

#[embassy_executor::task]
pub async fn process_task(runner: &'static DeviceMcuRunner) {
    runner.process_messages().await;
}

pub static DEVICE_MCU_RUNNER: OnceLock<DeviceMcuRunner> = OnceLock::new();

pub async fn spawn(spawner: embassy_executor::Spawner, config: Config) {
    let runner: &'static DeviceMcuRunner =
        &DEVICE_MCU_RUNNER.get_or_init(move || DeviceMcuRunner::new(config));

    spawner.spawn(broadcast_ecu(runner)).unwrap();
    spawner.spawn(run_engine_subsystem(runner)).unwrap();
    spawner.spawn(process_task(runner)).unwrap();
}
