use core::ops::{Deref, DerefMut};
use core::time;

use crate::config::config::ConfigDelta;
use crate::operations::config_updater::{ConfigUpdateState, ConfigUpdater};
use crate::{
    config::config::Config,
    controllers::shared::Lockable,
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
pub struct FcuConfig {
}

impl Default for FcuConfig {
    fn default() -> Self {
        FcuConfig {
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FcuState {
    throttle_req: Percentage,
    brake_req: Percentage,
    cur_state: ConfigUpdateState,
    cur_ws: Option<WheelSpeed>,
}

impl Default for FcuState {
    fn default() -> Self {
        FcuState {
            throttle_req: Percentage::zero(),
            brake_req: Percentage::zero(),
            cur_state: ConfigUpdateState::default(),
            cur_ws: None,
        }
    }
}

pub struct FcuController {
    pub config: Config,
    state: FcuState,

    config_updater: ConfigUpdater,
}

impl FcuController {
    pub fn new(config: Config) -> Self {
        FcuController {
            config,
            state: FcuState::default(),
            config_updater: ConfigUpdater::new(),
        }
    }

    pub fn process_message(&mut self, msg: Message) {
        match msg {
            Message::ConfigMessage(req) => {
                self.config.apply_delta(req);
            }
            Message::UpdateMessage(req) => {
                req.update(&mut self.config);
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

    pub fn broadcast_config(&self) -> Message {
        Message::ConfigMessage(ConfigDelta {
            engine: self.config.engine,
        })
    }
}
