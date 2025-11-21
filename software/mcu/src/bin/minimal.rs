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
    use stm32f4xx_hal::timer::{CounterHz, Event};

    use rtic_monotonics::systick::prelude::*;
    systick_monotonic!(Mono, 100);

    use super::*;

    #[shared]
    struct Shared {
        // nothing shared
    }

    #[local]
    struct Local {}

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        println!("init");

        Mono::start(ctx.core.SYST, 12_000_000);

        foo::spawn().ok();
        bar::spawn().ok();
        baz::spawn().ok();

        (Shared {}, Local {})
    }

    #[task]
    async fn foo(_cx: foo::Context) {
        loop {
            println!("hello from foo");
            Mono::delay(1000.millis()).await;
            println!("bye from foo");
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
