# Issue #41 — transition and pull-request Tracker effects

Date: 2026-07-17

## Implemented contract

- `tracker.transition` and `tracker.link_pull_request` use the same claim-scoped effect receipt model as `tracker.comment`.
- Preparation validates profile authority, writes a durable `executing` receipt, and returns a typed provider request. Completion records a bounded committed, failed, or ambiguous result. A stale lease cannot commit a late provider result.
- Transition requests carry both the refreshed expected state and canonical target state. Live Linear execution refreshes the Issue, confirms that it still belongs to the configured project, refuses changed or newly terminal source states, validates the target against the Issue team's current workflow states, and treats an already-applied target as committed without another mutation.
- Pull-request URLs accept only canonical HTTPS GitHub pull-request paths. Query strings, fragments, and a trailing slash are removed before the idempotency identity is calculated. The target Issue and project come from the retained claim rather than Workflow output.
- Live provider calls are exposed through task-authorized `OrchestraService` methods. They load the retained claim and its pinned profile revision; no renderer mutation method, raw GraphQL surface, scheduler, daemon, or alternate state store was added.
- Fixture execution writes synchronized JSONL mutation records and provider receipts. A committed terminal transition updates the retained claim state and prevents another continuation from being scheduled.
- The existing generic App Server receipt projection carries both effects without a new protocol type. The retained T3Code Automation panel logic test renders comment, transition, and pull-request receipts from that bounded projection.

## Automated evidence

- Root formatting and workspace suite: 97 tests passed; five evaluator-worker integration checks remained deliberately ignored for their pinned worker harness.
- Lifecycle doctor: Codex CLI `0.144.5`; manifest, configuration, four skills, and native capability checks passed.
- Pinned Codex `f90e7deea6a715bbd153044af6f475eefa749177`:
  - Orchestra core: 76 passed.
  - Orchestra extension: 16 passed, one live mutation test ignored by default.
  - Native Codex Orchestra core tests: 8 passed.
  - App Server protocol: 265 passed; generated TypeScript and JSON schema fixtures: 2 passed.
  - `cargo check -p codex-app-server`: passed.
- Retained T3Code `ecb35f75839925dd1ac6f854efeef5c9e291d11b`:
  - web typecheck: passed.
  - web unit suite: 1,285 passed across 149 files.
- Codex and T3Code integration patches applied cleanly with `git apply --check`; repository formatting and `git diff --check` passed.

## Explicit live mutation check

The live Linear test is compiled but ignored because it changes a user-selected Issue. Run it only against disposable test data with all of these values supplied:

```sh
ORCHESTRA_LINEAR_MUTATION_TEST=1 \
LINEAR_API_KEY=... \
ORCHESTRA_LINEAR_PROJECT_ID=... \
ORCHESTRA_LINEAR_ISSUE_ID=... \
ORCHESTRA_LINEAR_EXPECTED_STATE=... \
ORCHESTRA_LINEAR_TARGET_STATE=... \
ORCHESTRA_LINEAR_PULL_REQUEST_URL=https://github.com/OWNER/REPO/pull/NUMBER \
cargo test -p codex-orchestra-extension \
  tool::tests::live_linear_transition_and_pull_request_link_are_explicit_opt_in \
  -- --ignored --exact
```

The test was not run during this verification because no disposable Linear Issue or mutation authorization was provided.

## Unclaimed checks

- No human Electron rendering check was performed for this issue. The desktop evidence is automated projection and panel-logic coverage.
- Provider behavior beyond the explicit opt-in Linear check is not claimed from fixtures.

Verdict: automated acceptance evidence is complete; destructive live Linear and human desktop checks remain explicitly opt-in.
