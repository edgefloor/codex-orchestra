# Target repository structure

## Decision

Promote the contents of the current `codex-orchestra/` candidate to the repository root. Do not keep `codex-orchestra-framework/`, R2 design-package files, or the scaffolding wrapper beside the product. Their durable decisions are captured in `docs/adr/`; their remaining value is limited to behavioral evaluation cases that should be migrated before deletion.

## Final shape

```text
codex-orchestra/
├── .codex-plugin/
│   └── plugin.json
├── .gitignore
├── AGENTS.md
├── CONTEXT.md
├── LICENSE
├── README.md
├── pyproject.toml
├── uv.lock
├── skills/
│   ├── orchestrate/
│   ├── ground-project/
│   ├── create-charter/
│   ├── plan-delivery/
│   ├── form-taskforce/
│   ├── lead-workstream/
│   ├── package-context/
│   ├── execute-assignment/
│   ├── review-assignment/
│   ├── verify-milestone/
│   ├── recover-run/
│   ├── handoff-run/
│   └── manage-installation/
├── config/
│   ├── project.toml
│   ├── orchestra.config.toml
│   └── agents/
│       ├── consultant.toml
│       ├── team_leader.toml
│       ├── worker.toml
│       ├── reviewer.toml
│       └── verifier.toml
├── assets/
│   ├── policies/
│   ├── schemas/
│   └── templates/
├── scripts/
│   └── lifecycle.py
├── tests/
│   ├── fixtures/
│   └── test_scaffold.py
├── evals/
│   └── scenarios/
│       ├── bounded-delegation.md
│       ├── context-isolation.md
│       ├── independent-review.md
│       ├── write-conflict.md
│       ├── risk-derived-assurance.md
│       └── recovery.md
└── docs/
    ├── adr/
    ├── CONFIGURATION.md
    ├── LIFECYCLE.md
    ├── SELF-HOSTING.md
    └── VALIDATION.md
```

The `hooks/` directory should not exist until a real hook is supported and enabled. A placeholder directory has no interface or implementation and adds no depth.

## Consolidations before promotion

1. Replace duplicated `config/project/agents/` and `config/global/agents/` trees with one canonical `config/agents/` source. The lifecycle module maps that source to project or global destinations.
2. Rename `config/project/config.toml` to `config/project.toml` and `config/global/orchestra.config.toml` to `config/orchestra.config.toml`.
3. Add development-only `AGENTS.md` at the repository root. It governs work on Orchestra itself and is not installed into target repositories.
4. Add `evals/scenarios/` and migrate the useful behavioral cases from the legacy evaluation document before deletion.
5. Delete `docs/MIGRATION-INVENTORY.md` after the deletion ledger below is complete; ADRs become the permanent rationale.
6. Remove committed scaffolding-run artifacts from the outer `.codex/orchestra/` tree after their accepted decisions and verification evidence have been captured. Future Engagement artifacts may be kept locally or committed deliberately, but are not package source.

## Deletion ledger

Delete after the consolidations and behavioral-eval migration pass:

- `codex-orchestra-framework/` in full;
- `codex-orchestra-r2-assessment-and-design.md`;
- `codex-orchestra-r2-workflow.schema.json`;
- `codex-orchestra-r2-example-workflow.yaml`;
- legacy package `README.txt` and `SHA256SUMS.txt`;
- top-level `SCAFFOLDING-PLAN.md` after this structure is accepted;
- top-level `.codex/orchestra/` scaffolding evidence after material evidence is summarized in the retained validation history;
- candidate `docs/MIGRATION-INVENTORY.md` after migration completion;
- candidate placeholder `hooks/README.md` and the empty `hooks/` directory.

Do not carry forward the legacy SQLite control-plane utility, external App Server design, Workflow IR, fixed fourteen-role hierarchy, hard-coded model routes, experimental configuration, leases, or old schema/template catalog as compatibility baggage.

## ADR coverage

| Durable concern | Permanent home |
|---|---|
| Native-only product and durable truth | `docs/adr/0001-codex-native-sources-of-truth.md` |
| Plugin plus optional configuration installation | `docs/adr/0002-plugin-and-configuration-distribution.md` |
| Role authority without fixed topology | `docs/adr/0003-role-contracts-not-fixed-topology.md` |
| Lifecycle and self-hosting | `docs/adr/0004-checkpoint-lifecycle-and-self-hosting.md` |
| Delegation, joins, and context | `docs/adr/0005-bounded-delegation-and-context.md` |
| Read parallelism, writer isolation, integration | `docs/adr/0006-parallel-reading-isolated-writing.md` |
| Risk-derived assurance | `docs/adr/0007-risk-derived-assurance.md` |
| Drift, recovery, retry, and learning | `docs/adr/0008-drift-recovery-and-framework-learning.md` |

## Verified scaffold status

On Codex CLI 0.141.0:

- all 12 deterministic scaffold tests pass;
- the canonical plugin validator passes;
- lifecycle `doctor` discovers 13 skills and passes manifest, strict-config, profile-selection, and stable multi-agent capability checks;
- no MCP server, app, daemon, external scheduler, or plugin-local mutable state is present.

Remaining interactive Gates are a fresh-task plugin invocation, manual custom-agent selection in a supported Codex interface, and one live bounded self-hosting Engagement.
