use crate::utils::percentage::Percentage;

pub struct McuMessage {
    pub throttle: Percentage,
    pub brake: Percentage,
}

impl McuMessage {
    pub fn to_bytes(&self) -> [u8; 8] {
        [self.throttle.into(), self.brake.into(), 0, 0, 0, 0, 0, 0]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            throttle: data[0].into(),
            brake: data[1].into(),
        }
    }
}
