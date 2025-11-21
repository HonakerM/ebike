#![no_std]
#![no_main]

use stm32f4xx_hal::{
    gpio::{gpiog::PG13, Output, PushPull},
    pac,
    prelude::*,
    timer::Timer,
};
use rtic::app;
use mcu as _;

#[app(device = stm32f4xx_hal::pac, peripherals = true)]
mod app {
    use defmt::println;
    use stm32f4xx_hal::timer::{CounterHz, Event};

    use super::*;

    #[shared]
    struct Shared {
        // nothing shared
    }

    #[local]
    struct Local {
        led: PG13<Output<PushPull>>,
        tim2: CounterHz<pac::TIM2>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        let device: pac::Peripherals = ctx.device;

        // Enable clocks
        let mut rcc = device.RCC.constrain();
        let clocks = rcc.cfgr().write(|w|{ let new_w = w.sw().hse(); new_w });

        // Setup GPIO
        let gpiog = device.GPIOG.split(&mut rcc);
        let mut led = gpiog.pg13.into_push_pull_output();

        // Setup timer: 2 seconds
        let mut tim2 = Timer::new(device.TIM2, &mut rcc).counter_hz();
        tim2.start(1.Hz()).unwrap(); // 2 Hz = one toggle every 1 seconds
        tim2.listen(Event::Update );

        (Shared {}, Local { led, tim2 })
    }

    // Timer interrupt
    #[task(binds = TIM2, local = [led, tim2])]
    fn tim2_irq(ctx: tim2_irq::Context) {
        let tim2 = ctx.local.tim2;

        // toggle LED
        println!("We're toggleing");
        ctx.local.led.toggle();
    }
}
