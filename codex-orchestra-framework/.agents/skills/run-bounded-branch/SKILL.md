---
name: run-bounded-branch
description: Parent-agent procedure for spawning, waiting on, joining, and reporting a permitted V2 child branch without leaking child context upward.
---

1. Confirm your role is allowed to parent the requested archetype in `.orchestra/policies/collaboration-v2.yaml`.
2. Load the current Delegation Permit. Verify parent path, allowed child types, task/wave/gate refs, scope revision/digest, maximum children/concurrency, write limits, expiry, and stop condition.
3. Reserve one child at a time against the permit. For each new child, use a canonical name, approved agent type/model/effort, and `fork_turns: "none"` by default. Point to the Role Card, Context Capsule, output schema, and artifact directory; never paste the parent transcript.
4. Use `followup_task` to wake an existing permitted child for new work. Use `send_message` only for a passive delta or receipt that may wait. Do not steer a running turn with contradictory instructions; interrupt and issue a new attempt when meaning changed.
5. Continue your own parent work where useful, then call `wait_agent`. Do not poll agent status or filesystem queues. Treat timeout as a bounded reconciliation tick, not a failure.
6. On child completion/mailbox activity, validate schema, scope revision/digest, permit, lease, write domain, evidence, and independence before accepting the result. A completion notification is not proof of success.
7. Diagnose failures. Retry only transient failures; stale context is repackaged; semantic failure becomes a revised task/attempt; unsafe or runaway work is interrupted.
8. Join results at the closest competent parent. Keep raw child output and detailed findings in artifacts. Send upward only a `branch_report` containing outcome, at most five material findings, evidence refs, capacity/decision consequences, residual risks, and the exact next action.
9. Close the wave when its checkpoint/stop condition is met. Release unused permit capacity and retain/reuse only seats whose declared lifetime continues.

**Complete when:** all permitted children are reconciled, the branch has one authoritative joined result, and the parent above can act without reading child transcripts.
