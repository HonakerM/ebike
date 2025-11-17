use base64::Engine;
use embedded_can::StandardId;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

pub struct LocalMcuController {
    pub controller: Mutex<McuController>,
}

impl LocalMcuController {
    async fn broadcast_ecu(&self) {
        loop {
            let (sleep_time, msg) = {
                let mut lock = self.controller.lock().await;
                let msg = lock.broadcast_ecu();
                (lock.config.mcu.ecu_poll, msg)
            };
            eprintln!("{:?}", msg);
            tokio::time::sleep(Duration::from_millis(sleep_time.as_millis())).await;
        }
    }

    async fn run_engine_subsystem(&self) {
        loop {
            let sleep_time = {
                let mut lock = self.controller.lock().await;
                lock.run_engine_subsystem();
                lock.config.mcu.engine_poll
            };
            tokio::time::sleep(Duration::from_millis(sleep_time.as_millis())).await;
        }
    }

    async fn process_messages(&self) {
        // Get the Tokio handle to stdin
        let stdin = io::stdin();
        let mut reader = FramedRead::new(stdin, LinesCodec::new());
        loop {
            // Read a single line
            if let Some(line_result) = reader.next().await {
                let line = line_result.unwrap(); // Propagate any errors
                let line = line.replace("\n", "");

                let msg = {
                    let mut res = line.split(":");
                    let id_str = res.next().unwrap();
                    let data_str = res.next().unwrap();
                    let id: u16 = id_str.parse().unwrap();
                    let id = StandardId::new(id).unwrap();

                    let raw_data = base64::engine::general_purpose::STANDARD
                        .decode(data_str)
                        .unwrap();
                    Message::from_bytes(id, raw_data.as_ref()).unwrap()
                };
                {
                    let mut lock = self.controller.lock().await;
                    lock.process_message(msg);
                }
                eprintln!("{:?}", msg);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    let interface = HalInterface {
        get_timestamp: || {
            let now = SystemTime::now();
            Timestamp::from_micros(now.duration_since(UNIX_EPOCH).unwrap().as_micros() as u64)
        },
    };
    let config = Config::default();

    let controller = McuController::new(config, interface);
    let local_controller = LocalMcuController {
        controller: Mutex::new(controller),
    };

    // Spawn a new task
    let (_, _, _) = tokio::join!(
        local_controller.run_engine_subsystem(),
        local_controller.broadcast_ecu(),
        local_controller.process_messages(),
    );
}
