# Installation lifecycle

All commands preview by default. Add `--apply` only after inspecting the action list.

```bash
uv sync --locked --dev
uv run --locked python scripts/lifecycle.py doctor
uv run --locked python scripts/lifecycle.py project --target /repo [--apply]
uv run --locked python scripts/lifecycle.py profile --codex-home ~/.codex [--apply]
uv run --locked python scripts/lifecycle.py global-default --codex-home ~/.codex [--apply]
uv run --locked python scripts/lifecycle.py upgrade --target /repo [--apply]
uv run --locked python scripts/lifecycle.py rollback --target /repo [--apply]
uv run --locked python scripts/lifecycle.py uninstall --target /repo [--apply]
```

The helper records hashes only for files it creates. It refuses initial conflicts and refuses to upgrade, remove, or roll back locally modified files. Upgrade snapshots managed files under `.codex/orchestra/recovery/install-*`; rollback uses the newest snapshot. Uninstall removes unchanged managed configuration but preserves all `.codex/orchestra/` engagement artifacts and every user-owned file.

Plugin installation itself follows Codex marketplace commands. Keep the known-good version installed, validate the candidate manifest, update its cachebuster through the supported plugin workflow, install the candidate, and test it in a fresh task. Never edit a cached plugin installation.
