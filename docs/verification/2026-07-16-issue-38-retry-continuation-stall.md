# Issue #38 retry, continuation, and stall handling

Status: implemented at the native Automation claim and Codex extension seams.

## Contract

- The persisted Issue claim owns retry attempts, the bounded delay, due time, Workflow invocation
  count, current `max_turns` window, continuation count, and last-progress time.
- Transient failures use deterministic capped exponential backoff from the profile polling interval.
- A completed Workflow whose tracker issue remains active schedules a continuation. Exhausting
  `agent.max_turns` resets only the invocation window after the delay; it does not replace the claim,
  Issue task, worktree, profile digest, Child Run history, or effect receipts.
- The next explicit native task action dispatches due work. No detached timer, daemon, scheduler, or
  additional App Server process was added.
- Terminal tracker state and Automation cancellation clear pending work before another invocation.
- Stall state is derived from the persisted last-progress time and profile timeout when status or
  queue projection is read. Waiting gates and retry delays take precedence, and the read does not
  rewrite the checkpoint merely because wall time advanced.

## Recovery evidence

Focused fixtures cover capped retry delay, early-dispatch rejection, checkpoint reload, continuation
after `max_turns`, retained claim/worktree identity, terminal cancellation, explicit cancellation,
and active/waiting-retry/stalled classification. The native fixture runner reopens a due claim and
invokes the selected typed Workflow beneath its retained Issue task.

## Verification

- root Automation fixtures: 12 passed;
- pinned Codex Orchestra core: 72 passed;
- pinned Codex Orchestra extension: 11 passed;
- Rust formatting and scoped diff checks passed.
