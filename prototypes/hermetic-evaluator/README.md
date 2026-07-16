# Exact-Zod worker prototype

This disposable issue #16 harness proves the MVP coding-harness boundary, not hostile-code isolation.
Rust remains responsible for parsing and lowering workflow source. The one-request worker exercises a
pinned exact Zod bundle, canonical success and rejection results, provenance mismatch, request bounds,
timeout termination, and crash classification.

Run from the repository root:

```sh
scripts/hermetic-evaluator-prototype.sh
```

The script requires Bun 1.3.14 and installs exact Zod 4.4.3 from `bun.lock`. Generated dependencies and
standalone binaries stay under ignored prototype directories.
