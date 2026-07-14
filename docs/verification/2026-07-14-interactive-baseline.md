# Interactive verification baseline — 2026-07-14

This is the source-tree baseline for the first interactive run. It is not a substitute for the target repository's live `.codex/orchestra/verification/` record.

- Operator: not yet assigned
- Environment: macOS, Europe/Budapest
- Codex: `codex-cli 0.141.0`
- Candidate: plugin manifest `0.1.0`
- Source revision: pre-cutover rollback commit `75e7598`; final cutover revision is recorded after validation
- Tree status: structural cutover in progress when this baseline was updated
- Known-good N/cache: not yet recorded

## Automated evidence

| Check | Evidence | Status | Observation |
|---|---|---|---|
| Plugin validator | `uv run --locked python ~/.codex/skills/.system/plugin-creator/scripts/validate_plugin.py .` | pass | `Plugin validation passed` |
| Unit tests | `uv run --locked python -m unittest discover -s tests -v` | pass | 16 tests passed after the root cutover |
| Lifecycle doctor | `uv run --locked python scripts/lifecycle.py doctor` | pass | Codex 0.141.0; 13 skills; manifest, config, and native capability checks passed |
| Native product boundary | manifest and scaffold tests | pass | no manifest MCP/app declaration and no plugin-local `.codex/orchestra/` state |
| Development-only path scan | `colgrep` over `.codex-plugin`, `skills`, `assets`, `config`, and `scripts` | pass | no outer framework, R2, scaffolding-plan, fixture, or developer-home dependency found; the only external-runtime match is `doctor` rejecting `.mcp.json`/`.app.json` |
| Clean-room marketplace install/cache inventory | Stage 1 runbook | pending | requires a disposable marketplace and installed-cache observation |
| Repository and profile install hashes | Stages 1–2 runbook | pending | not applied to user-owned or disposable Codex state in this task |

## Human UI evidence

| Check | Status | Reason/next evidence |
|---|---|---|
| Completely fresh task and natural invocation | pending | open a new Codex task after clean-room install and preserve the UI observation |
| Installed skill/assets and run-directory creation | pending | inspect the installed cache and disposable target after invocation |
| Independence with source/legacy files unavailable | pending | repeat invocation with development source unavailable |
| Project agents visible/selectable | pending | manually exercise all five agents in a supported UI |
| Global profile agents visible/selectable | pending | repeat with a disposable Codex home and explicit profile |
| Metadata, inheritance, capability, switching, delegation | pending | record per-agent observations and native join behavior |
| N conducts one N+1 engagement | pending | choose a bounded improvement and record the live topology/artifacts |
| Checkpoint interruption and recovery | pending | resume the engagement in a transcript-free fresh task |

## Permanent evaluation readiness

Eight scenarios now define setup, perturbation, observable behavior, and pass/fail criteria for bounded workstreams, independent assurance, interruption/recovery, large-repository context, write-conflict isolation, semantic retry, risk-derived assurance, and self-hosting promotion. None has been executed against an installed candidate yet; all verdicts remain `pending`.

## Decision

- Verdict: `pending`
- Promotion blockers: all Stage 1–3 human gates and all live behavioral scenario results
- Current automated residual risk: the candidate is structurally valid on this environment, but installed discovery, UI agent behavior, transcript-independent recovery, and comparative orchestration quality are unobserved
