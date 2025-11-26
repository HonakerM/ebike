use base64::Engine;
use embedded_can::StandardId;
use std::{
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{io, sync::MutexGuard, task};

use futures::{FutureExt, StreamExt};
use shared::{
    config::config::Config, controllers::{fcu::FcuController, mcu::McuController}, messages::messages::Message, operations::config_updater::ConfigUpdateState, utils::{percentage::Percentage, time::Timestamp}
};
use tokio_util::codec::{FramedRead, LinesCodec}; // For the .next() method on FramedRead

use tokio::sync::Mutex;

use crate::wrappers::core::{broadcast_message, get_next_message, get_timestamp, local_sleep};


pub struct LocalFcuRunner {
    controller: Mutex<FcuController>,
}

impl LocalFcuRunner {
    fn new(config: Config) -> Self {
        let controler = FcuController::new(config);
        let controller = Mutex::from(controler);
        LocalFcuRunner { controller }
    }

    pub async fn broadcast_ctl(&self) {
        loop {
            let (sleep_time, msg) = {
                let mut controller = self.controller.lock().await;

                // get tc/break value:
                let tc_val = Percentage::zero();
                let break_val = Percentage::zero();


                let msg = controller.broadcast_ctl(tc_val, break_val);
                (controller.config.fcu.ctl_poll, msg)
            };
            //eprintln!("{}", Into::<String>::into(msg));
            (broadcast_message(msg)).await;
            local_sleep(sleep_time).await
        }
    }
    pub async fn broadcast_upload(&self) {
        loop {
            let (sleep_time, msg) = {
                let mut controller = self.controller.lock().await;
                // get tc/break value:
                let field_per = Percentage::zero();
                let val_per = Percentage::zero();
                let config_state = ConfigUpdateState::new(field_per, val_per);
                let msg = controller.run_config_update(config_state);
                (controller.config.fcu.update_poll, msg)
            };
            if let Some(msg) = msg {
                (broadcast_message(msg)).await;
            }
            local_sleep(sleep_time).await
        }
    }

    pub async fn update_user_display(&self) {
        {
            let mut controller = self.controller.lock().await;

            let state = controller.update_user_display();
        };
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
        let runner = LocalFcuRunner::new(config);
    
        // Spawn a new task
        let (_, _, _,_) = tokio::join!(
            runner.broadcast_ctl(),
            runner.broadcast_upload(),
            runner.update_user_display(),
            runner.process_messages(),
        );
    }

}

