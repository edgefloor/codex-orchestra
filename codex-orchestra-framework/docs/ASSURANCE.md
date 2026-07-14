# Assurance Profiles

## Why dimensions compose

A single “prototype / production / enterprise” field conflates maturity, harm, data, reversibility, availability, and external obligations. The Assurance Profile records these separately, and policy unions the controls triggered by each dimension.

This allows fast prototyping without pretending that a prototype touching restricted data is low risk. It also avoids demanding fuzzing, compliance evidence, or disaster-recovery work for a low-blast-radius production change where those controls are irrelevant.

## Dimensions

### Maturity

- `spike` — time-boxed learning; output is evidence, not shippable behavior.
- `prototype` — demonstrable behavior; limited users; shortcuts/caveats explicit.
- `beta` — realistic use; affected tests and basic operations expected.
- `production` — customer/organizational dependency; rollback and operations required.
- `regulated` — traceability and external control obligations in addition to production quality.

### Data sensitivity

- `public` — approved for public disclosure.
- `internal` — ordinary non-public repository/business data.
- `confidential` — meaningful harm from disclosure.
- `restricted` — credentials, regulated/personal/highly sensitive data or equivalent.

### Blast radius

- `local` — one developer/tool/environment.
- `team` — one internal team/workflow.
- `customer` — externally visible subset of users/customers.
- `platform` — broad shared infrastructure or many dependent systems.

### Reversibility

- `easy` — rollback/revert is routine and data-safe.
- `migration` — rollback or forward-fix requires coordinated data/schema transition.
- `irreversible` — action cannot be meaningfully undone or may destroy/externally commit state.

### Availability

- `none` — no uptime dependency.
- `standard` — ordinary service expectations.
- `critical` — material operational/business harm from outage or degradation.

### Compliance

- `none` — no named external/internal control framework.
- `policy` — binding organizational policy or contractual controls.
- `regulatory` — legal/regulatory/audit traceability requirements.

## Deriving gates

Start with baseline acceptance trace, changed-surface tests, Team Leader integration review, and residual-risk statement. Then union every matching conditional rule in `.orchestra/policies/assurance-policy.yaml`.

The Quality Governor may mark a rule `not_applicable` only when its trigger does not actually intersect the changed surface, with rationale. Removing a triggered mandatory gate requires a waiver with authority, expiry, residual risk, and compensating control.

## Examples

### Internal prototype using restricted customer-like data

Profile: prototype / restricted / team / easy / none / policy.

Likely gates: smoke/demo plus data flow, secrets, access boundary, logging redaction, threat model, independent security review, control mapping. Full platform load testing is not implied.

### Public documentation production site

Profile: production / public / customer / easy / standard / none.

Likely gates: full relevant suite, independent review, rollback, operational docs/observability, staged rollout, compatibility review. Threat modeling for restricted data and migration dry-runs are not implied unless changed surface adds those risks.

### Irreversible schema/data transition in a beta service

Profile: beta / confidential / customer / irreversible / standard / policy.

Likely gates include affected suite/peer review, data/access checks, staged rollout, compatibility, Operator approval, independent review, and recovery proof or explicit risk acceptance.

## Prototype shortcuts

Shortcuts belong in `caveats` and accepted risk, with invalidation conditions. Examples:

- single-node state; not horizontally safe;
- manual secret provisioning; not self-service;
- happy-path API only; malformed input behavior undefined;
- no backward compatibility; internal caller only;
- manual rollback; deployment limited to one environment.

A shortcut is acceptable when visible, bounded, and consistent with risk. Hidden shortcuts are defects in grounding or assurance.
