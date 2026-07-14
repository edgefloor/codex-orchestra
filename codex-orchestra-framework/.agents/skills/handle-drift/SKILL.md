---
name: handle-drift
description: Classify suspected scope/assumption drift and choose local repair, re-plan, or formal re-grounding.
---

1. State the observed evidence and the exact binding source it conflicts with. Distinguish a defect from a changed requirement or disproved assumption.
2. Determine blast radius: one task, one workstream, milestone, or project; identify queued/running capsules affected.
3. Choose the earliest correct repair level:
   - local implementation repair for a bounded defect;
   - Team Leader tactical re-plan for sequencing/technical mismatch;
   - Delivery Architect plan revision for dependency/system-boundary error;
   - Consultant/Operator re-grounding for product behavior, scope, risk, or external-constraint change.
4. Freeze only affected work. Recommend the reversible safe default for unaffected work.
5. Emit a drift envelope with evidence, impact, proposed owner, invalidated refs, and decision deadline. After an accepted revision, recompute the digest and repackage stale capsules.

**Complete when:** the conflict has one accountable owner, the affected set is explicit, and work resumes from the earliest affected gate without restarting unrelated work.
