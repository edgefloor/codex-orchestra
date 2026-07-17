use codex_orchestra_product::release::{
    ProductInstallState, PublicationEvidence, ReleaseRecord, ReleaseSlot,
    acquire_maintenance_lease, build_release_record, read_install_state, read_release_evidence,
    read_release_policy, verify_publication, write_install_state, write_release_record,
};
use codex_orchestra_product::{
    ArtifactInput, ProductError, ReleaseManifest, build_manifest, read_pins,
    verify_manifest_artifact, verify_manifest_identity, verify_repository, write_manifest,
};
use serde::de::DeserializeOwned;
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
        "verify-manifest" => verify_manifest(&remaining),
        "verify-artifact" => verify_artifact(&remaining),
        "release-gate" => release_gate(&remaining),
        "publication-gate" => publication_gate(&remaining),
        "update-init" => update_init(&remaining),
        "update-transition" => update_transition(&remaining),
        "desktop-update-stage" => desktop_update_stage(&remaining),
        "desktop-update-abort" => desktop_update_abort(&remaining),
        "desktop-startup-begin" => desktop_startup_begin(&remaining),
        "desktop-startup-commit" => desktop_startup_commit(&remaining),
        "desktop-startup-rollback" => desktop_startup_rollback(&remaining),
        _ => {
            println!(
                "usage:\n  orchestra-product doctor [--root PATH]\n  orchestra-product manifest --target TRIPLE --output PATH [--root PATH] [--artifact NAME=PATH]...\n  orchestra-product verify-manifest --manifest PATH\n  orchestra-product verify-artifact --manifest PATH --name NAME --artifact PATH\n  orchestra-product host-smoke --host PATH [--host-arg ARG]... --manifest PATH\n  orchestra-product release-gate --candidate PATH --evidence PATH --output PATH [--root PATH]\n  orchestra-product publication-gate --candidate PATH --record PATH --evidence PATH\n  orchestra-product update-init --codex-home PATH --state PATH --release PATH\n  orchestra-product update-transition --codex-home PATH --state PATH --action stage|activate|commit|rollback|reverse --recorded-at VALUE [--release PATH] [--manifest-sha SHA] [--allow-schema-reverse]\n  orchestra-product desktop-update-stage --codex-home PATH --state PATH --manifest PATH --app-bundle PATH --next-version VERSION --next-manifest-sha SHA --next-snapshot-schema ID --next-projection-schema ID --recorded-at VALUE\n  orchestra-product desktop-update-abort --codex-home PATH --state PATH --recorded-at VALUE\n  orchestra-product desktop-startup-begin --codex-home PATH --state PATH --manifest PATH --recorded-at VALUE\n  orchestra-product desktop-startup-commit --codex-home PATH --state PATH --recorded-at VALUE\n  orchestra-product desktop-startup-rollback --codex-home PATH --state PATH --app-bundle PATH --recorded-at VALUE"
            );
            Ok(())
        }
    }
}

fn verify_artifact(args: &[String]) -> Result<(), ProductError> {
    let manifest: ReleaseManifest = read_json(&required_path(args, "--manifest")?)?;
    let name =
        option(args, "--name").ok_or_else(|| ProductError::Message("--name is required".into()))?;
    let artifact = required_path(args, "--artifact")?;
    verify_manifest_artifact(&manifest, name, &artifact)?;
    println!("OK Product artifact: {name}");
    Ok(())
}

fn verify_manifest(args: &[String]) -> Result<(), ProductError> {
    let manifest: ReleaseManifest = read_json(&required_path(args, "--manifest")?)?;
    verify_manifest_identity(&manifest)?;
    println!("OK Product manifest: {}", manifest.manifest_sha256);
    Ok(())
}

fn release_gate(args: &[String]) -> Result<(), ProductError> {
    let root = option(args, "--root")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let candidate = required_path(args, "--candidate")?;
    let evidence_path = required_path(args, "--evidence")?;
    let output = required_path(args, "--output")?;
    let policy = read_release_policy(&root)?;
    let evidence = read_release_evidence(&evidence_path)?;
    let record = build_release_record(&candidate, &policy, &evidence)?;
    write_release_record(&output, &record)?;
    println!("OK release candidate: {}", record.record_sha256);
    Ok(())
}

fn publication_gate(args: &[String]) -> Result<(), ProductError> {
    let candidate = required_path(args, "--candidate")?;
    let record: ReleaseRecord = read_json(&required_path(args, "--record")?)?;
    let evidence: PublicationEvidence = read_json(&required_path(args, "--evidence")?)?;
    verify_publication(&candidate, &record, &evidence)?;
    println!("OK publication gate: {}", record.record_sha256);
    Ok(())
}

fn update_init(args: &[String]) -> Result<(), ProductError> {
    let codex_home = required_path(args, "--codex-home")?;
    let state_path = required_path(args, "--state")?;
    let release: ReleaseSlot = read_json(&required_path(args, "--release")?)?;
    let _lease = acquire_maintenance_lease(&codex_home, "update-init", "orchestra-product")?;
    if state_path.exists() {
        return Err(ProductError::Message(format!(
            "install state already exists: {}",
            state_path.display()
        )));
    }
    write_install_state(&state_path, &ProductInstallState::new(release))?;
    println!(
        "OK initialized Product install state: {}",
        state_path.display()
    );
    Ok(())
}

fn update_transition(args: &[String]) -> Result<(), ProductError> {
    let codex_home = required_path(args, "--codex-home")?;
    let state_path = required_path(args, "--state")?;
    let action = option(args, "--action")
        .ok_or_else(|| ProductError::Message("--action is required".into()))?;
    let recorded_at = option(args, "--recorded-at")
        .ok_or_else(|| ProductError::Message("--recorded-at is required".into()))?;
    let policy_root = option(args, "--root")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let policy = read_release_policy(&policy_root)?;
    let _lease = acquire_maintenance_lease(&codex_home, action, "orchestra-product")?;
    let mut state = read_install_state(&state_path)?;
    match action {
        "stage" => {
            let release: ReleaseSlot = read_json(&required_path(args, "--release")?)?;
            state.stage(release, recorded_at)?;
        }
        "activate" => state.activate(policy.distribution.retained_predecessors, recorded_at)?,
        "commit" => state.commit_first_launch(recorded_at)?,
        "rollback" => state.rollback_failed_first_launch(
            policy.distribution.automatic_rollback_attempts,
            recorded_at,
        )?,
        "reverse" => {
            let manifest = option(args, "--manifest-sha")
                .ok_or_else(|| ProductError::Message("--manifest-sha is required".into()))?;
            state.reverse_to_retained(
                manifest,
                has_flag(args, "--allow-schema-reverse"),
                policy.distribution.retained_predecessors,
                recorded_at,
            )?;
        }
        other => {
            return Err(ProductError::Message(format!(
                "unknown update action `{other}`"
            )));
        }
    }
    write_install_state(&state_path, &state)?;
    println!(
        "OK Product update {action}: {}",
        state.active.manifest_sha256
    );
    Ok(())
}

fn desktop_update_stage(args: &[String]) -> Result<(), ProductError> {
    let codex_home = required_path(args, "--codex-home")?;
    let state_path = required_path(args, "--state")?;
    let current_manifest: ReleaseManifest = read_json(&required_path(args, "--manifest")?)?;
    let current = ReleaseSlot::from_manifest(&current_manifest)?;
    let next = ReleaseSlot {
        product_version: required_option(args, "--next-version")?,
        manifest_sha256: required_option(args, "--next-manifest-sha")?,
        snapshot_schema: required_option(args, "--next-snapshot-schema")?,
        projection_schema: required_option(args, "--next-projection-schema")?,
    };
    let app_bundle = required_path(args, "--app-bundle")?;
    let recorded_at = required_option(args, "--recorded-at")?;
    let _lease =
        acquire_maintenance_lease(&codex_home, "desktop-update-stage", "orchestra-product")?;
    let mut state = if state_path.exists() {
        read_install_state(&state_path)?
    } else {
        ProductInstallState::new(current.clone())
    };
    if state.active.manifest_sha256 != current.manifest_sha256 {
        return Err(ProductError::Message(
            "running application does not match the active Product release".into(),
        ));
    }
    retain_application_bundle(&state_path, &current, &app_bundle)?;
    state.stage(next, &recorded_at)?;
    write_install_state(&state_path, &state)?;
    println!("OK desktop Product update staged");
    Ok(())
}

fn desktop_update_abort(args: &[String]) -> Result<(), ProductError> {
    let codex_home = required_path(args, "--codex-home")?;
    let state_path = required_path(args, "--state")?;
    let recorded_at = required_option(args, "--recorded-at")?;
    let _lease =
        acquire_maintenance_lease(&codex_home, "desktop-update-abort", "orchestra-product")?;
    let mut state = read_install_state(&state_path)?;
    state.abort_staged(&recorded_at)?;
    write_install_state(&state_path, &state)?;
    prune_application_bundles(&state_path, &state)?;
    println!("OK desktop Product update stage aborted");
    Ok(())
}

fn desktop_startup_begin(args: &[String]) -> Result<(), ProductError> {
    let codex_home = required_path(args, "--codex-home")?;
    let state_path = required_path(args, "--state")?;
    let manifest: ReleaseManifest = read_json(&required_path(args, "--manifest")?)?;
    let current = ReleaseSlot::from_manifest(&manifest)?;
    let recorded_at = required_option(args, "--recorded-at")?;
    let policy_root = option(args, "--root")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let policy = read_release_policy(&policy_root)?;
    let _lease =
        acquire_maintenance_lease(&codex_home, "desktop-startup-begin", "orchestra-product")?;
    if !state_path.exists() {
        write_install_state(&state_path, &ProductInstallState::new(current))?;
        println!("initialized");
        return Ok(());
    }
    let mut state = read_install_state(&state_path)?;
    let phase = match state.pending.as_ref() {
        Some(pending)
            if pending.release.manifest_sha256 == current.manifest_sha256
                && matches!(
                    pending.phase,
                    codex_orchestra_product::release::UpdatePhase::Staged
                ) =>
        {
            state.activate(policy.distribution.retained_predecessors, &recorded_at)?;
            write_install_state(&state_path, &state)?;
            "first-launch-pending"
        }
        Some(pending)
            if pending.release.manifest_sha256 == current.manifest_sha256
                && matches!(
                    pending.phase,
                    codex_orchestra_product::release::UpdatePhase::FirstLaunchPending
                ) =>
        {
            "first-launch-pending"
        }
        Some(_) if state.active.manifest_sha256 == current.manifest_sha256 => "staged",
        None if state.active.manifest_sha256 == current.manifest_sha256 => "steady",
        _ => {
            return Err(ProductError::Message(
                "running application is not the active or staged Product release".into(),
            ));
        }
    };
    println!("{phase}");
    Ok(())
}

fn desktop_startup_commit(args: &[String]) -> Result<(), ProductError> {
    let codex_home = required_path(args, "--codex-home")?;
    let state_path = required_path(args, "--state")?;
    let recorded_at = required_option(args, "--recorded-at")?;
    let _lease =
        acquire_maintenance_lease(&codex_home, "desktop-startup-commit", "orchestra-product")?;
    let mut state = read_install_state(&state_path)?;
    state.commit_first_launch(&recorded_at)?;
    write_install_state(&state_path, &state)?;
    prune_application_bundles(&state_path, &state)?;
    println!("OK desktop Product first launch committed");
    Ok(())
}

fn desktop_startup_rollback(args: &[String]) -> Result<(), ProductError> {
    let codex_home = required_path(args, "--codex-home")?;
    let state_path = required_path(args, "--state")?;
    let app_bundle = required_path(args, "--app-bundle")?;
    let recorded_at = required_option(args, "--recorded-at")?;
    let policy_root = option(args, "--root")
        .map(PathBuf::from)
        .unwrap_or(env::current_dir()?);
    let policy = read_release_policy(&policy_root)?;
    let _lease =
        acquire_maintenance_lease(&codex_home, "desktop-startup-rollback", "orchestra-product")?;
    let original = read_install_state(&state_path)?;
    let predecessor = original
        .pending
        .as_ref()
        .ok_or_else(|| ProductError::Message("no first launch is pending".into()))?
        .predecessor_manifest_sha256
        .clone();
    let mut rolled_back = original.clone();
    rolled_back.rollback_failed_first_launch(
        policy.distribution.automatic_rollback_attempts,
        &recorded_at,
    )?;
    restore_application_bundle(&state_path, &predecessor, &app_bundle)?;
    if let Err(error) = write_install_state(&state_path, &rolled_back) {
        let _ = write_install_state(&state_path, &original);
        return Err(error);
    }
    prune_application_bundles(&state_path, &rolled_back)?;
    println!("OK desktop Product first launch rolled back");
    Ok(())
}

fn required_option(args: &[String], name: &str) -> Result<String, ProductError> {
    option(args, name)
        .map(str::to_owned)
        .ok_or_else(|| ProductError::Message(format!("{name} is required")))
}

fn application_store_root(state_path: &Path) -> Result<PathBuf, ProductError> {
    Ok(state_path
        .parent()
        .ok_or_else(|| ProductError::Message("Product install state has no parent".into()))?
        .join("predecessor-apps"))
}

fn validate_manifest_key(value: &str) -> Result<(), ProductError> {
    if value.len() < 12 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ProductError::Message(
            "Product manifest identity is not a bounded hexadecimal key".into(),
        ));
    }
    Ok(())
}

fn retained_application_path(
    state_path: &Path,
    manifest_sha256: &str,
) -> Result<PathBuf, ProductError> {
    validate_manifest_key(manifest_sha256)?;
    Ok(application_store_root(state_path)?
        .join(manifest_sha256)
        .join("Orchestra.app"))
}

fn retain_application_bundle(
    state_path: &Path,
    release: &ReleaseSlot,
    app_bundle: &Path,
) -> Result<(), ProductError> {
    if !app_bundle.is_dir() {
        return Err(ProductError::Message(format!(
            "running application bundle does not exist: {}",
            app_bundle.display()
        )));
    }
    let destination = retained_application_path(state_path, &release.manifest_sha256)?;
    if destination.is_dir() {
        return Ok(());
    }
    let slot_root = destination
        .parent()
        .ok_or_else(|| ProductError::Message("retained application path has no parent".into()))?;
    let temporary = slot_root.with_extension(format!("tmp-{}", std::process::id()));
    if temporary.exists() {
        fs::remove_dir_all(&temporary)?;
    }
    fs::create_dir_all(&temporary)?;
    copy_application_bundle(app_bundle, &temporary.join("Orchestra.app"))?;
    fs::write(
        temporary.join("release.json"),
        serde_json::to_vec_pretty(release)?,
    )?;
    fs::create_dir_all(application_store_root(state_path)?)?;
    fs::rename(&temporary, slot_root)?;
    Ok(())
}

fn copy_application_bundle(source: &Path, destination: &Path) -> Result<(), ProductError> {
    let status = Command::new("ditto")
        .arg(source)
        .arg(destination)
        .status()?;
    if !status.success() {
        return Err(ProductError::Message(format!(
            "ditto failed to copy {} to {}",
            source.display(),
            destination.display()
        )));
    }
    Ok(())
}

fn restore_application_bundle(
    state_path: &Path,
    manifest_sha256: &str,
    current: &Path,
) -> Result<(), ProductError> {
    let retained = retained_application_path(state_path, manifest_sha256)?;
    if !retained.is_dir() {
        return Err(ProductError::Message(format!(
            "retained predecessor application is missing: {}",
            retained.display()
        )));
    }
    let parent = current
        .parent()
        .ok_or_else(|| ProductError::Message("application bundle has no parent".into()))?;
    let restored = parent.join(format!(".Orchestra.rollback-{}", std::process::id()));
    let failed = parent.join(format!(".Orchestra.failed-{}", std::process::id()));
    if restored.exists() {
        fs::remove_dir_all(&restored)?;
    }
    if failed.exists() {
        fs::remove_dir_all(&failed)?;
    }
    copy_application_bundle(&retained, &restored)?;
    fs::rename(current, &failed)?;
    if let Err(error) = fs::rename(&restored, current) {
        let _ = fs::rename(&failed, current);
        return Err(error.into());
    }
    fs::remove_dir_all(failed)?;
    Ok(())
}

fn prune_application_bundles(
    state_path: &Path,
    state: &ProductInstallState,
) -> Result<(), ProductError> {
    let root = application_store_root(state_path)?;
    if !root.is_dir() {
        return Ok(());
    }
    let retained = state
        .retained_predecessors
        .iter()
        .map(|release| release.manifest_sha256.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for entry in fs::read_dir(&root)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !retained.contains(name.as_ref()) {
            fs::remove_dir_all(entry.path())?;
        }
    }
    Ok(())
}

fn required_path(args: &[String], name: &str) -> Result<PathBuf, ProductError> {
    option(args, name)
        .map(PathBuf::from)
        .ok_or_else(|| ProductError::Message(format!("{name} is required")))
}

fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, ProductError> {
    Ok(serde_json::from_slice(&fs::read(path).map_err(
        |error| ProductError::Message(format!("failed to read {}: {error}", path.display())),
    )?)?)
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

fn has_flag(args: &[String], name: &str) -> bool {
    args.iter().any(|argument| argument == name)
}
