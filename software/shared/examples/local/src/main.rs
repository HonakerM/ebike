use local::wrappers::core::get_next_message;
use shared::messages::messages::common::Message;
use shared::messages::messages::control_req::ControlReqMessage;
use shared::utils::percentage::Percentage;
use std::fs::File;
use std::io::BufWriter;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;


pub async fn write_can_to_log() {
    let log_file = File::create("./logs.txt").unwrap();
    let mut writer = BufWriter::new(log_file);
    loop {
        let msg = get_next_message().await;
        writeln!(writer, "{:?}", msg).unwrap();
        writer.flush();
    }
}

fn main() {
    let (config, snd) = local::wrappers::setup();

    // Start a thread to save messages to a local text file
    thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(write_can_to_log())
    }); 
    
    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(local::wrappers::LocalMcuRunner::run(config))
    });   
    thread::spawn(move || {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(local::wrappers::LocalFcuRunner::run(config))
    });    
    local::ui::run().unwrap();


    /*
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
        if let Err(e) = snd.send(msg) {
            eprintln!("Failed to send message: {:?}", e);
        }
        thread::sleep(Duration::from_secs(5));
    } */
   loop {}
}
