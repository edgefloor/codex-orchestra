# Repository structure

## Decision

The repository root is the installable Codex Orchestra plugin. Product source is not wrapped in a second package directory, and retired design prototypes are preserved through Git history rather than shipped beside the plugin.

```text
codex-orchestra/
├── .codex-plugin/plugin.json
├── .gitignore
├── AGENTS.md
├── CONTEXT.md
├── LICENSE
├── README.md
├── pyproject.toml
├── uv.lock
├── skills/
├── config/
│   ├── project.toml
│   ├── orchestra.config.toml
│   └── agents/
├── assets/
│   ├── policies/
│   ├── schemas/
│   └── templates/
├── scripts/lifecycle.py
├── tests/
├── evals/scenarios/
└── docs/
    ├── adr/
    ├── verification/
    ├── CONFIGURATION.md
    ├── INTERACTIVE-VERIFICATION.md
    ├── LIFECYCLE.md
    ├── SELF-HOSTING.md
    └── VALIDATION.md
```

`config/agents/` is the only custom-agent source. The lifecycle helper maps those files into either repository or global Codex locations. There is no placeholder hooks directory; a hooks surface should appear only with a supported, implemented hook.

## Migration outcome

The pre-cutover scaffold is recoverable from Git commit `75e7598`. Its durable architectural content was grounded before removal:

| Durable concern | Permanent home |
|---|---|
| Native-only product and durable truth | `docs/adr/0001-codex-native-sources-of-truth.md` |
| Plugin plus optional configuration installation | `docs/adr/0002-plugin-and-configuration-distribution.md` |
| Role authority without fixed topology | `docs/adr/0003-role-contracts-not-fixed-topology.md` |
| Lifecycle and self-hosting | `docs/adr/0004-checkpoint-lifecycle-and-self-hosting.md` |
| Delegation, joins, and bounded context | `docs/adr/0005-bounded-delegation-and-context.md` |
| Read parallelism and writer isolation | `docs/adr/0006-parallel-reading-isolated-writing.md` |
| Risk-derived assurance | `docs/adr/0007-risk-derived-assurance.md` |
| Drift, recovery, retry, and learning | `docs/adr/0008-drift-recovery-and-framework-learning.md` |

Behavioral knowledge that needs executable evidence lives in `evals/scenarios/`. The repository does not carry forward the legacy SQLite control plane, external App Server design, Workflow IR, fixed role tree, hard-coded model routes, experimental configuration, leases, or the old schema/template catalog.

## Boundaries

- Mutable Engagement state belongs in a target repository at `.codex/orchestra/` and is ignored in this plugin repository by default.
- Project/global TOML files are templates, not hidden runtime state.
- Development instructions in `AGENTS.md` are not installed into target repositories.
- Installed plugin caches are immutable; upgrades use a newly packaged candidate and a fresh Codex task.
