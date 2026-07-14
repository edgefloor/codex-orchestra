# Worked Example — Analytics Import Feature

## 1. Operator intent

“Add CSV/JSON import to the analytics application, initially for internal analysts. Preserve existing dashboards; reject malformed records clearly; do not expose customer-confidential data.”

## 2. Grounding

Root spawns `/root/consultant`. The Consultant inspects repository evidence and runs one permitted read-only Explorer for current import/storage behavior.

Grounding artifacts establish:

- users: internal analysts;
- behavior: supported formats, validation/error examples, idempotency expectation;
- non-goals: public upload API and arbitrary schema mapping;
- assurance: internal maturity, confidential data, medium blast radius, reversible deploy but potentially irreversible bad imports;
- open material decision: retain rejected row payloads or only hashes/line numbers.

The Operator accepts the recommendation to avoid retaining confidential rejected payloads.

## 3. Delivery design

`/root/delivery_architect` defines:

- Workstream A — ingestion contract/parser/storage transaction;
- Workstream B — analyst upload/status surface;
- interface: normalized record + structured rejection report;
- integration order: contract/parser before UI wiring;
- evidence: golden format cases, rollback/transaction proof, confidential-data logging check, end-to-end smoke.

## 4. Staffing

Root wakes `/root/manager`. The Manager appoints:

- `/root/manager/tl_ingestion`;
- `/root/manager/tl_product`.

It gives each an initial three-child, two-concurrency wave permit while reserving quality/control capacity.

## 5. Ingestion Team Leader wave

`tl_ingestion` keeps the transaction/integration design itself and creates two separable tasks:

1. explore existing format/test fixtures (read-only Explorer);
2. implement CSV/JSON parser and rejection contract (Worker Terra).

It wakes its branch-local Context Engineer for both capsules. The parser task needs no new organizational capability, so no hire request is needed. The Team Leader spawns `explore_formats` and `w_parser_a1`, works on transaction boundaries, then waits.

The Explorer result and Worker result terminate at `tl_ingestion`, not root. The Team Leader reviews/integrates and spawns `review_parser_a1` because the interface and confidential error behavior are risk-sensitive. The Reviewer reports one high finding: raw rejected values appear in debug logs. The Team Leader creates a precise repair attempt, re-reviews, and receives explicit no-findings.

The Team Leader sends the Manager a branch report: ingestion wave integrated; no capacity change; evidence refs; residual risk is full end-to-end UI integration.

## 6. Product Team Leader hire

`tl_product` determines that existing capacity lacks accessibility-focused upload UI review. It authors a hire request with required outcomes, one-task lifetime, read-only review access, independence need, and one-seat budget.

The Manager approves the temporary seat and wakes `/root/manager/role_architect`. The Role Architect selects `reviewer_terra` with an accessibility-focused Role Card. The Manager extends `tl_product`'s permit. `tl_product` compiles the Context Capsule and spawns the reviewer as its child after implementation.

## 7. Integration and assurance

The named cross-workstream integration owner joins the normalized-record/rejection interface and UI. Root wakes `/root/quality_governor` with the integrated candidate.

The Governor derives gates from confidential data and import irreversibility, then spawns permitted verifiers for:

- transaction rollback/partial import behavior;
- log/artifact confidential-data leakage;
- end-to-end supported/invalid format behavior.

Verifiers return to the Governor. It issues a pass with evidence refs. The Manager recommends milestone acceptance; root gives the Operator one concise checkpoint. The Operator accepts.

## 8. V2 action map

| Event | Physical action | Context effect |
|---|---|---|
| Grounding | root spawns Consultant; Consultant may spawn Explorer | root receives one grounding result |
| Delivery design | root spawns/wakes Architect | evidence joined at Architect |
| Staffing | root spawns/wakes Manager; Manager spawns TLs | Manager sees only TL portfolio outputs |
| Work wave | TL spawns Context Engineer/workers/reviewer | worker detail remains at TL |
| Temporary specialist | TL request → Manager approval → Role Architect card → TL spawn | strict people/technical split |
| Assurance | root spawns/wakes QG; QG spawns verifiers | root receives one gate report |

This example demonstrates the intended context gradient: more detail lower in the tree, more decision compression upward.
