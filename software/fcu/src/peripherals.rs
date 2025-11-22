use std::cell::Ref;
use std::cell::RefCell;

// filepath: /esp32-led-control/esp32-led-control/src/main.rs
use embedded_can::Frame;
use embedded_can::StandardId;
use embedded_can::nb::Can;
use esp_idf_hal::adc::ADC1;
use esp_idf_hal::adc::oneshot::config::AdcChannelConfig;
use esp_idf_hal::adc::oneshot::{AdcDriver, AdcChannelDriver};
use esp_idf_hal::can;
use esp_idf_hal::can::CanDriver;
use esp_idf_hal::gpio::{Gpio32, Gpio33,Gpio12, Gpio36, Input, Output, PinDriver};
use esp_idf_hal::peripheral::Peripheral;
use esp_idf_hal::task::CriticalSection;
use esp_idf_hal::timer::Timer;
use esp_idf_svc::hal::prelude::*;
use esp_idf_svc::log::EspLogger;
use shared::messages::messages::Message;
use shared::utils::time::Duration;
use shared::messages::ids::CFG_MESG_ID;
use esp_idf_svc::sys::link_patches;
use esp_idf_hal::adc::attenuation::DB_11;
use esp_idf_sys as _;
use log::info;


type THROTTLE_INPUT_TYPE = AdcChannelDriver<'static, Gpio36, AdcDriver<'static, ADC1> >;
type CAN_TYPE = CanDriver<'static>;

pub static CAN: critical_section::Mutex<RefCell<Option<CAN_TYPE>>> = critical_section::Mutex::new(RefCell::new(None));
pub static THROTTLE_INPUT: critical_section::Mutex<RefCell<Option<THROTTLE_INPUT_TYPE>>> = critical_section::Mutex::new(RefCell::new(None));

pub fn setup() {
    // Take peripherals
    let peripherals = Peripherals::take().expect("Failed to take peripherals");

    // Configure CAN bus
    let filter = can::config::Filter::Standard { filter: CFG_MESG_ID.as_raw(),mask: 0x7FF };
    let timing = can::config::Timing::B500K;
    let config = can::config::Config::new().filter(filter).timing(timing);

    // Configure CAN driver
    let can_tx = peripherals.pins.gpio22;
    let can_rx = peripherals.pins.gpio23;
    let mut can = can::CanDriver::new(peripherals.can, can_tx, can_rx, &config).unwrap();

    //Configure Pin Drivers
    let ti_config  = AdcChannelConfig {
        attenuation: DB_11,
        ..Default::default()
    };
    let ti_driver = AdcDriver::new(peripherals.adc1).unwrap();
    let ti_adc = AdcChannelDriver::new(ti_driver, peripherals.pins.gpio36, &ti_config).unwrap();

    can.start().expect("Failed to start CAN driver");
    critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_can_driver: &mut Option<CanDriver> = &mut *CAN.borrow_ref_mut(cs);
        local_can_driver.replace(can);    


        let local_ti_driver: &mut Option<_> = &mut *THROTTLE_INPUT.borrow_ref_mut(cs);
        local_ti_driver.replace(ti_adc);
    });




}


pub fn broadcast_message(msg: Message) {
    critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_can_driver: &mut Option<CAN_TYPE> = &mut *CAN.borrow_ref_mut(cs);
        if let Some(can_driver) = local_can_driver {
            can_driver.transmit(&Frame::new(msg.to_embedded_id(), &msg.to_bytes()).unwrap()).unwrap();
        } else {
            panic!("broadcast before setup");
        }
    });
}


pub fn get_message(timeout: Duration)->Option<Message>{
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
        if let  embedded_can::Id::Standard(id) = frame.id() {
            Message::from_bytes(id.as_raw(), frame.data())
        } else {
            None
        }
    } else {
        None
    }
}

pub fn get_ti_value() -> u16 {
     critical_section::with(|cs| {
        // `RefCell::borrow` and `RefCell::borrow_mut` are renamed to
        // `borrow_ref` and `borrow_ref_mut` to avoid name collisions
        let local_ti: &mut Option<THROTTLE_INPUT_TYPE> = &mut *THROTTLE_INPUT.borrow_ref_mut(cs);
        if let Some(ti) = local_ti {
            ti.read().unwrap()
        } else {
            panic!("get_ti_value before setup")
        }
    })
}