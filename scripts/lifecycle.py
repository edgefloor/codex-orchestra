#!/usr/bin/env python3
"""Preview-first lifecycle helper for Codex Orchestra configuration templates."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import tomllib
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

PLUGIN = Path(__file__).resolve().parents[1]
STATE_DIRS = (
    "workflows",
    "runs",
    "install",
)


class LifecycleError(RuntimeError):
    pass


@dataclass(frozen=True)
class DesiredFile:
    source: Path
    target: Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rel(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def desired_project(target: Path) -> list[DesiredFile]:
    files = [DesiredFile(PLUGIN / "config/project.toml", target / ".codex/config.toml")]
    files.extend(
        DesiredFile(item, target / ".codex/agents" / item.name)
        for item in sorted((PLUGIN / "config/agents").glob("*.toml"))
    )
    return files


def desired_global(home: Path, *, default: bool) -> list[DesiredFile]:
    config_name = "config.toml" if default else "orchestra.config.toml"
    files = [DesiredFile(PLUGIN / "config/orchestra.config.toml", home / config_name)]
    files.extend(
        DesiredFile(item, home / "agents" / item.name)
        for item in sorted((PLUGIN / "config/agents").glob("*.toml"))
    )
    return files


def read_state(path: Path) -> dict:
    if not path.exists():
        return {"version": 1, "managed": {}}
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (json.JSONDecodeError, OSError) as exc:
        raise LifecycleError(f"invalid install state {path}: {exc}") from exc


def write_state(path: Path, state: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def classify(files: list[DesiredFile], root: Path, state: dict, *, upgrade: bool) -> tuple[list[str], list[str]]:
    actions: list[str] = []
    conflicts: list[str] = []
    managed = state.get("managed", {})
    for item in files:
        key = rel(item.target, root)
        if not item.target.exists():
            actions.append(f"CREATE {key}")
        elif digest(item.target) == digest(item.source):
            actions.append(f"KEEP   {key} (already current)")
        elif upgrade and key in managed and digest(item.target) == managed[key]["sha256"]:
            actions.append(f"UPDATE {key}")
        else:
            conflicts.append(f"CONFLICT {key} (existing or locally modified)")
    return actions, conflicts


def install(files: list[DesiredFile], root: Path, state_path: Path, *, apply: bool, upgrade: bool = False) -> int:
    state = read_state(state_path)
    actions, conflicts = classify(files, root, state, upgrade=upgrade)
    for line in actions + conflicts:
        print(line)
    if conflicts:
        print("No changes applied: reconcile conflicts and preview again.", file=sys.stderr)
        return 2
    if not apply:
        print("Preview only; rerun with --apply to make these changes.")
        return 0

    managed = dict(state.get("managed", {}))
    updates = [item for item in files if item.target.exists() and digest(item.target) != digest(item.source)]
    created_by_upgrade = [rel(item.target, root) for item in files if upgrade and not item.target.exists()]
    if updates or created_by_upgrade:
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        recovery = state_path.parent / "recovery" / f"install-{stamp}"
        recovery.mkdir(parents=True, exist_ok=True)
        for item in updates:
            backup = recovery / rel(item.target, root)
            backup.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(item.target, backup)
        state["latest_recovery"] = rel(recovery, root)
    if upgrade:
        state["latest_recovery_created"] = created_by_upgrade

    for item in files:
        key = rel(item.target, root)
        was_managed = key in managed
        if not item.target.exists() or digest(item.target) != digest(item.source):
            item.target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(item.source, item.target)
            managed[key] = {"sha256": digest(item.target)}
        elif was_managed:
            managed[key] = {"sha256": digest(item.target)}
    state.update(
        {
            "version": 1,
            "plugin_version": manifest()["version"],
            "managed": managed,
            "updated_at": datetime.now(timezone.utc).isoformat(),
        }
    )
    write_state(state_path, state)
    print(f"Applied {len(files)} managed files.")
    return 0


def init_project_state(target: Path, *, apply: bool) -> None:
    root = target / ".codex/orchestra"
    for name in STATE_DIRS:
        path = root / name
        print(f"{'KEEP  ' if path.is_dir() else 'CREATE'} .codex/orchestra/{name}/")
        if apply:
            path.mkdir(parents=True, exist_ok=True)


def uninstall(target: Path, *, apply: bool) -> int:
    state_path = target / ".codex/orchestra/install-state.json"
    state = read_state(state_path)
    conflicts: list[str] = []
    removable: list[Path] = []
    for key, record in sorted(state.get("managed", {}).items()):
        path = target / key
        if not path.exists():
            continue
        if digest(path) != record["sha256"]:
            conflicts.append(f"PRESERVE {key} (locally modified)")
        else:
            removable.append(path)
            print(f"REMOVE {key}")
    for line in conflicts:
        print(line)
    print("PRESERVE .codex/orchestra/ workflow definitions and run artifacts")
    if not apply:
        print("Preview only; rerun with --apply to make these changes.")
        return 0
    for path in removable:
        path.unlink()
    state["managed"] = {key: value for key, value in state.get("managed", {}).items() if (target / key).exists()}
    state["updated_at"] = datetime.now(timezone.utc).isoformat()
    write_state(state_path, state)
    return 0


def rollback(target: Path, *, apply: bool) -> int:
    state_path = target / ".codex/orchestra/install-state.json"
    state = read_state(state_path)
    recovery_ref = state.get("latest_recovery")
    if not recovery_ref:
        raise LifecycleError("no upgrade recovery snapshot is recorded")
    recovery = target / recovery_ref
    if not recovery.is_dir():
        raise LifecycleError(f"missing recovery snapshot: {recovery}")
    conflicts: list[str] = []
    restores: list[tuple[Path, Path]] = []
    removals: list[Path] = []
    for backup in sorted(path for path in recovery.rglob("*") if path.is_file()):
        destination = target / backup.relative_to(recovery)
        key = rel(destination, target)
        record = state.get("managed", {}).get(key)
        if destination.exists() and record and digest(destination) != record["sha256"]:
            conflicts.append(f"PRESERVE {key} (locally modified after upgrade)")
        else:
            restores.append((backup, destination))
            print(f"RESTORE {key}")
    for key in state.get("latest_recovery_created", []):
        destination = target / key
        record = state.get("managed", {}).get(key)
        if destination.exists() and record and digest(destination) != record["sha256"]:
            conflicts.append(f"PRESERVE {key} (locally modified after upgrade)")
        elif destination.exists():
            removals.append(destination)
            print(f"REMOVE {key} (created by upgrade)")
    for line in conflicts:
        print(line)
    if conflicts:
        print("No changes applied: rollback is atomic when managed files are unmodified.", file=sys.stderr)
        return 2
    if not apply:
        print("Preview only; rerun with --apply to make these changes.")
        return 0
    for backup, destination in restores:
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(backup, destination)
        state["managed"][rel(destination, target)] = {"sha256": digest(destination)}
    for destination in removals:
        destination.unlink()
        state["managed"].pop(rel(destination, target), None)
    state.pop("latest_recovery", None)
    state.pop("latest_recovery_created", None)
    state["updated_at"] = datetime.now(timezone.utc).isoformat()
    write_state(state_path, state)
    return 0


def manifest() -> dict:
    return json.loads((PLUGIN / ".codex-plugin/plugin.json").read_text(encoding="utf-8"))


def plugin_layout_matches_manifest(plugin: Path, data: dict) -> bool:
    name = data.get("name")
    version = data.get("version")
    return plugin.name == name or (plugin.parent.name == name and plugin.name == version)


def doctor() -> int:
    errors: list[str] = []
    notes: list[str] = []
    data = manifest()
    if not plugin_layout_matches_manifest(PLUGIN, data):
        errors.append("plugin name does not match its directory")
    if any((PLUGIN / name).exists() for name in (".mcp.json", ".app.json")):
        errors.append("external runtime integration is outside the scaffold boundary")
    for config in (PLUGIN / "config").rglob("*.toml"):
        try:
            tomllib.loads(config.read_text(encoding="utf-8"))
        except tomllib.TOMLDecodeError as exc:
            errors.append(f"invalid TOML {config.relative_to(PLUGIN)}: {exc}")
    skill_dirs = [path.parent for path in (PLUGIN / "skills").glob("*/SKILL.md")]
    if not any(path.name == "orchestrate" for path in skill_dirs):
        errors.append("primary orchestrate skill is missing")
    try:
        version = subprocess.run(["codex", "--version"], check=True, capture_output=True, text=True).stdout.strip()
        notes.append(version)
        features = subprocess.run(["codex", "features", "list"], check=True, capture_output=True, text=True).stdout
        if not any(line.startswith("multi_agent ") and "stable" in line for line in features.splitlines()):
            errors.append("installed Codex does not report stable multi_agent support")
        with tempfile.TemporaryDirectory() as tmp:
            shutil.copy2(PLUGIN / "config/project.toml", Path(tmp) / "config.toml")
            env = dict(os.environ, CODEX_HOME=tmp)
            result = subprocess.run(
                ["codex", "--strict-config", "doctor", "--json"], capture_output=True, text=True, env=env
            )
            report = json.loads(result.stdout)
            if report.get("checks", {}).get("config.load", {}).get("status") != "ok":
                errors.append("Codex strict configuration load failed for config.toml")
        with tempfile.TemporaryDirectory() as tmp:
            shutil.copy2(PLUGIN / "config/orchestra.config.toml", Path(tmp) / "orchestra.config.toml")
            env = dict(os.environ, CODEX_HOME=tmp)
            result = subprocess.run(
                ["codex", "--profile", "orchestra", "debug", "prompt-input", "Orchestra profile probe"],
                check=True,
                capture_output=True,
                text=True,
                env=env,
            )
            if not isinstance(json.loads(result.stdout), list):
                errors.append("Codex profile selection probe returned an unexpected response")
    except (FileNotFoundError, subprocess.CalledProcessError, json.JSONDecodeError) as exc:
        errors.append(f"Codex capability probe failed: {exc}")
    for note in notes:
        print(f"OK {note}")
    for error in errors:
        print(f"ERROR {error}")
    if not errors:
        print(f"OK {len(skill_dirs)} skills; manifest, config, and native capability checks passed")
    return 1 if errors else 0


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    sub = result.add_subparsers(dest="command", required=True)
    sub.add_parser("doctor")
    for name in ("project", "upgrade", "rollback", "uninstall"):
        command = sub.add_parser(name)
        command.add_argument("--target", type=Path, required=True)
        command.add_argument("--apply", action="store_true")
    for name in ("profile", "global-default"):
        command = sub.add_parser(name)
        command.add_argument("--codex-home", type=Path, default=Path.home() / ".codex")
        command.add_argument("--apply", action="store_true")
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "doctor":
            return doctor()
        if args.command in {"project", "upgrade"}:
            target = args.target.resolve()
            state_path = target / ".codex/orchestra/install-state.json"
            result = install(
                desired_project(target),
                target,
                state_path,
                apply=args.apply,
                upgrade=args.command == "upgrade",
            )
            if result == 0:
                init_project_state(target, apply=args.apply)
            return result
        if args.command == "uninstall":
            return uninstall(args.target.resolve(), apply=args.apply)
        if args.command == "rollback":
            return rollback(args.target.resolve(), apply=args.apply)
        home = args.codex_home.expanduser().resolve()
        state_path = home / "orchestra-install-state.json"
        return install(
            desired_global(home, default=args.command == "global-default"),
            home,
            state_path,
            apply=args.apply,
        )
    except LifecycleError as exc:
        print(f"ERROR {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
