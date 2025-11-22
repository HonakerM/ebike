// filepath: /esp32-led-control/esp32-led-control/src/main.rs
use embedded_can::Frame;
use embedded_can::StandardId;
use embedded_can::nb::Can;
use esp_idf_hal::can;
use esp_idf_hal::gpio::{Gpio32, Gpio33, Input, Output, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::timer::Timer;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;
use esp_idf_sys as _;
use fcu::peripherals::get_message;
use fcu::peripherals::setup;
use log::info;
use std::thread;
use fcu;
use shared::config::config::Config;
use shared::utils::time::Duration;
use shared::messages::messages::Message;

fn main() {
    // Required for ESP-IDF runtime patches
    link_patches();
    EspLogger::initialize_default();

    // Setup Peripherals
    setup();

    let mut config = Config::default();

    loop {
        // 
        let msg: Option<Message> = get_message(Duration::from_millis(1));


    }
}
