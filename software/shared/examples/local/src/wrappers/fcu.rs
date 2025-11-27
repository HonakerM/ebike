use base64::Engine;
use eframe::egui::{self, Color32};
use egui_async::Bind;
use embedded_can::StandardId;
use std::{
    sync::OnceLock,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::{io, sync::MutexGuard, task};

use futures::{FutureExt, StreamExt};
use shared::{
    config::config::Config, controllers::{fcu::{FcuController, FcuState}, mcu::McuController}, messages::messages::Message, operations::config_updater::ConfigUpdateState, utils::{percentage::Percentage, time::Timestamp}
};
use tokio_util::codec::{FramedRead, LinesCodec}; // For the .next() method on FramedRead

use tokio::sync::Mutex;

use crate::wrappers::core::{CurrentCarState, broadcast_message, get_car_state, get_next_message, get_req_throttle, get_timestamp, local_sleep, update_req_throttle};


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
                let tc_val = get_req_throttle().await;
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


pub struct FcuUiComponent {
    throttle_req: f32,
    update_state: Bind<(), ()>,

}

impl Default for FcuUiComponent {
    fn default() -> Self {
        FcuUiComponent { throttle_req: Percentage::zero().to_fractional(), update_state: Bind::new(false) }
    }
}
impl FcuUiComponent {
    pub fn ui(&mut self, ui: &mut egui::Ui, car_state: &CurrentCarState) {
            ui.add(egui::Slider::new(&mut self.throttle_req, 0.0..=100.0).text("Throttle Req"));
            ui.add(egui::ProgressBar::new(car_state.throttle.to_ui()).show_percentage().fill(Color32::from_rgb(0, 255, 0)));

        let local_perct = Percentage::from_ui(self.throttle_req);
        self.update_state.request_every( move || async move {
            update_req_throttle(local_perct).await;
            Ok(())
        }, Duration::from_millis(50));
    }
}