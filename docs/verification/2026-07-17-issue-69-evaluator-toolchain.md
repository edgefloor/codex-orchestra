# Issue #69 — Sealed evaluator toolchain provenance

Date: 2026-07-17

## Pinned dependency identity

Bun and Zod are pinned evaluator dependencies, not standalone Product forks. Their immutable source
and package identities are sealed as follows.

| Dependency | Version | Source identity | Package identity |
|---|---:|---|---|
| Bun | `1.3.14` | tag revision `0d9b296af33f2b851fcbf4df3e9ec89751734ba4` | packaged evaluator runtime |
| Zod | `4.4.3` | tag revision `1fb56a5c18c27102dbc92260a4007c7732a0ccca` | npm `gitHead` `f3c9ec03ba7a28ae72d25cc295f38674bee0f559` |

The Zod npm package is additionally sealed by:

- integrity: `sha512-ytENFjIJFl2UwYglde2jchW2Hwm4GJFLDiSXWdTrJQBIN9Fcyp7n4DhxJEiWNAJMV1/BqWfW/kkg71UDcHJyTQ==`
- shasum: `b680f172885d18bbebf21a834ea25e55a1bbf356`

The evaluator inputs are sealed by these SHA-256 digests:

| Input | Digest |
|---|---|
| `evaluator/worker.ts` | `169c4a10c0631a33f94c4bda6307f1d552bd7cc4db1da3b907b63df4847f4287` |
| `evaluator/bun.lock` | `c699079c216b7479aaf59d7856d9e7762a5b70ada858702a6034afd59bd33d1c` |
| `evaluator/package.json` | `471a1072db06bcb66aed6e8a6d215506a7fdc2169fb47295fc6154c2c83852c2` |

## Verification evidence

- The remote provenance verifier passed for the pinned Bun tag revision, Zod tag revision, Zod npm
  `gitHead`, integrity, and shasum.
- All 6 Rust evaluator tests passed.
- Fresh-process evaluator smoke tests covered canonical output, provenance, forbidden capabilities,
  malformed input, validation limits, and bounded diagnostics.
- The arm64 Mach-O evaluator passed the smoke suite with digest
  `eca0de99fcab3e38e086de2ad28abd0d37ddb8fa294f9534a3da8126d680c1b5`.
- The x86_64 Mach-O evaluator passed the smoke suite with digest
  `48deaca80dac51b190cdfe69d005aacd94a6888c519082772e841f539b9e9af8`.
- Product manifests record all 11 validation limits and the packaged binary digest.

## Execution boundary

The native Rust compiler remains the sole owner of parsing and lowering workflow TypeScript. Bun is
not permitted to execute workflow source: its only packaged entrypoint is `evaluator/worker.ts`,
which performs the sealed evaluator function behind the validation boundary. This preserves the
authoring-only restricted TypeScript SDK while making evaluator behavior reproducible across the two
supported macOS architectures.
