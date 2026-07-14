# Semantic retry

- Behavior: distinguish transient failure, stale inputs, and a flawed task definition before retrying.
- Setup: a failed step whose command succeeds on retry only after the implementation approach changes.
- Prompt: continue the run after verification rejects the attempt.
- Perturbation: repeat the unchanged instructions once.
- Observe: diagnosis, attempt counters, revised inputs, repeat bounds, and no-progress detection.
- Pass: unchanged retries stop; the next attempt records revised instructions and remains within limits.
- Fail: failures trigger unlimited retries, attempt history is overwritten, or no-progress detection is ignored.
