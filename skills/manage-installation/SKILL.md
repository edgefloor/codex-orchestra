---
name: manage-installation
description: Preview, install, check, upgrade, roll back, or uninstall Orchestra configuration templates safely.
---

Run `uv sync --project <plugin> --locked --dev`, then `uv run --project <plugin> --locked python <plugin>/scripts/lifecycle.py doctor`. Lifecycle commands preview by default and mutate only with `--apply`. Never overwrite conflicting or locally modified files. Use `project` for repository configuration, `profile` for `~/.codex/orchestra.config.toml`, and `global-default` only after explicit user intent. Consult `docs/LIFECYCLE.md`; preserve the preceding known-good plugin until a candidate succeeds in a fresh Codex task.
