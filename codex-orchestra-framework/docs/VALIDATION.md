# Validation

## Static doctor checks

`python tools/orchestra.py doctor` validates:

- required repository contracts/schemas/templates exist;
- V2 is enabled with visible routing/model overrides;
- `agents.max_depth = 3` and both thread ceilings are 10;
- all stable agent TOMLs contain name/description/instructions, least-privilege sandbox, and nickname candidates;
- no agent archetype defaults to Ultra;
- Manager remains read-only and high reasoning;
- builders use workspace-write; reviewers/advisors/governors remain read-only;
- expected custom-agent archetypes and canonical `.agents/skills` procedures exist;
- each skill has matching `name`/directory frontmatter and explicit invocation metadata;
- root instructions remain compact.

## Reference utility tests

Run:

```bash
python -m unittest discover -s tests -v
python tools/orchestra.py doctor
```

Coverage includes:

- V2 config invariants;
- JSON Schema syntax, direct template conformance, and full-history fork semantics;
- Manager execution prohibition;
- physical parent/role allowlist and depth limit;
- reporting-cycle rejection;
- exclusive leases and write-domain overlap;
- stale-result rejection after digest change;
- lease expiry/reconciliation;
- Team Leader task review;
- legal phase transitions and idempotent events.

## Required live V2 probes

The reference utility cannot prove Codex runtime behavior. Before relying on a new Codex release, run the capability probe in `CODEX-CONFIGURATION.md`, including a nested permitted worker and confirmation that its completion reaches the Team Leader parent rather than root.

## Behavioral evals

See `docs/EVALS.md`. Production promotion requires the Manager boundary, delegated squad, no-leaf-spawn, context compression, review no-findings, stale result, write isolation, drift, assurance, and recovery scenarios to pass.
