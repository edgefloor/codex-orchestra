use codex_orchestra_lifecycle::{
    LifecycleError, desired_global, desired_project, doctor, init_project_state, install,
    plugin_root, rollback, uninstall,
};
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    match run(env::args().skip(1).collect()) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("ERROR {error}");
            std::process::exit(2);
        }
    }
}

fn run(args: Vec<String>) -> Result<i32, LifecycleError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(usage());
    };
    let plugin = plugin_root();
    if command == "doctor" {
        if args.len() != 1 {
            return Err(usage());
        }
        return doctor(&plugin);
    }
    let apply = args.iter().any(|arg| arg == "--apply");
    let version = manifest_version(&plugin)?;
    match command {
        "project" | "upgrade" | "rollback" | "uninstall" => {
            let target = option_path(&args, "--target")?.canonicalize()?;
            match command {
                "project" | "upgrade" => {
                    let state = target.join(".codex/orchestra/install-state.json");
                    let result = install(
                        &desired_project(&plugin, &target),
                        &target,
                        &state,
                        &version,
                        apply,
                        command == "upgrade",
                    )?;
                    if result == 0 {
                        init_project_state(&target, apply)?;
                    }
                    Ok(result)
                }
                "rollback" => rollback(&target, apply),
                "uninstall" => uninstall(&target, apply),
                _ => unreachable!(),
            }
        }
        "profile" | "global-default" => {
            let home = optional_path(&args, "--codex-home")?.unwrap_or_else(default_codex_home);
            let home = absolute(home)?;
            install(
                &desired_global(&plugin, &home, command == "global-default"),
                &home,
                &home.join("orchestra-install-state.json"),
                &version,
                apply,
                false,
            )
        }
        _ => Err(usage()),
    }
}

fn manifest_version(plugin: &std::path::Path) -> Result<String, LifecycleError> {
    let value: Value =
        serde_json::from_slice(&fs::read(plugin.join(".codex-plugin/plugin.json"))?)?;
    value
        .get("version")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| LifecycleError::Message("plugin manifest version is missing".into()))
}

fn option_path(args: &[String], option: &str) -> Result<PathBuf, LifecycleError> {
    optional_path(args, option)?.ok_or_else(usage)
}

fn optional_path(args: &[String], option: &str) -> Result<Option<PathBuf>, LifecycleError> {
    let Some(index) = args.iter().position(|arg| arg == option) else {
        return Ok(None);
    };
    args.get(index + 1)
        .map(PathBuf::from)
        .map(Some)
        .ok_or_else(usage)
}

fn absolute(path: PathBuf) -> Result<PathBuf, LifecycleError> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn default_codex_home() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".codex")
}

fn usage() -> LifecycleError {
    LifecycleError::Message(
        "usage: orchestra-lifecycle doctor | (project|upgrade|rollback|uninstall) --target PATH [--apply] | (profile|global-default) [--codex-home PATH] [--apply]".into(),
    )
}
