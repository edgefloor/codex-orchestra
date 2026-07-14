---
name: verify-milestone
description: Quality Governor and verifier procedure for risk-derived gates and an independent evidence branch.
---

1. Read the accepted Assurance Profile, changed surface, milestone behavior, and policy triggers.
2. Compile a gate matrix: requirement, trigger, required evidence, independence, owner, status, and waiver authority. Exclude unrelated hardening.
3. Reuse evidence only when target revision, environment, scope, and independence remain valid.
4. Under an assurance Delegation Permit, spawn bounded independent Verifier/Advisor children for missing evidence. Give each an exact target, commands, environment, evidence schema, and no-source-edit rule. Wait and join locally.
5. Reproduce/validate important claims. Classify findings by gate and impact. Route repair needs to the relevant Team Leader; never patch them inside verification.
6. Issue pass, conditional_pass, or fail. Conditional pass requires a named time-bounded waiver and residual-risk owner. Send one bounded gate report upward.

**Complete when:** every derived gate has reproducible evidence or authorized waiver and the verdict does not depend on verifier transcripts.
