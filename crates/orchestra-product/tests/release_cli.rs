use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

#[cfg(target_os = "macos")]
use codex_orchestra_product::{ArtifactInput, build_manifest, read_pins, write_manifest};

fn run(binary: &Path, root: &Path, arguments: &[&str]) {
    let output = Command::new(binary)
        .args(arguments)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "command failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn update_cli_records_stage_activation_and_bounded_rollback() {
    let temporary = tempdir().unwrap();
    let codex_home = temporary.path().join("codex-home");
    let state = temporary.path().join("install-state.json");
    let initial = temporary.path().join("initial.json");
    let next = temporary.path().join("next.json");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let binary = Path::new(env!("CARGO_BIN_EXE_orchestra-product"));

    fs::write(
        &initial,
        serde_json::to_vec(&json!({
            "productVersion": "1.0.0",
            "manifestSha256": "aaaaaaaaaaaaaaaa",
            "snapshotSchema": "snapshot-v1",
            "projectionSchema": "projection-v1"
        }))
        .unwrap(),
    )
    .unwrap();
    fs::write(
        &next,
        serde_json::to_vec(&json!({
            "productVersion": "1.1.0",
            "manifestSha256": "bbbbbbbbbbbbbbbb",
            "snapshotSchema": "snapshot-v1",
            "projectionSchema": "projection-v2"
        }))
        .unwrap(),
    )
    .unwrap();

    run(
        binary,
        root,
        &[
            "update-init",
            "--codex-home",
            codex_home.to_str().unwrap(),
            "--state",
            state.to_str().unwrap(),
            "--release",
            initial.to_str().unwrap(),
        ],
    );
    run(
        binary,
        root,
        &[
            "update-transition",
            "--codex-home",
            codex_home.to_str().unwrap(),
            "--state",
            state.to_str().unwrap(),
            "--action",
            "stage",
            "--release",
            next.to_str().unwrap(),
            "--recorded-at",
            "stage",
        ],
    );
    for (action, recorded_at) in [("activate", "activate"), ("rollback", "rollback")] {
        run(
            binary,
            root,
            &[
                "update-transition",
                "--codex-home",
                codex_home.to_str().unwrap(),
                "--state",
                state.to_str().unwrap(),
                "--action",
                action,
                "--recorded-at",
                recorded_at,
            ],
        );
    }

    let recorded: serde_json::Value = serde_json::from_slice(&fs::read(state).unwrap()).unwrap();
    assert_eq!(recorded["active"]["productVersion"], "1.0.0");
    assert_eq!(recorded["automaticRollbacksUsed"], 1);
    assert_eq!(recorded["transitions"].as_array().unwrap().len(), 3);
    assert!(recorded["projectionGenerations"]["projection-v2"].is_string());
    assert!(!codex_home.join("orchestra/maintenance.lock").exists());
}

#[cfg(target_os = "macos")]
#[test]
fn desktop_lifecycle_retains_and_restores_the_predecessor_application() {
    let temporary = tempdir().unwrap();
    let codex_home = temporary.path().join("codex-home");
    let state = temporary.path().join("install-state.json");
    let app_bundle = temporary.path().join("Orchestra.app");
    let marker = app_bundle.join("Contents/version.txt");
    let current_artifact = temporary.path().join("current-artifact");
    let next_artifact = temporary.path().join("next-artifact");
    let current_manifest_path = temporary.path().join("current-manifest.json");
    let next_manifest_path = temporary.path().join("next-manifest.json");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let binary = Path::new(env!("CARGO_BIN_EXE_orchestra-product"));
    let pins = read_pins(root).unwrap();

    fs::create_dir_all(marker.parent().unwrap()).unwrap();
    fs::write(&marker, "current").unwrap();
    fs::write(&current_artifact, "current").unwrap();
    fs::write(&next_artifact, "next").unwrap();
    let current_manifest = build_manifest(
        pins.clone(),
        "aarch64-apple-darwin",
        &[ArtifactInput {
            name: "fixture".into(),
            path: current_artifact,
        }],
    )
    .unwrap();
    let next_manifest = build_manifest(
        pins,
        "aarch64-apple-darwin",
        &[ArtifactInput {
            name: "fixture".into(),
            path: next_artifact,
        }],
    )
    .unwrap();
    write_manifest(&current_manifest_path, &current_manifest).unwrap();
    write_manifest(&next_manifest_path, &next_manifest).unwrap();
    let snapshot = &next_manifest.schemas["snapshot"];
    let protocol = &next_manifest.schemas["protocol"];
    let projection = format!("{}:{}", protocol.identity, protocol.sha256);

    run(
        binary,
        root,
        &[
            "desktop-update-stage",
            "--codex-home",
            codex_home.to_str().unwrap(),
            "--state",
            state.to_str().unwrap(),
            "--manifest",
            current_manifest_path.to_str().unwrap(),
            "--app-bundle",
            app_bundle.to_str().unwrap(),
            "--next-version",
            &next_manifest.product_version,
            "--next-manifest-sha",
            &next_manifest.manifest_sha256,
            "--next-snapshot-schema",
            &snapshot.identity,
            "--next-projection-schema",
            &projection,
            "--recorded-at",
            "stage",
        ],
    );
    fs::write(&marker, "next").unwrap();
    run(
        binary,
        root,
        &[
            "desktop-startup-begin",
            "--codex-home",
            codex_home.to_str().unwrap(),
            "--state",
            state.to_str().unwrap(),
            "--manifest",
            next_manifest_path.to_str().unwrap(),
            "--policy-root",
            root.to_str().unwrap(),
            "--recorded-at",
            "activate",
        ],
    );
    run(
        binary,
        root,
        &[
            "desktop-startup-rollback",
            "--codex-home",
            codex_home.to_str().unwrap(),
            "--state",
            state.to_str().unwrap(),
            "--app-bundle",
            app_bundle.to_str().unwrap(),
            "--policy-root",
            root.to_str().unwrap(),
            "--recorded-at",
            "rollback",
        ],
    );

    assert_eq!(fs::read_to_string(&marker).unwrap(), "current");
    let recorded: serde_json::Value = serde_json::from_slice(&fs::read(state).unwrap()).unwrap();
    assert_eq!(
        recorded["active"]["manifestSha256"],
        current_manifest.manifest_sha256
    );
    assert_eq!(recorded["automaticRollbacksUsed"], 1);
}
