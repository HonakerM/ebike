use embedded_can::StandardId;

use crate::{
    config::config::Config,
    messages::messages::Message,
    utils::time::{Duration, Timestamp},
};


pub trait Lockable {
    type Target: ?Sized;
    type Guard<'a>: core::ops::DerefMut<Target = Self::Target> + 'a
    where
        Self: 'a;

    async fn lock<'a>(&'a self) -> Self::Guard<'a>;
}
