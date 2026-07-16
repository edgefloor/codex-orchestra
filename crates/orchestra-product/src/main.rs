use codex_orchestra_product::{
    ArtifactInput, ProductError, build_manifest, read_pins, verify_repository, write_manifest,
};
use std::env;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::time::Instant;

const MAX_FRAME_BYTES: usize = 64 * 1024;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ProductError> {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "help".into());
    let remaining = args.collect::<Vec<_>>();
    match command.as_str() {
        "doctor" => {
            let root = option(&remaining, "--root")
                .map(PathBuf::from)
                .unwrap_or(env::current_dir()?);
            let pins = read_pins(&root)?;
            verify_repository(&root, &pins)?;
            println!(
                "OK Product pins: Codex {}, T3Code {}",
                pins.sources["codex"], pins.sources["t3code"]
            );
            Ok(())
        }
        "manifest" => manifest(&remaining),
        "host-smoke" => host_smoke(&remaining),
        _ => {
            println!(
                "usage:\n  orchestra-product doctor [--root PATH]\n  orchestra-product manifest --target TRIPLE --output PATH [--root PATH] [--artifact NAME=PATH]...\n  orchestra-product host-smoke --host PATH [--host-arg ARG]... --manifest PATH"
            );
            Ok(())
        }
    }
}

fn host_smoke(args: &[String]) -> Result<(), ProductError> {
    let host = option(args, "--host")
        .map(PathBuf::from)
        .ok_or_else(|| ProductError::Message("--host is required".into()))?;
    let manifest = option(args, "--manifest")
        .map(PathBuf::from)
        .ok_or_else(|| ProductError::Message("--manifest is required".into()))?;
    let host_args = multi_option(args, "--host-arg");
    let expected: serde_json::Value = serde_json::from_slice(&fs::read(&manifest)?)?;
    let expected_sha = expected
        .get("manifestSha256")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| ProductError::Message("manifest has no manifestSha256".into()))?;
    let codex_home = env::temp_dir().join(format!("orchestra-host-smoke-{}", std::process::id()));
    fs::create_dir_all(&codex_home)?;

    let mut child = Command::new(&host)
        .args(host_args)
        .args(["--listen", "framed-stdio://"])
        .env("CODEX_HOME", &codex_home)
        .env("ORCHESTRA_RELEASE_MANIFEST", &manifest)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| {
            ProductError::Message(format!("failed to start {}: {error}", host.display()))
        })?;

    let request = serde_json::json!({
        "id": 1,
        "method": "initialize",
        "params": {
            "clientInfo": {
                "name": "orchestra-product-smoke",
                "title": "Orchestra Product Smoke",
                "version": env!("CARGO_PKG_VERSION")
            },
            "capabilities": { "experimentalApi": true }
        }
    });
    let body = serde_json::to_vec(&request)?;
    write_frame(
        child
            .stdin
            .as_mut()
            .ok_or_else(|| ProductError::Message("host stdin unavailable".into()))?,
        &body,
    )?;

    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| ProductError::Message("host stdout unavailable".into()))?;
    let expected_sha_owned = expected_sha.to_owned();
    let (response_tx, response_rx) = mpsc::channel();
    thread::spawn(move || {
        let result = read_initialize_response(&mut stdout, &expected_sha_owned);
        let initialized = result.is_ok();
        let _ = response_tx.send(result);

        // Keep ownership of stdout until the host exits. Codex may emit a final
        // notification after initialize; dropping the read end immediately makes
        // that normal shutdown path look like a protocol write failure.
        if initialized {
            let mut discard = [0_u8; 4096];
            while stdout.read(&mut discard).is_ok_and(|read| read != 0) {}
        }
    });
    let result = match response_rx.recv_timeout(Duration::from_secs(30)) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(ProductError::Message(
            "host initialize timed out after 30 seconds".into(),
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(ProductError::Message(
            "host initialize reader stopped without a result".into(),
        )),
    };

    drop(child.stdin.take());
    if result.is_err() {
        let _ = child.kill();
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(ProductError::Message(
                "host did not exit within 10 seconds after stdin closed".into(),
            ));
        }
        thread::sleep(Duration::from_millis(25));
    };
    let _ = fs::remove_dir_all(codex_home);
    result?;
    if !status.success() {
        return Err(ProductError::Message(format!(
            "host exited unsuccessfully: {status}"
        )));
    }
    println!("OK framed host handshake: {expected_sha}");
    Ok(())
}

fn read_initialize_response(
    reader: &mut impl Read,
    expected_sha: &str,
) -> Result<(), ProductError> {
    for _ in 0..32 {
        let response = read_frame(reader)?;
        if response.get("id") != Some(&serde_json::json!(1)) {
            continue;
        }
        let actual_sha = response
            .pointer("/result/orchestraProduct/manifestSha256")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                ProductError::Message(format!(
                    "host initialize omitted the Product tuple: {response}"
                ))
            })?;
        if actual_sha != expected_sha {
            return Err(ProductError::Message(format!(
                "host Product tuple mismatch: expected {expected_sha}, got {actual_sha}"
            )));
        }
        return Ok(());
    }
    Err(ProductError::Message(
        "host did not return initialize within 32 frames".into(),
    ))
}

fn write_frame(writer: &mut impl Write, body: &[u8]) -> Result<(), ProductError> {
    if body.is_empty() || body.len() > MAX_FRAME_BYTES {
        return Err(ProductError::Message(format!(
            "outbound frame length {} is outside 1..={MAX_FRAME_BYTES}",
            body.len()
        )));
    }
    writer.write_all(&(body.len() as u32).to_be_bytes())?;
    writer.write_all(body)?;
    writer.flush()?;
    Ok(())
}

fn read_frame(reader: &mut impl Read) -> Result<serde_json::Value, ProductError> {
    let mut length = [0_u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_be_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME_BYTES {
        return Err(ProductError::Message(format!(
            "inbound frame length {length} is outside 1..={MAX_FRAME_BYTES}"
        )));
    }
    let mut body = vec![0_u8; length];
    reader.read_exact(&mut body)?;
    Ok(serde_json::from_slice(&body)?)
}

fn manifest(args: &[String]) -> Result<(), ProductError> {
    let root = option(args, "--root")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let target = option(args, "--target")
        .ok_or_else(|| ProductError::Message("--target is required".into()))?;
    let output = option(args, "--output")
        .map(PathBuf::from)
        .ok_or_else(|| ProductError::Message("--output is required".into()))?;
    let artifacts = multi_option(args, "--artifact")
        .into_iter()
        .map(|value| parse_artifact(&root, value))
        .collect::<Result<Vec<_>, _>>()?;
    let pins = read_pins(&root)?;
    verify_repository(&root, &pins)?;
    let manifest = build_manifest(pins, target.to_owned(), &artifacts)?;
    write_manifest(&output, &manifest)?;
    println!("{}  {}", manifest.manifest_sha256, output.display());
    Ok(())
}

fn parse_artifact(root: &Path, value: &str) -> Result<ArtifactInput, ProductError> {
    let (name, path) = value
        .split_once('=')
        .ok_or_else(|| ProductError::Message("--artifact must use NAME=PATH syntax".into()))?;
    let path = PathBuf::from(path);
    Ok(ArtifactInput {
        name: name.into(),
        path: if path.is_absolute() {
            path
        } else {
            root.join(path)
        },
    })
}

fn option<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

fn multi_option<'a>(args: &'a [String], name: &str) -> Vec<&'a str> {
    args.windows(2)
        .filter(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
        .collect()
}
