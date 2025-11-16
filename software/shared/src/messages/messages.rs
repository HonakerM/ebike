use core::time;

use embedded_can::StandardId;

use crate::{messages::ids::{FCU_MESG_ID, MCU_MESG_ID, RCU_MESG_ID}, utils::{percentage::Percentage, speed::WheelSpeed}};


pub enum Message{
    McuMessage(McuMessage),
    FcuMessage(FcuMessage),
    RcuMessage(RcuMessage), 
}

impl Message {
    fn to_bytes(&self) -> [u8; 8] {
        match self {
            Message::McuMessage(msg) => msg.to_bytes(),
            Message::FcuMessage(msg) => msg.to_bytes(),
            Message::RcuMessage(msg) => msg.to_bytes(),
        }
    }

    fn from_bytes(id: StandardId, data: &[u8], ) -> Option<Self> {
        if id == MCU_MESG_ID {
            Some(Message::McuMessage(McuMessage::from_bytes(data)))
        } else if id == FCU_MESG_ID {
            Some(Message::FcuMessage(FcuMessage::from_bytes(data)))
        } else if  id == RCU_MESG_ID {
            Some(Message::RcuMessage(RcuMessage::from_bytes(data)))
        } else {
            None
        }
    }
}

pub struct McuMessage {
    pub throttle: Percentage,
    pub brake: Percentage,
}

impl McuMessage {
    fn to_bytes(&self) -> [u8; 8] {
        [self.throttle.into(), self.brake.into(), 0, 0, 0, 0, 0, 0]
    }

    fn from_bytes(data: &[u8]) -> Self {
        Self {
            throttle: data[0].into(),
            brake: data[1].into(),
        }
    }
}

pub struct FcuMessage {
    pub throttle_req: Percentage,
    pub brake_req: Percentage,
    pub tire_rpm: WheelSpeed,
}

impl  FcuMessage {
    fn to_bytes(&self) -> [u8; 8] {
        let tire_rpm = self.tire_rpm.to_packets();
        [
            self.throttle_req.into(),
            self.brake_req.into(),
            tire_rpm[0],
                tire_rpm[1],
            0,
            0,
            0,
            0,
        ]
    }

    fn from_bytes(data: &[u8]) -> Self {
        Self {
            throttle_req: data[0].into(),
            brake_req: data[1].into(),
            tire_rpm: WheelSpeed::from_packets(&[data[2], data[3]]),
        }
    }
}

pub struct RcuMessage {
    pub tire_rpm: WheelSpeed,
}

impl RcuMessage {
    fn to_bytes(&self) -> [u8; 8] {
        let packets = self.tire_rpm.to_packets();
        [   
            packets[0],
            packets[1],
            0,
            0,
            0,
            0,
            0,
            0,
        ]
    }

    fn from_bytes(data: &[u8]) -> Self {
        Self {
            tire_rpm:  WheelSpeed::from_packets(&[data[0], data[1]]),
        }
    }
}