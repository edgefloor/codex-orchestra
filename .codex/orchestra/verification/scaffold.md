# Scaffold verification

- Candidate: `codex-orchestra/` version `0.1.0`
- Status: local scaffold gates passed
- `uv lock --check` and `uv sync --locked --dev`: passed with `uv 0.11.21`; PyYAML resolved from the committed lockfile.
- `uv run --locked python -m unittest discover -s tests -v`: 12 passed.
- canonical plugin validator: passed inside the locked `uv` environment.
- `uv run --locked python scripts/lifecycle.py doctor`: passed on `codex-cli 0.141.0`; 13 skills discovered; project strict-config and profile selection probes passed.
- isolated marketplace: local marketplace add, plugin add, enabled listing, and cached installation at version `0.1.0` passed under an ephemeral `CODEX_HOME`.
- source review: no material findings; no MCP/App integration, daemon, scheduler, cached-install mutation, or plugin-local run state found.
- Seed handling: no seed file was moved or deleted; the candidate was created alongside `codex-orchestra-framework/`.
- Operator checkpoint: invoke `$codex-orchestra:orchestrate` from a fresh user task before promoting the candidate over the known-good version.
