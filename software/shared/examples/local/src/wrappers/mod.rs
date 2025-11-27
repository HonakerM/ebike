#[path = "./mcu.rs"]
pub mod mcu;

#[path = "./fcu.rs"]
pub mod fcu;

#[path = "./core.rs"]
pub mod core;

pub use core::setup;
pub use fcu::LocalFcuRunner;
pub use mcu::LocalMcuRunner;
