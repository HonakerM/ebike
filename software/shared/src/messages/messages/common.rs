use embedded_can::StandardId;

use crate::{
    messages::{
        ids::{CTL_MESG_ID, MCU_MESG_ID, TRS_MESG_ID},
        messages::{control_req::ControlReqMessage, mcu::McuMessage, tire_status::TireStatus},
    },
    utils::percentage::Percentage,
};

pub enum Message {
    McuMessage(McuMessage),
    TireStatusMessage(TireStatus),
    ControlReqMessage(ControlReqMessage),
}

impl Message {
    fn to_bytes(&self) -> [u8; 8] {
        match self {
            Message::McuMessage(msg) => msg.to_bytes(),
            Message::TireStatusMessage(msg) => msg.to_bytes(),
            Message::ControlReqMessage(msg) => msg.to_bytes(),
        }
    }

    fn from_bytes(id: StandardId, data: &[u8]) -> Option<Self> {
        if id == MCU_MESG_ID {
            Some(Message::McuMessage(McuMessage::from_bytes(data)))
        } else if id == CTL_MESG_ID {
            Some(Message::ControlReqMessage(ControlReqMessage::from_bytes(
                data,
            )))
        } else if id == TRS_MESG_ID {
            Some(Message::TireStatusMessage(TireStatus::from_bytes(data)))
        } else {
            None
        }
    }
}
