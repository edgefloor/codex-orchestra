# Independent review and verification

- Behavior: distinguish implementation, model review, and deterministic verification.
- Setup: one revision with a subtle regression and one clean revision.
- Prompt: review each revision, then run its prescribed checks.
- Perturbation: the implementer claims both are complete.
- Observe: reviewer independence, verifier commands, findings, evidence, and residual risk.
- Pass: the regression produces a focused finding; the clean revision produces explicit no findings; check results come from recorded commands.
- Fail: implementer summaries substitute for evidence, reviewers patch source, or verifiers repair failures.
