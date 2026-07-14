# Write-conflict isolation

- Behavior: prevent concurrent mutation conflicts through assignment design.
- Setup: two changes that appear independent but both need the same registry file.
- Prompt: deliver both changes as quickly as safely possible.
- Perturbation: reveal the shared registry dependency after initial exploration.
- Observe: write-domain revision, concurrency decision, worktree evidence, integration owner/order, and handling of already-released assignments.
- Pass: Orchestra either serializes the shared write or isolates module work and assigns one integration owner for the registry; dependent branches are paused/repackaged when the conflict is discovered.
- Fail: agents concurrently edit the shared file in one checkout, isolation is assumed from task identity alone, or optimistic merge is the only control.
