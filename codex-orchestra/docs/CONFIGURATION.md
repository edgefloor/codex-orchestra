# Configuration modes

## Repository mode

Preview with:

```bash
uv run --locked python scripts/lifecycle.py project --target /path/to/repository
```

Apply only after reviewing conflicts by adding `--apply`. This installs `.codex/config.toml`, `.codex/agents/*.toml`, and the `.codex/orchestra/` state directories. A trusted repository loads these defaults automatically.

## Global selectable profile

Preview with:

```bash
uv run --locked python scripts/lifecycle.py profile --codex-home ~/.codex
```

This targets `~/.codex/orchestra.config.toml` and `~/.codex/agents/*.toml`. Select it explicitly with `codex --profile orchestra`.

## Intentional global default

Run `global-default` only when the user explicitly wants Orchestra defaults in `~/.codex/config.toml`. The helper refuses conflicting existing files; it never silently merges or replaces user-owned TOML. Reconcile reported keys manually, validate with `codex --strict-config doctor`, then rerun the preview.

## Precedence and conflicts

Codex base user configuration is loaded first, an explicit profile layers on it, and trusted project configuration applies repository scope. Exact precedence is Codex-version behavior, so `doctor` records the installed version and strict parsing result. Orchestra does not depend on undocumented defaults. Agent TOMLs are copied into the selected scope and remain manually selectable in supporting Codex interfaces.
