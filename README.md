# Codex Orchestra

Codex Orchestra is a Codex-native orchestration plugin. It supplies skills, configuration templates, custom-agent templates, schemas, policies, behavioral evaluations, and lifecycle helpers. It does not ship an MCP server, daemon, scheduler, App Server client, or external control plane.

Invoke `$codex-orchestra:orchestrate` in a fresh Codex task after installing the plugin. Mutable engagement state belongs in the target repository at `.codex/orchestra/`; it must never be written into the installed or cached plugin.

Development and lifecycle tooling uses [uv](https://docs.astral.sh/uv/) with the committed `uv.lock`. From this directory, run `uv sync --locked --dev` once and use `uv run --locked` for every Python command.

Start with the [domain language](CONTEXT.md), [repository structure](docs/REPOSITORY-STRUCTURE.md), and [architecture decisions](docs/adr/). Operational guidance lives in [configuration](docs/CONFIGURATION.md), [lifecycle](docs/LIFECYCLE.md), [self-hosting](docs/SELF-HOSTING.md), [interactive verification](docs/INTERACTIVE-VERIFICATION.md), and [validation](docs/VALIDATION.md).
