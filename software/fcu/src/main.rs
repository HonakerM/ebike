// filepath: /esp32-led-control/esp32-led-control/src/main.rs
use esp_idf_hal::gpio::{Gpio32, Gpio33, Output, Input, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::can;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;
use esp_idf_sys as _;
use log::info;
use std::thread;
use std::time::Duration;
use esp_idf_hal::timer::Timer;
use embedded_can::nb::Can;
use embedded_can::Frame;
use embedded_can::StandardId;


enum LED_STATE {
    ON,
    OFF,
}

impl LED_STATE {
    fn export(&self) -> [u8; 8] {
        match self {
            LED_STATE::ON => [1,0,0,0,0,0,0,0],
            LED_STATE::OFF => [0,0,0,0,0,0,0,0],
        }
    }

    fn from_data(data: &[u8; 8]) -> Option<LED_STATE> {
        match data {
            [1,0,0,0,0,0,0,0] => Some(LED_STATE::ON),
            [0,0,0,0,0,0,0,0] => Some(LED_STATE::OFF),
            _ => None,
        }
    }
}
fn main() {
    // Required for ESP-IDF runtime patches
    link_patches();
    EspLogger::initialize_default();

    // Take peripherals
    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    // Configure GPIO pins
    let button = PinDriver::input(peripherals.pins.gpio32).expect("Failed to configure button pin");
    let mut led = PinDriver::output(peripherals.pins.gpio33).expect("Failed to configure LED pin");

    // Configure CAN bus
    let filter = can::config::Filter::standard_allow_all();
    let timing = can::config::Timing::B500K;
    let config = can::config::Config::new().filter(filter).timing(timing);

    // Configure CAN driver
    let can_tx = peripherals.pins.gpio22;
    let can_rx = peripherals.pins.gpio23;
    let mut can = can::CanDriver::new(peripherals.can, can_tx, can_rx, &config).unwrap();
    can.start().expect("Failed to start CAN driver");

    let mut led_state = false;
    let mut previous_state = false;


    let LED_ID = StandardId::new(0x01).unwrap();
    let on_frame = &Frame::new(LED_ID, &LED_STATE::ON.export()).unwrap();
    let off_frame = &Frame::new(LED_ID, &LED_STATE::OFF.export()).unwrap();


    info!("Starting main loop");
    loop {
        // Check button state
        if button.is_high() {
            if !previous_state {
                led_state = !led_state;
                if led_state {
                    led.set_high().expect("Failed to set LED high");
                    info!("LED ON");
                    can.transmit(on_frame, 100).expect("Failed to send CAN frame");

                } else {
                    led.set_low().expect("Failed to set LED low");
                    info!("LED OFF");
                    can.transmit(off_frame, 100).expect("Failed to send CAN frame");
                }
                previous_state = true;
            }
        } else {
            previous_state = false;
        }

        // Check for incoming CAN frames
        if let Ok(frame) = can.receive(100) {
            if frame.id() == embedded_can::Id::Standard(LED_ID) {
                if let Some(state) = LED_STATE::from_data(&frame.data()[0..8].try_into().unwrap()) {
                    match state {
                        LED_STATE::ON => {
                            led.set_high().expect("Failed to set LED high");
                            info!("LED ON (from CAN)");
                            led_state = true;
                        },
                        LED_STATE::OFF => {
                            led.set_low().expect("Failed to set LED low");
                            info!("LED OFF (from CAN)");
                            led_state = false;
                        },
                    }
                }
            }
        }

        // Sleep to debounce button
        thread::sleep(Duration::from_millis(100));
    }
}