use std::time::Duration;

use embedded_can::StandardId;

use crate::{config::config::Config, messages::messages::Message, utils::time::Timestamp};

pub struct HalInterface<F>
where
    F: std::future::Future,
{
    pub get_timestamp: fn() -> Timestamp,
    pub broadcast: fn(Message) -> F,
}
pub trait Controller<F>
where
    F: std::future::Future,
{
    fn new(config: Config, interface: HalInterface<F>) -> Self;
}
