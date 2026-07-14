# Model, Reasoning, and Fan-out Routing

## Decision axes

Route by complexity, interface/taste judgment, uncertainty, blast radius/reversibility, context coupling, tool needs, throughput/cost, and independence.

## Defaults

| Route | Model | Effort |
|---|---|---:|
| Consultant / Delivery Architect / Manager / Quality Governor | `gpt-5.6-sol` | high |
| Team Leader | `gpt-5.6-sol` | medium; high when critical |
| Context Engineer / Role Architect / Explorer / Reviewer / Verifier | `gpt-5.6-terra` | medium |
| Mechanical Worker | `gpt-5.6-luna` | low |
| Default Worker | `gpt-5.6-terra` | medium |
| Critical Worker / independent Advisor | `gpt-5.6-sol` | high |

## Escalation

Escalate after evidence such as irreversible/broad-risk consequence, unresolved cross-system ambiguity, diagnosed repeated semantic failure, security/compliance/data-loss criticality, architecture-sensitive cross-workstream integration, or a contested high-severity independent review.

Do not escalate merely because a task is large, a worker has not inspected evidence yet, or a stronger model feels prestigious. Large repetitive work may need bounded parallelism rather than a stronger single agent.

## Effort semantics

- `low`: exact, reversible transformation;
- `medium`: normal coding/repository synthesis;
- `high`: ambiguity, architecture, security, difficult debugging, consequential decisions;
- `xhigh`/`max`: rare extra compute for one agent, justified and recorded;
- `Ultra`: a multi-agent workflow topology.

No archetype defaults to Max or Ultra. Ultra is prohibited in leaves and requires an exceptional permit with owner, child/concurrency/token-time budgets, isolation, and stop conditions.

## Workflow decision

Use a single owner for coupled/high-context work. Use parallel map only for independent items or disjoint write domains. Use a sequential pipeline when one canonical artifact feeds the next. Use a review loop for independent implementer/reviewer. Use fan-out/verify only with a named join owner and explicit budgets.
