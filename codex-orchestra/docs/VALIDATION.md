# Scaffold validation

Run:

```bash
cd codex-orchestra
uv sync --locked --dev
uv run --locked python ~/.codex/skills/.system/plugin-creator/scripts/validate_plugin.py .
uv run --locked python -m unittest discover -s tests -v
uv run --locked python scripts/lifecycle.py doctor
```

The committed `uv.lock` pins the validator's PyYAML dependency and the supported Python runtime. Validation covers manifest shape, skill discovery and namespacing, TOML parsing, stable multi-agent configuration, project/profile/global-default separation, custom-agent scope, plugin/run-state separation, lifecycle preview safety, modified-file detection, upgrade snapshots, rollback, uninstall preservation, and the self-hosting evidence layout.

An isolated local-marketplace install can validate cache creation without touching user-owned Codex state. A live native delegation branch, manual custom-agent selection, and invocation from a freshly opened user task remain interactive checkpoints; record them under `.codex/orchestra/verification/` when performed.

Use [INTERACTIVE-VERIFICATION.md](INTERACTIVE-VERIFICATION.md) for the executable operator runbook and evidence rules. Permanent behavioral cases live under `evals/scenarios/`; their protocol is in `evals/README.md`. Automated success never substitutes for a UI checkpoint: the evidence ledger must mark every unobserved human check `pending`, not `pass`.

The current automated baseline and pending interactive gates are recorded in [verification/2026-07-14-interactive-baseline.md](verification/2026-07-14-interactive-baseline.md).
