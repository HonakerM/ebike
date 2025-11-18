#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::can::filter::Mask32;
use embassy_stm32::can::{
    Can, Fifo, Frame, Rx0InterruptHandler, Rx1InterruptHandler, SceInterruptHandler, StandardId,
    TxInterruptHandler,
};
use embassy_stm32::gpio::{Input, Pull};
use embassy_stm32::peripherals::CAN1;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::mutex::{Mutex, MutexGuard};
use embassy_time::Instant;
use embassy_time::{Delay, Duration, Timer};
use embedded_can::Id;
use mcu::perphierals::setup;
use mcu::wrappers::spawn;
use shared::config::config::Config;
use shared::messages::messages::Message;
use shared::utils::time::Timestamp;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");

    let mut p = embassy_stm32::init(Default::default());
    setup(p).await;
    spawn(spawner, Config::default()).await;
}
