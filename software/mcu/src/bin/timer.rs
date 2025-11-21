#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::peripherals::{TIM2, TIM4, TIM5};
use embassy_stm32::rcc::{APBPrescaler, LsConfig, clocks};
use embassy_stm32::timer;
use embassy_stm32::{
    Config, bind_interrupts, timer::CaptureCompareInterruptHandler, timer::UpdateInterruptHandler,
};
use embassy_time::{Instant, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    let mut config = Config::default();
    let p = embassy_stm32::init(config);

    let cur_clocks = clocks(&p.RCC);
    info!("Current live clocks: {:?}", cur_clocks);

    let mut local: u64 = 1;
    loop {
        //Timer::after_secs(1).await;
        local += 1;
        if local % 2 == 0 {
            info!("Current local: {}", local);
        }

        Timer::after_millis(10).await;
    }
}
