use base64;
use embedded_can::StandardId;
use shared::messages::messages::common::Message;
use shared::messages::messages::control_req::ControlReqMessage;
use shared::utils::percentage::Percentage;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[path = "./local.rs"]
pub mod local;

fn main() {
    let (config, snd) = local::setup();

    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(local::run(config))
    });
    let messages = vec![
        Message::ControlReqMessage(ControlReqMessage {
            throttle_req: Percentage::from_fractional(0.5),
            brake_req: Percentage::zero(),
        }),
        Message::ControlReqMessage(ControlReqMessage {
            throttle_req: Percentage::full(),
            brake_req: Percentage::zero(),
        }),
        Message::ControlReqMessage(ControlReqMessage {
            throttle_req: Percentage::zero(),
            brake_req: Percentage::full(),
        }),
    ];

    for msg in messages {
        println!("Sending: {:?}", msg);
        snd.send(msg);
        thread::sleep(Duration::from_secs(5));
    }

    // let log_file = std::fs::File::create("./logs.txt").unwrap();
    // let mut writer = std::io::BufWriter::new(log_file);
    // for msg in can_messages.lock().unwrap().iter() {
    //     writeln!(writer, "{:?}", msg).unwrap();
    // }

    // process.kill().unwrap();
    // process.wait().unwrap();
}
