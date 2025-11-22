use crate::{controllers::mcu::McuConfig, subsystems::mcu::engine::EngineConfig};

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    pub mcu: McuConfig,
    pub engine: EngineConfig,
}

impl Config {
    pub fn apply_delta(&mut self, delta: ConfigDelta) {
        self.engine = delta.engine;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConfigDelta {
    pub engine: EngineConfig,
}
impl ConfigDelta {
    pub fn to_bytes(&self) -> [u8; 8] {
        [
            self.engine.throttle_map_mode.into(),
            self.engine.traction_control_mode.into(),
            self.engine.desired_slip.into(),
            0,
            0,
            0,
            0,
            0,
        ]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            engine: EngineConfig {
                throttle_map_mode: data[0].into(),
                traction_control_mode: data[1].into(),
                desired_slip: data[2].into(),
            },
        }
    }
}
