use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_sync::lazy_lock::LazyLock;
use futures::task::UnsafeFutureObj;
use core::mem::MaybeUninit;
use core::pin::Pin;
use core::task::{Context, Poll};
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    
    Can, Fifo, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId,
    TxInterruptHandler,
};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN1;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
use embassy_time::Instant;
use shared::config::config::Config;
use shared::controllers::mcu::{McuController, McuRunner};
use shared::controllers::shared::{ControllerRunner, HalInterface, Lockable};
use shared::messages::messages::Message;
use shared::utils::time::Timestamp;
use crate::perphierals::CAN;

use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Delay, Duration, Timer};
use embedded_can::Id;


fn get_timestamp() -> Timestamp {
    let now = Instant::now();
    Timestamp::from_micros(now.as_micros())
}


async fn local_sleep(dur: shared::utils::time::Duration) {
    Timer::after_millis(dur.as_millis()).await
}

async fn get_can_message()-> Message {
    loop {
        // Wait for a new one
        let envelope = {
            loop {
                let cur_status = {
                    let mut can_lock = CAN.lock().await;
                    can_lock.as_mut().unwrap().try_read()
                };
                if let Ok(cur_frame) = cur_status {
                    break cur_frame
                } else {
                    local_sleep(shared::utils::time::Duration::from_millis(1)).await
                }
            }
        };

        if let Id::Standard(id) = envelope.frame.id() {
            if let Some(msg) = Message::from_bytes(*id, envelope.frame.data()) {
                return msg;
            }
        }
    }
}
async fn broadcast_can_message(msg: Message) {
    let frame = Frame::new_data(msg.to_id(), &msg.to_bytes()).unwrap();
    let mut can_lock = CAN.lock().await;
    can_lock.as_mut().unwrap().write(&frame).await;
}


pub struct McuMutex(Mutex<ThreadModeRawMutex, McuController>);

impl Lockable for McuMutex {
    type Target = McuController;

    type Guard<'a> = MutexGuard<'a, ThreadModeRawMutex, McuController>;

    async fn lock<'a>(&'a self) -> Self::Guard<'a> {
        self.0.lock().await
    }
}

impl From<McuController> for McuMutex {
    fn from(value: McuController) -> Self {
        McuMutex(Mutex::new(value))
    }
}


pub struct SomeType<G>  where G: Copy{
    output: Option<G>
}

impl<G> Future for SomeType<G> where G: Copy {
    type Output = G;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(val) = &self.output {
            Poll::Ready(val.clone())
        } else {
            Poll::Pending
        }
    }
}

pub struct DeviceRunner {

}
impl DeviceRunner {

}


#[embassy_executor::task]
pub async fn broadcast_ecu() {
    runner.broadcast_ecu().await;
}

#[embassy_executor::task]
pub async fn run_engine_subsystem(runner: McuRunner<McuMutex, SomeType<Message>, SomeType<()>, SomeType<()>>) {
    runner.run_engine_subsystem().await;
}

#[embassy_executor::task]
pub async fn process_task(runner: DeviceRunner) {
    runner.process_messages().await;
}



pub async fn spawn(spawner: embassy_executor::Spawner, config: Config) {
    let config = Config::default();
    let interface = HalInterface {
        get_timestamp: get_timestamp,
        get_can_message: get_can_message,
        broadcast_can_message: broadcast_can_message,
        sleep: local_sleep,
    };

    let runner: McuRunner<McuMutex,_,_,_>  = McuRunner::new(config, interface);

    spawner.spawn(broadcast_ecu(runner)).unwrap();
}