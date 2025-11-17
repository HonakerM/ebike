use embedded_can::StandardId;

use crate::{config::config::Config, messages::messages::Message, utils::time::{Duration, Timestamp}};

pub struct HalInterface<MF, EF, SF> where MF: core::future::Future<Output=Message>, EF: core::future::Future<Output=()>, SF:core::future::Future<Output=()>   {
    pub get_timestamp: fn() -> Timestamp,
    pub get_can_message: fn() -> MF,
    pub broadcast_can_message: fn(Message) -> EF,
    pub sleep: fn(Duration) -> SF,
}
pub trait Controller<MF, EF, SF> where MF: core::future::Future<Output=Message>, EF: core::future::Future<Output=()>,SF:core::future::Future<Output=()> {
    fn new(config: Config, interface: HalInterface<MF, EF, SF>) -> Self;
}
