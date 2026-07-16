# Issue 33 — task-owned Automation fixture

## Result

One eligible fixture issue now runs through the accepted native hierarchy:

`Automation task → resident Automation Root Run → Issue claim/worktree/Issue task → typed Workflow Run`

The Root Run and repository/project lease are persisted beside existing Orchestra Run state under
`.codex/orchestra/`; this is an extension of repository-owned execution state, not a Host store or
background scheduler. The profile snapshot, source revision, claim, native task identity, Workflow
Run identity, and cancellation target are durable before child work begins.

The pinned Codex App Server exposes generated `automation/runFixture` and `automation/cancel`
requests. The retained T3Code chat header validates first, runs the fixture, renders bounded root and
claim projections, exposes stable Issue-task and Workflow IDs, and can cancel the resident
Automation. Repository paths and authority remain derived from the owning Codex task.

## Automated evidence

- Root workspace: 58 `codex-orchestra-core` tests passed, including lease conflict, duplicate issue,
  stable claim/worktree identity, and compilation of the shipped Automation issue fixture. The full
  workspace suite passed before the fixture-only compiler assertion was added; lifecycle doctor
  passed with four skills and native capability checks.
- Pinned Codex: 58 Orchestra core tests, 8 Orchestra extension tests, 8 focused Codex native-control
  tests, and 2 Automation protocol tests passed. App Server compilation passed with generated
  request/response schemas.
- Pinned T3Code: contracts, web, and server typechecks passed. The full web unit project passed 149
  files and 1,283 tests, including the profile-to-desktop fixture projection and task-scoped request
  assertions. The final patch applies cleanly to pinned T3Code `ecb35f75839925dd1ac6f854efeef5c9e291d11b`.
- Both maintained fork patches pass `git apply --check`; the root worktree passes `git diff --check`.

## Boundaries

- This issue dispatches one supplied fixture issue. Tracker polling and real Linear reads begin in
  later queue/reconciliation issues.
- No tracker mutation is authorized here. The first `tracker.comment` receipt is issue #34.
- The shipped example profile is `examples/automation/WORKFLOW.md`; fixture validation works without
  `LINEAR_API_KEY`, while live Linear reads remain opt-in and require it. The profile selects
  `crates/orchestra-core/fixtures/automation-issue.workflow.ts`.
- A live model-backed desktop run remains a human/provider observation, not something the test suite
  fabricates. The deterministic product fixture proves the protocol and bounded desktop projection.
