# Workflow compilation and validation contract

This document is the accepted issue #11 MVP contract handed to the issue #16 evaluator prototype. The
prototype may ratify or reduce its candidate limits; if it falsifies determinism or bounded failure
behavior, issue #11 and ADR-0016 must be amended before implementation decomposition.

## Ownership and process boundary

The extended Codex host owns compilation and validation. Rust resolves, bounds, hash-verifies, parses,
and lowers the closed workflow graph without executing workflow source as JavaScript. It emits the
canonical execution plan, schema-only Validation bundle, guidance schemas, and artifact manifest.

`orchestra-validate-worker` is a pinned, one-request child process that receives only an exact
Validation bundle, schema ID, and canonical JSON value. It never imports or evaluates the workflow
graph.

The worker has no durable state and is not a daemon, scheduler, alternate host, or execution authority.
It ships inside the exact Product release described by ADR-0017.

## Closed authoring graph

Rust admits only exact, integrity-pinned instances of:

- the inert Agents-compatible authoring facade;
- the Product release's Zod build;
- workflow-local modules; and
- packages explicitly listed by identity and digest in the compile request.

The supported Agents surface is the issue #14 matrix: inert Agent declarations, declarative tools,
static handoffs, schemas, cloning, and pure composition helpers. Runner, providers, sessions, tracing,
realtime, model or tool execution, host callbacks, nested SDK execution, ambient imports, native
modules, dynamic import, top-level await, WebAssembly, `eval`, and `Function` are rejected.

Workflow-local helpers are accepted only when Rust can parse and lower their supported static semantics;
they are never invoked as JavaScript. Unsupported dynamic computation receives a source-located compile
diagnostic.

The MVP is a local coding harness and treats repository workflow definitions with the same trust as
repository build and test code. The host scrubs unnecessary environment, closes unrelated descriptors,
bounds IPC and time, and rejects unsupported validation source, but the stock evaluator is not claimed
as a hostile-code security sandbox. App-Sandboxed XPC hardening is deferred until the product accepts
untrusted workflow bundles or reaches production packaging.

## Exact validation

The supported Zod surface is explicitly versioned and synchronous. Ordinary schemas, synchronous
refinements, brands, and transforms are allowed inside the recorded validation source bundle. Async
parsing or effects, promises, host callbacks, unsupported VM objects, and transforms returning a
noncanonical value are rejected.

JSON Schema is recorded for model and tool guidance. Only the exact validation bundle decides runtime
acceptance. Validation records the raw canonical value and either the transformed canonical value or
normalized issues.

## Canonical values and custom codecs

Canonical values use RFC 8785 JSON Canonicalization Scheme over the I-JSON-compatible JSON subset.
Strings contain valid Unicode, numbers are finite and interoperable, arrays are dense, and objects
have ordinary string keys. Cycles, sparse arrays, `undefined`, symbols, functions, class instances,
`BigInt`, non-finite numbers, and VM identities are rejected. Rust canonicalizes before and after IPC.

Large integers, decimals, dates, and other custom values use explicit versioned JSON codecs rather
than JavaScript-number coercion. A codec declares a stable type ID, codec version, wire schema,
deterministic issues, canonical transformation, and compatibility. A type without canonical JSON is
compile-time-only.

## Authoritative artifact envelope

Compilation records a versioned canonical envelope containing:

- the graph manifest and source digests;
- the execution plan;
- the exact validation source bundle and schema identities;
- guidance JSON Schemas;
- compiler options and supported-surface revision; and
- the complete compatibility tuple.

The envelope and each material artifact receive SHA-256 identities over canonical bytes. Authoritative
source artifacts, not evaluator bytecode, are persisted. JSC bytecode and transpiler or machine caches
are disposable and must be provenance-checked on every hit.

The compatibility tuple contains the Agents commit, facade revision, Zod identity and integrity,
evaluator revision, Bun/JSC revision, adapter ABI, canonicalizer version, issue-format version, target
architecture, sandbox identity, effective limits, bundle hash, and Product release ID. Existing Runs
remain bound to their recorded tuple and are never silently recompiled after an update.

## Determinism and diagnostics

Identical authoritative inputs and compatibility tuple must yield byte-identical artifact envelopes,
transformed values, and normalized issues across fresh workers and cache states. Any divergence is an
evaluator infrastructure failure.

An issue has a versioned stable code, canonical value path, bounded message or template identity, and
optional source identity. Issues are deterministically sorted and capped. JavaScript exception stacks
are diagnostics sent to stderr and never become persisted validation issues.

Only ordinary schema rejection is eligible for a retryable Step failure. Timeout, crash, kill,
malformed output, provenance mismatch, nondeterminism, sandbox violation, or unsupported tuple is an
evaluator infrastructure failure. Partial responses are never accepted.

## Candidate MVP limits

All values are hard upper bounds recorded in the Product release manifest. Issue #16 must measure and
ratify or reduce them; it may not broaden them without amending this contract.

| Resource | Rust compilation | Validation worker |
| --- | ---: | ---: |
| Request bytes | 8 MiB | 2 MiB |
| Response bytes | 8 MiB | 2 MiB |
| Modules | 256 | validation bundle only |
| Total module source | 4 MiB | 2 MiB |
| Single module source | 1 MiB | 1 MiB |
| Parsed AST nodes | 500,000 | 250,000 |
| Canonical value bytes | 1 MiB | 1 MiB |
| Value depth | 64 | 64 |
| Total value nodes | 100,000 | 100,000 |
| Array entries or object keys | 10,000 | 10,000 |
| Single string bytes | 256 KiB | 256 KiB |
| Returned issues | 128 | 128 |
| Single issue text | 512 bytes | 512 bytes |
| Wall time | 5 seconds | 1 second |
| CPU time | 3 seconds | 500 milliseconds |

Rust enforces workflow parser, AST, graph, framing, byte, canonical-value, and response limits. The
validation worker enforces schema-bundle and structural limits. The host enforces wall time,
termination, descriptor closure, and one request per process. Production memory and OS-sandbox claims
are deliberately outside this MVP contract.

## Replay, upgrades, and acceptance

Validation is allowed only for a compatibility tuple explicitly supported by the current Product
release. Otherwise the Run remains inspectable but unsupported until an explicit recorded migration
or recompilation creates a new workflow artifact revision. Checkpoints and prior artifacts are never
rewritten.

Issue #16 closes with exact revisions, deterministic golden hashes across fresh processes, accepted
and rejected exact-Zod cases, byte-limit behavior, timeout termination, crash handling, provenance
checks, and a keep/change/reject verdict. OS sandbox and hostile-code research are not MVP exit gates.
