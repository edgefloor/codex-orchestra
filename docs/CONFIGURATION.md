# Configuration

`config/project.toml` and `config/orchestra.config.toml` enable multi-agent support, cap the thread tree, and set `max_depth = 1` so child recursive delegation is disabled by default. Workflows choose explicit models and reasoning settings per agent step; `config/agents/` role personalities are intentionally absent.

Use `cargo run -p codex-orchestra-lifecycle -- project --target <repo>` or `cargo run -p codex-orchestra-lifecycle -- profile --codex-home <home>` to preview, then add `--apply`. Existing or locally modified configuration is never overwritten silently.

These templates require the Orchestra-enabled build pinned by `integration/codex/UPSTREAM_REVISION`. A stock Codex installation may load the plugin skills but will not expose `orchestra_validate`, `orchestra_run`, `orchestra_resume`, `orchestra_status`, `orchestra_cancel`, or `orchestra_query`.
