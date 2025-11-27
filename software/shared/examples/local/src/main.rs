use local::wrappers::core::get_next_message;
use serde::ser::SerializeStruct;
use shared::messages::messages::common::Message;
use shared::messages::messages::control_req::ControlReqMessage;
use shared::utils::percentage::Percentage;
use std::fs::File;
use std::io::BufWriter;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, UNIX_EPOCH};
use std::time::SystemTime;
use serde::{Deserialize, Serialize, Serializer};

struct MessageStat {
    message: Message,
    timestamp: std::time::SystemTime,
}

impl MessageStat {
    pub fn new(msg: Message) ->Self{
        MessageStat {
            message: msg,
            timestamp: std::time::SystemTime::now(),
        }
    }
}
impl Serialize for MessageStat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Start serializing a struct with a specific number of fields
        let mut state = serializer.serialize_struct("MessageStat", 2)?;

        // Serialize individual fields with custom logic
        let some_val = self.timestamp.duration_since(UNIX_EPOCH).expect("time is bad");

        state.serialize_field("message", &format!("{:?}",self.message))?;
        state.serialize_field("timestamp", &some_val)?; // Example: Serialize as string

        // End the serialization of the struct
        state.end()
    }
}

pub async fn write_can_to_log() {
    let log_file = File::create("./logs.txt").unwrap();
    let mut writer = BufWriter::new(log_file);
    loop {
        let msg = get_next_message().await;
        let msg = serde_json::to_string(&MessageStat::new(msg)).expect("bad_msg");

        writeln!(writer, "{}", msg).unwrap();
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
    // start thread for thread updater
    thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(local::wrappers::core::state_updater())
    }); 
    


    
    // spawn threads for MCU/FCU
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

    std::process::exit(0);
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
