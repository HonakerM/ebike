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
    use bxcan::{Can as HalCan, Frame, Id, Mailbox, Rx0, Rx1, StandardId, Tx};
    use defmt::{debug, println};
    use shared::config::config::Config;
    use shared::messages::messages::Message;
    use shared::utils::time::Timestamp;
    use stm32f4xx_hal::can::Can;
    use stm32f4xx_hal::pac::CAN1;
    use stm32f4xx_hal::pac::dma1::st::par;
    use stm32f4xx_hal::timer::{CounterHz, Event};

    use shared::controllers::mcu::McuController;

    use rtic_monotonics::systick::prelude::*;
    systick_monotonic!(Mono, 100);

    use super::*;

    #[shared]
    struct Shared {
        controller: McuController,
        can_tx: Tx<Can<CAN1>>,
        prev_ecu_mailbox: Option<Mailbox>,
    }

    #[local]
    struct Local {
        can_rx_0: Rx0<Can<CAN1>>,
        can_rx_1: Rx1<Can<CAN1>>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local) {
        println!("init");
        let mut dp = ctx.device;

        let mut real_rcc = dp.RCC.freeze(stm32f4xx_hal::rcc::Config::hsi());
        let rcc = &mut real_rcc;

        println!("init timer");
        Mono::start(ctx.core.SYST, 12_000_000);

        println!("init can");
        let mut gpiob = dp.GPIOB.split(rcc);
        let mut can = dp.CAN1.can((gpiob.pb9, gpiob.pb8), rcc);
        println!("init hal can");
        let hal_can: HalCan<Can<CAN1>> = HalCan::builder(can).set_bit_timing(0x001c_0000).enable();
        let (can_tx, can_rx_0, can_rx_1) = hal_can.split();

        println!("init controller");
        let mut controller = McuController::new(Config::default());

        println!("init tasks");
        broadcast_ecu::spawn().unwrap();
        run_engine_subsystem::spawn().unwrap();
        process_messages::spawn().unwrap();

        (
            Shared {
                controller,
                can_tx,
                prev_ecu_mailbox: None,
            },
            Local { can_rx_0, can_rx_1 },
        )
    }

    #[task(shared = [controller, can_tx, prev_ecu_mailbox])]
    async fn broadcast_ecu(mut cx: broadcast_ecu::Context) {
        loop {
            let (sleep_time, msg) = cx.shared.controller.lock(|ctl| {
                let msg = ctl.broadcast_ecu();
                (ctl.config.mcu.ecu_poll, msg)
            });

            cx.shared.can_tx.lock(|cn| {
                let frame = Frame::new_data(
                    StandardId::new(msg.to_id().as_raw()).unwrap(),
                    msg.to_bytes(),
                );

                println!("Sending msg: {}", msg);
                let mail_box = match cn.transmit(&frame) {
                    Ok(status) => Some(status.mailbox()),
                    Err(nb::Error::WouldBlock) => {
                        // Identify the mailbox with the pending ECU message
                        let prev_ecu_val =
                            cx.shared.prev_ecu_mailbox.lock(|prev_ecu| prev_ecu.clone());
                        if let Some(prev_ecu_mailbox) = prev_ecu_val {
                            cn.abort(prev_ecu_mailbox);

                            match cn.transmit(&frame) {
                                Ok(status) => Some(status.mailbox()),
                                Err(err) => {
                                    println!("Failed to send message due to {:?}", err);
                                    None
                                }
                            }
                        } else {
                            None
                        }
                    }
                };
                cx.shared.prev_ecu_mailbox.lock(|prev_ecu| {
                    *prev_ecu = mail_box;
                });
            });

            Mono::delay((sleep_time.as_millis() as u32).millis()).await;
        }
    }

    #[task(shared = [controller])]
    async fn run_engine_subsystem(mut cx: run_engine_subsystem::Context) {
        loop {
            let (sleep_time) = cx.shared.controller.lock(|ctl| {
                ctl.run_engine_subsystem(Timestamp::from_micros(Mono::now().ticks() as u64));
                ctl.config.mcu.engine_poll
            });

            println!("Finished Engine Subsystem");

            Mono::delay((sleep_time.as_millis() as u32).millis()).await;
        }
    }

    #[task(shared = [controller], local=[can_rx_0, can_rx_1])]
    async fn process_messages(mut cx: process_messages::Context) {
        loop {
            let msg = {
                match cx.local.can_rx_0.receive() {
                    Ok(msg) => Some(msg),
                    Err(nb::Error::WouldBlock) => match cx.local.can_rx_1.receive() {
                        Ok(msg) => Some(msg),
                        Err(nb::Error::WouldBlock) => None,
                        Err(err) => {
                            panic!("{:?}", err)
                        }
                    },
                    Err(err) => {
                        panic!("{:?}", err);
                    }
                };
                if let Ok(msg) = cx.local.can_rx_0.receive() {
                    Some(msg)
                } else if let Ok(msg) = cx.local.can_rx_1.receive() {
                    Some(msg)
                } else {
                    None
                }
            };

            if let Some(msg) = msg {
                let raw_id = if let Id::Standard(id) = msg.id() {
                    id.as_raw()
                } else {
                    continue;
                };

                let parsed_msg = Message::from_bytes(
                    unsafe { embedded_can::StandardId::new_unchecked(raw_id) },
                    msg.data().unwrap(),
                );
                if let Some(parsed_msg) = parsed_msg {
                    println!("Processing Message: ");
                    cx.shared.controller.lock(|ctl| {
                        ctl.process_message(parsed_msg);
                    });
                }
            } else {
                Mono::delay((10).millis()).await;
            }
        }
    }
}
