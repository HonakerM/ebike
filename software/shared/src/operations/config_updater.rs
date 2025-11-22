use crate::{
    config::config::Config,
    messages::messages::{
        Message,
        update::{Update, UpdateField},
    },
    operations::{
        throttle_map::{ThottleMap, ThottleMapMode},
        traction_control::TractionControlMode,
    },
    utils::percentage::Percentage,
};
use micromath::F32Ext;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ConfigUpdateOptions {
    DSL(Percentage),
    TCM(TractionControlMode),
    TMM(ThottleMapMode),
}

impl ConfigUpdateOptions {
    pub fn to_bytes(&self) -> [u8; 7] {
        match &self {
            ConfigUpdateOptions::DSL(per) => [Percentage::into(*per), 0, 0, 0, 0, 0, 0],
            ConfigUpdateOptions::TCM(tcm) => [TractionControlMode::into(*tcm), 0, 0, 0, 0, 0, 0],
            ConfigUpdateOptions::TMM(tmm) => [ThottleMapMode::into(*tmm), 0, 0, 0, 0, 0, 0],
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ConfigUpdateState {
    pub field: UpdateField,
    pub val: ConfigUpdateOptions,
}
impl ConfigUpdateState {
    fn new(field_per: Percentage, val_per: Percentage) -> Self {
        let raw_val = field_per.to_fractional();
        let field = if raw_val <= 0.33 {
            UpdateField::DSL()
        } else if raw_val <= 0.66 {
            UpdateField::TCM()
        } else {
            UpdateField::TMM()
        };

        let fractional = val_per.to_fractional();
        let data = match field {
            UpdateField::DSL() => {
                ConfigUpdateOptions::DSL(Percentage::from_int((val_per.to_int() / 10)).into())
            }
            UpdateField::TCM() => ConfigUpdateOptions::TCM(if fractional <= 0.5 {
                TractionControlMode::Level0()
            } else {
                TractionControlMode::Level1()
            }),
            UpdateField::TMM() => ConfigUpdateOptions::TMM(if fractional <= 0.33 {
                ThottleMapMode::Level0()
            } else if fractional <= 0.66 {
                ThottleMapMode::Level1()
            } else {
                ThottleMapMode::Level2()
            }),
        };
        Self { field, val: data }
    }
}

impl Default for ConfigUpdateState {
    fn default() -> Self {
        Self {
            field: UpdateField::DSL(),
            val: ConfigUpdateOptions::DSL(Config::default().engine.desired_slip),
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
        Message::UpdateMessage(Update::new(state.field, &state.val.to_bytes()))
    }
}
