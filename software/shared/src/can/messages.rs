// ! All percentages are from 0 to u8 max

pub struct McuMessage {
    pub throttle: u8,
    pub brake: u8,
}

impl McuMessage {
    pub fn to_bytes(&self) -> [u8; 8] {
        [self.throttle, self.brake, 0, 0, 0, 0, 0, 0]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            throttle: data[0],
            brake: data[1],
        }
    }
}

pub struct FcuMessage {
    pub throttle_req: u8,
    pub brake_req: u8,
    pub tire_rpm: u16,
}

impl FcuMessage {
    pub fn to_bytes(&self) -> [u8; 8] {
        [
            self.throttle_req,
            self.brake_req,
            (self.tire_rpm & 0xFF) as u8,
            (self.tire_rpm >> 8) as u8,
            0,
            0,
            0,
            0,
        ]
    }

    pub fn from_bytes(data: &[u8]) -> Self {
        Self {
            throttle_req: data[0],
            brake_req: data[1],
            tire_rpm: (data[2] as u16) | ((data[3] as u16) << 8),
        }
    }
}

pub struct RcuMessage {
    pub tire_rpm: u16,
}

impl RcuMessage {
    pub fn to_bytes(&self) -> [u8; 8] {
        [
            (self.tire_rpm & 0xFF) as u8,
            (self.tire_rpm >> 8) as u8,
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
            tire_rpm: (data[0] as u16) | ((data[1] as u16) << 8),
        }
    }
}
