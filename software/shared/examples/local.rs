use base64::Engine;
use embedded_can::StandardId;
use std::{
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{io, sync::MutexGuard, task};

use futures::{FutureExt, StreamExt};
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
    tokio::time::sleep(Into::<Duration>::into(dur)).await;
}

static LOCAL_CAN_SEND: OnceLock<Mutex<std::sync::mpsc::Sender<Message>>> = OnceLock::new();
static LOCAL_CAN_RCV: OnceLock<Mutex<std::sync::mpsc::Receiver<Message>>> = OnceLock::new();

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
    loop {
        if let Some(recv) = LOCAL_CAN_RCV.get() {
            {
                let lock = recv.lock().await;
                let msg = lock.recv_timeout(Duration::from_millis(10));
                if let Ok(msg) = msg {
                    {
                        let mut lock = DEBUG_CAN_SEND.get().unwrap().lock().now_or_never().unwrap();
                        lock.send(Messagestats::new(msg));
                    }
                    return msg;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        } else {
            panic!("No message receiver for CAN")
        }
    }
}

pub async fn broadcast_message(msg: Message) {
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

pub struct LocalMcuRunner {
    controller: LocalMutex,
}

impl LocalMcuRunner {
    fn new(config: Config) -> Self {
        let controler = McuController::new(config);
        let controller = LocalMutex::from(controler);
        LocalMcuRunner { controller }
    }

    pub async fn broadcast_ecu(&self) {
        loop {
            let (sleep_time, msg) = {
                let controller = self.controller.lock().await;
                let msg = controller.broadcast_ecu();
                (controller.config.mcu.ecu_poll, msg)
            };
            //eprintln!("{}", Into::<String>::into(msg));
            (broadcast_message(msg)).await;
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
            let msg = get_next_message().await;
            {
                let mut controller = self.controller.lock().await;
                controller.process_message(msg);
            }
        }
    }
}

pub fn setup() -> (
    Config,
    std::sync::mpsc::Sender<Message>,
    std::sync::mpsc::Receiver<Messagestats>,
) {
    let (can_send, can_recv) = std::sync::mpsc::channel();
    LOCAL_CAN_SEND.set(Mutex::new(can_send.clone()));
    LOCAL_CAN_RCV.set(Mutex::new(can_recv));

    let (debug_send, debug_recv) = std::sync::mpsc::channel();
    DEBUG_CAN_SEND.set(Mutex::new(debug_send));

    (Config::default(), can_send, debug_recv)
}

pub async fn run(config: Config) {
    let runner = LocalMcuRunner::new(config);

    // Spawn a new task
    let (_, _, _) = tokio::join!(
        runner.run_engine_subsystem(),
        runner.broadcast_ecu(),
        runner.process_messages(),
    );
}

#[tokio::main]
async fn main() {
    let (config, _, _) = setup();
    run(config).await;
}
