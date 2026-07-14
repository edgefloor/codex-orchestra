#!/usr/bin/env python3
"""Validate Orchestra workflows and initialize durable run state."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

import jsonschema
import yaml

PLUGIN = Path(__file__).resolve().parents[1]
SCHEMA = PLUGIN / "assets/schemas/workflow.schema.json"
REFERENCE = re.compile(r"\$\{steps\.([a-z][a-z0-9-]*)\.([a-z][a-z0-9-]*)\}")
CONDITION = re.compile(
    r"^\$\{steps\.([a-z][a-z0-9-]*)\.([a-z][a-z0-9-]*)\}\s*==\s*(true|false|\"[^\"]*\"|'[^']*')$"
)


class WorkflowError(RuntimeError):
    pass


def load_workflow(path: Path) -> dict:
    try:
        value = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        raise WorkflowError(f"cannot read workflow {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise WorkflowError("workflow root must be an object")
    return value


def schema_errors(workflow: dict) -> list[str]:
    schema = json.loads(SCHEMA.read_text(encoding="utf-8"))
    validator = jsonschema.Draft202012Validator(schema)
    errors: list[str] = []
    for error in sorted(validator.iter_errors(workflow), key=lambda item: list(item.absolute_path)):
        location = ".".join(str(part) for part in error.absolute_path) or "workflow"
        errors.append(f"{location}: {error.message}")
    return errors


def overlaps(left: str, right: str) -> bool:
    left = left.rstrip("/")
    right = right.rstrip("/")
    return left == right or left.startswith(right + "/") or right.startswith(left + "/")


def string_values(value: object):
    if isinstance(value, str):
        yield value
    elif isinstance(value, dict):
        for child in value.values():
            yield from string_values(child)
    elif isinstance(value, list):
        for child in value:
            yield from string_values(child)


def semantic_errors(workflow: dict) -> list[str]:
    steps = workflow.get("steps", [])
    if not isinstance(steps, list):
        return []
    errors: list[str] = []
    ids = [step.get("id") for step in steps if isinstance(step, dict)]
    duplicates = sorted({item for item in ids if item is not None and ids.count(item) > 1})
    errors.extend(f"duplicate step id: {item}" for item in duplicates)
    by_id = {step.get("id"): step for step in steps if isinstance(step, dict) and isinstance(step.get("id"), str)}
    max_steps = workflow.get("limits", {}).get("max_steps") if isinstance(workflow.get("limits"), dict) else None
    if isinstance(max_steps, int) and len(steps) > max_steps:
        errors.append(f"workflow has {len(steps)} steps but limits.max_steps is {max_steps}")

    for step_id, step in by_id.items():
        needs = step.get("needs", [])
        if isinstance(needs, list):
            for dependency in needs:
                if dependency not in by_id:
                    errors.append(f"{step_id}: unknown dependency {dependency}")
                elif dependency == step_id:
                    errors.append(f"{step_id}: step cannot depend on itself")
        if step.get("type") == "agent":
            scopes = step.get("write_scope", [])
            if step.get("read_only") is True and scopes:
                errors.append(f"{step_id}: read-only agent must have an empty write_scope")
            if step.get("read_only") is False and not scopes:
                errors.append(f"{step_id}: writing agent must declare write_scope")
            limit = workflow.get("limits", {}).get("max_attempts") if isinstance(workflow.get("limits"), dict) else None
            if isinstance(limit, int) and isinstance(step.get("max_attempts"), int) and step["max_attempts"] > limit:
                errors.append(f"{step_id}: max_attempts exceeds workflow limit")
        if step.get("type") == "approval" and isinstance(step.get("decisions"), dict):
            if step.get("default") not in step["decisions"]:
                errors.append(f"{step_id}: default approval decision is not declared")
        for text in string_values(step):
            for source, output in REFERENCE.findall(text):
                if source not in by_id:
                    errors.append(f"{step_id}: output reference uses unknown step {source}")
                elif output not in by_id[source].get("outputs", []):
                    errors.append(f"{step_id}: {source} does not declare output {output}")

    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(step_id: str) -> None:
        if step_id in visiting:
            errors.append(f"dependency cycle includes {step_id}")
            return
        if step_id in visited:
            return
        visiting.add(step_id)
        for dependency in by_id[step_id].get("needs", []):
            if dependency in by_id:
                visit(dependency)
        visiting.remove(step_id)
        visited.add(step_id)

    for step_id in by_id:
        visit(step_id)

    def depends_on(left: str, right: str, seen: set[str] | None = None) -> bool:
        seen = set() if seen is None else seen
        if left in seen:
            return False
        seen.add(left)
        needs = by_id[left].get("needs", [])
        return right in needs or any(dep in by_id and depends_on(dep, right, seen) for dep in needs)

    for step_id, step in by_id.items():
        source = step.get("workspace_from")
        if not source:
            continue
        if source not in by_id:
            errors.append(f"{step_id}: workspace_from uses unknown step {source}")
        elif not depends_on(step_id, source):
            errors.append(f"{step_id}: workspace_from must reference a dependency")
        elif not {"workspace", "revision"}.issubset(set(by_id[source].get("outputs", []))):
            errors.append(f"{step_id}: workspace source {source} must declare workspace and revision outputs")

    writers = [
        (step_id, step)
        for step_id, step in by_id.items()
        if step.get("type") == "agent" and step.get("read_only") is False and step.get("worktree") == "shared"
    ]
    for index, (left_id, left) in enumerate(writers):
        for right_id, right in writers[index + 1 :]:
            if depends_on(left_id, right_id) or depends_on(right_id, left_id):
                continue
            if any(overlaps(a, b) for a in left.get("write_scope", []) for b in right.get("write_scope", [])):
                errors.append(f"unsafe parallel writers {left_id} and {right_id} have overlapping shared write scopes")
    return sorted(set(errors))


def validate(path: Path) -> list[str]:
    workflow = load_workflow(path)
    return schema_errors(workflow) + semantic_errors(workflow)


def atomic_json(path: Path, value: dict) -> None:
    temporary = path.with_suffix(path.suffix + ".tmp")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def step_attempt_limit(workflow: dict, step: dict) -> int:
    return int(step.get("max_attempts", workflow["limits"]["max_attempts"]))


def condition_met(expression: str, state: dict) -> bool:
    match = CONDITION.fullmatch(expression.strip())
    if not match:
        raise WorkflowError(f"unsupported condition: {expression}")
    source, output, raw_expected = match.groups()
    source_record = state["steps"].get(source)
    if source_record is None:
        raise WorkflowError(f"condition uses unknown step: {source}")
    if source_record["status"] == "skipped":
        return False
    values = source_record.get("outputs", {})
    if output not in values:
        raise WorkflowError(f"condition output is missing: {source}.{output}")
    if raw_expected == "true":
        expected: object = True
    elif raw_expected == "false":
        expected = False
    else:
        expected = raw_expected[1:-1]
    return values[output] == expected


def ready_step_ids(workflow: dict, state: dict) -> list[str]:
    """Return the next dependency-ready stage, bounded by max_parallel."""
    ready: list[str] = []
    for step in workflow["steps"]:
        record = state["steps"][step["id"]]
        if record["status"] != "pending":
            continue
        if not all(
            state["steps"][dependency]["status"] in {"complete", "skipped"}
            for dependency in step.get("needs", [])
        ):
            continue
        if step.get("when") and not condition_met(step["when"], state):
            record["status"] = "skipped"
            continue
        ready.append(step["id"])
    if all(item["status"] in {"complete", "skipped"} for item in state["steps"].values()):
        state["status"] = "complete"
        state["next_action"] = "Review the run summary."
    return ready[: workflow["limits"]["max_parallel"]]


def start_step(workflow: dict, state: dict, step_id: str) -> None:
    step = next(item for item in workflow["steps"] if item["id"] == step_id)
    record = state["steps"][step_id]
    if record["status"] != "pending" or step_id not in ready_step_ids(workflow, state):
        raise WorkflowError(f"step is not ready: {step_id}")
    if record["attempts"] >= step_attempt_limit(workflow, step):
        record["status"] = "failed"
        state["status"] = "failed"
        raise WorkflowError(f"step attempt limit exhausted: {step_id}")
    if step["type"] == "approval":
        record["status"] = "waiting_approval"
        state["status"] = "waiting_approval"
        state["next_action"] = step["reason"]
        return
    record["attempts"] += 1
    record["status"] = "running"
    state["status"] = "running"


def finish_step(
    workflow: dict,
    state: dict,
    step_id: str,
    *,
    success: bool,
    result: str | None = None,
    outputs: dict | None = None,
) -> None:
    step = next(item for item in workflow["steps"] if item["id"] == step_id)
    record = state["steps"][step_id]
    if record["status"] != "running":
        raise WorkflowError(f"step is not running: {step_id}")
    if success:
        record["status"] = "complete"
        if result:
            record["result"] = result
        if outputs is not None:
            undeclared = sorted(set(outputs) - set(step.get("outputs", [])))
            if undeclared:
                raise WorkflowError(f"step {step_id} returned undeclared outputs: {', '.join(undeclared)}")
            record["outputs"] = outputs
    elif record["attempts"] < step_attempt_limit(workflow, step):
        record["status"] = "pending"
    else:
        record["status"] = "failed"
        state["status"] = "failed"
        state["next_action"] = f"Repair or revise failed step {step_id}."
        return
    if all(item["status"] in {"complete", "skipped"} for item in state["steps"].values()):
        state["status"] = "complete"
        state["next_action"] = "Review the run summary."
    else:
        state["status"] = "running"
        state["next_action"] = "Run dependency-ready steps."


def decide_approval(workflow: dict, state: dict, step_id: str, decision: str) -> None:
    step = next(item for item in workflow["steps"] if item["id"] == step_id)
    record = state["steps"][step_id]
    if record["status"] != "waiting_approval":
        raise WorkflowError(f"step is not waiting for approval: {step_id}")
    if decision not in step["decisions"]:
        raise WorkflowError(f"unsupported approval decision for {step_id}: {decision}")
    record.update(
        {
            "status": "complete",
            "attempts": record["attempts"] + 1,
            "decision": decision,
            "outputs": {"decision": decision},
        }
    )
    outcome = step["decisions"][decision]
    if outcome == "revise":
        state["status"] = "blocked"
        state["next_action"] = f"Revise the workflow to address approval step {step_id}."
    elif outcome == "stop":
        state["status"] = "failed"
        state["next_action"] = f"Run stopped by approval decision at {step_id}."
    elif all(item["status"] in {"complete", "skipped"} for item in state["steps"].values()):
        state["status"] = "complete"
        state["next_action"] = "Review the run summary."
    else:
        state["status"] = "running"
        state["next_action"] = "Run dependency-ready steps."


def recover_interrupted(workflow: dict, state: dict) -> list[str]:
    """Return interrupted steps to pending only when another attempt remains."""
    exhausted: list[str] = []
    by_id = {step["id"]: step for step in workflow["steps"]}
    for step_id, record in state["steps"].items():
        if record["status"] != "running":
            continue
        if record["attempts"] < step_attempt_limit(workflow, by_id[step_id]):
            record["status"] = "pending"
        else:
            record["status"] = "failed"
            exhausted.append(step_id)
    state["status"] = "failed" if exhausted else "running"
    state["next_action"] = (
        f"Repair or revise exhausted steps: {', '.join(exhausted)}."
        if exhausted
        else "Reconcile write scopes, then run dependency-ready steps."
    )
    return exhausted


def should_repeat(step: dict, *, rounds: int, until_met: bool, progress: bool) -> bool:
    repeat = step.get("repeat")
    if not repeat or until_met or rounds >= repeat["max_rounds"]:
        return False
    if repeat.get("stop_on_no_progress", False) and not progress:
        return False
    return True


def initialize(workflow_path: Path, repository: Path, run_id: str) -> Path:
    errors = validate(workflow_path)
    if errors:
        raise WorkflowError("invalid workflow:\n" + "\n".join(f"- {item}" for item in errors))
    workflow = load_workflow(workflow_path)
    run = repository / ".codex/orchestra/runs" / run_id
    if run.exists():
        raise WorkflowError(f"run already exists: {run}")
    (run / "steps").mkdir(parents=True)
    (run / "evidence").mkdir()
    snapshot = run / "workflow.yaml"
    shutil.copy2(workflow_path, snapshot)
    try:
        revision = subprocess.run(
            ["git", "-C", str(repository), "rev-parse", "HEAD"], check=True, capture_output=True, text=True
        ).stdout.strip()
    except (FileNotFoundError, subprocess.CalledProcessError) as exc:
        raise WorkflowError("repository must have a readable Git HEAD") from exc
    now = datetime.now(timezone.utc).isoformat()
    state = {
        "run_id": run_id,
        "workflow": "workflow.yaml",
        "workflow_digest": hashlib.sha256(snapshot.read_bytes()).hexdigest(),
        "source_revision": revision,
        "status": "pending",
        "steps": {step["id"]: {"status": "pending", "attempts": 0} for step in workflow["steps"]},
        "updated_at": now,
        "next_action": "Run dependency-ready steps.",
    }
    atomic_json(run / "state.json", state)
    return run


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    commands = result.add_subparsers(dest="command", required=True)
    check = commands.add_parser("validate")
    check.add_argument("workflow", type=Path)
    init = commands.add_parser("init-run")
    init.add_argument("workflow", type=Path)
    init.add_argument("--repository", type=Path, required=True)
    init.add_argument("--run-id", required=True)
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "validate":
            errors = validate(args.workflow)
            for error in errors:
                print(f"ERROR {error}")
            if not errors:
                print(f"OK {args.workflow}")
            return 1 if errors else 0
        run = initialize(args.workflow.resolve(), args.repository.resolve(), args.run_id)
        print(f"OK initialized {run}")
        return 0
    except WorkflowError as exc:
        print(f"ERROR {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
