// filepath: /esp32-led-control/esp32-led-control/src/main.rs
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
use fcu;
use fcu::peripherals::broadcast_message;
use fcu::peripherals::get_message;
use fcu::peripherals::get_ti_value;
use fcu::peripherals::setup;
use fcu::wrapper::FcuWrapperController;
use log::info;
use shared::config::config::Config;
use shared::controllers::fcu::FcuController;
use shared::messages::messages::Message;
use shared::utils::time::Duration;
use shared::utils::{parts::Wheel, percentage::Percentage, speed::WheelSpeed, time::Timestamp};
use std::thread;
use std::time::Instant;

fn main() {
    // Required for ESP-IDF runtime patches
    link_patches();
    EspLogger::initialize_default();

    // Setup Peripherals
    setup();

    let mut controller = FcuWrapperController::new();

    let cur_time = Instant::now();
    loop {
        let elapsed_time = cur_time.elapsed();
        let (read_msgs, broad_ctl, broad_upd, refresh_disp) = {
            let mils = elapsed_time.as_millis();
            (
                mils % 250 == 0,
                mils % 15 == 0,
                mils % 500 == 0,
                mils % 100 == 0,
            )
        };

        // Process all messages up to 10 messages
        if read_msgs {
            controller.process_messages();
            println!("Processed Messages")
        }
        if broad_ctl {
            controller.broadcast_ctl();
            println!("Broadcasted Ctl")
        }
        if broad_upd {
            controller.broadcast_upload();
            println!("Broadcasted Upload")
        }
        if refresh_disp {
            controller.update_user_display();
            println!("Update Display")
        }
    }
}
