# Codex Orchestra

Codex Orchestra is an installable plugin for creating and running reviewable Codex-native workflows. It is inspired by [Anthropic Dynamic Workflows](https://code.claude.com/docs/en/workflows) and extends the pattern with repository-local run state, worktree isolation, deterministic checks, independent review, user approvals, recovery, and self-hosting.

Version 1 is model-executed: the active Codex agent reads a declarative YAML workflow and calls native collaboration tools. It is not a background script runtime, and it does not ship an MCP server, App Server client, daemon, sidecar, or external scheduler.

| Capability | Anthropic Dynamic Workflows | Orchestra v1 |
|---|---|---|
| Definition | Generated JavaScript | Declarative YAML |
| Executor | Background script runtime | Active Codex agent |
| Agent calls | Workflow runtime primitives | Native Codex collaboration tools |
| Intermediate state | Script variables | Repository step results and run state |
| Resume boundary | Same Claude session | Fresh Codex task from repository evidence |
| Extensions | Parallel and pipeline agent patterns | Checks, review, approvals, worktree policy, and self-hosting |

Invoke `$codex-orchestra:orchestrate` in a fresh Codex task after installing the plugin. Reusable workflows live in `.codex/orchestra/workflows/`; each run is snapshotted with durable state under `.codex/orchestra/runs/<run-id>/`. Installed plugin files remain read-only.

Development uses [uv](https://docs.astral.sh/uv/) with the committed `uv.lock`. Run `uv sync --locked --dev`, then use `uv run --locked` for Python commands.

Start with [CONTEXT.md](CONTEXT.md), [repository structure](docs/REPOSITORY-STRUCTURE.md), [configuration](docs/CONFIGURATION.md), and the [architecture decisions](docs/adr/).
