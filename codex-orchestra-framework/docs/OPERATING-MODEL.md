# Operating Model

## Decision plane and control plane

The decision plane contains the Operator and role agents. The control plane applies authorized decisions, validates policy/state, and manages runtime mechanics.

The root Conductor is the global control-plane owner. Authorized parent agents perform branch-local V2 orchestration only inside Delegation Permits. Their collaboration calls do not grant global authority.

## Authority matrix

| Activity | Operator | Consultant | Delivery Architect | Manager | Team Leader | Context Engineer | Role Architect | Quality Governor | Leaf | Root |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Outcomes/scope | A | R | C | C | C | — | — | C | — | apply |
| Assurance dimensions | A | R | C | C | C | — | — | R/C | — | apply |
| Milestone/workstream design | C | C | A/R | — | C | — | — | C | evidence only | apply |
| Appoint owners/capacity | C | — | C | A/R | requester | — | C | C | — | validate/apply |
| Tactical wave | — | — | C | — | A/R | C | — | C | — | validate |
| Technical hire requirements | — | — | C | approve only | A/R | — | C | C | — | validate |
| Role Card | — | — | — | A for approved seat | C | — | R | C | — | validate |
| Context Capsule | — | — | C | — | A for task meaning | R | — | C | — | validate |
| Spawn Team Leader | — | — | — | A/R | — | — | — | — | — | permit/global capacity |
| Spawn worker/reviewer | — | — | — | — | A/R | — | — | — | — | permit/global capacity |
| Implement/integrate | — | — | — | — | A/R | — | — | — | R | lease/worktree |
| Gate design/verdict | A for accepted risk | C | C | C | C | — | — | A/R | evidence | validate/apply |
| Global phase/state | — | — | — | recommend | evidence | — | — | evidence | envelope only | A/R |

## Manager contract

A Manager output must be organizational: owner, reporting line, capacity, priority, escalation, decision status, or checkpoint recommendation. The Manager may create/wake Team Leaders and its Role Architect, then wait for compact reports. It never contacts workers or decides technical implementation.

## Team Leader contract

The Team Leader is the workstream's technical owner and branch controller. It keeps central/high-context work, delegates separable work, defines hire requirements, uses branch-local context engineering, reviews child output, and integrates.

## Persistent and generated identity

- Stable TOMLs define recurring authority and personality.
- Canonical paths and roster events provide runtime identity.
- Durable role memory records current mandate/decisions/handoffs, not chat chronology.
- Role Cards add task-specific expertise to stable leaf archetypes.
- Promotion to a persistent TOML requires recurrence/continuity, evals, and approval.

## Coordination artifacts

There is no all-agent meeting. Shared alignment is projected through:

- grounding bundle;
- delivery plan;
- staffing decision and Delegation Permits;
- tactical wave;
- Context Capsules;
- branch reports;
- portfolio snapshot;
- gate report;
- alignment digest and decision log.
