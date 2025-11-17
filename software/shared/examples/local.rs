use base64::Engine;
use embedded_can::StandardId;
use std::{sync::OnceLock, time::{ Duration, SystemTime, UNIX_EPOCH}};
use tokio::{io, task};

use futures::StreamExt;
use shared::{
    config::config::Config,
    controllers::{
        mcu::McuController,
        shared::{Controller, HalInterface},
    },
    messages::messages::Message,
    utils::time::Timestamp,
};
use tokio_util::codec::{FramedRead, LinesCodec}; // For the .next() method on FramedRead

use tokio::sync::Mutex;


fn get_timestamp()->Timestamp {
    let now = SystemTime::now();
    Timestamp::from_micros(now.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64)
}

async fn local_sleep(dur: shared::utils::time::Duration) {
    tokio::time::sleep(dur.into()).await;
}


static LOCAL_CAN_SEND: OnceLock<Mutex<std::sync::mpsc::Sender<Message>>> = OnceLock::new();
static LOCAL_CAN_RCV: OnceLock<Mutex<std::sync::mpsc::Receiver<Message>>> = OnceLock::new();

async fn get_next_message()->Message {
    if let Some(recv) = LOCAL_CAN_RCV.get() {
        {
            let lock = recv.lock().await;
            return lock.recv().unwrap()
        }
    }
    panic!("No message receiver for CAN")
}


async fn broadcast_message(msg: Message) {
    if let Some(recv) = LOCAL_CAN_SEND.get() {
        let local_can = {
            let lock = recv.lock().await;
            lock.clone()
        };
        local_can.send(msg).unwrap()
    }
    panic!("No message sender for CAN")
}


#[tokio::main]
async fn main() {

    let (can_send, can_recv) = std::sync::mpsc::channel();
    LOCAL_CAN_SEND.set(Mutex::new(can_send));
    LOCAL_CAN_RCV.set(Mutex::new(can_recv));

    let interface= HalInterface {
        get_timestamp: get_timestamp,
        get_can_message: get_next_message,
        broadcast_can_message: broadcast_message,
        sleep: local_sleep,

    };
    let config = Config::default();

    let controller = McuController::new(config, interface);

    // Spawn a new task
    // let (_, _, _) = tokio::join!(
    //     local_controller.run_engine_subsystem(),
    //     local_controller.broadcast_ecu(),
    //     local_controller.process_messages(),
    // );
}
