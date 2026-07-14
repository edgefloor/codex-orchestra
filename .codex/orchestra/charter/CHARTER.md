# Scaffold engagement charter

- Run ID: `scaffold-2026-07-14`
- Revision: `1`
- Status: accepted from `SCAFFOLDING-PLAN.md`
- Objective: scaffold a Codex-native Orchestra plugin beside the untouched seed framework.
- In scope: plugin package, native skills, configuration/agent templates, lifecycle procedures, repository state contracts, tests, and a self-hosting validation path.
- Non-goals: MCP, App Server client, daemon, external scheduler, durable sidecar control plane, or operating-model redesign.
- Acceptance: the plan's scaffold surfaces exist, validate locally, preserve user files, and keep mutable run state outside the plugin.
- Bootstrap invariant: work only on source candidate `codex-orchestra/`; preserve `codex-orchestra-framework/` as fallback.
