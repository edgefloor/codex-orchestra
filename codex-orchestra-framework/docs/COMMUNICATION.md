# Communication and Context Protocol

## Three channels

| Channel | Purpose | Not for |
|---|---|---|
| V2 mailbox | live command, delta, receipt, completion activity | durable project truth |
| repository artifacts | charter, decisions, permits, capsules, results, evidence, handoffs | agent polling queue |
| runtime event store | paths, attempts, leases, phase, reconciliation | product/technical judgment |

## Default routing

Detailed work moves only to the closest competent parent. Upward communication is a reference-first summary:

```text
worker/reviewer -> Team Leader -> branch_report -> Manager -> portfolio checkpoint -> root/operator
verifier -> Quality Governor -> gate_report -> root/operator
explorer -> Consultant/Architect -> grounding/plan result -> root
```

No ambient group chat. Cross-branch messages require a named interface/service need and should point to a durable artifact.

## Message types

- **command** — start/wake/interrupt under a permit;
- **delta** — non-urgent evidence/interface update;
- **receipt** — accepted/started/completed/errored path status;
- **result** — leaf assignment outcome;
- **result (review profile)** — explicit target, no-findings or bounded findings;
- **branch_report** — joined outcome and organizational consequence;
- **decision_request** — one owner, deadline, options, recommendation/default;
- **blocker** — evidence-backed inability to proceed;
- **drift_alert** — binding mismatch and affected set;
- **gate_report** — risk-derived assurance verdict.

All envelopes pin project, sender/path, scope revision, alignment digest, and relevant permit/task/gate references.

## Context rules

A Context Capsule is sufficient, minimal, and revision-pinned. It includes only the goal/done/deliverables, boundaries, interfaces, must-read refs, triggered on-demand refs, execution/validation contract, escalation/default, and budgets.

Typical budgets:

- mechanical leaf: 2k–5k input tokens, 150–250 word result;
- normal implementation/review: 4k–10k, 250–400 words;
- hard specialist: 8k–20k only when coupling justifies it;
- branch report upward: 250 words normally, up to 600 at a checkpoint, at most five findings.

Budgets are policy thresholds, not promises about exact tokenizer counts.

## Status

Do not send periodic “still working” chatter. Send a status only when it changes the parent's decision: blocker, drift, budget pressure, changed ETA/critical path, safety issue, or checkpoint reached.

## No-findings

Independent reviews must state `no_findings: true` and identify the exact target/checks when no material issue is found. This prevents accidental re-review caused by an empty or ambiguous response.
