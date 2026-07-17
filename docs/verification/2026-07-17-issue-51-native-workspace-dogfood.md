# Issue #51 — Redesigned native workspace dogfood

## Correction — issue reopened

The original result below records useful native-state and lifecycle evidence, but it does not prove the
redesigned workspace. The unlocked Electron build did not match the canonical workspace composition,
and the required dark/light/narrow manual observations were never completed. Those observations cannot
be classified as non-blocking for this issue. Re-closure requires a real native-backed dogfood path and
direct screenshot comparison.

## Result

The redesigned workspace remains the normal pinned T3Code application. The exact Product tuple keeps
T3Code task navigation, chat, diff, files, context, terminal, settings, models, workspaces, commit, and
push surfaces while projecting native Codex subagents and task-scoped Orchestra state into that shell.
No detached workflow runner or renderer-owned execution authority was introduced.

The retained live Product scenario from issue #27 supplies the real native task evidence: the sealed
Codex fork created a native child, completed a Workflow, inspected bounded evidence, and recovered the
same task-scoped lifecycle entry after Electron and provider restart without a duplicate. Issues #44–#50
then replaced that scenario's presentation with the redesigned task tabs, native subagent panel, Run
Tree, Symphony workspace, Attention view, and opaque evidence-content expansion. A current exact-fork
dogfood contract composes those production DTOs and projection helpers as one task scenario rather than
inventing fixture-only UI state.

| Scenario transition | Current evidence |
|---|---|
| Open one native task | Workspace task tabs retain the active native `ThreadId` and do not create a workflow tab |
| Native child runs and completes | Native `collabAgentToolCall` projection resolves the direct child to one completed row |
| Workflow waits at a gate | The task timeline and Attention derive the same waiting step from the task-scoped replay event |
| Inspect evidence | The Run Tree requests content lazily by opaque evidence ID with bounded bytes and no renderer path |
| Resolve the gate | Revision 2 completes the same Run and clears Attention without a second Run identity |
| Control Symphony | Idle, validating, queued, running, waiting, paused, reconciling, completed, failed, cancelled, and unavailable presentations derive from native Automation state |
| Reload the renderer | Only the active task's stored Run cursor reattaches; another task's cursor is not selected |
| Restart the desktop | Two separate Electron processes reach `main window created` against the same temporary T3 home and shut down cleanly |

## Automated evidence

- Product manifest `02699e428583074de86811aea2101e3abd0c0bcc96ea23d258f1771482c538c3`
  seals Codex `f90e7deea6a715bbd153044af6f475eefa749177`, T3Code
  `ecb35f75839925dd1ac6f854efeef5c9e291d11b`, the evaluator, generated protocol, desktop main,
  preload, renderer, and retained server. Its framed host handshake passed.
- The full pinned T3Code workspace test command passed in all 14 workspace packages. The principal
  retained surfaces passed 1,322 web tests, 335 desktop tests, and 1,408 server tests with seven server
  tests skipped by the upstream suite. Web, server, Electron main/preload, and renderer production
  builds passed; web and desktop typechecks passed.
- The current `nativeWorkspaceDogfood.test.ts` adds three cross-surface contract tests. It covers the
  realistic task transition above; Workflow queued, running, waiting, paused, completed, failed,
  cancelled, and unavailable presentation; Symphony idle through terminal/unavailable presentation;
  Attention empty, loading, stale, and recovered presentation; and task-scoped reload selection.
- The desktop smoke now launches the built Electron application twice against one isolated T3 home.
  Both launches reached backend readiness and `main window created`, then emitted `before-quit received`
  and exited without an orphan process.
- Root formatting and `git diff --check` passed. The Rust workspace executed 99 tests successfully;
  the five evaluator-worker cases also passed through the pinned compiled Product worker. The lifecycle
  doctor passed the canonical plugin manifest, configuration, four skills, and native capability checks.
- Clean pinned Codex verification remained green after the evidence-content integration: 78 Orchestra
  core tests, 16 extension tests with the explicit live mutation test ignored, eight Codex integration
  tests, 267 protocol tests, two schema fixtures, and the App Server build check passed.

## Blocking human-only evidence

The Mac was locked during this pass. Fresh visual observation of dark mode, light mode, the narrow
desktop drawer, and the complete click path through retained diff/files/context/terminal/settings/model/
workspace/commit/push controls remains pending. The build and lifecycle evidence above does not replace
these visual observations. These items remain unobserved and block acceptance; they must not be recast
as manual evidence until a person actually performs them.
