---
name: architect-delivery
description: Convert accepted grounding into a milestone/workstream delivery architecture with dependencies, interfaces, integration, and evidence.
---

1. Verify the grounding bundle is accepted and revision-pinned. Do not infer missing product behavior.
2. Identify behavior slices, irreversible decisions, risk-reduction questions, architectural seams, and evidence needed for milestone acceptance.
3. When repository evidence is missing, use a bounded read-only Explorer/Advisor branch under `$run-bounded-branch`; ask self-contained questions and join evidence locally.
4. Produce the smallest milestone DAG and workstream split that enables parallelism without fragmenting coupled context. Name interfaces, dependency edges, integration owner/order, and Team Leader capability needs.
5. Define milestone checkpoints and evidence outcomes, not every implementation task. Reserve tactical decomposition for Team Leaders.
6. Trace every material plan element to grounding, repository evidence, or an accepted decision. Raise re-grounding only for a binding behavior/scope/risk contradiction.

**Complete when:** the Manager can appoint owners and Team Leaders can plan their first wave without inventing requirements.
