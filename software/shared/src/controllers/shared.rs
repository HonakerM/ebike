use std::time::Duration;

use crate::{config::config::Config, messages::messages::Message, utils::time::Timestamp};

pub struct HalInterface<F>
where
    F: std::future::Future,
{
    pub get_timestamp: fn() -> Timestamp,
    pub sleep: fn(Duration) -> F,
}
pub trait Controller<F>
where
    F: std::future::Future,
{
    fn new(config: Config, interface: HalInterface<F>) -> Self;
}
