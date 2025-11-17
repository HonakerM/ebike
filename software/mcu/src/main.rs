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
use mcu::perphierals::setup;
use shared::config::config::Config;
use shared::controllers::mcu::{McuController, McuRunner};
use shared::controllers::shared::{ControllerRunner, HalInterface, Lockable};
use shared::messages::messages::Message;
use shared::utils::time::Timestamp;
use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Delay, Duration, Timer};
use embedded_can::Id;


#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Hello World!");

    let mut p = embassy_stm32::init(Default::default());

    let mut p = setup(p).await;


    let config = Config::default();
    let interface = HalInterface {
        get_timestamp: get_timestamp,
        get_can_message: get_can_message,
        broadcast_can_message: broadcast_can_message,
        sleep: local_sleep,
    };

    let runner: McuRunner<McuMutex, _, _, _> = McuRunner::new(config, interface);
}
