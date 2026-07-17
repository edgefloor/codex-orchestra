# Issue #42 — Symphony-compatible Automation acceptance

Date: 2026-07-17

## Product contract

- Product manifest: `ee571c58b5f31c4710f6bf7ec518690b0de1f709468f714565bc8685625d8342`.
- Codex: `f90e7deea6a715bbd153044af6f475eefa749177`; T3Code:
  `ecb35f75839925dd1ac6f854efeef5c9e291d11b`.
- The executable acceptance test follows `WORKFLOW.md` through a normalized Linear fixture,
  Automation Root Run, persistent Issue claim/worktree/task, typed Workflow, policy-gated Tracker
  receipt, and bounded desktop projection. A sentinel from the raw child response is asserted absent
  from that projection.
- `SYMPHONY-COMPATIBILITY.md` classifies the supported profile surface and user-visible behavior.
  Missing `LINEAR_API_KEY` remains a validation warning and skips live reads; no raw GraphQL,
  daemon, detached scheduler, or alternate agent backend is exposed.

## Automated evidence

- Root formatting, 98 workspace tests, and lifecycle/plugin doctor passed. The five evaluator-worker
  cases then passed through the pinned Product worker.
- Clean pinned Codex verification passed: 77 Orchestra core tests; 16 extension tests with the
  explicit live Linear mutation test ignored; eight native Codex Orchestra tests; 267 App Server
  protocol tests; two generated-schema checks; App Server check; the end-to-end Automation
  acceptance; and the two affected upstream `thread/read` tests.
- Clean pinned T3Code web/desktop/server builds and typechecks passed. The full web suite passed
  1,285 tests across 149 files; the focused Automation/App Server suite passed 178 tests.
- Both integration patches applied cleanly. The framed host handshake, exact manifest identity, and
  Electron renderer-isolation smoke passed. Repository formatting and `git diff --check` passed.

## Retained desktop evidence

The retained Electron/React product visibly demonstrated:

- profile validation and rendered preview with the missing Linear credential reported as a warning;
- an asynchronous fixture start returning a running claim with persistent worktree, native Issue task
  ID, typed Workflow Run ID, Tracker effect, and bounded next action;
- Inspect and Refresh updating the bounded projection without child transcript text;
- Pause fencing the Root Run as `suspended` with reconciliation required, and Resume returning it to
  `running` only after reconciliation;
- the claim-scoped **Cancel issue** control and its durable suspended cancellation fence;
- explicit reattachment to a persisted root Run, followed by automatic reattachment after a full
  renderer reload through the native status projection;
- immediate **Cancel issue** acknowledgement in both a recovered run and a fresh isolated fixture.
  The target claim stopped dispatching, moved to `suspended`, and exposed the bounded
  `finish cancellation`/`retry native Issue descendant cancellation` state while native descendant
  confirmation remained pending. No other claim was modified. The core lifecycle test separately
  proves that confirmed descendant/effect reconciliation advances the claim to `cancelled` and makes
  cleanup eligible.
