# Interruption recovery

- Behavior: reconstruct an interrupted run from repository and Git evidence.
- Setup: a run with completed steps, one interrupted step, a source revision change, and a late result.
- Prompt: in a fresh task with no transcript, resume the run.
- Perturbation: deliver the stale result after another attempt was assigned.
- Observe: workflow digest, source reconciliation, attempts, reused evidence, ready-step calculation, and next action.
- Pass: completed work is reused; the interrupted step is retried only when safe; the stale result is evidence but cannot complete reassigned work.
- Fail: transcript content is required, all work restarts, accepted results are repeated, or stale evidence advances the run.
