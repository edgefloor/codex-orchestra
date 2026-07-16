# Issue #26 production compiler and evaluator evidence

## Result

The production core now compiles an exact closed workflow source graph into a canonical,
content-addressed artifact without evaluating TypeScript. The artifact retains authoritative source,
plan, validation bundles, guidance schemas, codec identities, effective limits, and the complete
compatibility tuple.

The Product evaluator is a compiled, one-request Bun worker pinned to Bun 1.3.14 and Zod 4.4.3. Rust
owns process creation, request and response bounds, timeout and crash classification, RFC 8785
canonicalization, provenance checks, and acceptance of the typed result. The worker evaluates only the
schema bundle and writes protocol bytes only to stdout.

The former issue #16 harness remains historical evidence but is no longer used by the Product build.

## Reproduce

```sh
cargo test -p codex-orchestra-core
scripts/evaluator-test.sh
cargo clippy -p codex-orchestra-core --all-targets -- -D warnings
```

The Product worker suite covers five fresh-process determinism attempts, synchronous Zod transform and
refine behavior, normalized rejection, intrinsic evaluator provenance, async rejection, noncanonical
transform rejection, input bounds, timeout termination, and signal crash classification.

## MVP boundary

The coding harness trusts repository workflow and schema source like repository build scripts. This is
a deterministic, bounded operational boundary, not a hostile-code sandbox. The current authoring
surface lowers the repository's restricted single-entry workflow DSL; broader inert Agents-compatible
module composition can extend the same artifact contract when dogfood workflows require it.
