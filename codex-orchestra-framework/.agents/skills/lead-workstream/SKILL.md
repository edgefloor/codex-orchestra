---
name: lead-workstream
description: Team Leader procedure for one tactical wave, branch-local context engineering, squad formation, review, direct execution, and integration.
---

1. Pin the accepted milestone/workstream, scope revision/digest, interfaces, and current Manager-issued Delegation Permit. Plan only the next bounded wave.
2. Choose the smallest useful shape: `single_owner`, `parallel_map`, `sequential_pipeline`, `review_loop`, or `fanout_verify`. Define tasks, dependencies, acceptance, write domains, join owner, child/concurrency budget, review policy, integration order, and checkpoint.
3. Keep the hardest, highest-context, or integration-critical task yourself when practical. Delegate only where parallelism or specialization saves more context than it costs.
4. For each delegated task, spawn/reuse a branch-local Context Engineer to compile a revision-pinned capsule. Reject stale, bloated, or meaning-changing capsules.
5. When capability/capacity is missing, emit a precise hire_request. After Manager approval and Role Architect output, verify the Role Card and permit, then spawn the approved leaf worker yourself.
6. Use `$run-bounded-branch`: dispatch only dependency-ready work, keep write domains disjoint, wait for child events, and review results at this branch. Add an independent Reviewer for interface/risk-sensitive work or when the plan requires it.
7. Diagnose semantic failures and create a changed repair task/capsule; do not replay vague instructions. Integrate accepted changes through your serialized workstream queue.
8. Emit a compact branch report to the Manager: wave outcome, milestone/priority/capacity consequences, at most five material risks/findings, evidence refs, and decisions needed. Keep technical detail in artifacts.

**Complete when:** the wave is integrated or has a decision-ready hire/blocker/drift request, and the Manager need not read worker output.
