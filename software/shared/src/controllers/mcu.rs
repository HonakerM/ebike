use core::time;
use std::ops::{Deref, DerefMut};

use crate::{
    config::config::Config,
    controllers::shared::{ControllerRunner, HalInterface, Lockable},
    messages::messages::{Message, ecu::EcuMessage},
    subsystems::{
        mcu::engine::{EngineRequest, EngineSubsystem},
        shared::Subsystem,
    },
    utils::{
        parts::Wheel,
        percentage::Percentage,
        speed::WheelSpeed,
        time::{Duration, Timestamp},
    },
};

#[derive(Debug, Clone, Copy)]
pub struct McuConfig {
    pub engine_poll: Duration,
    pub ecu_poll: Duration,
}

impl Default for McuConfig {
    fn default() -> Self {
        McuConfig {
            engine_poll: Duration::from_millis(100),
            ecu_poll: Duration::from_millis(500),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct McuState {
    throttle: Percentage,
    brake: Percentage,
    throttle_req: Percentage,
    brake_req: Percentage,
    rear_ws: Option<WheelSpeed>,
    front_ws: Option<WheelSpeed>,
}

impl Default for McuState {
    fn default() -> Self {
        McuState {
            throttle: Percentage::zero(),
            brake: Percentage::zero(),
            throttle_req: Percentage::zero(),
            brake_req: Percentage::zero(),
            rear_ws: None,
            front_ws: None,
        }
    }
}

pub struct McuController {
    pub config: Config,
    state: McuState,

    engine_subsystem: EngineSubsystem,
}

impl McuController {
    fn new(config: Config) -> Self {
        let engine_subsystem = EngineSubsystem::new(config.engine);
        McuController {
            config,
            state: McuState::default(),
            engine_subsystem,
        }
    }

    pub fn process_message(&mut self, msg: Message) {
        match msg {
            Message::TireStatusMessage(status) => match status.wheel {
                Wheel::Rear => {
                    self.state.rear_ws = Some(status.ws);
                }
                Wheel::Front => {
                    self.state.front_ws = Some(status.ws);
                }
            },
            Message::ControlReqMessage(req) => {
                self.state.throttle_req = req.throttle_req;
                self.state.brake_req = req.brake_req;
            }
            _ => {}
        }
    }

    pub fn run_engine_subsystem(&mut self, timestamp: Timestamp) {
        let req = EngineRequest {
            rear_ws: self.state.rear_ws,
            front_ws: self.state.front_ws,
            throttle_req: self.state.throttle_req,
            timestamp: timestamp,
        };
        let resp = self.engine_subsystem.run(req);
        self.state.throttle = resp.throttle_req;
    }

    pub fn broadcast_ecu(&self) -> Message {
        Message::EcuMessage(EcuMessage {
            throttle: self.state.throttle,
        })
    }
}

pub struct McuRunner<M, MF, EF, SF>
where
    M: From<McuController> + Lockable<Target = McuController>,
    MF: core::future::Future<Output = Message>,
    EF: core::future::Future<Output = ()>,
    SF: core::future::Future<Output = ()>,
{
    controller: M,
    interface: HalInterface<MF, EF, SF>,
}

impl<M, MF, EF, SF> ControllerRunner<MF, EF, SF> for McuRunner<M, MF, EF, SF>
where
    M: From<McuController> + Lockable<Target = McuController>,
    MF: core::future::Future<Output = Message>,
    EF: core::future::Future<Output = ()>,
    SF: core::future::Future<Output = ()>,
{
    fn new(config: Config, interface: HalInterface<MF, EF, SF>) -> Self {
        let controler = McuController::new(config);
        let controller = M::from(controler);
        McuRunner {
            controller,
            interface,
        }
    }
}

impl<M, MF, EF, SF> McuRunner<M, MF, EF, SF>
where
    M: From<McuController> + Lockable<Target = McuController>,
    MF: core::future::Future<Output = Message>,
    EF: core::future::Future<Output = ()>,
    SF: core::future::Future<Output = ()>,
{
    pub async fn broadcast_ecu(&self) {
        loop {
            let (sleep_time, msg) = {
                let controller = self.controller.lock().await;
                let msg = controller.broadcast_ecu();
                (controller.config.mcu.ecu_poll, msg)
            };
            eprintln!("{}", Into::<String>::into(msg));
            ((self.interface.broadcast_can_message)(msg)).await;
            (self.interface.sleep)(sleep_time).await
        }
    }
    pub async fn run_engine_subsystem(&self) {
        loop {
            let sleep_time = {
                let mut controller = self.controller.lock().await;
                controller.run_engine_subsystem(((self.interface.get_timestamp)()));
                controller.config.mcu.engine_poll
            };
            (self.interface.sleep)(sleep_time).await
        }
    }

    pub async fn process_messages(&self) {
        loop {
            let msg = (self.interface.get_can_message)().await;
            {
                let mut controller = self.controller.lock().await;
                controller.process_message(msg);
            }
        }
    }
}
