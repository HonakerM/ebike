use crate::{
    config::config::Config,
    operations::throttle_map::ThottleMapMode,
    utils::{parts::Wheel, speed::WheelSpeed},
};

#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum UpdateField {
    TMM(), // Throttle Map Mode
    TCM(), // Traction Control Mode
    DSL(), // Desired Slip Level

           // Add ECU settings later
           // ESP(), // Engine Subsystem Poll Time
}

impl From<u8> for UpdateField {
    fn from(value: u8) -> Self {
        match value {
            0 => UpdateField::TMM(),
            1 => UpdateField::TCM(),
            2 => UpdateField::DSL(),
            _ => UpdateField::TMM(),
        }
    }
}

impl Into<u8> for UpdateField {
    fn into(self) -> u8 {
        match self {
            UpdateField::TMM() => 0,
            UpdateField::TCM() => 1,
            UpdateField::DSL() => 2,
        }
    }
}
impl UpdateField {
    pub fn update_config(&self, config: &mut Config, data: [u8; 7]) {
        match self {
            UpdateField::TMM() => {
                config.engine.throttle_map_mode = data[0].into();
            }
            UpdateField::TCM() => {
                config.engine.traction_control_mode = data[0].into();
            }
            UpdateField::DSL() => {
                config.engine.desired_slip = data[0].into();
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Update {
    pub field: UpdateField,
    pub data: [u8; 7],
}

impl Update {
    pub fn to_bytes(&self) -> [u8; 8] {
        [
            self.field.into(),
            self.data[0],
            self.data[1],
            self.data[2],
            self.data[3],
            self.data[4],
            self.data[5],
            self.data[6],
        ]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        let parsed_data: [u8; 7] = data[1..8].try_into().unwrap();
        Self {
            field: data[0].into(),
            data: parsed_data,
        }
    }

    pub fn new(field: UpdateField, data: &[u8]) -> Self {
        let parsed_data: [u8; 7] = data[0..7].try_into().unwrap();
        Self {
            field,
            data: parsed_data,
        }
    }

    pub fn update(&self, config: &mut Config) {
        self.field.update_config(config, self.data);
    }
}
