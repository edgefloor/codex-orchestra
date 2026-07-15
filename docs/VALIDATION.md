# Validation

Repository verification has four layers:

1. `cargo test --workspace` covers compiler security, plan validation, DAG scheduling, exact context, retries, repeats, conflicts, fake native spawning, malformed output, approvals, checks, worktrees, recovery, cancellation primitives, plugin layout, and immutable lifecycle behavior.
2. `cargo run -p codex-orchestra-lifecycle -- doctor` validates manifest/config/plugin capabilities.
3. `scripts/characterize-pinned-skills.sh <pinned-codex-checkout>` verifies the exact native handoff and Codex skill-selection, policy, and cache behavior that skill-backed workflows rely on.
4. `scripts/codex-integration.sh <fresh-dir> verify` applies the patch to the pinned Codex source, tests both Orchestra crates, and checks `codex-app-server`.

Interactive UI rendering of native Orchestra tools and real provider-backed child completion remains human-only evidence; it must not be marked complete until observed.
