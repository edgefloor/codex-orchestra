# Interactive baseline — 2026-07-14

The legacy plugin-only baseline did not and could not verify direct `AgentControl` access. It is superseded by the native V2 architecture.

Automated evidence at OpenAI Codex `f90e7deea6a715bbd153044af6f475eefa749177`:

- The integration patch applied cleanly to a fresh detached checkout and `git diff --check` passed.
- The Rust workspace passed 29 tests: 16 core runtime/compiler tests and 13 lifecycle/plugin scaffold tests. The former Python lifecycle implementation and test environment have been removed.
- The canonical plugin validator, all four skill validators, and the lifecycle doctor passed.
- In the pinned Codex tree, the overlaid core passed 16 tests and the extension adapter passed 2 tests.
- `cargo check -p codex-orchestra-extension` and `cargo check -p codex-app-server` passed. Cargo workspace resolution first attempted to fetch the unrelated `realtime-webrtc` member's `libwebrtc` dependency and that external Git submodule fetch failed; the two affected package checks passed after excluding only that unrelated member in the temporary verification checkout. The repository patch does not alter upstream workspace membership.

- Fresh-task plugin discovery: pending
- Five native Orchestra tools: pending
- Provider-backed V2 vertical slice: pending
- Visible lineage, residency, and activity: pending
- Approval, cancellation, and transcript-free recovery: pending
- Installed-cache identity during self-hosting: pending

Verdict: `pending` until the Orchestra-enabled pinned Codex build is exercised interactively.
