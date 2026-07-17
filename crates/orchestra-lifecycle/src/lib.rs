use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LifecycleError {
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DesiredFile {
    pub source: PathBuf,
    pub target: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct InstallState {
    #[serde(default = "state_version")]
    pub version: u32,
    #[serde(default)]
    pub plugin_version: String,
    #[serde(default)]
    pub managed: BTreeMap<String, ManagedFile>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_recovery: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub latest_recovery_created: Vec<String>,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ManagedFile {
    pub sha256: String,
}

fn state_version() -> u32 {
    1
}

pub fn plugin_root() -> PathBuf {
    if let Some(root) = env::var_os("ORCHESTRA_PLUGIN_ROOT").map(PathBuf::from) {
        return root;
    }
    if let Ok(current) = env::current_dir()
        && let Some(root) = find_plugin_root(&current)
    {
        return root;
    }
    if let Ok(executable) = env::current_exe()
        && let Some(root) = executable.parent().and_then(find_plugin_root)
    {
        return root;
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("lifecycle crate must be inside the plugin repository")
        .to_path_buf()
}

fn find_plugin_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|candidate| candidate.join(".codex-plugin/plugin.json").is_file())
        .map(Path::to_path_buf)
}

pub fn desired_project(plugin: &Path, target: &Path) -> Vec<DesiredFile> {
    let mut files = vec![DesiredFile {
        source: plugin.join("config/project.toml"),
        target: target.join(".codex/config.toml"),
    }];
    files.extend(agent_files(
        &plugin.join("config/agents"),
        &target.join(".codex/agents"),
    ));
    files
}

pub fn desired_global(plugin: &Path, home: &Path, default: bool) -> Vec<DesiredFile> {
    let config = if default {
        "config.toml"
    } else {
        "orchestra.config.toml"
    };
    let mut files = vec![DesiredFile {
        source: plugin.join("config/orchestra.config.toml"),
        target: home.join(config),
    }];
    files.extend(agent_files(
        &plugin.join("config/agents"),
        &home.join("agents"),
    ));
    files
}

fn agent_files(source: &Path, target: &Path) -> Vec<DesiredFile> {
    let Ok(entries) = fs::read_dir(source) else {
        return Vec::new();
    };
    let mut files = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("toml"))
        .map(|path| DesiredFile {
            target: target.join(path.file_name().expect("agent file has a name")),
            source: path,
        })
        .collect::<Vec<_>>();
    files.sort_by(|left, right| left.source.cmp(&right.source));
    files
}

pub fn install(
    files: &[DesiredFile],
    root: &Path,
    state_path: &Path,
    plugin_version: &str,
    apply: bool,
    upgrade: bool,
) -> Result<i32, LifecycleError> {
    let mut state = read_state(state_path)?;
    let mut actions = Vec::new();
    let mut conflicts = Vec::new();
    for item in files {
        let key = relative(&item.target, root)?;
        if !item.target.exists() {
            actions.push(format!("CREATE {key}"));
        } else if digest(&item.target)? == digest(&item.source)? {
            actions.push(format!("KEEP   {key} (already current)"));
        } else if upgrade
            && state
                .managed
                .get(&key)
                .is_some_and(|managed| digest(&item.target).ok().as_ref() == Some(&managed.sha256))
        {
            actions.push(format!("UPDATE {key}"));
        } else {
            conflicts.push(format!("CONFLICT {key} (existing or locally modified)"));
        }
    }
    print_lines(actions.iter().chain(conflicts.iter()));
    if !conflicts.is_empty() {
        eprintln!("No changes applied: reconcile conflicts and preview again.");
        return Ok(2);
    }
    if !apply {
        println!("Preview only; rerun with --apply to make these changes.");
        return Ok(0);
    }

    let updates = files
        .iter()
        .filter(|item| {
            item.target.exists() && digest(&item.target).ok() != digest(&item.source).ok()
        })
        .collect::<Vec<_>>();
    let created = files
        .iter()
        .filter(|item| upgrade && !item.target.exists())
        .map(|item| relative(&item.target, root))
        .collect::<Result<Vec<_>, _>>()?;
    if !updates.is_empty() || !created.is_empty() {
        let recovery = state_path
            .parent()
            .ok_or_else(|| LifecycleError::Message("state path has no parent".into()))?
            .join("recovery")
            .join(format!("install-{}", timestamp()));
        for item in updates {
            let backup = recovery.join(relative(&item.target, root)?);
            copy_file(&item.target, &backup)?;
        }
        state.latest_recovery = Some(relative(&recovery, root)?);
        state.latest_recovery_created = created;
    }

    for item in files {
        let key = relative(&item.target, root)?;
        let was_managed = state.managed.contains_key(&key);
        let changed = !item.target.exists() || digest(&item.target)? != digest(&item.source)?;
        if changed {
            copy_file(&item.source, &item.target)?;
        }
        if changed || was_managed {
            state.managed.insert(
                key,
                ManagedFile {
                    sha256: digest(&item.target)?,
                },
            );
        }
    }
    state.version = 1;
    state.plugin_version = plugin_version.into();
    state.updated_at = timestamp();
    write_state(state_path, &state)?;
    println!("Applied {} managed files.", files.len());
    Ok(0)
}

pub fn init_project_state(target: &Path, apply: bool) -> Result<(), LifecycleError> {
    let runs = target.join(".codex/orchestra/runs");
    println!(
        "{} .codex/orchestra/runs/",
        if runs.is_dir() { "KEEP  " } else { "CREATE" }
    );
    if apply {
        fs::create_dir_all(runs)?;
    }
    Ok(())
}

pub fn uninstall(target: &Path, apply: bool) -> Result<i32, LifecycleError> {
    let state_path = target.join(".codex/orchestra/install-state.json");
    let mut state = read_state(&state_path)?;
    let mut removable = Vec::new();
    for (key, record) in &state.managed {
        let path = target.join(key);
        if !path.exists() {
            continue;
        }
        if digest(&path)? != record.sha256 {
            println!("PRESERVE {key} (locally modified)");
        } else {
            println!("REMOVE {key}");
            removable.push((key.clone(), path));
        }
    }
    println!("PRESERVE .codex/orchestra/runs/ runtime-owned run artifacts");
    if !apply {
        println!("Preview only; rerun with --apply to make these changes.");
        return Ok(0);
    }
    for (key, path) in removable {
        fs::remove_file(path)?;
        state.managed.remove(&key);
    }
    state.updated_at = timestamp();
    write_state(&state_path, &state)?;
    Ok(0)
}

pub fn rollback(target: &Path, apply: bool) -> Result<i32, LifecycleError> {
    let state_path = target.join(".codex/orchestra/install-state.json");
    let mut state = read_state(&state_path)?;
    let recovery_ref = state.latest_recovery.clone().ok_or_else(|| {
        LifecycleError::Message("no upgrade recovery snapshot is recorded".into())
    })?;
    let recovery = target.join(recovery_ref);
    if !recovery.is_dir() {
        return Err(LifecycleError::Message(format!(
            "missing recovery snapshot: {}",
            recovery.display()
        )));
    }
    let mut restores = Vec::new();
    collect_files(&recovery, &mut restores)?;
    restores.sort();
    let mut conflicts = Vec::new();
    for backup in &restores {
        let key = relative(backup, &recovery)?;
        let destination = target.join(&key);
        if destination.exists()
            && state
                .managed
                .get(&key)
                .is_some_and(|managed| digest(&destination).ok().as_ref() != Some(&managed.sha256))
        {
            conflicts.push(format!("PRESERVE {key} (locally modified after upgrade)"));
        } else {
            println!("RESTORE {key}");
        }
    }
    for key in &state.latest_recovery_created {
        let destination = target.join(key);
        if destination.exists()
            && state
                .managed
                .get(key)
                .is_some_and(|managed| digest(&destination).ok().as_ref() != Some(&managed.sha256))
        {
            conflicts.push(format!("PRESERVE {key} (locally modified after upgrade)"));
        } else if destination.exists() {
            println!("REMOVE {key} (created by upgrade)");
        }
    }
    print_lines(conflicts.iter());
    if !conflicts.is_empty() {
        eprintln!("No changes applied: rollback is atomic when managed files are unmodified.");
        return Ok(2);
    }
    if !apply {
        println!("Preview only; rerun with --apply to make these changes.");
        return Ok(0);
    }
    for backup in restores {
        let key = relative(&backup, &recovery)?;
        let destination = target.join(&key);
        copy_file(&backup, &destination)?;
        state.managed.insert(
            key,
            ManagedFile {
                sha256: digest(&destination)?,
            },
        );
    }
    for key in state.latest_recovery_created.clone() {
        let path = target.join(&key);
        if path.exists() {
            fs::remove_file(path)?;
        }
        state.managed.remove(&key);
    }
    state.latest_recovery = None;
    state.latest_recovery_created.clear();
    state.updated_at = timestamp();
    write_state(&state_path, &state)?;
    Ok(0)
}

pub fn doctor(plugin: &Path) -> Result<i32, LifecycleError> {
    let manifest: Value =
        serde_json::from_slice(&fs::read(plugin.join(".codex-plugin/plugin.json"))?)?;
    let mut errors = Vec::new();
    let name = manifest
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let version = manifest
        .get("version")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !plugin_layout_matches_manifest(plugin, name, version) {
        errors.push("plugin name does not match its directory".into());
    }
    if plugin.join(".mcp.json").exists() || plugin.join(".app.json").exists() {
        errors.push("external runtime integration is outside the scaffold boundary".into());
    }
    match fs::read_to_string(plugin.join("product/pins.toml"))
        .ok()
        .and_then(|source| source.parse::<toml::Table>().ok())
        .and_then(|pins| pins.get("sources").and_then(toml::Value::as_table).cloned())
    {
        Some(sources) => {
            for key in ["orchestra_codex", "orchestra_desktop"] {
                let valid = sources
                    .get(key)
                    .and_then(toml::Value::as_str)
                    .is_some_and(|value| {
                        value.len() == 40
                            && value
                                .bytes()
                                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
                    });
                if !valid {
                    errors.push(format!("Product source `{key}` is not a full commit hash"));
                }
            }
            for key in ["orchestra_codex_repository", "orchestra_desktop_repository"] {
                let valid = sources
                    .get(key)
                    .and_then(toml::Value::as_str)
                    .is_some_and(|value| {
                        value.starts_with("https://github.com/") && value.ends_with(".git")
                    });
                if !valid {
                    errors.push(format!(
                        "Product source `{key}` is not a sealed GitHub repository"
                    ));
                }
            }
        }
        None => errors.push("Product source pins are missing or invalid".into()),
    }
    if !plugin.join("Cargo.toml").is_file() {
        errors.push("Rust workspace is missing".into());
    }
    let mut configs = Vec::new();
    collect_extension(&plugin.join("config"), "toml", &mut configs)?;
    for path in configs {
        if let Err(error) = fs::read_to_string(&path)?.parse::<toml::Table>() {
            errors.push(format!(
                "invalid TOML {}: {error}",
                path.strip_prefix(plugin).unwrap_or(&path).display()
            ));
        }
    }
    let mut skills = Vec::new();
    collect_named(&plugin.join("skills"), "SKILL.md", &mut skills)?;
    if !skills.iter().any(|path| {
        path.parent()
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            == Some("orchestrate")
    }) {
        errors.push("primary orchestrate skill is missing".into());
    }
    match codex_capability_probe(plugin) {
        Ok(note) => println!("OK {note}"),
        Err(error) => errors.push(format!("Codex capability probe failed: {error}")),
    }
    for error in &errors {
        println!("ERROR {error}");
    }
    if errors.is_empty() {
        println!(
            "OK {} skills; manifest, config, and native capability checks passed",
            skills.len()
        );
        Ok(0)
    } else {
        Ok(1)
    }
}

pub fn plugin_layout_matches_manifest(plugin: &Path, name: &str, version: &str) -> bool {
    plugin.file_name().and_then(|v| v.to_str()) == Some(name)
        || (plugin
            .parent()
            .and_then(Path::file_name)
            .and_then(|v| v.to_str())
            == Some(name)
            && plugin.file_name().and_then(|v| v.to_str()) == Some(version))
}

fn codex_capability_probe(plugin: &Path) -> Result<String, LifecycleError> {
    let version = command_output(Command::new("codex").arg("--version"))?;
    let features = command_output(Command::new("codex").args(["features", "list"]))?;
    if !features
        .lines()
        .any(|line| line.starts_with("multi_agent ") && line.contains("stable"))
    {
        return Err(LifecycleError::Message(
            "installed Codex does not report stable multi_agent support".into(),
        ));
    }
    let temp = temp_dir("doctor-project")?;
    copy_file(
        &plugin.join("config/project.toml"),
        &temp.join("config.toml"),
    )?;
    let report = command_stdout(
        Command::new("codex")
            .args(["--strict-config", "doctor", "--json"])
            .env("CODEX_HOME", &temp),
    )?;
    let report: Value = serde_json::from_str(&report)?;
    if report
        .pointer("/checks/config.load/status")
        .and_then(Value::as_str)
        != Some("ok")
    {
        return Err(LifecycleError::Message(
            "Codex strict configuration load failed for config.toml".into(),
        ));
    }
    fs::remove_dir_all(&temp)?;
    let temp = temp_dir("doctor-profile")?;
    copy_file(
        &plugin.join("config/orchestra.config.toml"),
        &temp.join("orchestra.config.toml"),
    )?;
    let report = command_output(
        Command::new("codex")
            .args([
                "--profile",
                "orchestra",
                "debug",
                "prompt-input",
                "Orchestra profile probe",
            ])
            .env("CODEX_HOME", &temp),
    )?;
    if !serde_json::from_str::<Value>(&report)?.is_array() {
        return Err(LifecycleError::Message(
            "Codex profile selection probe returned an unexpected response".into(),
        ));
    }
    fs::remove_dir_all(temp)?;
    Ok(version.trim().into())
}

fn command_output(command: &mut Command) -> Result<String, LifecycleError> {
    let output = command.output()?;
    if !output.status.success() {
        return Err(LifecycleError::Message(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn command_stdout(command: &mut Command) -> Result<String, LifecycleError> {
    let output = command.output()?;
    if output.stdout.is_empty() {
        return Err(LifecycleError::Message(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn read_state(path: &Path) -> Result<InstallState, LifecycleError> {
    if !path.exists() {
        return Ok(InstallState {
            version: 1,
            ..InstallState::default()
        });
    }
    serde_json::from_slice(&fs::read(path)?).map_err(Into::into)
}

fn write_state(path: &Path, state: &InstallState) -> Result<(), LifecycleError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut data = serde_json::to_vec_pretty(state)?;
    data.push(b'\n');
    fs::write(path, data)?;
    Ok(())
}

fn digest(path: &Path) -> Result<String, LifecycleError> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

fn relative(path: &Path, root: &Path) -> Result<String, LifecycleError> {
    path.strip_prefix(root)
        .map(|value| value.to_string_lossy().replace('\\', "/"))
        .map_err(|_| {
            LifecycleError::Message(format!("{} is outside {}", path.display(), root.display()))
        })
}

fn copy_file(source: &Path, target: &Path) -> Result<(), LifecycleError> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, target)?;
    Ok(())
}

fn collect_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), LifecycleError> {
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, output)?;
        } else if path.is_file() {
            output.push(path);
        }
    }
    Ok(())
}

fn collect_extension(
    root: &Path,
    extension: &str,
    output: &mut Vec<PathBuf>,
) -> Result<(), LifecycleError> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_extension(&path, extension, output)?;
        } else if path.extension().and_then(|v| v.to_str()) == Some(extension) {
            output.push(path);
        }
    }
    Ok(())
}

fn collect_named(root: &Path, name: &str, output: &mut Vec<PathBuf>) -> Result<(), LifecycleError> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_named(&path, name, output)?;
        } else if path.file_name().and_then(|v| v.to_str()) == Some(name) {
            output.push(path);
        }
    }
    Ok(())
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn temp_dir(label: &str) -> Result<PathBuf, LifecycleError> {
    let path = env::temp_dir().join(format!(
        "codex-orchestra-{label}-{}-{}",
        std::process::id(),
        timestamp()
    ));
    fs::create_dir_all(&path)?;
    Ok(path)
}

fn print_lines<'a>(lines: impl Iterator<Item = &'a String>) {
    for line in lines {
        println!("{line}");
    }
}
