---
status: accepted
---

# Maintain Codex as a long-lived pinned product fork

At OpenAI Codex revision `f90e7deea6a715bbd153044af6f475eefa749177`,
`codex-extension-api` supports constructor-injected capabilities and native tools, but stock plugin
packages cannot register arbitrary Rust extensions and App Server constructs a static registry.
More importantly, Orchestra's product goal requires native workflow semantics to deepen alongside
Codex tasks, turns, permissions, tools, and V2 agents rather than remain an external integration.

Orchestra therefore maintains a long-lived Codex product fork. The independent Orchestra core
repository supplies a pinned overlay that:

- registers `codex-orchestra-extension` in the static registry;
- exposes `OrchestraControl`, a narrow wrapper over the active thread's `AgentControl` and sandbox executor;
- makes task-bound Orchestra invocation and observation available in the bundled Codex host mode,
  backed by the same `OrchestraControl`, so execution retains canonical V2 child lineage; and
- leaves scheduling and state in the independent `codex-orchestra-core` crate.

The fork's existing `app-server-protocol` crate owns the added `orchestra/*` and `host/*` wire DTOs,
notification metadata, JSON Schema, and TypeScript generation. `codex-orchestra-core` owns Runs,
Steps, scheduling, checkpoints, gates, effects, and recovery. Explicit conversions connect the wire
contract to the domain model; protocol DTOs never become checkpoint authority.

This boundary is intentionally honest. There is no fallback to SDK threads, MCP, `codex exec`, an App Server sidecar, a daemon, or separate agent processes.

The fork selectively incorporates upstream Codex changes while keeping an exact reviewed revision
pin. Existing Codex primitives are reused first. New behavior is added to Codex only when its
semantics belong to native task, turn, permission, tool, or agent lifecycle; workflow plans,
scheduling, evidence, gates, effects, and recovery remain Orchestra-owned. Divergence stays
concentrated in explicit extension and host seams so upstream synchronization remains practical.

Dynamic extension support upstream may reduce the maintained patch surface, but removing the fork
is not an architectural objective. Every upstream sync must revalidate the native lineage,
residency, steering, event, final-response, cancellation, sandbox, and host-protocol contracts.
