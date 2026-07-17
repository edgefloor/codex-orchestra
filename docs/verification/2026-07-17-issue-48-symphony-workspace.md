# Issue #48 — Task-scoped Symphony workspace

Date: 2026-07-17

## Product contract

- Symphony is a contextual workspace inside the normal Codex task shell. It does not replace the
  timeline or composer and does not create a detached renderer-owned Run.
- Existing native validation, queue, lifecycle, reconciliation, and cancellation commands remain
  authoritative. Renderer state derives presentation and control availability from their bounded
  projections.
- The renderer persists only the current task's Run identifier as a reattachment cursor. Opening
  the workspace reads native status again; it never reconstructs a Run from browser storage.

## Implementation

- The former profile dialog is now a bounded inline workspace beneath the task header and native
  subagent summary. A task-keyed header action opens or closes it without changing task identity.
- The workspace presents idle, validating, queued, running, waiting, paused, reconciling,
  completed, failed, cancelled, and unavailable states derived from native validation and Run
  projections.
- Validate, start, inspect, pause, resume, refresh, and cancel controls are exposed only when the
  current native projection permits them. Existing profile, queue, intake, claim, receipt, cleanup,
  and recovery detail remains available in the task context.

## Automated evidence

- Logic tests cover lifecycle derivation and native control capability gating. Component coverage
  verifies a task workspace is rendered and rejects a detached dialog surface.
- The pinned integration suite passed 127 web tests and 76 server tests. Web and desktop typechecks
  passed.
- Production web/server/Electron builds passed against pinned T3Code revision
  `ecb35f75839925dd1ac6f854efeef5c9e291d11b`; the canonical patch verifies in reverse and
  `git diff --check` passed.
- Root formatting, 98 executed workspace tests, and the lifecycle/plugin doctor passed. Five
  evaluator-worker tests remain intentionally delegated to their pinned Product-worker harness.

## Deferred observation

- Direct authenticated-shell observation of workspace sizing and control transitions remains
  non-blocking evidence debt while the Mac is locked. No implementation or automated release gate
  depends on that observation.
