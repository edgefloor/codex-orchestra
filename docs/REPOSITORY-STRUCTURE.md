# Repository structure

```text
crates/orchestra-core/              restricted compiler, plan validation, runtime, state, context, host trait
crates/orchestra-lifecycle/         preview-first plugin configuration lifecycle CLI and tests
crates/orchestra-product/           Product manifest, release gates, update state, and verification CLI
evaluator/                          pinned one-request exact-Zod Product worker
sdk/                                TypeScript authoring declarations (never executed)
integration/codex/                  pinned revision, maintained Codex patch, and source overlay
integration/t3code/                 pinned revision and maintained retained-desktop patch
product/                            exact Product pins, release policy, and evidence schemas
scripts/codex-integration.sh        deterministic apply/build verification
scripts/t3code-integration.sh       retained desktop apply/build/test verification
scripts/product-source-prepare.sh   prepare both exact maintained fork trees
scripts/product-dev-build.sh        build and verify the dogfood Product tuple
scripts/product-release.sh          two-architecture signed release and publication gates
scripts/characterize-pinned-skills.sh pinned native skill boundary verification
skills/                             user-facing native-tool guidance
config/                             optional project/global Codex configuration
assets/templates/                   .workflow.ts and summary templates
evals/workflows/                    compiler/runtime fixtures
evals/scenarios/                    behavioral acceptance scenarios
docs/adr/                           architecture decisions
docs/ARCHITECTURE.md                decision-complete product-fork synthesis
docs/WORKFLOW-COMPILATION.md        accepted evaluator artifact and trust contract
docs/agents/                        tracker and agent-operation contracts
landing/                            static product landing page and its browser tests
```

Runtime-owned target-repository data:

```text
.codex/orchestra/runs/<run-id>/
├── workflow.json                   immutable compiled-plan snapshot
├── inputs.json                     immutable resolved inputs
├── state.json                      atomic checkpoint and hashes
├── outputs/<step>.json             validated outputs
├── evidence/checks/                command evidence
├── evidence/skills/                immutable skill manifest, instructions, and declared resources
├── evidence/changes/               isolated patches plus the aggregate promoted.patch
├── approvals/                      explicit decisions
└── summary.md                      paused or terminal summary
```

Temporary worktrees may exist under `.codex/orchestra/worktrees/` while a run is active. They are removed after success, rejection, cancellation, or ordinary failure; a shared worktree is retained after a promotion conflict so resume can retry. No workflow engine or mutable state is stored in the installed plugin cache.
