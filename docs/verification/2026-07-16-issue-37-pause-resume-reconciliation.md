# Issue 37 — pause, resume, and reconciliation

Status: automated implementation evidence passed; live Linear and visible desktop interaction remain human-only.

## Implemented contract

- Automation checkpoints carry a monotonically advancing lease epoch and revision. Every write compares both values with durable state; a stale checkpoint or provider callback cannot commit.
- Pause durably advances the epoch before native Child Runs are interrupted. Executing Tracker effects become ambiguous rather than being replayed.
- The native Workflow runtime now pauses active agents into the existing checkpoint and resumes the same Run ID.
- A graceful Codex host shutdown fences every tracked running Automation Root Run. A later refresh or resume also fences a checkpoint still marked running before reconciliation.
- Resume has an explicit required/in-progress/blocked/complete reconciliation state. It verifies the pinned profile digest and lease, refreshes Tracker state, inspects retained worktrees, Issue tasks, Child Runs, and effect receipts, and only returns the Root Run to `running` after the pass completes.
- Existing worktree, Issue-task, Child-Run, claim, and receipt identities are retained. Missing retained resources block reconciliation instead of causing replacement creation.
- An externally terminal Tracker issue cancels descendants first. Ambiguous effects keep the claim out of cleanup eligibility.
- `automation/status`, `automation/pause`, `automation/refresh`, and `automation/resume` are typed App Server operations. T3Code routes them through its existing provider boundary and renders normal task-dialog actions.

## Automated evidence

- Root workspace: `cargo fmt --all -- --check`, 90 executed tests passed, 5 evaluator integration tests remained intentionally ignored, and lifecycle doctor passed with four skills.
- Core lifecycle tests cover epoch fencing, stale provider results, retained identity reuse, terminal reconciliation, and active native Workflow pause/resume.
- Clean pinned Codex patch application passed. The clean Codex checkout compiled `codex-orchestra-extension` and `codex-app-server`; the working pinned checkout passed 69 core, 11 extension, and 265 protocol tests plus schema-fixture checks.
- Clean pinned T3Code patch application passed. Contracts, server, and web typechecks passed; 35 server tests and 4 Automation dialog logic tests passed.

## Human-only checks

- A real Linear credential was not present, so provider-backed refresh and external terminal cancellation were not exercised against Linear.
- The desktop buttons were typechecked but not visually clicked in a launched Electron window.
- Graceful shutdown is covered by the Codex service drop fence. An uncatchable process kill cannot run cleanup code; the next typed refresh/resume fences the still-running durable checkpoint before reconciliation.
