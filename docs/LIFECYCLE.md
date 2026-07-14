# Lifecycle commands

Run commands through the locked development environment:

```text
uv run --locked python scripts/lifecycle.py doctor
uv run --locked python scripts/lifecycle.py project --target <repo>
uv run --locked python scripts/lifecycle.py project --target <repo> --apply
uv run --locked python scripts/lifecycle.py upgrade --target <repo> --apply
uv run --locked python scripts/lifecycle.py rollback --target <repo> --apply
uv run --locked python scripts/lifecycle.py uninstall --target <repo> --apply
```

`doctor` validates the manifest, TOML, native multi-agent availability, and strict Codex configuration loading. Project initialization creates only `.codex/orchestra/workflows/`, `runs/`, and `install/`.

Preview is the default. Managed-file hashes prevent an upgrade, rollback, or uninstall from overwriting user modifications. Workflow definitions and run artifacts are always preserved.
