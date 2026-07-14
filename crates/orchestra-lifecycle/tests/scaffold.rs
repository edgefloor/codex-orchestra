use codex_orchestra_lifecycle::{
    DesiredFile, desired_project, init_project_state, install, plugin_layout_matches_manifest,
    rollback, uninstall,
};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap()
        .to_path_buf()
}

fn manifest() -> Value {
    serde_json::from_slice(&fs::read(root().join(".codex-plugin/plugin.json")).unwrap()).unwrap()
}

#[test]
fn manifest_describes_native_v2_surface_without_external_runtime() {
    let manifest = manifest();
    assert_eq!(manifest["name"], "orchestra");
    assert!(
        manifest["version"]
            .as_str()
            .unwrap()
            .starts_with("0.2.0+codex.f90e7dee")
    );
    assert_eq!(manifest["skills"], "./skills/");
    for forbidden in ["mcpServers", "apps", "hooks"] {
        assert!(manifest.get(forbidden).is_none());
    }
    assert!(
        manifest["interface"]["longDescription"]
            .as_str()
            .unwrap()
            .contains("native V2")
    );
}

#[test]
fn legacy_python_yaml_runtime_and_fixed_roles_are_removed() {
    let root = root();
    for path in [
        "scripts/lifecycle.py",
        "scripts/workflow.py",
        "tests/test_scaffold.py",
        "pyproject.toml",
        "uv.lock",
        "assets/templates/WORKFLOW.yaml",
        "assets/schemas/workflow.schema.json",
        "evals/workflows/native-vertical-slice.yaml",
    ] {
        assert!(!root.join(path).exists(), "{path}");
    }
    let agents = match fs::read_dir(root.join("config/agents")) {
        Ok(entries) => entries
            .flatten()
            .filter(|entry| {
                entry.path().extension().and_then(|value| value.to_str()) == Some("toml")
            })
            .count(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0,
        Err(error) => panic!("failed to inspect config/agents: {error}"),
    };
    assert_eq!(agents, 0);
    assert!(
        root.join("evals/workflows/native-vertical-slice.workflow.ts")
            .is_file()
    );
}

#[test]
fn rust_runtime_lifecycle_and_restricted_sdk_are_present() {
    let root = root();
    let cargo: toml::Table = fs::read_to_string(root.join("Cargo.toml"))
        .unwrap()
        .parse()
        .unwrap();
    let members = cargo["workspace"]["members"].as_array().unwrap();
    assert!(
        members
            .iter()
            .any(|item| item.as_str() == Some("crates/orchestra-core"))
    );
    assert!(
        members
            .iter()
            .any(|item| item.as_str() == Some("crates/orchestra-lifecycle"))
    );
    let sdk = fs::read_to_string(root.join("sdk/index.d.ts")).unwrap();
    for call in [
        "agent", "parallel", "pipeline", "check", "approval", "worktree", "repeat",
    ] {
        assert!(sdk.contains(&format!("function {call}")));
    }
    let compiler = fs::read_to_string(root.join("crates/orchestra-core/src/compiler.rs")).unwrap();
    for rejected in [
        "side effects or trailing statements",
        "non-Orchestra identifier",
        "import('./x')",
    ] {
        assert!(compiler.contains(rejected));
    }
}

#[test]
fn pinned_integration_is_explicit_and_minimal() {
    let root = root();
    assert_eq!(
        fs::read_to_string(root.join("integration/codex/UPSTREAM_REVISION"))
            .unwrap()
            .trim(),
        "f90e7deea6a715bbd153044af6f475eefa749177"
    );
    let patch = fs::read_to_string(root.join("integration/codex/codex-f90e7dee.patch")).unwrap();
    assert!(patch.contains("codex_orchestra_extension::install"));
    assert!(patch.contains("orchestra_control"));
    let adapter =
        fs::read_to_string(root.join("integration/codex/overlay/codex-rs/core/src/orchestra.rs"))
            .unwrap();
    assert!(adapter.contains("AgentControl"));
    assert!(adapter.contains("process_exec_tool_call"));
    for forbidden in ["reqwest", "mcp", "codex exec", "Command::new(\"codex\")"] {
        assert!(!adapter.contains(forbidden));
    }
}

#[test]
fn five_native_tools_are_registered() {
    let source = fs::read_to_string(
        root().join("integration/codex/overlay/codex-rs/ext/orchestra/src/tool.rs"),
    )
    .unwrap();
    for name in [
        "orchestra_validate",
        "orchestra_run",
        "orchestra_resume",
        "orchestra_status",
        "orchestra_cancel",
    ] {
        assert!(source.contains(name));
    }
}

#[test]
fn skills_delegate_to_runtime_not_model_scheduling() {
    let mut combined = String::new();
    for entry in fs::read_dir(root().join("skills")).unwrap().flatten() {
        let path = entry.path().join("SKILL.md");
        if path.is_file() {
            combined.push_str(&fs::read_to_string(path).unwrap());
        }
    }
    for name in [
        "orchestra_validate",
        "orchestra_run",
        "orchestra_resume",
        "orchestra_status",
        "orchestra_cancel",
    ] {
        assert!(combined.contains(name));
    }
    assert!(!combined.contains("workflow.yaml"));
    assert!(!combined.contains("active Codex agent is the executor"));
}

#[test]
fn configuration_parses_and_disables_recursive_children() {
    for entry in fs::read_dir(root().join("config")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }
        let config: toml::Table = fs::read_to_string(path).unwrap().parse().unwrap();
        assert_eq!(config["features"]["multi_agent"].as_bool(), Some(true));
        assert_eq!(config["agents"]["max_depth"].as_integer(), Some(1));
    }
}

#[test]
fn project_install_is_preview_first_and_cache_immutable() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("repo");
    fs::create_dir(&target).unwrap();
    let state = target.join(".codex/orchestra/install-state.json");
    let files = desired_project(&root(), &target);
    assert_eq!(
        install(&files, &target, &state, "test", false, false).unwrap(),
        0
    );
    assert!(!target.join(".codex").exists());
    init_project_state(&target, true).unwrap();
    assert_eq!(
        install(&files, &target, &state, "test", true, false).unwrap(),
        0
    );
    assert!(target.join(".codex/config.toml").is_file());
    assert!(target.join(".codex/orchestra/runs").is_dir());
    assert!(!target.join(".codex/orchestra/workflows").exists());
}

#[test]
fn conflicts_are_refused_and_uninstall_preserves_runs() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("repo");
    fs::create_dir_all(target.join(".codex")).unwrap();
    let config = target.join(".codex/config.toml");
    fs::write(&config, "model = \"user-owned\"\n").unwrap();
    let state = target.join(".codex/orchestra/install-state.json");
    let files = desired_project(&root(), &target);
    assert_eq!(
        install(&files, &target, &state, "test", true, false).unwrap(),
        2
    );
    assert_eq!(
        fs::read_to_string(&config).unwrap(),
        "model = \"user-owned\"\n"
    );

    fs::remove_file(&config).unwrap();
    assert_eq!(
        install(&files, &target, &state, "test", true, false).unwrap(),
        0
    );
    let artifact = target.join(".codex/orchestra/runs/r1/summary.md");
    fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    fs::write(&artifact, "keep\n").unwrap();
    assert_eq!(uninstall(&target, true).unwrap(), 0);
    assert!(artifact.exists());
}

#[test]
fn identical_preexisting_configuration_remains_user_owned() {
    let temp = tempdir().unwrap();
    let target = temp.path().join("repo");
    fs::create_dir_all(target.join(".codex")).unwrap();
    let config = target.join(".codex/config.toml");
    fs::copy(root().join("config/project.toml"), &config).unwrap();
    let state = target.join(".codex/orchestra/install-state.json");
    let files = desired_project(&root(), &target);
    assert_eq!(
        install(&files, &target, &state, "test", true, false).unwrap(),
        0
    );
    assert_eq!(uninstall(&target, true).unwrap(), 0);
    assert!(config.exists());
}

#[test]
fn upgrade_is_reversible_and_rollback_refuses_post_upgrade_edits() {
    let temp = tempdir().unwrap();
    let plugin = temp.path().join("plugin");
    let target = temp.path().join("repo");
    fs::create_dir_all(&plugin).unwrap();
    fs::create_dir_all(&target).unwrap();
    let source = plugin.join("config.toml");
    let destination = target.join(".codex/config.toml");
    let state = target.join(".codex/orchestra/install-state.json");
    fs::write(&source, "value = 1\n").unwrap();
    let files = vec![DesiredFile {
        source: source.clone(),
        target: destination.clone(),
    }];
    install(&files, &target, &state, "one", true, false).unwrap();
    fs::write(&source, "value = 2\n").unwrap();
    assert_eq!(
        install(&files, &target, &state, "two", true, true).unwrap(),
        0
    );
    assert_eq!(fs::read_to_string(&destination).unwrap(), "value = 2\n");
    assert_eq!(rollback(&target, true).unwrap(), 0);
    assert_eq!(fs::read_to_string(&destination).unwrap(), "value = 1\n");

    fs::write(&source, "value = 3\n").unwrap();
    install(&files, &target, &state, "three", true, true).unwrap();
    fs::write(&destination, "user edit\n").unwrap();
    assert_eq!(rollback(&target, true).unwrap(), 2);
    assert_eq!(fs::read_to_string(destination).unwrap(), "user edit\n");
}

#[test]
fn source_and_versioned_cache_layouts_are_valid() {
    let manifest = manifest();
    let name = manifest["name"].as_str().unwrap();
    let version = manifest["version"].as_str().unwrap();
    let temp = tempdir().unwrap();
    let source = temp.path().join(name);
    fs::create_dir(&source).unwrap();
    assert!(plugin_layout_matches_manifest(&source, name, version));
    let cache = PathBuf::from("/tmp").join(name).join(version);
    assert!(plugin_layout_matches_manifest(&cache, name, version));
}

#[test]
fn mutable_run_state_is_not_tracked() {
    let output = Command::new("git")
        .args(["ls-files", "--", ".codex/orchestra"])
        .current_dir(root())
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
}
