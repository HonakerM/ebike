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
    controllers::{
        mcu::{McuController},
    },
    messages::messages::Message,
    utils::time::Timestamp,
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

#[derive(Debug)]
pub struct Messagestats {
    pub msg: Message,
    pub time: Timestamp,
}
impl Messagestats {
    pub fn new(msg: Message) -> Self {
        Messagestats {
            msg,
            time: get_timestamp(),
        }
    }
}

static DEBUG_CAN_SEND: OnceLock<Mutex<std::sync::mpsc::Sender<Messagestats>>> = OnceLock::new();

pub async fn get_next_message() -> Message {
    let mut subsriber = LOCAL_CAN_SEND.get().unwrap().subscribe();
    let msg = subsriber.recv().await.unwrap();
    {
        let mut lock = DEBUG_CAN_SEND.get().unwrap().lock().now_or_never().unwrap();
        lock.send(Messagestats::new(msg));
    }
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


pub fn setup() -> (
    Config,
    tokio::sync::broadcast::Sender<Message>,
    std::sync::mpsc::Receiver<Messagestats>,
) {
    let (can_send, can_recv) = broadcast::channel(16);
    LOCAL_CAN_SEND.set(can_send.clone());

    let (debug_send, debug_recv) = std::sync::mpsc::channel();
    DEBUG_CAN_SEND.set(Mutex::new(debug_send));

    (Config::default(), can_send, debug_recv)
}


