use std::time::Duration;

use crate::messages::messages::Message;


pub trait Executor {
}

pub trait Controller<Config, Req, Res> {
    fn new() -> Self;
    async fn run_tick(&mut self);
    async fn run_event(&mut self, message: Message);
}