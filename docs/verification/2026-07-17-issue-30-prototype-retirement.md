# Issue #30 — Experimental integration retirement

Date: 2026-07-17

## Result

The disposable issue #16 evaluator and issue #20 direct-host, MessagePort, Electron, and reducer
implementations were removed. Their dated verification records remain unchanged except for explicit
retirement notices. No compatibility backend or legacy state reader was added.

The pinned Codex source overlay and T3Code patch remain. They are the maintained source inputs copied
or applied to clean exact-revision trees by the Product build; neither is a process, protocol client,
store, scheduler, or runtime fallback. Removing them before the long-lived fork repositories become
the source checkout would delete the production implementation rather than retire an experiment.

## Deleted-claim parity

| Deleted prototype claim | Maintained Product coverage | Disposition |
| --- | --- | --- |
| Exact bundle negotiation and terminal mismatch | Product manifest identity/substitution tests; retained Codex driver compatibility tests; real Product handshake | migrated |
| `thread/read`, task snapshot, stable lifecycle revisions, restart replay, and bounded replay tail | Codex `StateRuntime` tests `duplicate_revision_is_idempotent_and_restart_replays_latest` and `replay_tail_is_bounded_and_reports_pruning`; retained provider hydration/recovery tests | migrated |
| Parent lifecycle references a child without copying child history | Native Codex child identity plus retained T3Code `nativeSubagents` and `NativeSubagentsPanel` tests | migrated |
| Lazy child detail loading | Task-scoped native subagent and Workflow Run Tree query tests in the retained renderer | migrated |
| Replaceable bounded Run Digest in World State | Native `world_state` snapshot-noop/replacement tests and bounded digest tests in `orchestra-core::query` | migrated |
| Renderer/native-tool query parity and authorization denial | Shared `ExecutionQueryService` authorization, selector, pagination, history, evidence, and hard-budget tests; both native adapters use that service | migrated |
| Renderer reload and provider crash recovery | Retained T3Code orchestration recovery tests and the real issue #28 Product crash/restart observation | migrated |
| Expired cursor, duplicate delivery, and slow-consumer recovery | Native bounded replay/idempotency tests plus retained renderer replay coordinator retry/gap/fallback tests | migrated |
| Redacted diagnostics and malformed/oversized lifecycle rejection | Retained provider diagnostic-redaction and shared Effect Schema contract tests | migrated |
| Sandboxed Electron renderer and one normal application shell | Retained desktop packaging/isolation tests and real Product startup/dogfood evidence | migrated |
| Length-prefixed MessagePort host, one renderer port, and private fd 3 | The accepted Product uses the retained local UI server and one `codex app-server`, not this topology | obsolete; no alternate transport retained |
| Privileged confirmation traverses fd 3 | No Orchestra privileged decision RPC is exposed to the renderer; a direction-authenticated channel is deferred until concrete native confirmation UX exists | obsolete until product requirement |
| Five fresh exact-Zod transform/refine attempts are identical | Product pinned-worker test `exact_zod_transform_refine_is_deterministic_across_fresh_workers` | migrated |
| Rejections are normalized, stable, sorted, and bounded | Product pinned-worker test `ordinary_rejection_is_stable_sorted_and_bounded` | migrated |
| Evaluator provenance mismatch fails closed | Product pinned-worker test `evaluator_revision_is_intrinsic_worker_provenance` plus Rust bundle-hash checks | migrated |
| Oversize, timeout, crash, async, and noncanonical transforms are distinct infrastructure failures | Product pinned-worker tests `oversized_timeout_and_crash_have_distinct_failures` and `async_and_noncanonical_transforms_are_infrastructure_failures` | migrated |
| Evaluator stdout is protocol-only and diagnostics use stderr | Production one-request worker/adapter contract and Product build smoke | migrated |

## Removed paths

- `crates/orchestra-host-prototype/`
- `prototypes/desktop-host/`
- `prototypes/hermetic-evaluator/`
- `scripts/desktop-host-prototype.sh`
- `scripts/electron-host-prototype.sh`
- `scripts/hermetic-evaluator-prototype.sh`

The Rust workspace no longer resolves or runs the five disposable host-fixture tests. Current
developer and interactive-verification documentation points to the Product worker, exact Codex
integration, retained T3Code integration, and release preflight instead.

## State and evidence preservation

Retirement deletes no `$CODEX_HOME` content, Codex rollout, `StateRuntime` database, target checkout,
or repository `.codex/orchestra/runs/` content. Existing Run checkpoints retain their recorded Product
tuple and evaluator provenance. The issue #16 and #20 dated records preserve the unsupported legacy
results without keeping runnable parallel implementations.

## Verification

Run on 2026-07-17:

- `cargo fmt --all --check` passed.
- `cargo test --workspace` passed 103 active tests; the five pinned-worker tests remained
  intentionally ignored by the ordinary workspace runner.
- `scripts/evaluator-test.sh` passed all five pinned-worker tests.
- `cargo run -p codex-orchestra-lifecycle -- doctor` passed the manifest, configuration, native
  capability, and four-skill checks.
- `scripts/codex-integration.sh /tmp/orchestra-issue30-codex-20260717 verify` freshly cloned the
  exact `f90e7deea6a715bbd153044af6f475eefa749177` revision and passed 78 Orchestra core tests,
  16 extension tests, eight native Codex `orchestra::tests`, 267 generated protocol tests, and the
  real `codex-app-server` check.
- A clean `ecb35f75` T3Code worktree accepted and verified the maintained patch, the 28 recovered
  source-overlay files, and freshly generated protocol types. This also migrated the stale desktop
  product-name assertion into the maintained patch instead of leaving it in a temporary checkout.
- Standalone retained desktop and web suites passed 335 and 1,322 tests. The repository-wide T3Code
  run also reported all 187 server tests passing before its unrelated `infra/relay` process failed
  to exit under local Node 26; the pinned T3Code engine is Node 24.
- Unlocked Product observation confirmed one normal retained project/task/chat shell, inline
  task-scoped Symphony, native-subagent and attention summaries, a clean Electron quit/relaunch, and
  persisted project registration. No replacement dashboard, detached Run, or prototype transport
  appeared. The unsent draft correctly did not become canonical task history.
- `git diff --check` passed.

## Release dependency resolved for the MVP

Issue #29 now provides Electron updater-driven predecessor retention, first-launch commit, and bounded
automatic rollback. Public signed/notarized publication is intentionally separated into issue #56;
it is not a reason to keep any disposable agent, protocol, state, evaluator, or dashboard
implementation. The maintained exact-revision patch and source overlay remain the source integration
seam until the two long-lived forks become direct repository checkouts.
