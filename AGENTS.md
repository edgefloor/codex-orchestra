# Orchestra development guide

Orchestra is a native V2 Rust runtime with an installable plugin for skills, configuration, and documentation. Keep orchestration on the active thread's native `AgentControl` path through the pinned Codex integration.

- Invoke workflows inside their parent Codex task; UI affordances may initiate the same task action but must not start detached host Runs.
- Optimize context at every boundary: keep canonical detail in its native task or authority, provide bounded summaries by default, and expand typed detail only on demand.
- Do not add an MCP server, App Server client, daemon, sidecar, external scheduler, or hidden control plane.
- Treat `CONTEXT.md` as the domain-language source and `docs/adr/` as the architectural decision record.
- Keep the restricted TypeScript SDK authoring-only; workflow source must be parsed and lowered by Rust, never executed as JavaScript.
- Keep runtime-owned snapshots, checkpoints, validated outputs, evidence, decisions, and summaries under a target repository's `.codex/orchestra/runs/`, never in plugin source or an installed cache.
- Keep Product assembly pinned to the exact public fork and upstream-base identities in `product/pins.toml`; normal builds must never reconstruct source with patches or overlays.
- Treat Codex and the T3Code-derived desktop as long-lived product forks: reuse upstream primitives first, extend them where product semantics belong, and keep divergence in explicit reviewed seams.
- Use `colgrep` as the primary semantic search tool and `rg --files` for filename discovery.
- Run the unit suite, canonical plugin validator, and lifecycle doctor after structural changes.
- Record assumptions and leave human-only UI checks pending until they are actually observed.
- Promote framework lessons only when they are general, testable, versioned, and placed in the narrowest durable source.
- If you need a paragraph-long comment to justify why the workaround is OK, the code is wrong - fix the code.
