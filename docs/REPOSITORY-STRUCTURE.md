# Repository structure

```text
crates/orchestra-core/              restricted compiler, plan validation, runtime, state, context, host trait
crates/orchestra-lifecycle/         preview-first plugin configuration lifecycle CLI and tests
sdk/                                TypeScript authoring declarations (never executed)
integration/codex/                  pinned revision, minimal patch, and Codex overlay crates
scripts/codex-integration.sh        deterministic apply/build verification
skills/                             user-facing native-tool guidance
config/                             optional project/global Codex configuration
assets/templates/                   .workflow.ts and summary templates
evals/workflows/                    compiler/runtime fixtures
evals/scenarios/                    behavioral acceptance scenarios
docs/adr/                           architecture decisions
```

Runtime-owned target-repository data:

```text
.codex/orchestra/runs/<run-id>/
├── workflow.json                   immutable compiled-plan snapshot
├── state.json                      atomic checkpoint and hashes
├── outputs/<step>.json             validated outputs
├── evidence/checks/                command evidence
├── approvals/                      explicit decisions
└── summary.md                      paused or terminal summary
```

Temporary worktrees may exist under `.codex/orchestra/worktrees/` while a run is active and are removed by the runtime. No workflow engine or mutable state is stored in the installed plugin cache.
