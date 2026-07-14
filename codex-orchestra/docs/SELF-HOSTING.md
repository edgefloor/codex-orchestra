# Self-hosting vertical slice

Orchestra version N conducts work against the source tree for candidate N+1. The conductor must not edit its own installed cache.

Minimum topology:

```text
Operator -> Consultant -> Team Leader -> Worker -> Reviewer -> Operator checkpoint
```

1. Ground the Orchestra source repository and save context evidence.
2. Accept a charter for one small candidate improvement.
3. Plan one workstream with disjoint implementation and independent review assignments.
4. Use native Codex agents and worktrees; persist task capsules and results in the source repository's `.codex/orchestra/` tree.
5. Run locked manifest, config, and test validation through `uv run` and save exact evidence.
6. Update the development plugin cachebuster through the supported plugin tooling; do not modify a cached install.
7. Install the candidate while preserving the known-good version, then invoke `$codex-orchestra:orchestrate` in a fresh Codex task.
8. If candidate validation fails, remove it and reopen the run with the known-good version plus the recovery artifact.

The repository includes a bounded scaffold-improvement engagement under its top-level `.codex/orchestra/`. Installation into user-owned Codex state and the fresh-task checkpoint remain explicit operator actions.

Execute the live slice as Stage 3 of [INTERACTIVE-VERIFICATION.md](INTERACTIVE-VERIFICATION.md). Score it with `evals/scenarios/self-hosting-promotion.md`; do not promote N+1 from deterministic checks alone.
