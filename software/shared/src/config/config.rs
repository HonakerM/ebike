use crate::{controllers::mcu::McuConfig, subsystems::mcu::engine::EngineConfig};

#[derive(Debug, Clone, Copy, Default)]
pub struct Config {
    pub mcu: McuConfig,
    pub engine: EngineConfig,
}
