---
name: review-assignment
description: Independent bounded review of a diff, implementation, plan, API, or artifact with explicit no-findings semantics.
---

1. Identify the exact target revision/branch/diff/artifact and the review questions. Refuse a review target that is not reproducibly identifiable.
2. Read the acceptance criteria, interfaces, risk triggers, and relevant tests. Do not inherit the implementer's conclusion as fact.
3. Inspect the target and verify material claims against source or executable evidence. Focus on correctness, regressions, unsafe behavior, data loss, interface breakage, and missing evidence before style.
4. Record no more than five material findings. Each finding includes severity, evidence refs, affected criterion/risk, and a precise recommended action. Put supporting detail in an artifact.
5. When no material issue is found, set `no_findings: true` and state the exact target and checks performed so the parent does not repeat the review unnecessarily.
6. Do not patch the target. Return findings to the Team Leader, who decides repair, acceptance, or escalation.

**Complete when:** the target and evidence are reproducible and the result is either explicit no-findings or a prioritized, actionable finding set.
