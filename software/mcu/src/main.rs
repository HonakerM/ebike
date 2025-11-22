#![no_std]
#![no_main]

use mcu as _;
use rtic::app;
use stm32f4xx_hal::{
    gpio::{Output, PushPull, gpiog::PG13},
    pac,
    prelude::*,
    timer::Timer,
};

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use defmt::{debug, println};
    use shared::config::config::Config;
    use stm32f4xx_hal::pac::CAN1;
    use stm32f4xx_hal::timer::{CounterHz, Event};
    use stm32f4xx_hal::can::Can;

    use shared::controllers::mcu::McuController;

    use rtic_monotonics::systick::prelude::*;
    systick_monotonic!(Mono, 100);

    use super::*;

    #[shared]
    struct Shared {
        controller: McuController,
        can: Can<CAN1>,
    }

    #[local]
    struct Local {
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        println!("init");
        let mut dp = ctx.device;
        let rcc = &mut dp.RCC;

        println!("init timer");
        Mono::start(ctx.core.SYST, 12_000_000);
        
        println!("init can");
        let mut gpioa = dp.GPIOA.split(rcc);
        let mut can = Can::new(dp.CAN1,(gpioa.pa12,gpioa.pa11),  rcc);

        println!("init controller");
        let mut controller = McuController::new(Config::default());


        (Shared {controller, can: can}, Local {})
    }

    #[task(shared = [controller, can])]
    async fn broadcast_ecu(mut cx: broadcast_ecu::Context) {
        loop {
            let (sleep_time, msg) =  cx.shared.controller.lock(|ctl| {
                    let msg = ctl.broadcast_ecu();
                    (ctl.config.mcu.ecu_poll, msg)
                });

            cx.shared.can.lock(|cn| {
            });
                
            //eprintln!("{}", Into::<String>::into(msg));
            //(broadcast_can_message(msg)).await;

            Mono::delay((sleep_time.as_millis() as u32).millis()).await;
        }
    }

    #[task]
    async fn bar(_cx: bar::Context) {
        loop {
            println!("hello from bar");
            Mono::delay(2000.millis()).await;
            println!("bye from bar");
        }
    }

    #[task]
    async fn baz(_cx: baz::Context) {
        println!("hello from baz");
        Mono::delay(3000.millis()).await;
        println!("bye from baz");

        return;
    }
}
