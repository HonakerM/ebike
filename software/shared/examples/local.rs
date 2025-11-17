use base64::Engine;
use embedded_can::StandardId;
use std::{
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{io, sync::MutexGuard, task};

use futures::StreamExt;
use shared::{
    config::config::Config,
    controllers::{
        mcu::{McuController, McuRunner},
        shared::{ControllerRunner, HalInterface, Lockable},
    },
    messages::messages::Message,
    utils::time::Timestamp,
};
use tokio_util::codec::{FramedRead, LinesCodec}; // For the .next() method on FramedRead

use tokio::sync::Mutex;

fn get_timestamp() -> Timestamp {
    let now = SystemTime::now();
    Timestamp::from_micros(now.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64)
}

async fn local_sleep(dur: shared::utils::time::Duration) {
    println!(
        "Asking to sleep: {:?} sleeping {:?}",
        dur,
        Into::<Duration>::into(dur)
    );
    tokio::time::sleep(Into::<Duration>::into(dur)).await;
    println!("Done sleeping");
}

static LOCAL_CAN_SEND: OnceLock<Mutex<std::sync::mpsc::Sender<Message>>> = OnceLock::new();
static LOCAL_CAN_RCV: OnceLock<Mutex<std::sync::mpsc::Receiver<Message>>> = OnceLock::new();

async fn get_next_message() -> Message {
    loop {
        if let Some(recv) = LOCAL_CAN_RCV.get() {
            {
                let lock = recv.lock().await;
                let msg = lock.recv_timeout(Duration::from_millis(10));
                if let Ok(msg) = msg {
                    return msg;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        } else {
            panic!("No message receiver for CAN")
        }
    }
}

async fn broadcast_message(msg: Message) {
    if let Some(recv) = LOCAL_CAN_SEND.get() {
        let local_can = {
            let lock = recv.lock().await;
            lock.clone()
        };
        local_can.send(msg).unwrap();
        return;
    }
    panic!("No message sender for CAN")
}

pub struct LocalMutex(Mutex<McuController>);

impl Lockable for LocalMutex {
    type Target = McuController;

    type Guard<'a> = MutexGuard<'a, McuController>;

    async fn lock<'a>(&'a self) -> Self::Guard<'a> {
        self.0.lock().await
    }
}

impl From<McuController> for LocalMutex {
    fn from(value: McuController) -> Self {
        LocalMutex(Mutex::new(value))
    }
}

#[tokio::main]
async fn main() {
    let (can_send, can_recv) = std::sync::mpsc::channel();
    LOCAL_CAN_SEND.set(Mutex::new(can_send));
    LOCAL_CAN_RCV.set(Mutex::new(can_recv));

    let interface = HalInterface {
        get_timestamp: get_timestamp,
        get_can_message: get_next_message,
        broadcast_can_message: broadcast_message,
        sleep: local_sleep,
    };
    let config = Config::default();

    let runner: McuRunner<LocalMutex, _, _, _> = McuRunner::new(config, interface);

    // Spawn a new task
    let (_, _, _) = tokio::join!(
        runner.run_engine_subsystem(),
        runner.broadcast_ecu(),
        runner.process_messages(),
    );
}
