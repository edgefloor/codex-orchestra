# Codex Orchestra

Codex Orchestra is a Rust workflow runtime for native Codex V2 agents, plus an installable plugin that supplies authoring skills and configuration. Workflows are written in a restricted `.workflow.ts` data language, compiled to an internal Rust execution plan, and never evaluated as JavaScript.

The runtime owns DAG scheduling, exact context materialization and hashing, retries, bounded repeats, sandbox-aware checks, approvals, isolated Git worktrees, checkpoints, recovery, cancellation, and summaries. Agent steps use the active task's parent-linked V2 `AgentControl`, canonical task paths, completion watchers, residency, and explicit model/reasoning/service-tier/fork settings. Child delegation is disabled by default.

Stock plugin packages cannot register arbitrary Rust extensions, so the current delivery has two honest parts:

- this independent repository and its installable plugin layer;
- a small integration patch pinned to OpenAI Codex revision `f90e7deea6a715bbd153044af6f475eefa749177`.

There is no fallback to SDK threads, MCP, `codex exec`, an App Server client, a daemon, a sidecar, or model-authored run state.

## Develop and verify

```bash
cargo test --workspace
cargo run -p codex-orchestra-lifecycle -- doctor
scripts/codex-integration.sh /tmp/codex-orchestra-codex verify
```

The last command clones the pinned upstream revision when needed, applies the overlay, tests the Orchestra crates, and checks `codex-app-server`.

Start with [CONTEXT.md](CONTEXT.md), [repository structure](docs/REPOSITORY-STRUCTURE.md), [configuration](docs/CONFIGURATION.md), and [ADR 0011](docs/adr/0011-temporary-codex-fork-boundary.md).
