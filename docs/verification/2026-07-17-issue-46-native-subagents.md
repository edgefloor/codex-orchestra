# Issue #46 — Native subagents in the task experience

Date: 2026-07-17

## Product contract

- Subagents remain native Codex child threads on the parent task's existing provider runtime.
- The parent stores only bounded child identity, lifecycle, and recent-activity projections. Full
  child history is requested lazily and never copied into a separate Orchestra registry.
- Child detail requires the ordinary orchestration read scope and a direct native
  `parentThreadId` relationship verified by the Codex runtime.

## Implementation

- The retained task shell now exposes a slim subagent strip with stable native child identity and
  pending, running, waiting, completed, failed, cancelled, and unavailable presentations.
- Selecting a child opens bounded native detail with an explicit return to the parent. The read
  uses Codex `thread/read`, returns at most 24 recent items, and rejects unrelated threads.
- Existing native collaboration activity supplies the parent summary. No detached run, eager
  child-history load, extra registry, or non-native control action was introduced.

## Automated evidence

- Projection and component tests cover stable identity, lifecycle updates, bounded parent state,
  ordinary tasks with no children, and rendered child state.
- Runtime tests cover direct-child authorization, unrelated-thread rejection, waiting state,
  bounded summaries, and bounded child history.
- The pinned integration suite passed 118 web tests and 76 server tests. Web/server/contracts
  typechecks, production web/server/Electron builds, and desktop typecheck passed.
- The exact integration patch applied and verified against
  `ecb35f75839925dd1ac6f854efeef5c9e291d11b`; `git diff --check` passed.
- Root formatting, 98 executed workspace tests, and lifecycle/plugin doctor passed. Five evaluator
  worker integration tests remained intentionally ignored for their pinned Product-worker harness.

## Deferred observation

- Direct authenticated-shell observation remains non-blocking evidence debt while the Mac is
  locked. Automated renderer, protocol, authorization, build, and retained-task evidence passed.
