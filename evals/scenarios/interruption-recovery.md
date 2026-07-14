# Checkpoint interruption and recovery

- Behavior: resume from durable accepted evidence after the coordinating task disappears.
- Setup: reach an accepted charter, plan, one completed assignment, and a pending dependent assignment; persist a checkpoint.
- Prompt: in a fresh task with no transcript, ask Orchestra to recover the engagement.
- Perturbation: include one late result from superseded source revision.
- Observe: reconciliation of charter, plan, decisions, source/worktrees, results, verification, and exact next action.
- Pass: recovery identifies accepted state and earliest incomplete gate, treats the late result as evidence but not advancement, and resumes only dependent work with a revision-pinned capsule.
- Fail: transcript content is required, all work restarts, accepted work is repeated, or the stale result advances the engagement.
