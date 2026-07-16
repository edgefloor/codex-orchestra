# Issue 34 — policy-gated tracker comment

## Result

The task-owned Automation fixture can publish one `tracker.comment` effect through a native
pre-mutation gate. The effective profile, current claim, and tracker project supply authority;
workflow output can supply only the bounded comment body. A prepared effect receipt is persisted
before execution, and a claim is successful only after a durable provider receipt is stored.

The MVP provider is an explicit fixture adapter at
`.codex/orchestra/fixture-tracker/comments.jsonl`. Live Linear access remains issue #35.

## Replay and failure behavior

- The idempotency key binds profile digest, claim ID, effect kind, and request digest.
- A committed receipt replays without another provider call, including after claim completion.
- New effects on inactive or foreign claims fail before mutation.
- Reject and ask-human gates never call the provider.
- Interrupted `executing` receipts and uncertain provider results become `ambiguous`; they are not
  retried automatically.
- The normal T3Code task dialog shows at most four bounded receipt summaries per claim. It does not
  expose raw workflow output or provider authority.

## Automated evidence

- Root workspace: formatting passed; 81 tests passed (60 core, 5 host prototype, 14 lifecycle,
  2 product, plus package targets); lifecycle doctor passed with four skills and native capability
  checks.
- Fresh pinned Codex checkout at `f90e7deea6a715bbd153044af6f475eefa749177`:
  60 Orchestra core tests, 10 extension tests, 8 focused native-control tests, all 262 protocol
  tests, and `codex-app-server` compilation passed.
- Fresh pinned T3Code checkout at `ecb35f75839925dd1ac6f854efeef5c9e291d11b`:
  contracts, web, and server typechecks passed; all 1,283 web tests passed.
- Both integration patches applied cleanly to fresh detached checkouts, and generated Codex types
  include the effect receipt, gate policy, and ambiguity status.

## Pending human evidence

Provider-backed execution through the packaged desktop remains unobserved. Do not treat the
fixture JSONL adapter as proof of live Linear behavior.
