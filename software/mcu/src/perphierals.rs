#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{PeripheralType, Peripherals, bind_interrupts};
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
use shared::config::config::Config;
use shared::controllers::mcu::{McuController, McuRunner};
use shared::controllers::shared::{ControllerRunner, HalInterface, Lockable};
use shared::messages::messages::Message;
use shared::utils::time::Timestamp;
use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Delay, Duration, Timer};
use embedded_can::Id;

bind_interrupts!(struct Irqs {
    CAN1_RX0 => Rx0InterruptHandler<CAN1>;
    CAN1_RX1 => Rx1InterruptHandler<CAN1>;
    CAN1_SCE => SceInterruptHandler<CAN1>;
    CAN1_TX => TxInterruptHandler<CAN1>;
});

pub static CAN: Mutex<ThreadModeRawMutex, Option<Can>> = Mutex::new(None);

pub async fn setup(mut p: Peripherals){
    // The next two lines are a workaround for testing without transceiver.
    // To synchronise to the bus the RX input needs to see a high level.
    // Use `mem::forget()` to release the borrow on the pin but keep the
    // pull-up resistor enabled.
    let rx_pin = Input::new(p.PA11.reborrow(), Pull::Up);
    core::mem::forget(rx_pin);

    let mut can = Can::new(p.CAN1, p.PA11, p.PA12, Irqs);

    can.modify_filters()
        .enable_bank(0, Fifo::Fifo0, Mask32::accept_all());

    can.modify_config()
        .set_loopback(true) // Receive own frames
        .set_silent(true)
        .set_bitrate(1_000_000);

    can.enable().await;

    *CAN.lock().await = Some(can);
}