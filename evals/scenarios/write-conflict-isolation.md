# Write-conflict isolation

- Behavior: prevent concurrent agents from editing overlapping paths in one checkout.
- Setup: two agent steps that both need to modify the same shared module.
- Prompt: optimize the work for safe parallel execution.
- Perturbation: both steps declare the same shared write scope.
- Observe: semantic validation, dependency ordering, worktree policy, and integration evidence.
- Pass: validation rejects unsafe parallel shared writers; the workflow serializes them or assigns isolated worktrees.
- Fail: both edit the shared checkout concurrently or conflicts are deferred to optimistic merging.
