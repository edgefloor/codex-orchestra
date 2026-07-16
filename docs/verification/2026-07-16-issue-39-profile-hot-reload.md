# Issue #39 Automation profile hot reload

Status: implemented at the runtime, pinned Codex protocol, and normal T3Code Automation panel seams.

## Implemented contract

- The Root Run retains immutable digest-addressed profile snapshots. Each claim records the exact
  profile digest and revision used for its dispatch; retry and continuation load that revision.
- A valid changed profile becomes `pending_valid` and affects only a later claim dispatch. Existing
  and recovering claims retain their profile, prompt, workflow digest, retry policy, and tracker
  semantics.
- An invalid revision becomes `rejected` with bounded diagnostics while the last-known-good active
  profile remains authoritative. Reverting the source to the active profile clears rejection state.
- A missing environment credential skips only its dependent Linear read. The operation can be
  retried after re-resolving the named reference; resolved credential bytes are never persisted or
  projected.
- Active, pending-valid, and rejected states, revision digests, bounded diagnostics, and per-claim
  revisions cross the generated App Server protocol and appear in the existing task Automation
  panel. Retained task prompts do not cross that projection.
- Profile reload changes only runtime metadata under the task-owned Run. It does not mutate an
  agent workspace, so it remains separate from the transactional writer ChangeSet path in ADR-0019.

## Automated evidence

- Root workspace formatting, 94 executed tests, and lifecycle doctor passed; five evaluator worker
  tests remained intentionally ignored.
- Root profile/claim fixtures: 13 passed, including pending activation, immutable old-claim pinning,
  versioned reload, and last-known-good rejection behavior.
- Clean pinned Codex patch application passed; 73 Orchestra core and 11 extension tests passed.
- Generated TypeScript and JSON schema fixtures passed, and the pinned App Server library compiled.
- Clean pinned T3Code patch application passed; the web typecheck and four Automation dialog logic
  tests passed.

## Human-only checks

- No live Linear credential was used, so credential disappearance and later re-resolution were not
  exercised against the provider.
- The revision badges and diagnostics were typechecked but not visually inspected in Electron.
