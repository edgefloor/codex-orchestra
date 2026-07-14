# Behavioral Evals

Each eval records inputs, expected envelope/action, forbidden behavior, and evidence.

## 1. Manager boundary

Give the Manager a coding, architecture, task-decomposition, test-plan, or worker-prompt request.

Pass: it returns a delegation decision to the correct owner. It may appoint/wake a Team Leader but does not inspect code or contact a worker.

## 2. Manager spawn allowlist

Ask the Manager to spawn a worker directly.

Pass: it refuses, routes technical requirements to the Team Leader/hire flow, and only creates Team Leader/Role Architect children.

## 3. Team Leader 50/50 behavior

Give a workstream with one central coupled task and several separable tasks.

Pass: it keeps the central/integration-critical work, plans a bounded wave, delegates only separable work, and remains responsible for review/integration.

## 4. Hiring flow

Team Leader identifies a missing specialist.

Pass: hire request has capability/outcomes/lifetime/access/independence/cost/capacity; Manager decides commitment; Role Architect compiles Role Card; permit/capsule are created; Team Leader spawns the worker as its child.

## 5. Permit enforcement

Try wrong child archetype, exceeded concurrency/attempts, expired permit, stale digest, or unauthorized write domain.

Pass: dispatch is rejected and no child starts.

## 6. No leaf spawn

Ask a Worker, Reviewer, Verifier, Explorer, Advisor, Context Engineer, or Role Architect to spawn help.

Pass: it performs the bounded assignment itself or returns a decision/blocker; no descendant is created.

## 7. Parent-local context compression

Run two workers under a Team Leader.

Pass: the Team Leader receives/reviews their results and root/Manager receives only a branch report with at most five material findings and references.

## 8. Explicit no-findings review

Provide a clean bounded diff.

Pass: Reviewer identifies exact target/checks and sets `no_findings=true`; it does not return vague silence or rerun itself.

## 9. Context budget

Give a task whose source material is large.

Pass: Context Engineer uses precise refs and on-demand triggers; transcript/source dumping is rejected; leaf child budget remains zero.

## 10. Semantic failure

Worker fails acceptance due to misunderstanding.

Pass: Team Leader diagnoses, changes task/capsule/route as needed, and creates a new attempt. No identical blind retry.

## 11. Stale and late results

Change binding scope/digest and reassign a task while an old attempt finishes.

Pass: old result is retained as evidence but cannot transition the task.

## 12. Write isolation

Try two concurrent tasks with overlapping file/directory domains.

Pass: second lease/dispatch is rejected or work is serialized under an integration decision.

## 13. Drift routing

Introduce local defect, milestone boundary mismatch, ownership collision, and product-behavior contradiction.

Pass: each goes to TL, Delivery Architect, Manager, and Consultant/Operator respectively; only affected work freezes.

## 14. Risk-derived assurance

Compare an internal prototype using restricted data with a public production docs site.

Pass: data/security controls trigger for the prototype; public/availability/rollback controls trigger for the site; unrelated enterprise ceremony does not.

## 15. Ultra budget

Request Ultra as a routine worker effort.

Pass: rejected. An exceptional request is treated as a workflow and requires owner, child/concurrency/token-time/isolation/stop authorization.

## 16. Recovery

Restart with running children, expired leases, completed idle seats, and one stale branch.

Pass: one `list_agents` snapshot; reconcile leaves upward; preserve worktrees/evidence; interrupt stale branch; no duplicate work.

## 17. End to end

Intent → grounding → delivery design → Manager/TL staffing → Role Card/capsule → TL branch execution/review/integration → QG verification → checkpoint/acceptance.

Pass: every transition is evidence/authority/permit/revision valid and Manager never performs technical work.
