use crate::{
    config::config::Config,
    controllers::shared::{Controller, HalInterface},
    messages::messages::{ecu::EcuMessage, Message},
    subsystems::{
        mcu::engine::{EngineRequest, EngineSubsystem},
        shared::Subsystem,
    },
    utils::{parts::Wheel, percentage::Percentage, speed::WheelSpeed, time::Duration},
};

#[derive(Debug, Clone, Copy)]
pub struct McuConfig {
    pub engine_poll: Duration,
    pub ecu_poll: Duration,
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

pub struct McuController<F>
where
    F: std::future::Future,
{
    pub config: Config,
    state: McuState,
    interface: HalInterface<F>,

    engine_subsystem: EngineSubsystem,
}

impl<F> Controller<F> for McuController<F>
where
    F: std::future::Future,
{
    fn new(config: Config, interface: HalInterface<F>) -> Self {
        let engine_subsystem = EngineSubsystem::new(config.engine);
        McuController::<F> {
            config,
            state: McuState::default(),
            interface,
            engine_subsystem,
        }
    }
}

impl<F> McuController<F>
where
    F: std::future::Future,
{
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

    pub fn run_engine_subsystem(&mut self) {
        let req = EngineRequest {
            rear_ws: self.state.rear_ws,
            front_ws: self.state.front_ws,
            throttle_req: self.state.throttle_req,
            timestamp: (self.interface.get_timestamp)(),
        };
        let resp = self.engine_subsystem.run(req);
        self.state.throttle = resp.throttle_req;
    }

    pub async fn broadcast_ecu(&self) {
        let ecu_message = Message::EcuMessage(EcuMessage {
            throttle: self.state.throttle,
        });
        (self.interface.broadcast)(ecu_message).await;
    }
}
