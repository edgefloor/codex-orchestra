---
status: accepted
---

# Keep Orchestra Codex-native and repository-grounded

Orchestra is an installable Codex-native operating layer. Skills and custom agents conduct live work through Codex tasks, native collaboration, worktrees, approvals, and ordinary tools; repository artifacts preserve accepted intent, plans, decisions, assignments, results, verification, recovery, and handoffs; Git preserves code and change evidence. Native messages and transcripts are coordination context, not durable truth.

Do not add an MCP server, App Server client, daemon, external scheduler, event store, SQLite control plane, or separate workflow platform unless a concrete requirement is proven impossible through supported native Codex facilities. This supersedes the legacy R2 external-control-plane and Workflow IR proposals while retaining their useful separation between live coordination, durable work records, and code evidence.

## Consequences

- The installed plugin contains read-only behavior and templates, never live Engagement state.
- Mutable state is rooted at `<repository>/.codex/orchestra/`.
- Recovery reconstructs work from repository and Git evidence rather than a transcript or external database.
- Native capability gaps are recorded and validated before introducing a new integration seam.
