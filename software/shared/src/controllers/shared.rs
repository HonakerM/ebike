use embedded_can::StandardId;

use crate::{
    config::config::Config,
    messages::messages::Message,
    utils::time::{Duration, Timestamp},
};

pub struct HalInterface<MF, EF, SF>
where
    MF: core::future::Future<Output = Message>,
    EF: core::future::Future<Output = ()>,
    SF: core::future::Future<Output = ()>,
{
    pub get_timestamp: fn() -> Timestamp,
    pub get_can_message: fn() -> MF,
    pub broadcast_can_message: fn(Message) -> EF,
    pub sleep: fn(Duration) -> SF,
}


pub trait HalInterfaceType {
    // async fn get_can_message() -> Message
    type GetCanMessageFuture<'a>: Future<Output = Message> + 'a
    where
        Self: 'a;

    // async fn broadcast_can_message(msg)
    type BroadcastFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    // async fn sleep()
    type SleepFuture<'a>: Future<Output = ()> + 'a
    where
        Self: 'a;

    /// sync function is fine
    fn get_timestamp(&self) -> Timestamp;

    /// async functions expressed using GATs
    fn get_can_message(&self) -> Self::GetCanMessageFuture<'_>;
    fn broadcast_can_message(&self, msg: Message) -> Self::BroadcastFuture<'_>;
    fn sleep(&self, duration: Duration) -> Self::SleepFuture<'_>;
}
pub trait ControllerRunner<MF, EF, SF>
where
    MF: core::future::Future<Output = Message>,
    EF: core::future::Future<Output = ()>,
    SF: core::future::Future<Output = ()>,
{
    fn new(config: Config, interface: HalInterface<MF, EF, SF>) -> Self;
}

pub trait Lockable {
    type Target: ?Sized;
    type Guard<'a>: core::ops::DerefMut<Target = Self::Target> + 'a
    where
        Self: 'a;

    async fn lock<'a>(&'a self) -> Self::Guard<'a>;
}
