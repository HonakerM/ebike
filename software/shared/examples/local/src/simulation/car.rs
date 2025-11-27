use shared::{
    messages::messages::{Message, tire_status::TireStatus},
    utils::speed::WheelSpeed,
};

use crate::simulation::ecu::EcuState;

#[derive(Debug, Clone, Copy)]
pub struct CarState {
    pub ecu: EcuState,
    pub front_wheel: TireStatus,
    pub rear_wheel: TireStatus,
}

impl Default for CarState {
    fn default() -> Self {
        CarState {
            ecu: EcuState::default(),
            front_wheel: TireStatus::new(shared::utils::parts::Wheel::Front, WheelSpeed::zero()),
            rear_wheel: TireStatus::new(shared::utils::parts::Wheel::Rear, WheelSpeed::zero()),
        }
    }
}

impl CarState {
    pub fn update(&mut self, msg: Message) {
        match msg {
            Message::EcuMessage(msg) => {
                self.ecu.throttle = msg.throttle;
            }
            _ => {}
        }
    }
}
