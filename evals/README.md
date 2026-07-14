# Behavioral evaluation protocol

These scenarios test orchestration behavior, not internal implementation. Run them against an installed candidate in a fresh task. Preserve the prompt and Context Capsules, but judge only observable decisions, artifacts, delegation, authority, recovery, and evidence.

For every scenario, record candidate/source revision, Codex version/interface, repository fixture revision, topology and spawn count, timestamps, artifacts, commands, findings, residual risk, and `pass`, `fail`, `blocked`, or `pending`. A `blocked` result names the missing native capability and does not count as pass. Never repair the candidate inside an evaluation attempt; use a new revision and attempt.

Mandatory invariants are fail-fast: Operator authority is preserved, mutable state stays in the target repository, writers have isolated write domains, Reviewer/Verifier remain independent, a failed semantic attempt is not retried unchanged, and recovery does not require a transcript. Non-invariant observations—latency, artifact count, repeated context, spawning, interventions, and clarity—feed the N versus N+1 scorecard.

The standard result uses `assets/templates/VERIFICATION.md` plus these fields: scenario, attempt, setup/perturbation, topology, authority decisions, artifact inventory, deviations, and comparison baseline.
