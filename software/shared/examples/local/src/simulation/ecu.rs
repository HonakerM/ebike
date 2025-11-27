use shared::utils::percentage::Percentage;

#[derive(Debug, Clone, Copy)]
pub struct EcuState {
    pub throttle: Percentage,
}

impl Default for EcuState {
    fn default() -> Self {
        Self {
            throttle: Percentage::zero(),
        }
    }
}
