use std::time::Duration;

use embedded_can::StandardId;

use crate::{config::config::Config, messages::messages::Message, utils::time::Timestamp};

pub struct HalInterface {
    pub get_timestamp: fn() -> Timestamp,
}
pub trait Controller {
    fn new(config: Config, interface: HalInterface) -> Self;
}
