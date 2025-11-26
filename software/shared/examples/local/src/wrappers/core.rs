use base64::Engine;
use embedded_can::StandardId;
use std::{
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{io, sync::{MutexGuard, broadcast}, task};

use futures::{FutureExt, StreamExt};
use shared::{
    config::config::Config,
    controllers::mcu::McuController,
    messages::messages::{Message, ecu::EcuMessage},
    utils::{percentage::Percentage, time::Timestamp},
};
use tokio_util::codec::{FramedRead, LinesCodec}; // For the .next() method on FramedRead

use tokio::sync::Mutex;

pub fn get_timestamp() -> Timestamp {
    let now = SystemTime::now();
    Timestamp::from_micros(now.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64)
}

pub async fn local_sleep(dur: shared::utils::time::Duration) {
    tokio::time::sleep(Into::<Duration>::into(dur)).await;
}

static LOCAL_CAN_SEND: OnceLock<tokio::sync::broadcast::Sender<Message>> = OnceLock::new();


pub async fn get_next_message() -> Message {
    let mut subsriber = LOCAL_CAN_SEND.get().unwrap().subscribe();
    let msg = subsriber.recv().await.unwrap();
    return msg;
}

pub async fn broadcast_message(msg: Message) {
    if let Some(sender) = LOCAL_CAN_SEND.get() {
        match sender.send(msg) {
            Err(err) => {eprintln!("Failed to send message: {:?} most likely due to no recievers", err)}
            _ => {}
        }
        return;
    }
    panic!("No message sender for CAN")
}



#[derive(Debug, Clone, Copy)]
pub struct CurrentOutsideState {
    pub throttle: Percentage
}

impl Default for CurrentOutsideState {
    fn default() -> Self {
        CurrentOutsideState { throttle: Percentage::zero() }
    }
}



static CURRENT_OUTSIDE_STATE: OnceLock<Mutex<CurrentOutsideState>> = OnceLock::new();

pub async fn update_req_throttle(val: Percentage) {
    let mut local_mut = CURRENT_OUTSIDE_STATE.get().unwrap().lock().await;
    local_mut.throttle = val;
}

pub async fn get_req_throttle()->Percentage {
    let mut local_mut = CURRENT_OUTSIDE_STATE.get().unwrap().lock().await;
    local_mut.throttle
}

pub async fn get_outside_state()->CurrentOutsideState {
    let mut local_mut = CURRENT_OUTSIDE_STATE.get().unwrap().lock().await;
    local_mut.clone()
}




#[derive(Debug, Clone, Copy)]
pub struct CurrentCarState {
    pub throttle: Percentage,
    pub brake: Percentage,
}

impl Default for CurrentCarState {
    fn default() -> Self {
        CurrentCarState { throttle: Percentage::zero(), brake: Percentage::zero(), }
    }
}

static CURRENT_CAR_STATE: OnceLock<Mutex<CurrentCarState>> = OnceLock::new();



pub async fn state_updater() {
    loop {
        let msg = get_next_message().await;
        match msg {
            Message::EcuMessage(msg) => {
                {
                    let mut local_mut = CURRENT_CAR_STATE.get().unwrap().lock().await;
                    local_mut.throttle = msg.throttle;
                }
            },
            _ => {}
        }
    }
}
pub async fn get_ecu_throttle()->Percentage {
    let mut local_mut = CURRENT_CAR_STATE.get().unwrap().lock().await;
    local_mut.throttle
}

pub async fn get_car_state()->CurrentCarState {
    let mut local_mut = CURRENT_CAR_STATE.get().unwrap().lock().await;
    local_mut.clone()
}


pub fn setup() -> (
    Config,
    tokio::sync::broadcast::Sender<Message>,
) {
    let (can_send, can_recv) = broadcast::channel(16);
    LOCAL_CAN_SEND.set(can_send.clone()).unwrap();

    CURRENT_OUTSIDE_STATE.set(Mutex::new(CurrentOutsideState::default())).unwrap();
    CURRENT_CAR_STATE.set(Mutex::new(CurrentCarState::default())).unwrap();

    (Config::default(), can_send)
}
