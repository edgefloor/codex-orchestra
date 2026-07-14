---
status: accepted
---

# Use a small pinned Codex integration patch until Rust extensions are loadable

At OpenAI Codex revision `f90e7deea6a715bbd153044af6f475eefa749177`, `codex-extension-api` supports constructor-injected capabilities and native tools, but stock plugin packages cannot register arbitrary Rust extensions and App Server constructs a static registry.

Orchestra therefore remains an independent repository and supplies a small pinned overlay that:

- registers `codex-orchestra-extension` in the static registry;
- exposes `OrchestraControl`, a narrow wrapper over the active thread's `AgentControl` and sandbox executor;
- leaves scheduling and state in the independent `codex-orchestra-core` crate.

This boundary is intentionally honest. There is no fallback to SDK threads, MCP, `codex exec`, an App Server sidecar, a daemon, or separate agent processes.

Exit the fork when stock Codex can discover and load a third-party Rust extension with equivalent per-thread capability injection, native tool registration, V2 lineage/residency/events/final-response/cancellation access, and sandbox-aware command execution. At that point remove only the pin/apply overlay; the core host trait and plugin authoring surface should remain.
