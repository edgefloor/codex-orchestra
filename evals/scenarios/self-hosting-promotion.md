# Self-hosting promotion

- Behavior: installed Orchestra N runs the native vertical-slice workflow against candidate N+1.
- Setup: immutable installed N, source candidate N+1, and a clean repository-local run directory.
- Prompt: use Orchestra to improve and validate itself through the supplied workflow.
- Perturbation: interrupt after the parallel read stage, then resume in a fresh task.
- Observe: installed-cache digests, workflow snapshot, parallel agents, worktree isolation, checks, review, approval, recovery, and final summary.
- Pass: N stays byte-identical; recovery uses durable state; acceptance promotes the verified N+1 patch unstaged into its target checkout; N+1 is installed separately and exercised in a fresh task.
- Fail: the installed cache changes, transcript continuity is required, rejection changes the target, a promotion conflict overwrites user work, or the candidate replaces N before validation.
