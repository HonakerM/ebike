use shared::{
    messages::{
        ids::ECU_MESG_ID,
        messages::{Message, ecu::EcuMessage},
    },
    utils::percentage::Percentage,
};

fn main() {
    let msg = Message::EcuMessage(EcuMessage {
        throttle: Percentage::full(),
    });
    let bytes = msg.to_bytes();
    let byte_str = bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<String>>()
        .join(" ");
    println!("{:?}:{:?}", ECU_MESG_ID.as_raw(), byte_str)
}
