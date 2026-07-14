# Codex Multi-Agent V2 Collaboration Contract

## 1. Runtime role

V2 is the live collaboration transport. It is not the durable project database and it is not the Manager.

- `/root` owns global phase/state, policy, portfolio capacity, recovery, and Operator dialogue.
- Authorized parent roles own one bounded child branch.
- `.orchestra/` artifacts and the runtime event store own durable truth.
- Mailboxes carry compact commands, deltas, receipts, and result references.

## 2. Bounded physical topology

```text
/root
├── consultant
│   └── explorer/advisor (optional)
├── delivery_architect
│   └── explorer/advisor (optional)
├── manager
│   ├── role_architect
│   └── tl_<workstream>
│       ├── context
│       ├── worker
│       └── reviewer
└── quality_governor
    └── verifier/advisor
```

`agents.max_depth = 3`. The raw configuration limit is only one guard; `.orchestra/policies/collaboration-v2.yaml` also enforces the role-to-child allowlist and permit budgets.

Canonical task-name patterns:

- stable seat: `consultant`, `manager`, `quality_governor`, `tl_<workstream>`;
- service: `role_architect`, `context`;
- worker: `w_<task>_a<attempt>`;
- reviewer: `review_<task>_a<attempt>`;
- verifier: `verify_<gate>_a<attempt>`;
- read-only evidence: `explore_<question>` or `advise_<decision>`.

Paths, not random nicknames, are runtime identity.

## 3. Spawn authority matrix

| Caller role | Allowed child archetypes | Purpose |
|---|---|---|
| root | Consultant, Delivery Architect, Manager, Quality Governor | phase owners |
| Consultant | Explorer, Advisor | grounding evidence only |
| Delivery Architect | Explorer, Advisor | plan evidence/challenge only |
| Manager | Team Leader, Role Architect | people management only |
| Team Leader | Context Engineer, Explorer, Advisor, Worker, Reviewer | one approved workstream wave |
| Quality Governor | Verifier, Advisor | assurance evidence |
| leaf/service role | none | no descendants |

A V2 tool being available does not grant organizational authority. Every non-root spawn requires a current Delegation Permit.

## 4. Tool semantics and policy

### `spawn_agent`

Create a new seat or fresh attempt. The parent supplies:

- canonical `task_name`;
- approved `agent_type`, model, and reasoning effort;
- `fork_turns` (normally `none`);
- compact message reference;
- Role Card and Context Capsule references where applicable;
- output schema/artifact location;
- Delegation Permit reference.

The parent records a command/receipt and owns the child join.

### `send_message`

Queue a passive delta without starting a new turn. Use for:

- evidence or interface deltas that may wait;
- compact receipts;
- bounded branch reports to an ancestor/root while it is waiting.

Do not use it to contradict a running assignment. Interrupt/reissue when meaning changed.

### `followup_task`

Wake an existing resident seat for new work or immediate review. It consumes execution capacity. Reuse only when the role/config/security boundary and durable handoff remain valid.

`followup_task` cannot target `/root`; use `send_message` for a compact root receipt/report.

### `wait_agent`

Normal event loop for every branch parent. It ends on mailbox activity, user steering, or timeout. A timeout is a reconciliation tick—not proof that a child failed.

### `interrupt_agent`

Use for cancellation, stale binding revision, runaway behavior, write-domain conflict, or policy/safety stop. Record the reason first.

### `list_agents`

Use once at startup/recovery or when path identity is uncertain. Do not turn it into polling.

## 5. Fork policy

| Mode | Policy |
|---|---|
| `none` | default; Context Capsule provides task context |
| bounded N | immediate parent-child continuation only, with rationale |
| `all` | exceptional; recorded authorization; cannot combine with role/model/effort overrides |

A durable handoff is preferred over transcript inheritance for long-horizon continuity.

## 6. Parent branch loop

```text
validate role authority + Delegation Permit
reserve child capacity / work domain
spawn or wake permitted child
continue useful parent work
wait_agent
on event:
  validate envelope + permit + revision/digest + lease + evidence
  classify failure or accept result
  review/join at this parent
  dispatch another dependency-ready child if budget remains
at checkpoint:
  emit bounded branch report upward
  release/renew permit and seats
```

The parent above never needs to read child transcripts.

## 7. Hiring and squad formation

1. Team Leader identifies a real capability/capacity/independence gap and writes the technical hire request.
2. Manager approves/rejects/rebalances the organizational commitment.
3. Manager wakes/spawns its Role Architect with the approved request.
4. Role Architect compiles a bounded Role Card using a stable archetype.
5. Manager issues or revises the Team Leader Delegation Permit.
6. Team Leader wakes its Context Engineer for the assignment capsule.
7. Team Leader spawns the worker as its child and owns the result/review.

The Manager does not write the worker prompt or see raw worker output.

## 8. Review and verification

A Reviewer is a Team Leader child for implementation/plan/API/diff review. It returns explicit `no_findings` or at most five findings. It never patches.

A Verifier is a Quality Governor child for executable evidence against a risk-derived gate. It remains independent and does not modify production source.

High-risk work may have both: Reviewer for implementation quality, then Verifier for milestone evidence.

## 9. Fan-out budget

Ordinary defaults:

- 10 live threads globally;
- at most 2 active Team Leaders;
- at most 3 new children per Team Leader wave;
- at most 2 parallel writers per workstream, 4 globally;
- at most 4 read-only children in a Consultant/Architect/QG analysis branch;
- at least one control and one quality/emergency slot reserved;
- at most 12 dynamic workflow nodes without Operator override.

Every fan-out names a join owner, child/concurrency/attempt budget, target artifact, review reserve, isolation boundary, and checkpoint.

## 10. Reasoning modes

`max` is treated as extended reasoning by one agent. It may be used for a rare consequential decision with cost justification.

`Ultra` or equivalent autonomous subagent behavior is treated as a workflow topology. It is not allowed in leaf TOMLs. An exception requires a named owner, explicit child/concurrency/token-time budgets, isolation, stop conditions, and recorded authorization.

## 11. User steering

User input interrupts waits. The receiving parent/root:

1. preserves event cursor, permits, leases, and worktrees;
2. classifies whether the steer changes scope, priority, authorization, or presentation;
3. queues a safe passive delta when current work remains valid;
4. interrupts only invalidated attempts;
5. records a binding revision/repackages capsules when required;
6. resumes without duplicate dispatch.

## 12. Recovery

1. Load durable events, roster paths, permits, leases, worktrees, revision/digest.
2. Call `list_agents` once.
3. Reconcile from leaves upward.
4. Reject stale/late results from transitioning reassigned work.
5. Interrupt invalid survivors and preserve evidence/worktrees.
6. Reuse a resident path only when its role/config/security and handoff remain valid.
7. Emit one reconciliation report and resume the nearest safe gate.
