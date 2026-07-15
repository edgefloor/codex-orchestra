---
status: accepted
---

ADR 0014 supersedes this decision's prohibition on bounded host-local product projection state.
Repository-local checkpoints remain authoritative for workflow execution and effects.

# Keep execution Codex-native and state repository-local

The active Codex agent executes workflows through native collaboration, worktrees, approvals, and ordinary tools. Workflow definitions, run state, step results, evidence, and summaries are repository artifacts; Git preserves source and change history. Messages and transcripts are coordination context, not durable state.

Do not add an MCP server, App Server client, daemon, sidecar, external scheduler, event database, or separate workflow platform unless a proven requirement cannot be met through supported native Codex capabilities.

## Consequences

- Installed plugin files are read-only behavior and templates.
- Mutable state lives under `<repository>/.codex/orchestra/`.
- Recovery uses repository and Git evidence rather than transcript continuity.
- Native capability gaps are documented instead of hidden behind unsupported integrations.
