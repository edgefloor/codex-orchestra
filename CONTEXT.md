# Codex Orchestra

## Domain language

**Workflow source**: A restricted `.workflow.ts` file using only the Orchestra import, literals, references, arrays, objects, templates, and approved DSL calls.

**Execution plan**: The validated internal Rust representation produced without executing TypeScript.

**Run**: One runtime-owned execution of a plan against a repository revision and parent Codex task.

**Step**: One agent, check, or approval action with dependencies, attempt bounds, optional repeat bounds, context, outputs, and worktree policy.

**Stage**: Dependency-ready steps executed concurrently up to `max_parallel`.

**Context bundle**: Exact bytes materialized from declared files, line ranges, revisions, diffs, and dependency outputs, with a SHA-256 digest.

**Native host**: The narrow Codex capability for parent-linked V2 spawn, status, wait, final response, cancellation, and sandboxed command execution.

**Checkpoint**: Atomic runtime state recording attempts, rounds, statuses, context hashes, validated outputs, evidence, and approvals.

**Approval**: An explicit human decision that pauses a run and can be supplied only during resume.

**Run summary**: The transcript-independent terminal or paused record under `.codex/orchestra/runs/<run-id>/summary.md`.

## Invariants

- TypeScript workflow source is parsed, never evaluated.
- The runtime, not a model, owns scheduling and durable state.
- Agent steps use the active task's V2 `AgentControl`; there is no alternate dispatcher.
- `fork_turns` defaults to `none`; exact declared context replaces the parent transcript.
- Models and reasoning settings are step data, not fixed role personalities.
- Installed plugin and runtime artifacts are immutable. Mutable snapshots and state live only in the target repository.
- Stock Codex cannot load this Rust extension dynamically today; the pinned integration patch is explicit and temporary.
