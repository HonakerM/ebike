use crate::utils::percentage::Percentage;

pub struct ControlReqMessage {
    pub throttle_req: Percentage,
    pub brake_req: Percentage,
}

impl ControlReqMessage {
    pub fn to_bytes(&self) -> [u8; 8] {
        [
            self.throttle_req.into(),
            self.brake_req.into(),
            0,
            0,
            0,
            0,
            0,
            0,
        ]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            throttle_req: data[0].into(),
            brake_req: data[1].into(),
        }
    }
}
