use crate::utils::percentage::Percentage;

#[derive(Debug, Clone, Copy)]
pub struct EcuMessage {
    pub throttle: Percentage,
}

impl EcuMessage {
    pub fn to_bytes(&self) -> [u8; 8] {
        [self.throttle.into(), 0, 0, 0, 0, 0, 0, 0]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            throttle: data[0].into(),
        }
    }
}
