---
status: accepted
---

# Keep execution Codex-native and state repository-local

Orchestra executes through native Codex collaboration, worktrees, approvals, and tools rather than
an MCP server, sidecar, daemon, or separate workflow platform. Repository checkpoints and Git remain
authoritative for workflow execution and effects because transcripts and installed plugin files are
not durable Run state.

ADR 0014 later permits bounded host-local projections for product replay and hydration; those
projections remain rebuildable and do not weaken repository execution authority.
