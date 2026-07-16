# Issue #40 Issue worktree lifecycle

Status: implemented at the runtime, pinned Codex, generated protocol, and normal T3Code Automation
panel seams.

## Implemented contract

- Claim worktree names combine a bounded safe Issue identifier, a stable Issue hash, and the
  Attempt number. The runtime resolves the configured workspace root first and rejects any path
  that is not a contained child.
- `after_create`, `before_run`, `after_run`, and `before_remove` execute through native Codex
  command control in the retained Issue worktree. Each invocation persists only a command digest,
  status, exit code, bounded output previews, and a bounded failure—not the command source.
- Retry, continuation, pause, resume, and reconciliation retain the same claim, native Issue task,
  worktree, and recorded source revision. An existing worktree is reusable only when native Git
  reports the exact recorded base; a stale path fails instead of being silently replaced.
- Externally terminal claims become cleanup-eligible only after native descendants have stopped and
  no Tracker effect remains executing or ambiguous. Cleanup runs the pinned profile's removal hook,
  then native Git removal; failures persist as `retry_pending` and are retried by a later native
  reconciliation.
- Handoff and explicit cancellation retain resources for inspection. Successful terminal cleanup
  removes the worktree but keeps the claim, hook receipts, cleanup receipt, effects, and evidence.
- The current per-claim Git worktree remains a transitional safety adapter. This issue does not add
  another store, scheduler, control plane, or agent-writable shared worktree.

## Automated evidence

- Root workspace formatting, 96 executed tests, and lifecycle doctor passed; five evaluator worker
  tests remained intentionally ignored. Coverage includes path escape/collision resistance,
  bounded hook and cleanup receipts, cleanup retry, retained handoff identities, terminal ordering,
  and the existing promotion-conflict preservation case.
- Clean pinned Codex patch application passed; 75 Orchestra core and 12 extension tests passed,
  including exact-base worktree reuse rejection. All 265 App Server protocol tests and both schema
  fixture tests passed, and the pinned App Server compiled.
- Clean pinned T3Code patch application passed. T3Code web typechecking and its 1,285 unit tests
  passed with the new compact hook and cleanup projection.

## Human-only checks

- Hook execution and terminal cleanup were not exercised through a live signed Electron build.
- The hook receipt disclosure and cleanup badges were typechecked but not visually inspected.
