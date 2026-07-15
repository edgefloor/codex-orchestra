//! PROTOTYPE ONLY: stdio host with a renderer-inaccessible inherited fd 3.

use codex_orchestra_host_prototype::{PrototypeHost, read_frame, write_frame};
use serde_json::{Value, json};
use std::fs::OpenOptions;
use std::io::{BufReader, BufWriter};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let instance = std::env::var("ORCHESTRA_PROTOTYPE_HOST_INSTANCE")
        .unwrap_or_else(|_| "prototype-host".into());
    eprintln!("orchestra-host-prototype instance={instance}");
    let mut host = PrototypeHost::new(instance);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut input = BufReader::new(stdin.lock());
    let mut output = BufWriter::new(stdout.lock());

    while let Some(request) = read_frame(&mut input)? {
        let request_id = request.get("id").cloned().unwrap_or(Value::Null);
        let action = host.handle(request);
        for message in action.data {
            write_frame(&mut output, &message)?;
        }
        if let Some(challenge) = action.control_challenge {
            let mut control = OpenOptions::new()
                .read(true)
                .write(true)
                .open("/dev/fd/3")?;
            write_frame(&mut control, &challenge)?;
            let response =
                read_frame(&mut control)?.unwrap_or_else(|| json!({"decision": "decline"}));
            write_frame(
                &mut output,
                &host.confirmation_response(request_id, response),
            )?;
        }
        if action.close {
            break;
        }
    }
    Ok(())
}
