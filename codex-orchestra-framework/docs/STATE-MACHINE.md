# Orchestration State Machine

## States

```text
NEW
  -> GROUNDING
  -> GROUNDED
  -> DELIVERY_DESIGN
  -> ARCHITECTED
  -> STAFFING
  -> WAVE_READY
  -> EXECUTING
  -> INTEGRATING
  -> ASSURANCE
  -> MILESTONE_CHECKPOINT
  -> WAVE_READY | DELIVERY
  -> OPERATOR_ACCEPTANCE
  -> CLOSED
```

Cross-cutting states: `REGROUNDING`, `PAUSED`, `ABORTED`.

## Gates and owners

| State/gate | Decision owner | Required durable outputs | Root action |
|---|---|---|---|
| `NEW -> GROUNDING` | Operator/root | captured intent | spawn/wake Consultant |
| `GROUNDING -> GROUNDED` | Operator + Consultant | Brief, Scope, Assurance, acceptance, decisions/defaults | record revision/digest |
| `GROUNDED -> DELIVERY_DESIGN` | root | accepted grounding ref | spawn/wake Delivery Architect |
| `DELIVERY_DESIGN -> ARCHITECTED` | Delivery Architect | delivery plan and evidence contract | validate/record |
| `ARCHITECTED -> STAFFING` | Manager | workstream ownership, TL appointments, capacity | spawn/wake Manager |
| `STAFFING -> WAVE_READY` | Manager + TL | Delegation Permit, tactical plan, hires/Role Cards, capsules | validate readiness |
| `WAVE_READY -> EXECUTING` | Team Leader | dependency-ready assignments, leases/worktrees | TL branch dispatch |
| `EXECUTING -> INTEGRATING` | Team Leader | reviewed results | TL integrates |
| `INTEGRATING -> ASSURANCE` | Team Leader/root | integrated candidate and evidence refs | spawn/wake QG |
| `ASSURANCE -> MILESTONE_CHECKPOINT` | Quality Governor | gate report | validate verdict |
| checkpoint -> next wave | Manager/root | portfolio recommendation and renewed permits | resume TL wave |
| checkpoint -> delivery | Manager + Operator as required | accepted milestone/final evidence | advance |
| `DELIVERY -> OPERATOR_ACCEPTANCE` | Operator | delivery brief, residual risk, runbooks/docs as triggered | decision brief |
| acceptance -> `CLOSED` | Operator/root | acceptance record | close/release seats |

## Branch-local subloops

### Consultant/Architect/QG

```text
permit read-only question(s)
spawn bounded evidence children
wait
validate/join
return one grounding/plan/gate artifact
```

### Manager

```text
appoint/wake Team Leaders
approve/reject hires and issue permits
wait for TL branch reports
resolve ownership/capacity/priority
return portfolio/checkpoint decision
```

### Team Leader

```text
plan one wave
compile capsules
spawn approved leaves
wait/review/repair
integrate
return branch report
```

## Re-grounding

A drift alert is classified before transition:

- local defect/tactical issue stays with Team Leader;
- milestone/workstream boundary returns to Delivery Architect;
- ownership/capacity stays with Manager;
- product behavior/scope/external obligation/assurance assumption returns to Consultant + Operator;
- evidence/gate interpretation stays with Quality Governor.

Only affected permits, capsules, leases, and tasks are invalidated. Resume from the earliest affected gate.

## Completion rule

V2 terminal notification cannot pass a gate. Every transition requires current revision/digest, valid authority/permit, required artifacts, evidence, and deterministic state commit.
