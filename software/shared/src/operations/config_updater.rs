use crate::{config::config::Config, messages::messages::{Message, update::{Update, UpdateField}}, operations::{throttle_map::{ThottleMap, ThottleMapMode}, traction_control::TractionControlMode}, utils::percentage::Percentage};
use micromath::F32Ext;


#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConfigUpdateState {
    pub field: UpdateField,
    pub val: Percentage,
}
impl Default for ConfigUpdateState {
    fn default() -> Self {
        Self {
            field: UpdateField::DSL(),
            val: Percentage::from_int(
                Config::default().engine.desired_slip.to_int()*100
            )
        }
    }
}
impl Eq for ConfigUpdateState {}

pub struct ConfigUpdater {}

impl ConfigUpdater {
    pub fn new() -> Self {
        Self {}
    }
    pub fn run(&self, state: ConfigUpdateState) -> Message {
        let data = match state.field {
            UpdateField::DSL() => {
                [
                    Percentage::from_int((state.val.to_int()/10)).into(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]
            },
            UpdateField::TCM() => {
                let level = if state.val.to_fractional() <= 0.5 {
                    TractionControlMode::Level0()
                } else {
                    TractionControlMode::Level1()
                };
                [
                    level.into(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]
            },
            UpdateField::TMM() => {
                let level = if state.val.to_fractional() <= 0.33 {
                    ThottleMapMode::Level0()
                } else if state.val.to_fractional() <= 0.66 {
                    ThottleMapMode::Level1()
                } else {
                    ThottleMapMode::Level2()
                };
                [
                    level.into(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]
            },
        };

        Message::UpdateMessage(Update::new(state.field, &data))
    }
}
