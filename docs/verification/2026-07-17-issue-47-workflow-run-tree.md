# Issue #47 — Lazy native Workflow Run Tree

Date: 2026-07-17

## Product contract

- The normal Codex task timeline and composer remain the primary work surface. A native Orchestra
  lifecycle entry is the run-tree root; no detached run screen or renderer-owned authority exists.
- The collapsed root uses only the bounded task-local lifecycle projection. Run, step, output,
  evidence, decision, failure, and recovery detail comes from the existing authorized
  `orchestra/query` surface when the corresponding node is opened.
- Existing task replay and provider-session recovery remain the reload and runtime-recovery paths.
  Unsupported actions and states are not simulated.

## Implementation

- The lifecycle row now renders an accessible tree with a six-step attention summary, total and
  completed counts, native next action, and queued/running/waiting/completed/failed/cancelled or
  recovered presentation.
- Opening the root loads bounded run and ordered step projections. Opening a step loads only that
  step's outputs and evidence references; inline output rendering is separately bounded. Decisions
  and failures remain hidden until their step is inspected.
- Recovery and decision history is a separate lazy node backed by rollout history. Continuation
  markers make omitted native detail explicit instead of eagerly paging or copying it.
- Query failures preserve the lifecycle digest and present native detail as unavailable. The
  current pinned runtime does not emit a paused run status, so no synthetic pause control or state
  was added.

## Automated evidence

- Logic tests cover the six-step initial bound, attention priority, deterministic step ordering,
  recovery mapping, authorized query budgets, and bounded output rendering.
- Component tests verify that the collapsed task entry exposes the native digest while withholding
  final response, decision/history, and other lazy detail.
- The pinned integration suite passed 124 web tests and 76 server tests. Contracts, web, and server
  typechecks passed.
- Production web/server/Electron builds and desktop typecheck passed. The exact patch applied and
  verified against `ecb35f75839925dd1ac6f854efeef5c9e291d11b`; `git diff --check` passed.
- Root formatting, 98 executed workspace tests, and lifecycle/plugin doctor remained green in the
  same working tree. Five evaluator-worker tests remain intentionally delegated to their pinned
  Product-worker harness.

## Deferred observation

- Direct authenticated-shell observation of tree expansion remains non-blocking evidence debt
  while the Mac is locked. No pairing, authorization, or native task boundary was bypassed.
