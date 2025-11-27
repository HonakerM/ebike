use base64::Engine;
use embedded_can::StandardId;
use std::{
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{io, sync::MutexGuard, task};

use futures::{FutureExt, StreamExt};
use shared::{
    config::config::Config, controllers::mcu::McuController, messages::messages::Message,
    utils::time::Timestamp,
};
use tokio_util::codec::{FramedRead, LinesCodec}; // For the .next() method on FramedRead

use tokio::sync::Mutex;

use crate::wrappers::core::{broadcast_message, get_next_message, get_timestamp, local_sleep};

pub struct LocalMcuRunner {
    controller: Mutex<McuController>,
}

impl LocalMcuRunner {
    fn new(config: Config) -> Self {
        let controler = McuController::new(config);
        let controller = Mutex::from(controler);
        LocalMcuRunner { controller }
    }

    pub async fn broadcast_ecu(&self) {
        loop {
            let (sleep_time, msg) = {
                let controller = self.controller.lock().await;
                let msg = controller.broadcast_ecu();
                (controller.config.mcu.ecu_poll, msg)
            };
            (broadcast_message(msg)).await;
            local_sleep(sleep_time).await
        }
    }
    pub async fn broadcast_config(&self) {
        loop {
            let (sleep_time, msg) = {
                let controller = self.controller.lock().await;
                let msg = controller.broadcast_config();
                (controller.config.mcu.config_poll, msg)
            };
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

    pub async fn run(config: Config) {
        let runner = LocalMcuRunner::new(config);

        // Spawn a new task
        let (_, _, _, _) = tokio::join!(
            runner.run_engine_subsystem(),
            runner.broadcast_ecu(),
            runner.process_messages(),
            runner.broadcast_config(),
        );
    }
}
