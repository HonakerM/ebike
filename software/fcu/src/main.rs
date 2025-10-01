use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::channel::Receiver;
use embassy_sync::channel::Sender;
use embedded_hal_async::digital::Wait;
use embedded_hal_async::can::{Can, Frame};
use esp_idf_hal::can;
use esp_idf_hal::gpio::{Gpio32, Gpio33, Input, Output, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;
use esp_idf_sys as _;
use log::info;

#[derive(Debug, Clone, Copy)]
enum LED_STATE {
    ON,
    OFF,
}

impl LED_STATE {
    fn export(&self) -> [u8; 8] {
        match self {
            LED_STATE::ON => [1, 0, 0, 0, 0, 0, 0, 0],
            LED_STATE::OFF => [0, 0, 0, 0, 0, 0, 0, 0],
        }
    }

    fn from_data(data: &[u8; 8]) -> Option<LED_STATE> {
        match data {
            [1, 0, 0, 0, 0, 0, 0, 0] => Some(LED_STATE::ON),
            [0, 0, 0, 0, 0, 0, 0, 0] => Some(LED_STATE::OFF),
            _ => None,
        }
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
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

    let (sender, receiver) = Channel::<NoopRawMutex, LED_STATE, 1>::new();

    spawner.spawn(button_task(button, sender.clone())).unwrap();
    spawner.spawn(can_task(can, sender.clone())).unwrap();
    spawner.spawn(led_task(receiver, led)).unwrap();
}

#[embassy_executor::task]
async fn button_task(mut button: PinDriver<'static, Gpio32, Input>, sender: Sender<'static, NoopRawMutex, LED_STATE, 1>) {
    let mut led_state = LED_STATE::OFF;
    loop {
        if button.is_high() {
            led_state = match led_state {
                LED_STATE::ON => LED_STATE::OFF,
                LED_STATE::OFF => LED_STATE::ON,
            };
            sender.send(led_state).await.unwrap();
        }
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task]
async fn can_task(mut can: can::CanDriver<'static>, sender: Sender<'static, NoopRawMutex, LED_STATE, 1>) {
    let led_id = embedded_can::StandardId::new(0x01).unwrap();
    loop {
        if let Ok(frame) = can.receive(100).await {
            if frame.id() == embedded_can::Id::Standard(led_id) {
                if let Some(state) = LED_STATE::from_data(&frame.data()[0..8].try_into().unwrap()) {
                    sender.send(state).await.unwrap();
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn led_task(mut receiver: Receiver<'static, NoopRawMutex, LED_STATE, 1>, mut led: PinDriver<'static, Gpio33, Output>) {
    loop {
        if let Ok(state) = receiver.receive().await {
            match state {
                LED_STATE::ON => {
                    led.set_high().expect("Failed to set LED high");
                    info!("LED ON");
                }
                LED_STATE::OFF => {
                    led.set_low().expect("Failed to set LED low");
                    info!("LED OFF");
                }
            }
        }
    }
}
