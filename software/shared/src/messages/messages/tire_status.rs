use crate::utils::{parts::Wheel, speed::WheelSpeed};

pub struct TireStatus {
    pub wheel: Wheel,
    pub ws: WheelSpeed,
}

impl TireStatus {
    pub fn to_bytes(&self) -> [u8; 8] {
        let packets = self.ws.to_packets();
        [self.wheel.into(), packets[0], packets[1], 0, 0, 0, 0, 0]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            wheel: data[0].into(),
            ws: WheelSpeed::from_packets(&[data[1], data[2]]),
        }
    }
}
