#!/usr/bin/env python3
"""Small reference control-plane utility for Codex Orchestra.

This is intentionally not a full Codex App Server client. It demonstrates the
state, event, digest, phase, task, and lease invariants that the production
Conductor should preserve.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sqlite3
import sys
import tomllib
import uuid
from collections import Counter
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Any, Iterable, Sequence


DB_RELATIVE_PATH = Path(".orchestra/runtime/orchestra.db")

PHASES = {
    "NEW",
    "GROUNDING",
    "GROUNDED",
    "DELIVERY_DESIGN",
    "ARCHITECTED",
    "STAFFING",
    "WAVE_READY",
    "EXECUTING",
    "INTEGRATING",
    "ASSURANCE",
    "MILESTONE_CHECKPOINT",
    "DELIVERY",
    "OPERATOR_ACCEPTANCE",
    "CLOSED",
    "REGROUNDING",
    "PAUSED",
    "ABORTED",
}

ALLOWED_TRANSITIONS: dict[str, set[str]] = {
    "NEW": {"GROUNDING", "PAUSED", "ABORTED"},
    "GROUNDING": {"GROUNDED", "REGROUNDING", "PAUSED", "ABORTED"},
    "GROUNDED": {"DELIVERY_DESIGN", "REGROUNDING", "PAUSED", "ABORTED"},
    "DELIVERY_DESIGN": {"ARCHITECTED", "REGROUNDING", "PAUSED", "ABORTED"},
    "ARCHITECTED": {"STAFFING", "REGROUNDING", "PAUSED", "ABORTED"},
    "STAFFING": {"WAVE_READY", "REGROUNDING", "PAUSED", "ABORTED"},
    "WAVE_READY": {"EXECUTING", "REGROUNDING", "PAUSED", "ABORTED"},
    "EXECUTING": {"INTEGRATING", "REGROUNDING", "PAUSED", "ABORTED"},
    "INTEGRATING": {"EXECUTING", "ASSURANCE", "REGROUNDING", "PAUSED", "ABORTED"},
    "ASSURANCE": {"EXECUTING", "MILESTONE_CHECKPOINT", "REGROUNDING", "PAUSED", "ABORTED"},
    "MILESTONE_CHECKPOINT": {"WAVE_READY", "DELIVERY", "REGROUNDING", "PAUSED", "ABORTED"},
    "DELIVERY": {"OPERATOR_ACCEPTANCE", "REGROUNDING", "PAUSED", "ABORTED"},
    "OPERATOR_ACCEPTANCE": {"CLOSED", "WAVE_READY", "REGROUNDING", "PAUSED", "ABORTED"},
    "CLOSED": set(),
    "REGROUNDING": {"GROUNDING", "GROUNDED", "DELIVERY_DESIGN", "ARCHITECTED", "STAFFING", "WAVE_READY", "EXECUTING", "PAUSED", "ABORTED"},
    "PAUSED": PHASES - {"PAUSED", "CLOSED"},
    "ABORTED": set(),
}


CANONICAL_FIXED_FILES = (
    Path("AGENTS.md"),
    Path(".orchestra/charter/BRIEF.md"),
    Path(".orchestra/charter/SCOPE.md"),
    Path(".orchestra/charter/ASSURANCE.yaml"),
    Path(".orchestra/plan/MILESTONES.yaml"),
    Path(".orchestra/plan/WORKSTREAMS.yaml"),
    Path(".orchestra/roster/ROLE_CATALOG.yaml"),
)

REQUIRED_AGENT_FIELDS = {"name", "description", "developer_instructions"}
READ_ONLY_ROLES = {
    "consultant",
    "delivery_architect",
    "manager",
    "context_engineer",
    "role_architect",
    "quality_governor",
    "explorer_terra",
    "advisor_sol",
    "reviewer_terra",
}
KNOWN_AGENT_TYPES = {
    "consultant",
    "delivery_architect",
    "manager",
    "context_engineer",
    "role_architect",
    "team_leader",
    "quality_governor",
    "explorer_terra",
    "advisor_sol",
    "reviewer_terra",
    "worker_luna",
    "worker_terra",
    "worker_sol",
    "verifier_terra",
}

TASK_AGENT_TYPES = {
    "team_leader",
    "explorer_terra",
    "advisor_sol",
    "reviewer_terra",
    "worker_luna",
    "worker_terra",
    "worker_sol",
    "verifier_terra",
}

PHYSICAL_CHILD_ROLE_RULES: dict[str, set[str]] = {
    "root": {"consultant", "delivery_architect", "manager", "quality_governor"},
    "consultant": {"explorer_terra", "advisor_sol"},
    "delivery_architect": {"explorer_terra", "advisor_sol"},
    "manager": {"team_leader", "role_architect"},
    "team_leader": {
        "context_engineer", "explorer_terra", "advisor_sol",
        "reviewer_terra", "worker_luna", "worker_terra", "worker_sol",
    },
    "quality_governor": {"verifier_terra", "advisor_sol"},
    "role_architect": set(),
    "context_engineer": set(),
    "explorer_terra": set(),
    "advisor_sol": set(),
    "reviewer_terra": set(),
    "worker_luna": set(),
    "worker_terra": set(),
    "worker_sol": set(),
    "verifier_terra": set(),
}


class OrchestraError(RuntimeError):
    """A user-correctable control-plane invariant violation."""


def utc_now() -> datetime:
    return datetime.now(timezone.utc)


def iso(dt: datetime | None = None) -> str:
    return (dt or utc_now()).astimezone(timezone.utc).isoformat().replace("+00:00", "Z")


def parse_iso(value: str) -> datetime:
    normalized = value[:-1] + "+00:00" if value.endswith("Z") else value
    parsed = datetime.fromisoformat(normalized)
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def repo_root(path: str | Path | None = None) -> Path:
    start = Path(path or ".").resolve()
    if start.is_file():
        start = start.parent
    for candidate in (start, *start.parents):
        if (candidate / ".orchestra").is_dir() and (candidate / ".codex").is_dir():
            return candidate
    raise OrchestraError(f"No Codex Orchestra repository found from {start}")


def db_path(root: Path) -> Path:
    return root / DB_RELATIVE_PATH


def connect(root: Path, *, must_exist: bool = True) -> sqlite3.Connection:
    path = db_path(root)
    if must_exist and not path.exists():
        raise OrchestraError(f"Runtime database does not exist: {path}. Run `orchestra.py init` first.")
    path.parent.mkdir(parents=True, exist_ok=True)
    conn = sqlite3.connect(path, timeout=10.0, isolation_level=None)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA busy_timeout = 10000")
    return conn


def schema_sql() -> str:
    return """
    PRAGMA journal_mode = WAL;

    CREATE TABLE IF NOT EXISTS projects (
        project_id TEXT PRIMARY KEY,
        phase TEXT NOT NULL,
        previous_phase TEXT,
        scope_revision INTEGER NOT NULL,
        alignment_digest TEXT NOT NULL,
        current_milestone TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS events (
        seq INTEGER PRIMARY KEY AUTOINCREMENT,
        event_id TEXT NOT NULL UNIQUE,
        project_id TEXT NOT NULL,
        event_type TEXT NOT NULL,
        actor TEXT NOT NULL,
        idempotency_key TEXT,
        payload_json TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY(project_id) REFERENCES projects(project_id),
        UNIQUE(project_id, idempotency_key)
    );

    CREATE TABLE IF NOT EXISTS agents (
        agent_id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL,
        role TEXT NOT NULL,
        reports_to TEXT,
        physical_parent_id TEXT,
        agent_path TEXT NOT NULL,
        depth INTEGER NOT NULL,
        delegation_permit_ref TEXT,
        status TEXT NOT NULL,
        config_ref TEXT NOT NULL,
        role_card_ref TEXT,
        thread_id TEXT,
        updated_at TEXT NOT NULL,
        FOREIGN KEY(project_id) REFERENCES projects(project_id),
        UNIQUE(project_id, agent_path)
    );

    CREATE TABLE IF NOT EXISTS tasks (
        task_id TEXT PRIMARY KEY,
        project_id TEXT NOT NULL,
        title TEXT NOT NULL,
        workstream TEXT NOT NULL,
        milestone TEXT NOT NULL,
        scope_revision INTEGER NOT NULL,
        alignment_digest TEXT NOT NULL,
        status TEXT NOT NULL,
        recommended_agent_type TEXT NOT NULL,
        reports_to TEXT,
        context_ref TEXT NOT NULL,
        write_domain_json TEXT NOT NULL,
        acceptance_json TEXT NOT NULL,
        current_attempt_id TEXT,
        result_ref TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY(project_id) REFERENCES projects(project_id)
    );

    CREATE TABLE IF NOT EXISTS attempts (
        attempt_id TEXT PRIMARY KEY,
        task_id TEXT NOT NULL,
        project_id TEXT NOT NULL,
        agent_id TEXT NOT NULL,
        status TEXT NOT NULL,
        started_at TEXT NOT NULL,
        ended_at TEXT,
        result_ref TEXT,
        error_class TEXT,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL,
        FOREIGN KEY(task_id) REFERENCES tasks(task_id),
        FOREIGN KEY(project_id) REFERENCES projects(project_id)
    );

    CREATE TABLE IF NOT EXISTS leases (
        task_id TEXT PRIMARY KEY,
        attempt_id TEXT NOT NULL UNIQUE,
        project_id TEXT NOT NULL,
        agent_id TEXT NOT NULL,
        lease_token TEXT NOT NULL UNIQUE,
        expires_at TEXT NOT NULL,
        heartbeat_at TEXT NOT NULL,
        created_at TEXT NOT NULL,
        FOREIGN KEY(task_id) REFERENCES tasks(task_id),
        FOREIGN KEY(attempt_id) REFERENCES attempts(attempt_id),
        FOREIGN KEY(project_id) REFERENCES projects(project_id)
    );

    CREATE INDEX IF NOT EXISTS idx_events_project_seq ON events(project_id, seq);
    CREATE INDEX IF NOT EXISTS idx_tasks_project_status ON tasks(project_id, status);
    CREATE INDEX IF NOT EXISTS idx_attempts_task ON attempts(task_id, created_at);
    CREATE INDEX IF NOT EXISTS idx_leases_expiry ON leases(project_id, expires_at);
    """


def canonical_paths(root: Path) -> list[Path]:
    paths: list[Path] = [p for p in CANONICAL_FIXED_FILES if (root / p).is_file()]
    for pattern in (
        ".orchestra/decisions/ADR-*.md",
        ".orchestra/decisions/WAIVER-*.md",
        ".orchestra/policies/*.yaml",
    ):
        paths.extend(p.relative_to(root) for p in root.glob(pattern) if p.is_file())
    return sorted(set(paths), key=lambda p: p.as_posix())


def compute_alignment_digest(root: Path) -> tuple[str, list[str]]:
    hasher = hashlib.sha256()
    included: list[str] = []
    for relative in canonical_paths(root):
        path = root / relative
        included.append(relative.as_posix())
        hasher.update(relative.as_posix().encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(path.read_bytes())
        hasher.update(b"\0")
    return f"sha256:{hasher.hexdigest()}", included


def get_project(conn: sqlite3.Connection, project_id: str) -> sqlite3.Row:
    row = conn.execute("SELECT * FROM projects WHERE project_id = ?", (project_id,)).fetchone()
    if row is None:
        raise OrchestraError(f"Unknown project_id: {project_id}")
    return row


def emit_event(
    conn: sqlite3.Connection,
    *,
    project_id: str,
    event_type: str,
    actor: str,
    payload: dict[str, Any],
    idempotency_key: str | None = None,
) -> dict[str, Any]:
    if idempotency_key:
        existing = conn.execute(
            "SELECT event_id, seq, created_at FROM events WHERE project_id = ? AND idempotency_key = ?",
            (project_id, idempotency_key),
        ).fetchone()
        if existing:
            return {"event_id": existing["event_id"], "seq": existing["seq"], "created_at": existing["created_at"], "duplicate": True}
    event_id = f"evt-{uuid.uuid4()}"
    created_at = iso()
    cursor = conn.execute(
        "INSERT INTO events(event_id, project_id, event_type, actor, idempotency_key, payload_json, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        (event_id, project_id, event_type, actor, idempotency_key, json.dumps(payload, sort_keys=True), created_at),
    )
    return {"event_id": event_id, "seq": cursor.lastrowid, "created_at": created_at, "duplicate": False}


def migrate_schema(conn: sqlite3.Connection) -> None:
    """Add columns introduced by later framework revisions to an existing pilot DB."""
    columns = {row[1] for row in conn.execute("PRAGMA table_info(agents)").fetchall()}
    additions = {
        "physical_parent_id": "TEXT",
        "agent_path": "TEXT",
        "depth": "INTEGER",
        "delegation_permit_ref": "TEXT",
    }
    for name, sql_type in additions.items():
        if name not in columns:
            conn.execute(f"ALTER TABLE agents ADD COLUMN {name} {sql_type}")


def configured_max_depth(root: Path) -> int:
    try:
        config = tomllib.loads((root / ".codex/config.toml").read_text(encoding="utf-8"))
        value = config.get("agents", {}).get("max_depth", 3)
    except (OSError, tomllib.TOMLDecodeError):
        value = 3
    return int(value) if isinstance(value, int) else 3


def normalize_task_name(value: str) -> str:
    normalized = re.sub(r"[^a-z0-9_]+", "_", value.strip().lower()).strip("_")
    if not normalized:
        raise OrchestraError("Agent task_name must contain a lowercase letter, digit, or underscore")
    return normalized


def initialize(root: Path, project_id: str = "default") -> dict[str, Any]:
    conn = connect(root, must_exist=False)
    try:
        conn.executescript(schema_sql())
        migrate_schema(conn)
        digest, included = compute_alignment_digest(root)
        now = iso()
        conn.execute("BEGIN IMMEDIATE")
        try:
            existing = conn.execute("SELECT project_id FROM projects WHERE project_id = ?", (project_id,)).fetchone()
            if existing is None:
                conn.execute(
                    "INSERT INTO projects(project_id, phase, previous_phase, scope_revision, alignment_digest, current_milestone, created_at, updated_at) VALUES (?, 'NEW', NULL, 0, ?, NULL, ?, ?)",
                    (project_id, digest, now, now),
                )
                event = emit_event(
                    conn,
                    project_id=project_id,
                    event_type="project.initialized",
                    actor="conductor",
                    payload={"phase": "NEW", "scope_revision": 0, "alignment_digest": digest, "included_files": included},
                    idempotency_key=f"project-init:{project_id}",
                )
            else:
                event = {"duplicate": True}
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"project_id": project_id, "database": str(db_path(root)), "alignment_digest": digest, "event": event}
    finally:
        conn.close()


def doctor(root: Path) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    warnings: list[str] = []

    required_paths = [
        "README.md",
        "DESIGN.md",
        "AGENTS.md",
        ".codex/config.toml",
        ".orchestra/charter/BRIEF.md",
        ".orchestra/charter/SCOPE.md",
        ".orchestra/charter/ASSURANCE.yaml",
        ".orchestra/plan/MILESTONES.yaml",
        ".orchestra/plan/WORKSTREAMS.yaml",
        ".orchestra/roster/ROLE_CATALOG.yaml",
        ".orchestra/policies/concurrency.yaml",
        ".orchestra/policies/collaboration-v2.yaml",
        ".orchestra/policies/assurance-policy.yaml",
        ".orchestra/schemas/collaboration-command.schema.json",
        ".orchestra/schemas/delegation-permit.schema.json",
        ".orchestra/schemas/branch-report.schema.json",
        ".orchestra/schemas/portfolio-checkpoint.schema.json",
        ".orchestra/schemas/context-capsule.schema.json",
        ".orchestra/schemas/result-envelope.schema.json",
        ".orchestra/templates/PORTFOLIO_CHECKPOINT.yaml",
        "docs/V2-COLLABORATION.md",
    ]
    for relative in required_paths:
        if not (root / relative).is_file():
            errors.append(f"missing required file: {relative}")

    agents_md = root / "AGENTS.md"
    if agents_md.is_file() and agents_md.stat().st_size > 32 * 1024:
        errors.append(f"AGENTS.md is {agents_md.stat().st_size} bytes; keep root instructions below 32 KiB")

    config_path = root / ".codex/config.toml"
    if config_path.is_file():
        try:
            config = tomllib.loads(config_path.read_text(encoding="utf-8"))
            features = config.get("features", {})
            v2 = features.get("multi_agent_v2") if isinstance(features, dict) else None
            if not isinstance(v2, dict) or v2.get("enabled") is not True:
                errors.append(".codex/config.toml must enable features.multi_agent_v2.enabled = true")
                v2 = {}
            if v2.get("hide_spawn_agent_metadata") is not False:
                errors.append("multi_agent_v2.hide_spawn_agent_metadata must be false so routing fields stay visible")
            if v2.get("expose_spawn_agent_model_overrides") is not True:
                errors.append("multi_agent_v2.expose_spawn_agent_model_overrides must be true")
            if v2.get("tool_namespace") != "collaboration":
                warnings.append("multi_agent_v2.tool_namespace should be 'collaboration' for the documented skill contract")
            v2_threads = v2.get("max_concurrent_threads_per_session")
            if not isinstance(v2_threads, int) or v2_threads < 4:
                errors.append("multi_agent_v2.max_concurrent_threads_per_session must be an integer >= 4")

            agent_config = config.get("agents", {})
            if agent_config.get("max_depth") != 3:
                errors.append(".codex/config.toml must set agents.max_depth = 3 for the bounded hierarchy")
            max_threads = agent_config.get("max_threads")
            if not isinstance(max_threads, int) or max_threads < 6:
                errors.append("agents.max_threads must be an integer >= 6 to reserve control/review capacity")
            elif max_threads > 12:
                warnings.append("agents.max_threads > 12: validate write contention, cost, and fan-out behavior")
            if isinstance(max_threads, int) and isinstance(v2_threads, int) and max_threads != v2_threads:
                warnings.append("agents.max_threads and V2 session thread ceiling differ; document the intended lower bound")
        except (OSError, tomllib.TOMLDecodeError) as exc:
            errors.append(f"invalid .codex/config.toml: {exc}")

    names: dict[str, Path] = {}
    agent_dir = root / ".codex/agents"
    for path in sorted(agent_dir.glob("*.toml")) if agent_dir.is_dir() else []:
        try:
            data = tomllib.loads(path.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as exc:
            errors.append(f"invalid agent TOML {path.relative_to(root)}: {exc}")
            continue
        missing = REQUIRED_AGENT_FIELDS - data.keys()
        if missing:
            errors.append(f"{path.relative_to(root)} missing fields: {', '.join(sorted(missing))}")
            continue
        name = data["name"]
        if not isinstance(name, str) or not name.strip():
            errors.append(f"{path.relative_to(root)} has invalid name")
            continue
        if name in names:
            errors.append(f"duplicate agent name {name}: {names[name].name} and {path.name}")
        names[name] = path
        if path.stem != name:
            warnings.append(f"agent filename {path.name} does not match name={name!r}")
        sandbox = data.get("sandbox_mode")
        if name in READ_ONLY_ROLES and sandbox != "read-only":
            errors.append(f"read-only role {name} must set sandbox_mode = 'read-only'")
        effort = data.get("model_reasoning_effort")
        if effort == "ultra":
            errors.append(f"agent {name} may not default to multi-agent orchestration effort 'ultra'")
        elif effort == "max":
            warnings.append(f"agent {name} defaults to costly single-agent effort 'max'; prefer per-attempt justification")
        if name == "manager" and effort != "high":
            warnings.append("Manager should use high reasoning for consequential, low-frequency decisions")
        if name.startswith("worker_") and sandbox != "workspace-write":
            errors.append(f"builder {name} must default to workspace-write in an isolated worktree")
        nicknames = data.get("nickname_candidates")
        if not isinstance(nicknames, list) or not nicknames or not all(isinstance(n, str) and n.strip() for n in nicknames):
            warnings.append(f"agent {name} should define a non-empty nickname_candidates list for readable V2 activity")
        elif len(set(nicknames)) != len(nicknames):
            errors.append(f"agent {name} has duplicate nickname_candidates")

    missing_agents = KNOWN_AGENT_TYPES - names.keys()
    if missing_agents:
        errors.append(f"missing expected agent archetypes: {', '.join(sorted(missing_agents))}")

    skill_dir = root / ".agents/skills"
    if not skill_dir.is_dir():
        errors.append("missing .agents/skills")
    else:
        for skill in sorted(p for p in skill_dir.iterdir() if p.is_dir()):
            skill_md = skill / "SKILL.md"
            if not skill_md.is_file():
                errors.append(f"skill {skill.name} missing SKILL.md")
            else:
                text = skill_md.read_text(encoding="utf-8")
                frontmatter = re.match(r"^---\s*\n(?P<body>.*?)\n---\s*\n", text, re.DOTALL)
                if frontmatter is None:
                    errors.append(f"skill {skill.name} SKILL.md missing YAML frontmatter")
                else:
                    body = frontmatter.group("body")
                    name_match = re.search(r"(?m)^name:\s*[\"']?([^\n\"']+)", body)
                    description_match = re.search(r"(?m)^description:\s*(.+)$", body)
                    if name_match is None:
                        errors.append(f"skill {skill.name} SKILL.md missing name")
                    elif name_match.group(1).strip() != skill.name:
                        errors.append(
                            f"skill directory {skill.name} does not match frontmatter name={name_match.group(1).strip()!r}"
                        )
                    if description_match is None or not description_match.group(1).strip():
                        errors.append(f"skill {skill.name} SKILL.md missing description")
            metadata = skill / "agents/openai.yaml"
            if not metadata.is_file():
                warnings.append(f"skill {skill.name} missing agents/openai.yaml invocation policy")
            else:
                metadata_text = metadata.read_text(encoding="utf-8")
                if "allow_implicit_invocation: false" not in metadata_text:
                    warnings.append(
                        f"skill {skill.name} should disable implicit invocation for explicit orchestration procedures"
                    )

    schema_dir = root / ".orchestra/schemas"
    for path in sorted(schema_dir.glob("*.json")) if schema_dir.is_dir() else []:
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
            if data.get("$schema") != "https://json-schema.org/draft/2020-12/schema":
                warnings.append(f"{path.relative_to(root)} does not declare JSON Schema 2020-12")
        except (OSError, json.JSONDecodeError) as exc:
            errors.append(f"invalid JSON schema {path.relative_to(root)}: {exc}")

    gitignore = root / ".gitignore"
    if not gitignore.is_file() or ".orchestra/runtime/*" not in gitignore.read_text(encoding="utf-8"):
        errors.append(".gitignore must exclude .orchestra/runtime/*")

    return errors, warnings


def record_digest(
    root: Path,
    *,
    project_id: str,
    actor: str,
    affected_tasks: Sequence[str] = (),
    idempotency_key: str | None = None,
) -> dict[str, Any]:
    digest, included = compute_alignment_digest(root)
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            project = get_project(conn, project_id)
            old_digest = project["alignment_digest"]
            now = iso()
            conn.execute("UPDATE projects SET alignment_digest = ?, updated_at = ? WHERE project_id = ?", (digest, now, project_id))
            stale: list[str] = []
            for task_id in affected_tasks:
                task = conn.execute("SELECT status FROM tasks WHERE project_id = ? AND task_id = ?", (project_id, task_id)).fetchone()
                if task is None:
                    raise OrchestraError(f"Unknown affected task: {task_id}")
                if task["status"] not in {"done", "integrated", "cancelled"}:
                    conn.execute("UPDATE tasks SET status = 'stale', updated_at = ? WHERE task_id = ?", (now, task_id))
                    stale.append(task_id)
            event = emit_event(
                conn,
                project_id=project_id,
                event_type="alignment.digest_recorded",
                actor=actor,
                payload={"old_digest": old_digest, "new_digest": digest, "included_files": included, "stale_tasks": stale},
                idempotency_key=idempotency_key,
            )
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"project_id": project_id, "old_digest": old_digest, "alignment_digest": digest, "included_files": included, "stale_tasks": stale, "event": event}
    finally:
        conn.close()


def set_phase(
    root: Path,
    *,
    project_id: str,
    target: str,
    actor: str,
    gate_ref: str,
    expected_current: str | None = None,
    idempotency_key: str | None = None,
) -> dict[str, Any]:
    target = target.upper()
    if target not in PHASES:
        raise OrchestraError(f"Unknown phase: {target}")
    if not gate_ref.strip():
        raise OrchestraError("A gate/evidence reference is required for every phase transition")
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            project = get_project(conn, project_id)
            current = project["phase"]
            if expected_current and current != expected_current.upper():
                raise OrchestraError(f"Phase compare-and-set failed: expected {expected_current.upper()}, found {current}")
            if target == current:
                raise OrchestraError(f"Project is already in phase {target}")
            if target not in ALLOWED_TRANSITIONS[current]:
                raise OrchestraError(f"Illegal phase transition: {current} -> {target}")
            now = iso()
            conn.execute(
                "UPDATE projects SET previous_phase = ?, phase = ?, updated_at = ? WHERE project_id = ?",
                (current, target, now, project_id),
            )
            event = emit_event(
                conn,
                project_id=project_id,
                event_type="phase.changed",
                actor=actor,
                payload={"from": current, "to": target, "gate_ref": gate_ref},
                idempotency_key=idempotency_key,
            )
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"project_id": project_id, "from": current, "to": target, "gate_ref": gate_ref, "event": event}
    finally:
        conn.close()


def revise_scope(
    root: Path,
    *,
    project_id: str,
    revision: int,
    actor: str,
    reason: str,
    affected_tasks: Sequence[str],
    idempotency_key: str | None = None,
) -> dict[str, Any]:
    digest, included = compute_alignment_digest(root)
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            project = get_project(conn, project_id)
            old_revision = int(project["scope_revision"])
            if revision <= old_revision:
                raise OrchestraError(f"New scope revision {revision} must be greater than current {old_revision}")
            now = iso()
            stale: list[str] = []
            for task_id in affected_tasks:
                task = conn.execute("SELECT status FROM tasks WHERE project_id = ? AND task_id = ?", (project_id, task_id)).fetchone()
                if task is None:
                    raise OrchestraError(f"Unknown affected task: {task_id}")
                if task["status"] not in {"done", "integrated", "cancelled"}:
                    conn.execute("UPDATE tasks SET status = 'stale', updated_at = ? WHERE task_id = ?", (now, task_id))
                    stale.append(task_id)
            conn.execute(
                "UPDATE projects SET scope_revision = ?, alignment_digest = ?, updated_at = ? WHERE project_id = ?",
                (revision, digest, now, project_id),
            )
            event = emit_event(
                conn,
                project_id=project_id,
                event_type="scope.revised",
                actor=actor,
                payload={"old_revision": old_revision, "new_revision": revision, "alignment_digest": digest, "reason": reason, "affected_tasks": stale, "included_files": included},
                idempotency_key=idempotency_key,
            )
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"project_id": project_id, "old_revision": old_revision, "scope_revision": revision, "alignment_digest": digest, "stale_tasks": stale, "event": event}
    finally:
        conn.close()


def register_agent(
    root: Path,
    *,
    project_id: str,
    agent_id: str,
    role: str,
    reports_to: str | None,
    status: str,
    config_ref: str,
    role_card_ref: str | None,
    thread_id: str | None,
    actor: str,
    physical_parent_id: str | None = None,
    task_name: str | None = None,
    delegation_permit_ref: str | None = None,
) -> dict[str, Any]:
    if role not in KNOWN_AGENT_TYPES and not role_card_ref:
        raise OrchestraError("Unknown role requires a role_card_ref")
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            get_project(conn, project_id)
            if physical_parent_id is None:
                parent_role = "root"
                parent_path = "/root"
                depth = 1
            else:
                parent = conn.execute(
                    "SELECT agent_id, role, agent_path, depth FROM agents WHERE project_id = ? AND agent_id = ?",
                    (project_id, physical_parent_id),
                ).fetchone()
                if parent is None:
                    raise OrchestraError(f"Unknown physical parent agent: {physical_parent_id}")
                parent_role = parent["role"]
                parent_path = parent["agent_path"] or f"/root/{normalize_task_name(physical_parent_id)}"
                depth = int(parent["depth"] or 1) + 1
                if reports_to is None:
                    reports_to = physical_parent_id
                if reports_to != physical_parent_id:
                    raise OrchestraError("Operational agents must logically report to their physical parent unless an explicit exception is implemented")

            allowed = PHYSICAL_CHILD_ROLE_RULES.get(parent_role, set())
            if role in KNOWN_AGENT_TYPES and role not in allowed:
                raise OrchestraError(f"Physical parent role {parent_role} may not spawn child role {role}")
            if role not in KNOWN_AGENT_TYPES and parent_role not in {"consultant", "delivery_architect", "team_leader", "quality_governor"}:
                raise OrchestraError(f"Physical parent role {parent_role} may not spawn a generated unknown role")
            max_depth = configured_max_depth(root)
            if depth > max_depth:
                raise OrchestraError(f"Agent depth {depth} exceeds configured max_depth {max_depth}")
            if depth > 1 and not delegation_permit_ref:
                raise OrchestraError("Nested agent registration requires a delegation_permit_ref")

            segment = normalize_task_name(task_name or agent_id)
            agent_path = f"{parent_path}/{segment}"
            now = iso()
            conn.execute(
                "INSERT INTO agents(agent_id, project_id, role, reports_to, physical_parent_id, agent_path, depth, delegation_permit_ref, status, config_ref, role_card_ref, thread_id, updated_at) "
                "VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) "
                "ON CONFLICT(agent_id) DO UPDATE SET role=excluded.role, reports_to=excluded.reports_to, physical_parent_id=excluded.physical_parent_id, agent_path=excluded.agent_path, depth=excluded.depth, delegation_permit_ref=excluded.delegation_permit_ref, status=excluded.status, config_ref=excluded.config_ref, role_card_ref=excluded.role_card_ref, thread_id=excluded.thread_id, updated_at=excluded.updated_at",
                (agent_id, project_id, role, reports_to, physical_parent_id, agent_path, depth, delegation_permit_ref, status, config_ref, role_card_ref, thread_id, now),
            )
            # Logical reporting lines must remain acyclic.
            cursor_id = reports_to
            seen = {agent_id}
            while cursor_id:
                if cursor_id in seen:
                    raise OrchestraError(f"Reporting cycle detected while assigning {agent_id} -> {reports_to}")
                seen.add(cursor_id)
                parent = conn.execute(
                    "SELECT reports_to FROM agents WHERE project_id = ? AND agent_id = ?",
                    (project_id, cursor_id),
                ).fetchone()
                cursor_id = parent["reports_to"] if parent else None
            event = emit_event(
                conn,
                project_id=project_id,
                event_type="agent.registered",
                actor=actor,
                payload={
                    "agent_id": agent_id, "role": role, "reports_to": reports_to,
                    "physical_parent_id": physical_parent_id, "agent_path": agent_path,
                    "depth": depth, "delegation_permit_ref": delegation_permit_ref,
                    "status": status, "config_ref": config_ref,
                    "role_card_ref": role_card_ref, "thread_id": thread_id,
                },
            )
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {
            "agent_id": agent_id, "role": role, "reports_to": reports_to,
            "physical_parent_id": physical_parent_id, "agent_path": agent_path,
            "depth": depth, "delegation_permit_ref": delegation_permit_ref,
            "status": status, "event": event,
        }
    finally:
        conn.close()


def add_task(
    root: Path,
    *,
    project_id: str,
    task_id: str,
    title: str,
    workstream: str,
    milestone: str,
    context_ref: str,
    recommended_agent_type: str,
    reports_to: str | None,
    write_domain: Sequence[str],
    acceptance: Sequence[str],
    actor: str,
    scope_revision: int | None = None,
    alignment_digest: str | None = None,
) -> dict[str, Any]:
    if not acceptance:
        raise OrchestraError("Every task requires at least one acceptance criterion")
    if recommended_agent_type not in KNOWN_AGENT_TYPES:
        raise OrchestraError(f"Unknown recommended_agent_type: {recommended_agent_type}")
    if recommended_agent_type not in TASK_AGENT_TYPES:
        raise OrchestraError(f"Role {recommended_agent_type} cannot be assigned an execution task; route it through its phase/decision envelope")
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            project = get_project(conn, project_id)
            revision = int(project["scope_revision"]) if scope_revision is None else scope_revision
            digest = project["alignment_digest"] if alignment_digest is None else alignment_digest
            if revision != int(project["scope_revision"]) or digest != project["alignment_digest"]:
                raise OrchestraError("Task must be created against the current project scope revision and alignment digest")
            now = iso()
            conn.execute(
                "INSERT INTO tasks(task_id, project_id, title, workstream, milestone, scope_revision, alignment_digest, status, recommended_agent_type, reports_to, context_ref, write_domain_json, acceptance_json, current_attempt_id, result_ref, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, 'ready', ?, ?, ?, ?, ?, NULL, NULL, ?, ?)",
                (task_id, project_id, title, workstream, milestone, revision, digest, recommended_agent_type, reports_to, context_ref, json.dumps(list(write_domain)), json.dumps(list(acceptance)), now, now),
            )
            event = emit_event(conn, project_id=project_id, event_type="task.ready", actor=actor, payload={"task_id": task_id, "title": title, "workstream": workstream, "milestone": milestone, "scope_revision": revision, "alignment_digest": digest, "context_ref": context_ref, "recommended_agent_type": recommended_agent_type, "write_domain": list(write_domain), "acceptance": list(acceptance)})
            conn.commit()
        except sqlite3.IntegrityError as exc:
            conn.rollback()
            raise OrchestraError(f"Could not create task {task_id}: {exc}") from exc
        except Exception:
            conn.rollback()
            raise
        return {"task_id": task_id, "status": "ready", "scope_revision": revision, "alignment_digest": digest, "event": event}
    finally:
        conn.close()


def _expire_lease_if_needed(conn: sqlite3.Connection, task_id: str, now: datetime) -> bool:
    lease = conn.execute("SELECT * FROM leases WHERE task_id = ?", (task_id,)).fetchone()
    if lease is None or parse_iso(lease["expires_at"]) > now:
        return False
    timestamp = iso(now)
    conn.execute("DELETE FROM leases WHERE task_id = ?", (task_id,))
    conn.execute("UPDATE attempts SET status = 'expired', ended_at = ?, updated_at = ? WHERE attempt_id = ?", (timestamp, timestamp, lease["attempt_id"]))
    conn.execute("UPDATE tasks SET status = 'ready', current_attempt_id = NULL, updated_at = ? WHERE task_id = ? AND status = 'in_progress'", (timestamp, task_id))
    emit_event(conn, project_id=lease["project_id"], event_type="lease.expired", actor="conductor", payload={"task_id": task_id, "attempt_id": lease["attempt_id"], "agent_id": lease["agent_id"], "expires_at": lease["expires_at"]})
    return True


def _normalize_domain_path(value: str) -> str:
    value = value.strip().replace("\\", "/")
    while value.startswith("./"):
        value = value[2:]
    value = value.rstrip("/")
    wildcard_positions = [pos for ch in ("*", "?", "[") if (pos := value.find(ch)) >= 0]
    if wildcard_positions:
        value = value[: min(wildcard_positions)].rstrip("/")
    return value or "."


def _domains_overlap(left: Iterable[str], right: Iterable[str]) -> list[tuple[str, str]]:
    overlaps: list[tuple[str, str]] = []
    normalized_left = [(raw, _normalize_domain_path(raw)) for raw in left]
    normalized_right = [(raw, _normalize_domain_path(raw)) for raw in right]
    for left_raw, left_path in normalized_left:
        for right_raw, right_path in normalized_right:
            if (
                left_path == "."
                or right_path == "."
                or left_path == right_path
                or left_path.startswith(right_path + "/")
                or right_path.startswith(left_path + "/")
            ):
                overlaps.append((left_raw, right_raw))
    return overlaps


def claim_task(
    root: Path,
    *,
    project_id: str,
    task_id: str,
    agent_id: str,
    ttl_seconds: int,
    attempt_id: str | None = None,
) -> dict[str, Any]:
    if ttl_seconds <= 0:
        raise OrchestraError("ttl_seconds must be positive")
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            project = get_project(conn, project_id)
            task = conn.execute("SELECT * FROM tasks WHERE project_id = ? AND task_id = ?", (project_id, task_id)).fetchone()
            if task is None:
                raise OrchestraError(f"Unknown task: {task_id}")
            now_dt = utc_now()
            _expire_lease_if_needed(conn, task_id, now_dt)
            task = conn.execute("SELECT * FROM tasks WHERE task_id = ?", (task_id,)).fetchone()
            if task["status"] != "ready":
                raise OrchestraError(f"Task {task_id} is not claimable; status={task['status']}")
            if task["scope_revision"] != project["scope_revision"] or task["alignment_digest"] != project["alignment_digest"]:
                conn.execute("UPDATE tasks SET status = 'stale', updated_at = ? WHERE task_id = ?", (iso(now_dt), task_id))
                raise OrchestraError(f"Task {task_id} is stale and must be repackaged")
            overlap = conn.execute(
                "SELECT t.task_id, t.write_domain_json FROM tasks t JOIN leases l ON l.task_id = t.task_id WHERE t.project_id = ? AND t.task_id <> ?",
                (project_id, task_id),
            ).fetchall()
            requested_paths = list(json.loads(task["write_domain_json"]))
            for active in overlap:
                active_paths = list(json.loads(active["write_domain_json"]))
                conflicts = _domains_overlap(requested_paths, active_paths) if requested_paths and active_paths else []
                if conflicts:
                    formatted = [f"{left} <-> {right}" for left, right in conflicts]
                    raise OrchestraError(f"Write-domain overlap with active task {active['task_id']}: {formatted}")
            attempt = attempt_id or f"attempt-{uuid.uuid4()}"
            token = f"lease-{uuid.uuid4()}"
            now = iso(now_dt)
            expires = iso(now_dt + timedelta(seconds=ttl_seconds))
            conn.execute(
                "INSERT INTO attempts(attempt_id, task_id, project_id, agent_id, status, started_at, ended_at, result_ref, error_class, created_at, updated_at) VALUES (?, ?, ?, ?, 'running', ?, NULL, NULL, NULL, ?, ?)",
                (attempt, task_id, project_id, agent_id, now, now, now),
            )
            conn.execute(
                "INSERT INTO leases(task_id, attempt_id, project_id, agent_id, lease_token, expires_at, heartbeat_at, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                (task_id, attempt, project_id, agent_id, token, expires, now, now),
            )
            conn.execute("UPDATE tasks SET status = 'in_progress', current_attempt_id = ?, updated_at = ? WHERE task_id = ?", (attempt, now, task_id))
            event = emit_event(conn, project_id=project_id, event_type="task.claimed", actor=agent_id, payload={"task_id": task_id, "attempt_id": attempt, "expires_at": expires})
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"task_id": task_id, "attempt_id": attempt, "lease_token": token, "expires_at": expires, "event": event}
    finally:
        conn.close()


def heartbeat_task(root: Path, *, project_id: str, task_id: str, lease_token: str, ttl_seconds: int) -> dict[str, Any]:
    if ttl_seconds <= 0:
        raise OrchestraError("ttl_seconds must be positive")
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            lease = conn.execute("SELECT * FROM leases WHERE project_id = ? AND task_id = ?", (project_id, task_id)).fetchone()
            if lease is None or lease["lease_token"] != lease_token:
                raise OrchestraError("Invalid or missing lease token")
            now_dt = utc_now()
            if parse_iso(lease["expires_at"]) <= now_dt:
                _expire_lease_if_needed(conn, task_id, now_dt)
                raise OrchestraError("Lease has expired")
            heartbeat = iso(now_dt)
            expires = iso(now_dt + timedelta(seconds=ttl_seconds))
            conn.execute("UPDATE leases SET heartbeat_at = ?, expires_at = ? WHERE task_id = ?", (heartbeat, expires, task_id))
            conn.execute("UPDATE attempts SET updated_at = ? WHERE attempt_id = ?", (heartbeat, lease["attempt_id"]))
            event = emit_event(conn, project_id=project_id, event_type="lease.heartbeat", actor=lease["agent_id"], payload={"task_id": task_id, "attempt_id": lease["attempt_id"], "expires_at": expires})
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"task_id": task_id, "attempt_id": lease["attempt_id"], "expires_at": expires, "event": event}
    finally:
        conn.close()


def submit_task(
    root: Path,
    *,
    project_id: str,
    task_id: str,
    lease_token: str,
    result_ref: str,
    alignment_digest: str,
) -> dict[str, Any]:
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            project = get_project(conn, project_id)
            lease = conn.execute("SELECT * FROM leases WHERE project_id = ? AND task_id = ?", (project_id, task_id)).fetchone()
            if lease is None or lease["lease_token"] != lease_token:
                raise OrchestraError("Invalid or missing lease token")
            now_dt = utc_now()
            if parse_iso(lease["expires_at"]) <= now_dt:
                _expire_lease_if_needed(conn, task_id, now_dt)
                raise OrchestraError("Lease expired; late result retained externally but cannot transition task")
            task = conn.execute("SELECT * FROM tasks WHERE task_id = ?", (task_id,)).fetchone()
            if alignment_digest != project["alignment_digest"] or task["alignment_digest"] != project["alignment_digest"]:
                conn.execute("UPDATE tasks SET status = 'stale', updated_at = ? WHERE task_id = ?", (iso(now_dt), task_id))
                conn.execute("UPDATE attempts SET status = 'rejected_stale', ended_at = ?, result_ref = ?, updated_at = ? WHERE attempt_id = ?", (iso(now_dt), result_ref, iso(now_dt), lease["attempt_id"]))
                conn.execute("DELETE FROM leases WHERE task_id = ?", (task_id,))
                emit_event(conn, project_id=project_id, event_type="task.rejected_stale", actor="conductor", payload={"task_id": task_id, "attempt_id": lease["attempt_id"], "result_ref": result_ref, "submitted_digest": alignment_digest, "current_digest": project["alignment_digest"]})
                conn.commit()
                raise OrchestraError("Result rejected because its alignment digest is stale")
            now = iso(now_dt)
            conn.execute("UPDATE attempts SET status = 'submitted', ended_at = ?, result_ref = ?, updated_at = ? WHERE attempt_id = ?", (now, result_ref, now, lease["attempt_id"]))
            conn.execute("UPDATE tasks SET status = 'in_review', result_ref = ?, updated_at = ? WHERE task_id = ?", (result_ref, now, task_id))
            conn.execute("DELETE FROM leases WHERE task_id = ?", (task_id,))
            event = emit_event(conn, project_id=project_id, event_type="task.submitted", actor=lease["agent_id"], payload={"task_id": task_id, "attempt_id": lease["attempt_id"], "result_ref": result_ref})
            conn.commit()
        except OrchestraError:
            # A stale rejection may already have committed the durable rejection.
            if conn.in_transaction:
                conn.rollback()
            raise
        except Exception:
            conn.rollback()
            raise
        return {"task_id": task_id, "attempt_id": lease["attempt_id"], "status": "in_review", "result_ref": result_ref, "event": event}
    finally:
        conn.close()


def decide_task_review(
    root: Path,
    *,
    project_id: str,
    task_id: str,
    actor: str,
    decision: str,
    reason: str,
) -> dict[str, Any]:
    if decision not in {"integrated", "done", "repair", "cancelled"}:
        raise OrchestraError("review decision must be integrated, done, repair, or cancelled")
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            task = conn.execute("SELECT * FROM tasks WHERE project_id = ? AND task_id = ?", (project_id, task_id)).fetchone()
            if task is None:
                raise OrchestraError(f"Unknown task: {task_id}")
            if task["status"] != "in_review":
                raise OrchestraError(f"Task {task_id} is not in_review; status={task['status']}")
            target = "ready" if decision == "repair" else decision
            now = iso()
            current_attempt = task["current_attempt_id"]
            if decision == "repair":
                if current_attempt:
                    conn.execute("UPDATE attempts SET status = 'rejected_semantic', error_class = 'semantic', updated_at = ? WHERE attempt_id = ?", (now, current_attempt))
                conn.execute("UPDATE tasks SET status = 'ready', current_attempt_id = NULL, result_ref = NULL, updated_at = ? WHERE task_id = ?", (now, task_id))
            else:
                if current_attempt:
                    conn.execute("UPDATE attempts SET status = 'accepted', updated_at = ? WHERE attempt_id = ?", (now, current_attempt))
                conn.execute("UPDATE tasks SET status = ?, updated_at = ? WHERE task_id = ?", (target, now, task_id))
            event = emit_event(conn, project_id=project_id, event_type=f"task.{decision}", actor=actor, payload={"task_id": task_id, "attempt_id": current_attempt, "reason": reason, "result_ref": task["result_ref"]})
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"task_id": task_id, "decision": decision, "status": target, "event": event}
    finally:
        conn.close()


def release_task(
    root: Path,
    *,
    project_id: str,
    task_id: str,
    lease_token: str,
    error_class: str,
    reason: str,
) -> dict[str, Any]:
    allowed = {"transient", "semantic", "stale_context", "authority_access", "safety_policy", "cancelled"}
    if error_class not in allowed:
        raise OrchestraError(f"error_class must be one of: {', '.join(sorted(allowed))}")
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            lease = conn.execute("SELECT * FROM leases WHERE project_id = ? AND task_id = ?", (project_id, task_id)).fetchone()
            if lease is None or lease["lease_token"] != lease_token:
                raise OrchestraError("Invalid or missing lease token")
            now = iso()
            attempt_status = {
                "transient": "failed_transient",
                "semantic": "failed_semantic",
                "stale_context": "rejected_stale",
                "authority_access": "blocked",
                "safety_policy": "blocked_policy",
                "cancelled": "cancelled",
            }[error_class]
            task_status = {
                "transient": "ready",
                "semantic": "failed",
                "stale_context": "stale",
                "authority_access": "failed",
                "safety_policy": "failed",
                "cancelled": "cancelled",
            }[error_class]
            conn.execute("UPDATE attempts SET status = ?, ended_at = ?, error_class = ?, updated_at = ? WHERE attempt_id = ?", (attempt_status, now, error_class, now, lease["attempt_id"]))
            conn.execute("UPDATE tasks SET status = ?, current_attempt_id = NULL, updated_at = ? WHERE task_id = ?", (task_status, now, task_id))
            conn.execute("DELETE FROM leases WHERE task_id = ?", (task_id,))
            event = emit_event(conn, project_id=project_id, event_type=f"task.released.{error_class}", actor=lease["agent_id"], payload={"task_id": task_id, "attempt_id": lease["attempt_id"], "reason": reason, "next_status": task_status})
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"task_id": task_id, "attempt_id": lease["attempt_id"], "status": task_status, "error_class": error_class, "event": event}
    finally:
        conn.close()


def reconcile(root: Path, *, project_id: str) -> dict[str, Any]:
    conn = connect(root)
    expired: list[dict[str, Any]] = []
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            now = utc_now()
            rows = conn.execute("SELECT task_id, attempt_id, agent_id, expires_at FROM leases WHERE project_id = ?", (project_id,)).fetchall()
            for row in rows:
                if parse_iso(row["expires_at"]) <= now:
                    if _expire_lease_if_needed(conn, row["task_id"], now):
                        expired.append(dict(row))
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return {"project_id": project_id, "expired_count": len(expired), "expired": expired}
    finally:
        conn.close()


def emit_manual_event(
    root: Path,
    *,
    project_id: str,
    event_type: str,
    actor: str,
    payload: dict[str, Any],
    idempotency_key: str | None,
) -> dict[str, Any]:
    conn = connect(root)
    try:
        conn.execute("BEGIN IMMEDIATE")
        try:
            get_project(conn, project_id)
            event = emit_event(conn, project_id=project_id, event_type=event_type, actor=actor, payload=payload, idempotency_key=idempotency_key)
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        return event
    finally:
        conn.close()


def status(root: Path, *, project_id: str) -> dict[str, Any]:
    conn = connect(root)
    try:
        project = dict(get_project(conn, project_id))
        task_rows = conn.execute("SELECT status, COUNT(*) AS n FROM tasks WHERE project_id = ? GROUP BY status", (project_id,)).fetchall()
        task_counts = {row["status"]: row["n"] for row in task_rows}
        agent_rows = conn.execute("SELECT status, COUNT(*) AS n FROM agents WHERE project_id = ? GROUP BY status", (project_id,)).fetchall()
        agent_counts = {row["status"]: row["n"] for row in agent_rows}
        leases = [dict(row) for row in conn.execute("SELECT task_id, attempt_id, agent_id, expires_at, heartbeat_at FROM leases WHERE project_id = ? ORDER BY expires_at", (project_id,)).fetchall()]
        recent_events = []
        for row in conn.execute("SELECT seq, event_type, actor, payload_json, created_at FROM events WHERE project_id = ? ORDER BY seq DESC LIMIT 10", (project_id,)).fetchall():
            item = dict(row)
            item["payload"] = json.loads(item.pop("payload_json"))
            recent_events.append(item)
        return {"project": project, "tasks": task_counts, "agents": agent_counts, "active_leases": leases, "recent_events": recent_events}
    finally:
        conn.close()


def load_payload(value: str | None, path: str | None) -> dict[str, Any]:
    if value and path:
        raise OrchestraError("Use either --payload or --payload-file, not both")
    if path:
        try:
            data = json.loads(Path(path).read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as exc:
            raise OrchestraError(f"Could not load payload file: {exc}") from exc
    elif value:
        try:
            data = json.loads(value)
        except json.JSONDecodeError as exc:
            raise OrchestraError(f"Invalid JSON payload: {exc}") from exc
    else:
        data = {}
    if not isinstance(data, dict):
        raise OrchestraError("Event payload must be a JSON object")
    return data


def print_json(value: Any) -> None:
    print(json.dumps(value, indent=2, sort_keys=True))


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Codex Orchestra reference control-plane utility")
    parser.add_argument("--root", default=".", help="Repository root or a path inside it")
    parser.add_argument("--project-id", default="default")
    sub = parser.add_subparsers(dest="command", required=True)

    sub.add_parser("init", help="Initialize the runtime database and project record")
    sub.add_parser("doctor", help="Validate checked-in framework contracts")

    digest = sub.add_parser("digest", help="Compute or record the alignment digest")
    digest.add_argument("--record", action="store_true")
    digest.add_argument("--actor", default="conductor")
    digest.add_argument("--affected-task", action="append", default=[])
    digest.add_argument("--idempotency-key")

    phase = sub.add_parser("phase", help="Read or change project phase")
    phase_sub = phase.add_subparsers(dest="phase_command", required=True)
    phase_sub.add_parser("get")
    phase_set = phase_sub.add_parser("set")
    phase_set.add_argument("target")
    phase_set.add_argument("--actor", required=True)
    phase_set.add_argument("--gate-ref", required=True)
    phase_set.add_argument("--expected-current")
    phase_set.add_argument("--idempotency-key")

    scope = sub.add_parser("scope", help="Record a binding scope revision")
    scope_sub = scope.add_subparsers(dest="scope_command", required=True)
    scope_revise = scope_sub.add_parser("revise")
    scope_revise.add_argument("revision", type=int)
    scope_revise.add_argument("--actor", required=True)
    scope_revise.add_argument("--reason", required=True)
    scope_revise.add_argument("--affected-task", action="append", default=[])
    scope_revise.add_argument("--idempotency-key")

    agent = sub.add_parser("agent", help="Register/update a logical agent record")
    agent_sub = agent.add_subparsers(dest="agent_command", required=True)
    agent_register = agent_sub.add_parser("register")
    agent_register.add_argument("--agent-id", required=True)
    agent_register.add_argument("--role", required=True)
    agent_register.add_argument("--reports-to")
    agent_register.add_argument("--physical-parent-id")
    agent_register.add_argument("--task-name")
    agent_register.add_argument("--delegation-permit-ref")
    agent_register.add_argument("--status", default="idle")
    agent_register.add_argument("--config-ref", required=True)
    agent_register.add_argument("--role-card-ref")
    agent_register.add_argument("--thread-id")
    agent_register.add_argument("--actor", default="conductor")

    task = sub.add_parser("task", help="Create, lease, submit, and review tasks")
    task_sub = task.add_subparsers(dest="task_command", required=True)
    task_add = task_sub.add_parser("add")
    task_add.add_argument("--task-id", required=True)
    task_add.add_argument("--title", required=True)
    task_add.add_argument("--workstream", required=True)
    task_add.add_argument("--milestone", required=True)
    task_add.add_argument("--context-ref", required=True)
    task_add.add_argument("--agent-type", required=True)
    task_add.add_argument("--reports-to")
    task_add.add_argument("--write-domain", action="append", default=[])
    task_add.add_argument("--acceptance", action="append", required=True)
    task_add.add_argument("--actor", required=True)
    task_add.add_argument("--scope-revision", type=int)
    task_add.add_argument("--alignment-digest")

    task_claim = task_sub.add_parser("claim")
    task_claim.add_argument("--task-id", required=True)
    task_claim.add_argument("--agent-id", required=True)
    task_claim.add_argument("--ttl", type=int, default=900)
    task_claim.add_argument("--attempt-id")

    task_heartbeat = task_sub.add_parser("heartbeat")
    task_heartbeat.add_argument("--task-id", required=True)
    task_heartbeat.add_argument("--lease-token", required=True)
    task_heartbeat.add_argument("--ttl", type=int, default=900)

    task_submit = task_sub.add_parser("submit")
    task_submit.add_argument("--task-id", required=True)
    task_submit.add_argument("--lease-token", required=True)
    task_submit.add_argument("--result-ref", required=True)
    task_submit.add_argument("--alignment-digest", required=True)

    task_review = task_sub.add_parser("review")
    task_review.add_argument("--task-id", required=True)
    task_review.add_argument("--actor", required=True)
    task_review.add_argument("--decision", required=True, choices=["integrated", "done", "repair", "cancelled"])
    task_review.add_argument("--reason", required=True)

    task_release = task_sub.add_parser("release")
    task_release.add_argument("--task-id", required=True)
    task_release.add_argument("--lease-token", required=True)
    task_release.add_argument("--error-class", required=True)
    task_release.add_argument("--reason", required=True)

    sub.add_parser("reconcile", help="Expire orphaned leases and return tasks to ready")
    sub.add_parser("status", help="Print a concise runtime projection")

    event = sub.add_parser("event", help="Append a manual typed event")
    event_sub = event.add_subparsers(dest="event_command", required=True)
    event_emit = event_sub.add_parser("emit")
    event_emit.add_argument("--type", required=True)
    event_emit.add_argument("--actor", required=True)
    event_emit.add_argument("--payload")
    event_emit.add_argument("--payload-file")
    event_emit.add_argument("--idempotency-key")

    return parser


def main(argv: Sequence[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        root = repo_root(args.root)
        project_id = args.project_id

        if args.command == "init":
            print_json(initialize(root, project_id))
        elif args.command == "doctor":
            errors, warnings = doctor(root)
            print_json({"ok": not errors, "errors": errors, "warnings": warnings})
            return 1 if errors else 0
        elif args.command == "digest":
            if args.record:
                print_json(record_digest(root, project_id=project_id, actor=args.actor, affected_tasks=args.affected_task, idempotency_key=args.idempotency_key))
            else:
                digest, included = compute_alignment_digest(root)
                print_json({"alignment_digest": digest, "included_files": included})
        elif args.command == "phase":
            if args.phase_command == "get":
                conn = connect(root)
                try:
                    print_json(dict(get_project(conn, project_id)))
                finally:
                    conn.close()
            else:
                print_json(set_phase(root, project_id=project_id, target=args.target, actor=args.actor, gate_ref=args.gate_ref, expected_current=args.expected_current, idempotency_key=args.idempotency_key))
        elif args.command == "scope":
            print_json(revise_scope(root, project_id=project_id, revision=args.revision, actor=args.actor, reason=args.reason, affected_tasks=args.affected_task, idempotency_key=args.idempotency_key))
        elif args.command == "agent":
            print_json(register_agent(root, project_id=project_id, agent_id=args.agent_id, role=args.role, reports_to=args.reports_to, status=args.status, config_ref=args.config_ref, role_card_ref=args.role_card_ref, thread_id=args.thread_id, actor=args.actor, physical_parent_id=args.physical_parent_id, task_name=args.task_name, delegation_permit_ref=args.delegation_permit_ref))
        elif args.command == "task":
            if args.task_command == "add":
                print_json(add_task(root, project_id=project_id, task_id=args.task_id, title=args.title, workstream=args.workstream, milestone=args.milestone, context_ref=args.context_ref, recommended_agent_type=args.agent_type, reports_to=args.reports_to, write_domain=args.write_domain, acceptance=args.acceptance, actor=args.actor, scope_revision=args.scope_revision, alignment_digest=args.alignment_digest))
            elif args.task_command == "claim":
                print_json(claim_task(root, project_id=project_id, task_id=args.task_id, agent_id=args.agent_id, ttl_seconds=args.ttl, attempt_id=args.attempt_id))
            elif args.task_command == "heartbeat":
                print_json(heartbeat_task(root, project_id=project_id, task_id=args.task_id, lease_token=args.lease_token, ttl_seconds=args.ttl))
            elif args.task_command == "submit":
                print_json(submit_task(root, project_id=project_id, task_id=args.task_id, lease_token=args.lease_token, result_ref=args.result_ref, alignment_digest=args.alignment_digest))
            elif args.task_command == "review":
                print_json(decide_task_review(root, project_id=project_id, task_id=args.task_id, actor=args.actor, decision=args.decision, reason=args.reason))
            elif args.task_command == "release":
                print_json(release_task(root, project_id=project_id, task_id=args.task_id, lease_token=args.lease_token, error_class=args.error_class, reason=args.reason))
        elif args.command == "reconcile":
            print_json(reconcile(root, project_id=project_id))
        elif args.command == "status":
            print_json(status(root, project_id=project_id))
        elif args.command == "event":
            payload = load_payload(args.payload, args.payload_file)
            print_json(emit_manual_event(root, project_id=project_id, event_type=args.type, actor=args.actor, payload=payload, idempotency_key=args.idempotency_key))
        else:
            parser.error(f"Unhandled command: {args.command}")
        return 0
    except OrchestraError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
