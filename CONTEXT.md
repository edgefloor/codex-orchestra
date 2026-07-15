# Orchestra

## Domain language

**Workflow source**: A restricted `.workflow.ts` file using only the Orchestra import, literals, references, arrays, objects, templates, and approved DSL calls.

**Execution plan**: The validated internal Rust representation produced without executing TypeScript.

**Run**: One runtime-owned execution of a plan against a repository revision and parent Codex task.

**Step**: One agent, check, or approval action with dependencies, attempt bounds, optional repeat bounds, context, outputs, and worktree policy.

**Stage**: Dependency-ready steps executed concurrently up to `max_parallel`.

**Context bundle**: Exact bytes materialized from declared files, line ranges, revisions, diffs, and dependency outputs, with a SHA-256 digest.

**Run input**: A typed, run-specific value resolved before scheduling, canonically serialized, hashed, and persisted independently of the parent transcript.

**Skill requirement**: An exact enabled skill identity plus its declared transitive skills and resources, resolved through the native host and snapshotted before an agent starts.

**Human input**: A durable free-text or structured response that resumes a paused workflow without granting acceptance authority.

**Native host**: The narrow Codex capability for parent-linked V2 spawn, status, wait, final response, cancellation, and sandboxed command execution.

**Checkpoint**: Atomic runtime state recording attempts, rounds, statuses, context hashes, validated outputs, evidence, and approvals.

**Approval**: An explicit human decision that pauses a run. The first declared choice continues an accepted run; any other declared choice rejects and cancels it.

**External effect**: A declared mutation outside the target checkout, such as publishing or closing an issue, bounded by explicit authority and recorded with a reconciliation receipt.

**Promotion**: Conflict-checked application of the aggregate verified isolated-worktree patch into the target checkout after every step and approval succeeds.

**Run summary**: The transcript-independent terminal or paused record under `.codex/orchestra/runs/<run-id>/summary.md`.

## Invariants

- TypeScript workflow source is parsed, never evaluated.
- The runtime, not a model, owns scheduling and durable state.
- Agent steps use the active task's V2 `AgentControl`; there is no alternate dispatcher.
- `fork_turns` defaults to `none`; exact declared context replaces the parent transcript.
- Models and reasoning settings are step data, not fixed role personalities.
- Installed plugin and runtime artifacts are immutable. Mutable snapshots and state live only in the target repository.
- Resumed runs use their recorded inputs, skill snapshots, human responses, and external-effect receipts rather than re-resolving mutable ambient state.
- Verified isolated changes reach the target checkout only after successful checks and acceptance; rejection or a promotion conflict never overwrites target files.
- Stock Codex cannot load this Rust extension dynamically today; the pinned integration patch is explicit and temporary.
