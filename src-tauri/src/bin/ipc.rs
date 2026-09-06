use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tradex::{ControlPlane, protocol::EventSink};

fn main() -> io::Result<()> {
    let Some(path) = std::env::args_os().nth(1) else {
        eprintln!("Usage: tradex-ipc <isolated-workspace-directory>");
        std::process::exit(2);
    };
    let mut control = ControlPlane::new(PathBuf::from(path));
    let output = Arc::new(Mutex::new(io::stdout()));
    let event_output = output.clone();
    let sink: EventSink = Arc::new(move |event| {
        write_frame(&event_output, &json!({"kind":"event", "event":event})).is_ok()
    });
    let mut input = io::stdin().lock();
    let mut frame = Vec::new();
    let mut oversized = false;
    loop {
        let bytes = input.fill_buf()?;
        if bytes.is_empty() {
            break;
        }
        let newline = bytes.iter().position(|b| *b == b'\n');
        let length = newline.map_or(bytes.len(), |n| n + 1);
        if frame.len() + length > 65_536 {
            oversized = true;
        }
        if !oversized {
            frame.extend_from_slice(&bytes[..length]);
        }
        input.consume(length);
        if newline.is_some() {
            let request: Value = if oversized {
                Value::Null
            } else {
                serde_json::from_slice(&frame).unwrap_or(Value::Null)
            };
            let result = control.dispatch_with_events(request, "stdio", Some(sink.clone()));
            write_frame(&output, &json!({"kind":"result", "result":result}))?;
            frame.clear();
            oversized = false;
        }
    }
    Ok(())
}

fn write_frame(output: &Mutex<io::Stdout>, frame: &Value) -> io::Result<()> {
    let mut output = output
        .lock()
        .map_err(|_| io::Error::other("output unavailable"))?;
    serde_json::to_writer(&mut *output, frame)?;
    output.write_all(b"\n")?;
    output.flush()
}
