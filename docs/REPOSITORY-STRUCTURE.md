# Repository structure

```text
crates/orchestra-core/              restricted compiler, plan validation, runtime, state, context, host trait
crates/orchestra-lifecycle/         preview-first plugin configuration lifecycle CLI and tests
crates/orchestra-product/           Product manifest, release gates, update state, and verification CLI
evaluator/                          pinned one-request exact-Zod Product worker
sdk/                                TypeScript authoring declarations (never executed)
product/                            exact Product pins, release policy, and evidence schemas
scripts/product-source-prepare.sh   clone both exact public hard-fork commits
scripts/product-source-verify.sh    verify fork, upstream, runtime, and protocol identities
scripts/orchestra-desktop.sh        standalone desktop verify/build/test/smoke helper
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
