---
name: resume-workflow
description: Resume or recover a native Orchestra run from its runtime-owned checkpoint.
---

# Resume a workflow

1. Read the run with `orchestra_status`; the transcript is not authoritative.
2. If the run is waiting for approval, pass `approval_decision` to `orchestra_resume` only when the user explicitly chose it. The first declared choice accepts and permits promotion; every other choice rejects and cancels without promotion.
3. Otherwise call `orchestra_resume` without a decision. The runtime reconciles interrupted attempts against budgets and the immutable workflow snapshot.
4. Report the returned summary, promotion status, or next approval exactly. Use `orchestra_query`
   for bounded failed evidence or history rather than copying the full Run state.
5. Use `orchestra_cancel` when the user asks to stop the run.

Never repair checkpoint JSON or synthesize step outputs manually.
