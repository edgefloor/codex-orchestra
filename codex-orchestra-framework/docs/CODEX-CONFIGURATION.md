# Codex Configuration Guide

## Project configuration

```toml
model = "gpt-5.6-sol"
model_reasoning_effort = "medium"

[features.multi_agent_v2]
enabled = true
hide_spawn_agent_metadata = false
expose_spawn_agent_model_overrides = true
max_concurrent_threads_per_session = 10
tool_namespace = "collaboration"

[agents]
max_threads = 10
max_depth = 3
job_max_runtime_seconds = 1800
interrupt_message = true
```

Visible spawn metadata/model overrides are deliberate: each permit/command must record the effective role, model, effort, and parent path.

## Why depth 3

Depth is used as a context boundary, not permission for arbitrary recursion:

```text
root(0) -> phase owner/manager(1) -> Team Leader/service(2) -> leaf worker/reviewer(3)
```

Role-to-child allowlists, permits, and budgets are stricter than the raw depth setting. Leaves cannot spawn.

## Custom TOMLs

Each `.codex/agents/*.toml` contains:

- `name` and task-discovery `description`;
- model and reasoning default;
- least-privilege sandbox;
- readable nickname candidates;
- durable authority/personality instructions.

Task-specific expertise belongs in a Role Card and Context Capsule, not a new TOML per task.

## Persistent identity

Persistent means stable TOML + canonical path + roster/memory + eval. Reuse a resident agent with `followup_task` only when the role/config/security boundary and durable handoff remain valid. Start fresh after material scope/security/role changes.

## Capability probe

After a Codex upgrade:

1. confirm V2 tools and configured namespace;
2. inspect `spawn_agent` fields for agent/model/effort/fork support;
3. confirm max depth/thread behavior;
4. spawn one read-only child with `fork_turns: none`;
5. from an authorized parent, spawn one permitted leaf and confirm completion reaches that parent;
6. verify a leaf cannot exceed depth/role policy;
7. confirm `send_message` is passive, `followup_task` wakes, and `wait_agent` is event-driven;
8. record the probe version/result before production use.

## Effective configuration snapshot

Every attempt records model, effort, sandbox, cwd/worktree, tools/MCP, Role Card, Context Capsule, permit, base revision, and output schema. Changing them creates a new attempt.
