# Issue #16 evaluator MVP evidence

## Verdict

Keep the pinned one-request validation-worker design for the coding-harness MVP. Rust parses and lowers
workflow source; the worker evaluates only the recorded schema bundle. This prototype does not claim
that stock Bun is a hostile-code security sandbox.

## Reproduce

> Historical note (2026-07-17): issue #30 removed this disposable harness after the Product worker
> suite superseded every assertion. The command below records the original run and is not present in
> the maintained tree; use `scripts/evaluator-test.sh` for current verification.

```sh
scripts/hermetic-evaluator-prototype.sh
```

Observed on arm64 macOS with Bun 1.3.14 and exact Zod 4.4.3:

- five fresh workers produced one SHA-256 output identity for the same accepted transform;
- synchronous refine/transform returned canonical raw and transformed JSON plus bundle/evaluator
  provenance;
- ordinary rejection returned stable, sorted, bounded issues;
- a false bundle hash failed before evaluation with exit 70;
- a request over 2 MiB failed with exit 70;
- a busy worker was killed by the host wall timer after approximately 250 ms; and
- an aborted worker produced no partial protocol response and was classified by signal.

## MVP boundary

Repository workflow definitions are trusted like repository build and test code. The worker is bounded
for determinism and operational recovery, not isolated against malicious JavaScript. App-Sandboxed XPC,
hard memory enforcement, adversarial escape testing, notarized signing, both production architectures,
and the final forked evaluator revision belong to production hardening and release verification.

Issue #26 integrated the same request, provenance, canonicalization, and failure semantics into the
pinned Product fork. Issue #30 removed the harness rather than shipping it as a second runtime.
