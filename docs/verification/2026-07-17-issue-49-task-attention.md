# Issue #49 — Native task Attention view

Date: 2026-07-17

## Product contract

- Attention is a bounded projection of existing task and runtime authority, not a dismissible
  renderer inbox. Native updates add, resolve, recover, or remove items.
- The canonical count distinguishes native approvals, waiting workflow or effect gates, failed or
  ambiguous effects, blocked reconciliation, and provider failures. Detail is task-scoped and
  loaded only when an item is opened.
- Approval prompts and changed-file notices retain their existing semantic surfaces above the
  composer. Attention summarizes them without turning them into chat messages.

## Implementation

- The normal task shell now has a compact, collapsible Attention row with a canonical total and a
  twelve-item priority bound. The total remains unchanged when lower-priority items are omitted.
- Workflow gates use the latest native lifecycle revision. Expanded workflow detail calls the
  existing authorized `orchestra/query` surface with bounded budgets.
- Automation counts reconcile through the active task's Run-id cursor and native status command on
  mount, task revision updates, and manual refresh. Expanded effect and reconciliation detail
  re-reads native status; no renderer dismissal or speculative state transition exists.
- Approval actions reuse the existing native composer approval handler. Workflow gates route back
  to the native approval surface, while Automation items open the task-scoped Symphony workspace.
  Provider failures expose no invented action.
- Empty, loading, stale, error, recovered, and ready Automation snapshot states are represented.

## Automated evidence

- Logic tests cover count parity, category distinction, latest-revision resolution, list bounds,
  action routing, bounded authorized queries, runtime states, and task-scoped reload reattachment.
- Component tests cover the normal task-shell and empty/approval summary presentations while
  rejecting a detached dialog or duplicated approval prompt.
- The pinned integration suite passed 136 web tests and 76 server tests. Web and desktop typechecks
  passed.
- Production web/server/Electron builds passed against pinned T3Code revision
  `ecb35f75839925dd1ac6f854efeef5c9e291d11b`; canonical patch verification and
  `git diff --check` passed.
- Root formatting, 98 executed workspace tests, and the lifecycle/plugin doctor remained green in
  the same working tree. Five evaluator-worker tests remain intentionally delegated to their
  pinned Product-worker harness.

## Deferred observation

- Direct authenticated-shell observation of collapsed/expanded density and notice placement
  remains non-blocking evidence debt while the Mac is locked. Automated structure, authority,
  action-routing, recovery, and build gates passed.
