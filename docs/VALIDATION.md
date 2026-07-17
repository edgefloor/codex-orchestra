# Validation

Repository verification has four layers:

1. `cargo test --workspace` covers compiler security, plan validation, DAG scheduling, exact context, retries, repeats, conflicts, fake native spawning, malformed output, approvals, checks, worktrees, recovery, cancellation primitives, plugin layout, and immutable lifecycle behavior.
2. `cargo run -p codex-orchestra-lifecycle -- doctor` validates manifest/config/plugin capabilities.
3. `scripts/characterize-pinned-skills.sh <pinned-codex-checkout>` verifies the exact native handoff and Codex skill-selection, policy, and cache behavior that skill-backed workflows rely on.
4. `scripts/product-source-prepare.sh <fresh-dir>` followed by `scripts/product-source-verify.sh <fresh-dir>` clones the exact public hard forks and verifies repository, commit, tree, upstream ancestry, runtime snapshot, generated protocol, and retained desktop capability identities.

Interactive UI rendering of native Orchestra tools and real provider-backed child completion remains human-only evidence; it must not be marked complete until observed.

The accepted evaluator determinism, provenance, bounded-failure, and candidate resource-limit contract
is defined in [WORKFLOW-COMPILATION.md](./WORKFLOW-COMPILATION.md). The MVP makes no hostile-code
sandbox claim; those behavioral claims remain gated by issue #16 evidence.
