---
name: run-workflow
description: Execute a validated Orchestra YAML workflow through native Codex agents and repository tools. Use when a user supplies or selects a workflow and wants bounded parallel execution, checks, review, approvals, durable state, and a final run summary.
---

# Run a workflow

The active Codex agent is the executor. A workflow is declarative data, not executable code.

1. Validate the workflow and refuse cycles, unknown dependencies, undeclared output references, unbounded repeats, or unsafe parallel writers.
2. Create `.codex/orchestra/runs/<run-id>/`, snapshot the workflow as `workflow.yaml`, record its digest and source revision in `state.json`, and initialize each step as `pending`.
3. Select pending steps whose `needs` are complete. Run independent read-only steps in parallel up to `limits.max_parallel`; use native `spawn_agent` with explicit `fork_turns: none` when available.
4. Give each agent only its step instructions, resolved inputs, repository revision, read/write scope, checks, output contract, attempt limit, and result path. The parent that spawns it waits and records its structured result.
5. Never run concurrent writers in one checkout. For an isolated writer, create a worktree before spawning it and record the absolute workspace path plus candidate revision in the step outputs. A downstream `workspace_from` step must execute in that exact workspace and revision. Do not integrate or package the candidate until its checks, review, and any approval are accepted.
6. Execute `check` commands directly in the declared workspace, record command, working directory, revision, exit status, and output under `evidence/`, and never treat an agent's claim as check evidence.
7. Run review work through a fresh reviewer agent. If it reports a material finding, mark the downstream approval step ready and pause for the user; do not let the reviewer repair its own finding.
8. For `approval`, evaluate `when` from recorded step outputs. Mark a false condition `skipped`. Otherwise persist `waiting_approval`, state the exact decisions and default, then end the turn. A `continue` decision completes the step, `revise` blocks the run for a revised workflow, and `stop` ends the run as failed.
9. Apply repeats only while `until` is false, progress is observed, and `max_rounds` is not exhausted. Record each attempt separately.
10. After every transition, atomically update `state.json`. On completion, write `summary.md` with outputs, changed files, checks, review findings, approvals, and residual risk.

Do not use an MCP server, App Server client, daemon, sidecar, or external scheduler.
