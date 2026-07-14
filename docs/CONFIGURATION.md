# Configuration

Plugin installation exposes namespaced skills without changing user configuration. Optional templates enable native multi-agent support and custom agents.

- Project: preview `scripts/lifecycle.py project --target <repo>`, then repeat with `--apply`.
- Global profile: preview `scripts/lifecycle.py profile`, then repeat with `--apply`; select with `codex --profile orchestra`.
- Global default: use `global-default` only after explicitly reviewing conflicts.

The canonical custom agents are planner, worker, reviewer, and verifier. The active root agent orchestrates runs. Templates avoid pinned models and experimental feature flags.

Lifecycle operations preserve user-owned files, refuse unresolved conflicts, and track only files Orchestra created. Upgrades create rollback evidence before replacing managed files.
