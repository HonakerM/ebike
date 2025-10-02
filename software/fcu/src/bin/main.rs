#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::peripherals::Peripherals;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::timer::timg::TimerGroup;
use embedded_can::{Frame, StandardId};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

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

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let button = Input::new(peripherals.GPIO22, InputConfig::default());
    let mut led = Output::new(peripherals.GPIO23, Level::Low, OutputConfig::default());
    
    let timer0 = TimerGroup::new(peripherals.TIMG1);
    esp_hal_embassy::init(timer0.timer0);

    loop {
        println!("Bing! Is button High: {}", button.is_high());
        Timer::after(Duration::from_millis(100)).await;
    }
    /* 
    let filter = can::config::Filter::standard_allow_all();
    let timing = can::config::Timing::B500K;
    let config = can::config::Config::new().filter(filter).timing(timing);

    let can_tx = peripherals.GPIO22;
    let can_rx = peripherals.GPIO23;
    let mut can = can::CanDriver::new(peripherals.can, can_tx, can_rx, &config).unwrap();
    can.start().expect("Failed to start CAN driver");

    let led_id = StandardId::new(0x01).unwrap();
    let on_frame = Frame::new(led_id, &LED_STATE::ON.export()).unwrap();
    let off_frame = Frame::new(led_id, &LED_STATE::OFF.export()).unwrap();

    let mut led_state = false;
    let mut previous_state = false;

    loop {
        // Check button state
        if button.is_high() {
            if !previous_state {
                led_state = !led_state;
                if led_state {
                    led.set_high().expect("Failed to set LED high");
                    println!("LED ON");
                    can.transmit(&on_frame, 100).expect("Failed to send CAN frame");
                } else {
                    led.set_low().expect("Failed to set LED low");
                    println!("LED OFF");
                    can.transmit(&off_frame, 100).expect("Failed to send CAN frame");
                }
                previous_state = true;
            }
        } else {
            previous_state = false;
        }

        // Check for incoming CAN frames
        if let Ok(frame) = can.receive(100) {
            if frame.id() == embedded_can::Id::Standard(led_id) {
                if let Some(state) = LED_STATE::from_data(&frame.data()[0..8].try_into().unwrap()) {
                    match state {
                        LED_STATE::ON => {
                            led.set_high().expect("Failed to set LED high");
                            println!("LED ON (from CAN)");
                            led_state = true;
                        }
                        LED_STATE::OFF => {
                            led.set_low().expect("Failed to set LED low");
                            println!("LED OFF (from CAN)");
                            led_state = false;
                        }
                    }
                }
            }
        }

        // Sleep to debounce button
        Timer::after(Duration::from_millis(100)).await;
    }
    */
}
