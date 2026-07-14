# Seed migration inventory

The source inventory is `../codex-orchestra-framework/`. It remains untouched and runnable as the fallback conductor.

| Seed material | Classification | Scaffold treatment |
|---|---|---|
| 13 `.agents/skills/*` procedures | Normalize for plugin packaging | Consolidated into the primary `orchestrate` skill and 12 focused plugin skills; native collaboration semantics retained. |
| 14 `.codex/agents/*.toml` archetypes | Normalize for plugin packaging | Reduced to consultant, Team Leader, worker, reviewer, and verifier templates in project and global scopes. Task-specific model names and nicknames removed. |
| `.codex/config.toml` | Replace after capability validation | Replaced V2-specific table/flags with stable `features.multi_agent`, `agents.max_threads`, and `agents.max_depth`; no model default. |
| 6 `.orchestra/policies/*.yaml` | Retain as design reference | Core native-only, bounded-delegation, assurance, and bootstrap invariants summarized in `assets/policies/orchestration.yaml`. |
| 20 `.orchestra/schemas/*.json` | Retain as design reference | Scaffold exposes only minimal run-state and assignment-result schemas. Expand through later versioned work. |
| 14 `.orchestra/templates/*` | Normalize for plugin packaging | Replaced by six markdown templates matching `.codex/orchestra/` state locations. |
| Charter, plan, roster, runtime seed data | Replace | Mutable state is initialized in each target repository, never bundled as live plugin state. |
| `tools/orchestra.py` SQLite/reference control plane | Retain as design reference | Not migrated. `scripts/lifecycle.py` only manages transparent installation files and is not a scheduler. |
| `tests/test_orchestra.py` | Retain as design reference | New tests cover plugin shape, config parsing, lifecycle safety, state separation, and skill discovery. |
| `DESIGN.md` and 13 supporting docs | Retain as design reference | Scaffold docs focus on configuration, lifecycle, self-hosting, and validation; detailed operating-model redesign is deferred. |
| Seed `.gitignore`, `AGENTS.md`, and package metadata | Replace | Target package is self-contained and does not inject repository instructions without an explicit lifecycle action. |

## Capability-validation findings

- Seed model identifiers (`gpt-5.6-sol`, `terra`, `luna`) are environment-specific and are not safe configuration defaults.
- Seed `[features.multi_agent_v2]` options such as routing metadata visibility, namespace selection, and wait-time limits are version-sensitive. On Codex CLI 0.141.0, `multi_agent` is stable while `multi_agent_v2` is under development and disabled.
- `agents.max_threads` and `agents.max_depth` strictly parse on CLI 0.141.0. Other seed agent runtime keys were not required by this scaffold.
- Custom-agent model selection is intentionally omitted so the active Codex installation and user policy choose it.
- The plugin manifest must not declare `hooks` with the current validator; the empty hooks surface is documented but inactive.

These findings must be rechecked with `doctor` after every Codex upgrade.
