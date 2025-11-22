use std::cell::Ref;
use std::cell::RefCell;
use std::io::Write;

// filepath: /esp32-led-control/esp32-led-control/src/main.rs
use embedded_can::nb::Can;
use embedded_can::Frame;
use embedded_can::StandardId;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcChannelDriver, AdcDriver};
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::can;
use esp_idf_hal::can::CanDriver;
use esp_idf_hal::gpio::{Gpio12, Gpio32, Gpio33, Gpio36, Input, Output, PinDriver};
use esp_idf_hal::i2c;
use esp_idf_hal::i2c::I2cDriver;
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::task::CriticalSection;
use esp_idf_hal::timer::Timer;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::sys::link_patches;
use esp_idf_sys as _;
use lcd_i2c_rs::Lcd;
use log::info;
use shared::controllers::fcu::FcuState;
use shared::messages::ids::CFG_MESG_ID;
use shared::messages::messages::Message;
use shared::operations::config_updater::ConfigUpdateOptions;
use shared::utils::time::Duration;
use shared::utils::{parts::Wheel, percentage::Percentage, speed::WheelSpeed, time::Timestamp};

type THROTTLE_INPUT_TYPE = AdcChannelDriver<'static, Gpio36, AdcDriver<'static, ADC1>>;
type CAN_TYPE = CanDriver<'static>;
type LCD_TYPE = Lcd<'static>;

pub static CAN: critical_section::Mutex<RefCell<Option<CAN_TYPE>>> =
    critical_section::Mutex::new(RefCell::new(None));
pub static THROTTLE_INPUT: critical_section::Mutex<RefCell<Option<THROTTLE_INPUT_TYPE>>> =
    critical_section::Mutex::new(RefCell::new(None));
pub static LCD: critical_section::Mutex<RefCell<Option<LCD_TYPE>>> =
    critical_section::Mutex::new(RefCell::new(None));

pub fn setup() {
    // Take peripherals
    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    // Configure CAN bus
    let filter = can::config::Filter::Standard {
        filter: CFG_MESG_ID.as_raw(),
        mask: 0x7FF,
    };
    let timing = can::config::Timing::B500K;
    let config = can::config::Config::new().filter(filter).timing(timing);

    // Configure CAN driver
    let can_tx = peripherals.pins.gpio1;
    let can_rx = peripherals.pins.gpio3;
    let mut can = can::CanDriver::new(peripherals.can, can_tx, can_rx, &config).unwrap();

    //Configure Pin Drivers
    let ti_config = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let ti_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let ti_adc = AdcChannelDriver::new(ti_driver, peripherals.pins.gpio36, &ti_config).unwrap();

    // Configure L2C Driver
    let i2c = I2cDriver::new(
        peripherals.i2c0,
        peripherals.pins.gpio21,
        peripherals.pins.gpio22,
        &i2c::config::Config::new(),
    )
    .unwrap();
    let mut lcd: Lcd = Lcd::new(Ok(i2c), 16, 2); // Initialize for a 16x2 LCD display
    lcd.backlight_on();

    can.start().expect("Failed to start CAN driver");
    critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_can_driver: &mut Option<CanDriver> = &mut *CAN.borrow_ref_mut(cs);
        local_can_driver.replace(can);

        let local_ti_driver: &mut Option<_> = &mut *THROTTLE_INPUT.borrow_ref_mut(cs);
        local_ti_driver.replace(ti_adc);

        let local_lcd: &mut Option<_> = &mut *LCD.borrow_ref_mut(cs);
        local_lcd.replace(lcd);
    });
}

pub fn broadcast_message(msg: Message) {
    critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_can_driver: &mut Option<CAN_TYPE> = &mut *CAN.borrow_ref_mut(cs);
        if let Some(can_driver) = local_can_driver {
            can_driver
                .transmit(&Frame::new(msg.to_embedded_id(), &msg.to_bytes()).unwrap())
                .unwrap();
        } else {
            panic!("broadcast before setup");
        }
    });
}

pub fn get_message(timeout: Duration) -> Option<Message> {
    let frame_res: Option<esp_idf_hal::can::Frame> = critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_can_driver: &mut Option<CAN_TYPE> = &mut *CAN.borrow_ref_mut(cs);
        if let Some(can_driver) = local_can_driver {
            esp_idf_hal::can::CanDriver::receive(&can_driver, timeout.as_millis() as u32).ok()
        } else {
            panic!("get_message before setup");
        }
    });

    if let Some(frame) = frame_res {
        if let embedded_can::Id::Standard(id) = frame.id() {
            Message::from_bytes(id.as_raw(), frame.data())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_ti_value() -> Percentage {
    critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_ti: &mut Option<THROTTLE_INPUT_TYPE> = &mut *THROTTLE_INPUT.borrow_ref_mut(cs);
        if let Some(ti) = local_ti {
            Percentage::from_fractional(ti.read().unwrap() as f32 / u16::MAX as f32)
        } else {
            panic!("get_ti_value before setup")
        }
    })
}

pub fn update_display(state: FcuState) {
    critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_lcd: &mut Option<LCD_TYPE> = &mut *LCD.borrow_ref_mut(cs);
        if let Some(lcd) = local_lcd {
            lcd.clear().unwrap();
            lcd.print_str(
                format!(
                    "TR: {:03} BR: {:03}",
                    state.throttle_req.to_int(),
                    state.brake_req.to_int()
                )
                .as_str(),
            )
            .unwrap();
            lcd.set_cursor(0, 1).unwrap(); // Move to first column of the second row
            lcd.print_str(state.update.field.to_small_str()).unwrap();

            let mut ref_val_buffer = [0u8; 3];
            match state.update.val {
                ConfigUpdateOptions::DSL(per) => {
                    lcd.print_str(format!("{:03}", per.to_int()).as_str())
                        .unwrap();
                }
                ConfigUpdateOptions::TCM(tcm) => {
                    lcd.print_str(tcm.to_small_str()).unwrap();
                }
                ConfigUpdateOptions::TMM(tmm) => {
                    lcd.print_str(tmm.to_small_str()).unwrap();
                }
            };
        } else {
            panic!("update_display before setup")
        }
    });
}
