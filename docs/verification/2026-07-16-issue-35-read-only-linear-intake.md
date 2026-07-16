# Issue 35 — read-only Linear intake

## Result

Automation profiles can read active candidates, terminal issues, or one refreshed issue through
the native task-owned Codex path. Rust normalizes pagination, labels, blockers, priorities, states,
and terminal filtering before returning a typed response. The normal T3Code task dialog renders a
maximum of 25 normalized summaries; raw GraphQL, provider URLs, descriptions, and credentials do
not cross that projection.

The endpoint is pinned to `https://api.linear.app/graphql`. Live requests use Codex's native HTTP
client and its custom-CA handling. Profiles retain only credential references. Missing environment
credentials return `skipped`, while fixture profile validation remains valid with a warning.
Inline credential values are reduced to a digest during validation and cannot enable live reads.

## Automated evidence

- Root normalizer tests cover two-page cursor behavior, deterministic ordering, labels, blockers,
  priorities, states, issue refresh, and typed GraphQL failures.
- Fresh pinned Codex checkout at `f90e7deea6a715bbd153044af6f475eefa749177`:
  63 Orchestra core tests, 11 extension tests, 8 focused native-control tests, all 263 protocol
  tests and schema fixtures, and `codex-app-server` compilation passed.
- Fresh pinned T3Code checkout at `ecb35f75839925dd1ac6f854efeef5c9e291d11b`:
  contracts, server, and web typechecks passed; all 149 web test files and 1,284 tests passed.
- Both integration patches applied cleanly to fresh detached checkouts. The generated protocol
  exposes only the read kind, task/profile identity, bounded pagination, optional issue identifier,
  normalized issues, and a bounded next action.

## Pending live evidence

`LINEAR_API_KEY` was not configured, so the opt-in live check was skipped, not passed. The exact
project lookup and relation field shape therefore remain unobserved against a real Linear
workspace. Fixture and contract evidence do not claim production-provider parity until that check
is run with an authorized project and credential reference.
