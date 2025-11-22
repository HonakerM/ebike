use core::ops::{Deref, DerefMut};
use core::time;

use crate::config::config::ConfigDelta;
use crate::messages::messages::control_req::ControlReqMessage;
use crate::messages::messages::tire_status::TireStatus;
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
pub struct FcuConfig {}

impl Default for FcuConfig {
    fn default() -> Self {
        FcuConfig {}
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FcuState {
    pub throttle_req: Percentage,
    pub brake_req: Percentage,
    pub update: ConfigUpdateState,
    pub cur_ws: Option<WheelSpeed>,
}

impl Default for FcuState {
    fn default() -> Self {
        FcuState {
            throttle_req: Percentage::zero(),
            brake_req: Percentage::zero(),
            update: ConfigUpdateState::default(),
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

    pub fn run_config_update(&mut self, state: ConfigUpdateState) -> Option<Message> {
        if state != self.state.update {
            self.state.update = state;
            Some(self.config_updater.run(state))
        } else {
            None
        }
    }

    pub fn broadcast_ctl(&mut self, throttle: Percentage, brake: Percentage) -> Message {
        self.state.brake_req = brake;
        self.state.throttle_req = throttle;
        Message::ControlReqMessage(ControlReqMessage {
            throttle_req: throttle,
            brake_req: brake,
        })
    }

    pub fn broadcast_wheel(&mut self, ws: WheelSpeed) -> Message {
        self.state.cur_ws = Some(ws);
        Message::TireStatusMessage(TireStatus {
            wheel: Wheel::Front,
            ws: ws,
        })
    }

    pub fn update_user_display(&self) -> FcuState {
        self.state
    }
}
