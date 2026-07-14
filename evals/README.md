# Behavioral evaluations

These scenarios test observable workflow behavior against an installed candidate in a fresh Codex task. Internal implementation details do not satisfy them.

For each scenario, record source revision, workflow snapshot, run state, step results, evidence, elapsed time, spawned agents, user interventions, and residual risk. Fail fast on authority loss, unsafe shared writes, missing evidence, unbounded retries, cache mutation, or transcript-dependent recovery.

`workflows/native-vertical-slice.yaml` is the executable self-hosting fixture.
