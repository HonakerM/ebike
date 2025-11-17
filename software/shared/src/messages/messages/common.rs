use embedded_can::StandardId;

use crate::{
    messages::{
        ids::{CTL_MESG_ID, ECU_MESG_ID, TRS_MESG_ID, UPD_MESG_ID},
        messages::{
            control_req::ControlReqMessage, ecu::EcuMessage, tire_status::TireStatus,
            update::Update,
        },
    },
    utils::percentage::Percentage,
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    EcuMessage(EcuMessage),
    TireStatusMessage(TireStatus),
    ControlReqMessage(ControlReqMessage),
    UpdateMessage(Update),
}

impl Message {
    pub fn to_bytes(&self) -> [u8; 8] {
        match self {
            Message::EcuMessage(msg) => msg.to_bytes(),
            Message::TireStatusMessage(msg) => msg.to_bytes(),
            Message::ControlReqMessage(msg) => msg.to_bytes(),
            Message::UpdateMessage(msg) => msg.to_bytes(),
        }
    }

    pub fn from_bytes(id: StandardId, data: &[u8]) -> Option<Self> {
        if id == ECU_MESG_ID {
            Some(Message::EcuMessage(EcuMessage::from_bytes(data)))
        } else if id == CTL_MESG_ID {
            Some(Message::ControlReqMessage(ControlReqMessage::from_bytes(
                data,
            )))
        } else if id == TRS_MESG_ID {
            Some(Message::TireStatusMessage(TireStatus::from_bytes(data)))
        } else if id == UPD_MESG_ID {
            Some(Message::UpdateMessage(Update::from_bytes(data)))
        } else {
            None
        }
    }

    fn to_id(&self) -> StandardId {
        match self {
            Message::EcuMessage(_) => ECU_MESG_ID,
            Message::TireStatusMessage(_) => TRS_MESG_ID,
            Message::ControlReqMessage(_) => CTL_MESG_ID,
            Message::UpdateMessage(_) => UPD_MESG_ID,
        }
    }
}
