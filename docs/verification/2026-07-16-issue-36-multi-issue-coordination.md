# Issue 36 — multi-issue coordination

## Result

The Automation Root Run now persists a deterministic issue queue beside its native Issue claims.
Normalized pages are deduplicated by issue identity and latest observation, then classified by
active/terminal state, required labels, and nonterminal blockers. Dispatch sorts Linear priority
ascending, then creation time, identifier, and provider ID.

Global and case-insensitive per-state limits count claimed, running, and suspended claims. A
saturated state is skipped while the scan continues, so another eligible state can use remaining
global capacity. Active issue identities cannot be claimed twice, and the existing
repository/project Root lease remains authoritative.

The generated App Server surface adds read-only `automation/queue/read`. Root projections contain
six counts, at most 25 claims, and at most eight preview items; category reads clamp pages to 50.
The normal T3Code task dialog shows queue counters and opens typed 25-item pages with bounded row
detail. The renderer cannot choose a repository, tracker query, or execution authority.

## Automated evidence

- Multi-issue Rust fixtures cover normalized two-page Linear input, duplicate observations,
  blocker decisions, priority ordering, global/per-state capacity, head-of-line skipping, duplicate
  claim prevention, external terminal-state observations, and 50-item page bounds.
- Fresh pinned Codex checkout at `f90e7deea6a715bbd153044af6f475eefa749177`:
  66 Orchestra core tests, 11 extension tests, 8 focused native-control tests, all 264 protocol
  tests and schema fixtures, and `codex-app-server` compilation passed.
- Fresh pinned T3Code checkout at `ecb35f75839925dd1ac6f854efeef5c9e291d11b`:
  contracts, server, and web typechecks passed; all 149 web test files and 1,284 tests passed.
- Both integration patches applied cleanly to fresh detached checkouts.

## Boundary

The existing fixture execution request supplies one issue and now uses this coordinator before it
starts the native Issue task. Multi-page queue selection is exercised at the persisted Rust seam;
continuous live polling and resume-time dispatch are intentionally left for the later resident-loop
and reconciliation issues rather than introducing a second scheduler here.
