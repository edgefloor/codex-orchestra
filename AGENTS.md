# Codex Orchestra development guide

Codex Orchestra is a native V2 Rust runtime with an installable plugin for skills, configuration, and documentation. Keep orchestration on the active thread's native `AgentControl` path through the pinned Codex integration.

- Do not add an MCP server, App Server client, daemon, sidecar, external scheduler, or hidden control plane.
- Treat `CONTEXT.md` as the domain-language source and `docs/adr/` as the architectural decision record.
- Keep the restricted TypeScript SDK authoring-only; workflow source must be parsed and lowered by Rust, never executed as JavaScript.
- Keep runtime-owned snapshots, checkpoints, validated outputs, evidence, decisions, and summaries under a target repository's `.codex/orchestra/runs/`, never in plugin source or an installed cache.
- Keep the integration patch pinned to `integration/codex/UPSTREAM_REVISION` and verify assumptions against that exact source.
- Use `colgrep` as the primary semantic search tool and `rg --files` for filename discovery.
- Run the unit suite, canonical plugin validator, and lifecycle doctor after structural changes.
- Record assumptions and leave human-only UI checks pending until they are actually observed.
