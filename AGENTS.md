# Codex Orchestra development guide

Codex Orchestra is a Codex-native plugin. Keep implementation inside supported Codex surfaces: skills, plugin assets, repository/global TOML, custom-agent TOML, native collaboration, worktrees, and repository artifacts.

- Do not add an MCP server, App Server client, daemon, sidecar, external scheduler, or hidden control plane.
- Treat `CONTEXT.md` as the domain-language source and `docs/adr/` as the architectural decision record.
- Keep `config/agents/` canonical; lifecycle code maps it into project or global installation targets.
- Keep mutable engagement state under a target repository's `.codex/orchestra/`, never in plugin source or an installed cache.
- Use `colgrep` as the primary semantic search tool and `rg --files` for filename discovery.
- Run the unit suite, canonical plugin validator, and lifecycle doctor after structural changes.
- Record assumptions and leave human-only UI gates pending until they are actually observed.
