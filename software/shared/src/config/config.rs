use crate::{controllers::mcu::McuConfig, subsystems::mcu::engine::EngineConfig};

pub struct Config {
    pub mcu: McuConfig,
    pub engine: EngineConfig,
}
