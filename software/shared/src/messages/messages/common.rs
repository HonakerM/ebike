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

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
impl From<String> for Message {
    fn from(s: String) -> Self {
        // Deserialize the string into a Message
        // Format: "<ID>:<DATA>"
        let parts: Vec<&str> = s.splitn(2, ':').collect();
        if parts.len() != 2 {
            panic!("Invalid Message string format: '{s}'");
        }

        let id = parts[0].parse::<u16>().expect("Invalid ID format");
        let data = hex_to_bytes::<8>(parts[1]).expect("Invalid hex data");

        if id == ECU_MESG_ID.as_raw() {
            Message::EcuMessage(EcuMessage::from_bytes(&data))
        } else if id == CTL_MESG_ID.as_raw() {
            Message::ControlReqMessage(ControlReqMessage::from_bytes(&data))
        } else if id == TRS_MESG_ID.as_raw() {
            Message::TireStatusMessage(TireStatus::from_bytes(&data))
        } else if id == UPD_MESG_ID.as_raw() {
            Message::UpdateMessage(Update::from_bytes(&data))
        } else {
            panic!("Unknown Message ID");
        }
    }
}

#[cfg(feature = "std")]
impl Into<String> for Message {
    fn into(self) -> String {
        // Serialize the Message into a string
        // Format: "<ID>:<DATA>"
        let id = self.to_id().as_raw();
        let data = match self {
            Message::EcuMessage(msg) => msg.to_bytes(),
            Message::TireStatusMessage(msg) => msg.to_bytes(),
            Message::ControlReqMessage(msg) => msg.to_bytes(),
            Message::UpdateMessage(msg) => msg.to_bytes(),
        };

        let hex_data = bytes_to_hex(&data);
        format!("{}:{}", id, hex_data)
    }
}

#[cfg(feature = "std")]
// Helper function to convert a byte slice to a hexadecimal string
fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut hex = String::new();
    for byte in bytes {
        hex.push_str(&format!("{:02x}", byte));
    }
    hex
}

#[cfg(feature = "std")]
// Helper function to convert a hexadecimal string back to a byte vector
fn hex_to_bytes<const N: usize>(hex: &str) -> Result<[u8; N], String> {
    if hex.len() < N * 2 {
        return Err(format!(
            "Hex string length mismatch: expected {}, got {}",
            N * 2,
            hex.len()
        ));
    }

    let mut array = [0u8; N];
    for i in 0..N {
        array[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| "Invalid hex character".to_string())?;
    }
    Ok(array)
}
