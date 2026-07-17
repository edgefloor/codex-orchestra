# Issue #45 — Project, task, and tab navigation

Date: 2026-07-17

## Correction — issue reopened

The original acceptance was invalid. Component tests proved that project and task controls were wired
to native state, but the unlocked Electron build still composed them as a lightly themed T3Code shell
rather than the canonical `orchestra-workspace.html` information architecture. Native subagent and
Attention projections were stacked as full-width diagnostic rows above chat, the task-tab strip sat
below the task header, and the required contextual rail was absent. The issue remains open until the
real Electron product is compared directly with the canonical workspace at desktop and narrow-desktop
widths.

The first corrective pass is captured in
`docs/verification/assets/2026-07-17-orchestra-workspace-correction.jpeg`. It moves the compact task
tabs above the native task header, adds the worktree/status bar, composes native Subagents and Attention
into a contextual right rail, and removes the redundant project hierarchy for the selected project.
This is progress evidence, not closure evidence; project overview/capability navigation, narrow-drawer
behavior, populated native task states, and direct pixel comparison still remain.

## Product contract

- The workspace remains the pinned T3Code/Codex task application. Project selection, task tabs,
  task history, composer actions, auxiliary panels, and recovery all derive from existing native
  renderer authorities.
- No open-tab database, fixture task model, detached Orchestra Run, or replacement dashboard was
  introduced. The tab row is a bounded projection of native tasks in the active project.
- The existing responsive sidebar and right-panel sheet remain the narrow-window disclosure paths;
  this ticket does not create a mobile product.

## Implementation

- The retained sidebar now exposes a compact project picker backed by logical project snapshots.
  Choosing a project opens its native recent task, or uses the existing native draft flow when the
  project has no task.
- The task shell now renders a bounded, horizontally scrollable task tab row above the unchanged
  timeline and composer. The active task is always retained, task state is projected from native
  thread shells, and the plus action calls the existing new-thread handler.
- Arrow keys wrap between tabs; Home and End select the first and last visible tasks. Focus,
  selection, attention, running, error, empty-project, and overflow behavior are explicit.
- Diff, files, context, preview, terminal, settings, models, workspaces, commit, and push affordances
  remain on their existing T3Code paths. The existing contextual right rail still collapses to its
  accessible sheet at narrow desktop widths.

## Automated evidence

- Navigation logic and component tests cover bounded recent-task projection, retention of an older
  active task, deduplication, archived-task exclusion, status priority, wrapping keyboard
  navigation, selected rendering, and the empty-project new-task action.
- Focused navigation/regression tests passed 62 tests. The pinned integration suite passed 187
  web/server tests.
- T3Code formatting, focused lint, web typecheck, production web build, server bundle, Electron
  main/preload builds, and desktop typecheck passed.
- The exact pinned integration patch applied and verified against
  `ecb35f75839925dd1ac6f854efeef5c9e291d11b`; `git diff --check` passed.

## Invalid deferred observation

- Direct observation of the authenticated shell at multiple desktop widths was incorrectly treated as
  non-blocking while the Mac was locked. It is blocking evidence for this visual issue.
