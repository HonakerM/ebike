use core::ops::{Deref, DerefMut};
use core::time;

use crate::{
    config::config::Config,
    controllers::shared::{ Lockable},
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
    pub fn new(config: Config) -> Self {
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