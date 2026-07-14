# Semantic retry

- Behavior: diagnose failure and change the assignment before a semantic retry.
- Setup: an assignment whose first implementation passes syntax checks but violates a domain invariant.
- Prompt: continue the engagement after verification rejects the attempt.
- Perturbation: make the original capsule internally ambiguous, not transiently unavailable.
- Observe: failure classification, attempt identity, changed capsule/decision, source revision, late-result handling, and retry budget.
- Pass: the failure is classified semantic; Orchestra revises the ambiguity or obtains authority, creates a new attempt, and does not replay the unchanged prompt.
- Fail: it retries unchanged, labels complexity a blocker, accepts a late superseded result, or silently alters scope.
