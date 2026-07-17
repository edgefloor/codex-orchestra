# Issue #44 — Pinned T3Code visual foundation

Date: 2026-07-17

## Product contract

- T3Code is pinned to `ecb35f75839925dd1ac6f854efeef5c9e291d11b`; the visual foundation is
  carried by the reviewed integration patch rather than a detached renderer or replacement shell.
- Existing T3Code/Codex navigation, task, chat, composer, and native Orchestra behavior remain the
  product surface. This change only establishes the approved Orchestra palette, surface hierarchy,
  typography, focus, selection, and semantic status tokens.
- Dark and light themes use quiet layered surfaces with restrained iris accents. SF Pro and SF Mono
  are requested through system font stacks, with platform fallbacks when they are unavailable.

## Implementation

- The pinned renderer now owns explicit light and dark Orchestra tokens and maps T3Code's existing
  semantic Tailwind variables onto them.
- Sidebar chrome uses the existing sidebar semantic variables, keeping layout and behavior intact.
- Legacy bundled DM Sans and JetBrains Mono imports were removed. Selection, focus, composer
  elevation, and success/warning/danger/info states use the approved tokens.
- The integration verifier and focused T3Code suite assert the palette, typography, semantic
  mappings, sidebar seam, and absence of the legacy font imports.

## Automated evidence

- The exact T3Code integration patch applied and verified against the pinned revision.
- The focused renderer suite passed 15 tests. T3Code web typecheck, focused lint, formatting, and
  production build passed.
- The pinned integration test command passed 181 web/server tests. The pinned integration build
  passed the web production build, server bundle, Electron main/preload builds, and web/desktop
  typechecks.
- Root `cargo fmt --all -- --check`, the complete Rust workspace suite, lifecycle/plugin doctor,
  and `git diff --check` passed.

## Visual evidence

- The rebuilt Electron product started successfully and created its main window. Its compiled dark
  renderer was inspected at the desktop pairing boundary: the near-black layered surfaces, SF font
  stack, restrained iris action, and approved canvas/chrome/surface/focus/status values were present.
- After the Mac was unlocked, the real retained shell was observed directly. The Orchestra repository
  opened through the normal local-project path; project navigation, task tabs, native-subagent summary,
  attention summary, composer, model/reasoning/runtime/workspace selectors, Git actions, and terminal
  and right-panel toggles remained available. Symphony rendered inline inside the active draft rather
  than as a replacement dashboard.
- The explicit Light setting rendered the approved light canvas, layered sidebar and settings surfaces,
  iris controls, borders, and focus treatment. The original System preference was restored after the
  observation.
- A graceful quit emitted `before-quit received`; relaunch reached backend readiness, recreated the main
  window, and retained the registered Orchestra project. The unsent draft was correctly not promoted
  into canonical task history.
- The observation exposed separate branding-copy debt: the macOS window title is `Orchestra (Alpha)`,
  while retained renderer chrome and settings copy still say `T3 Code`. This does not invalidate the
  visual-token foundation, but Product branding is not yet internally consistent.
