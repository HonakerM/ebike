use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use base64;
use shared::messages::messages::control_req::ControlReqMessage;
use shared::utils::percentage::Percentage;
use std::io::{BufRead, BufReader, Write};
use shared::messages::messages::common::Message;
use embedded_can::StandardId;

fn forward_debug(output: Arc<Mutex<BufReader<std::process::ChildStdout>>>) {
    loop {
        let line = {
            let mut reader = output.lock().unwrap();
            let mut buffer = String::new();
            if reader.read_line(&mut buffer).is_ok() {
                Some(buffer)
            } else {
                None
            }
        };

        if let Some(val) = line {
            let val = val.trim().to_string(); // Remove any trailing newlines
            println!("{}", val);
        }
    }
}

fn capture_can(output: Arc<Mutex<BufReader<std::process::ChildStderr>>>, can_messages: Arc<Mutex<Vec<Message>>>) {
    loop {
        let line = {
            let mut reader = output.lock().unwrap();
            let mut buffer = String::new();
            if reader.read_line(&mut buffer).is_ok() {
                Some(buffer)
            } else {
                None
            }
        };

        if let Some(val) = line {
            let val = val.trim().to_string(); // Remove any trailing newlines
            println!("{}", val);
            can_messages.lock().unwrap().push(val.into());
        }
    }
}

fn main() {
    let rust_binary = "./target/debug/examples/local.exe";

    let mut process = Command::new(rust_binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start Rust binary");

    let stdout = Arc::new(Mutex::new(BufReader::new(process.stdout.take().unwrap())));
    let stderr = Arc::new(Mutex::new(BufReader::new(process.stderr.take().unwrap())));
    let can_messages = Arc::new(Mutex::new(Vec::new()));

    let stdout_clone = Arc::clone(&stdout);
    let stderr_clone = Arc::clone(&stderr);
    let can_messages_clone = Arc::clone(&can_messages);

    thread::spawn(move || forward_debug(stdout_clone));
    thread::spawn(move || capture_can(stderr_clone, can_messages_clone));

    let stdin = process.stdin.as_mut().unwrap();
    let messages = vec![
        Message::ControlReqMessage(ControlReqMessage{ throttle_req: Percentage::from_fractional(0.5), brake_req: Percentage::zero()}),
        Message::ControlReqMessage(ControlReqMessage{ throttle_req: Percentage::full(), brake_req: Percentage::zero()}),
        Message::ControlReqMessage(ControlReqMessage{ throttle_req: Percentage::zero(), brake_req: Percentage::full()})
    ];

    for msg in messages {
        println!("Sending: {:?}", msg);
        writeln!(stdin, "{}", Into::<String>::into(msg)).unwrap();
        stdin.flush().unwrap();
        thread::sleep(Duration::from_secs(5));
    }

    let log_file = std::fs::File::create("./logs.txt").unwrap();
    let mut writer = std::io::BufWriter::new(log_file);
    for msg in can_messages.lock().unwrap().iter() {
        writeln!(writer, "{:?}", msg).unwrap();
    }

    process.kill().unwrap();
    process.wait().unwrap();
}