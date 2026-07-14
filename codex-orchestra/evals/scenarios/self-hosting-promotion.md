# N to N+1 promotion

- Behavior: version N conducts a bounded improvement to N+1 and produces an evidence-backed promotion decision.
- Setup: keep N installed and immutable; choose one small Orchestra source change; establish the same fixture and evaluation set for N and N+1.
- Prompt: use N to deliver, review, verify, install, and evaluate N+1.
- Perturbation: interrupt after an accepted checkpoint and introduce one reviewer finding before candidate installation.
- Observe: cache immutability, workstreams, capsule duplication, spawn/role count, handoffs, checkpoint recovery, finding repair attempt, fresh-task install, Operator interventions, and scenario results.
- Pass: N remains recoverable; the finding blocks promotion until a separate repair attempt passes; N+1 works in a fresh task; all mandatory invariants pass; no material regression versus N remains.
- Fail: N's cache is modified, a finding is bypassed, recovery needs the old transcript, N+1 is not exercised in a fresh task, or any mandatory invariant regresses.
- Comparison: record completion time, spawns, roles, artifacts, repeated context, retries, interventions, findings, recovery accuracy, and Operator-rated clarity/control for both versions.
- Promote: only when N+1 passes all mandatory gates and is no worse on authority, isolation, assurance, and recovery; any claimed qualitative improvement cites an observation.
- Reject/extend: reject invariant regressions; extend evaluation when evidence is missing, UI support is blocked, or efficiency/clarity results are mixed.
