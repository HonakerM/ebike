// filepath: /esp32-led-control/esp32-led-control/src/main.rs
use crate::peripherals::broadcast_message;
use crate::peripherals::get_message;
use crate::peripherals::get_ti_value;
use crate::peripherals::update_display;
use embedded_can::nb::Can;
use embedded_can::Frame;
use embedded_can::StandardId;
use esp_idf_hal::can;
use esp_idf_hal::gpio::{Gpio32, Gpio33, Input, Output, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::timer::Timer;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;
use esp_idf_sys as _;
use log::info;
use shared::config::config::Config;
use shared::controllers::fcu::FcuController;
use shared::messages::messages::Message;
use shared::operations::config_updater::{ConfigUpdateState, ConfigUpdater};
use shared::utils::time::Duration;
use shared::utils::{parts::Wheel, percentage::Percentage, speed::WheelSpeed, time::Timestamp};
use std::thread;
use std::time::Instant;

pub struct FcuWrapperController {
    controller: FcuController,
}

impl FcuWrapperController {
    pub fn new() -> Self {
        let config = Config::default();
        let controller = FcuController::new(config);

        Self { controller }
    }

    pub fn process_messages(&mut self) {
        let mut count: u8 = 0;
        loop {
            if let Some(msg) = get_message(Duration::from_millis(1)) {
                self.controller.process_message(msg);
                count += 1;
                if count > 10 {
                    break;
                }
            } else {
                break;
            }
        }
    }

    pub fn broadcast_ctl(&mut self) {
        // Read the raw values
        let tc_val = get_ti_value();
        // let break_val = get_ti_value();
        let break_val = Percentage::zero();

        broadcast_message(self.controller.broadcast_ctl(tc_val, break_val));
    }

    pub fn broadcast_upload(&mut self) {
        // Read the raw values
        //let tc_val = get_ti_value();
        // let break_val = get_ti_value();
        //let field_per= Percentage::zero();
        //let val_per = Percentage::zero();
        //let config_state = ConfigUpdateState::new(field_per, val_per);
        let config_state = ConfigUpdateState::default();

        if let Some(msg) = self.controller.run_config_update(config_state) {
            broadcast_message(msg);
        }
    }

    pub fn update_user_display(&mut self) {
        let state = self.controller.update_user_display();
        update_display(state);
    }
}
