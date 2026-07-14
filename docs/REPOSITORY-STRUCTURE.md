# Repository structure

```text
.codex-plugin/plugin.json      installable plugin manifest
skills/                        workflow creation, execution, and recovery behavior
config/agents/                 optional planner, worker, reviewer, verifier agents
assets/schemas/                workflow, run-state, and step-result contracts
assets/templates/              workflow, step-input, run-summary, and verification templates
assets/policies/               default execution limits and safety rules
scripts/lifecycle.py           preview-first configuration lifecycle
scripts/workflow.py            workflow validation and run initialization
docs/adr/                      architecture decisions
evals/workflows/               executable workflow fixtures
evals/scenarios/               behavioral acceptance scenarios
tests/                         deterministic repository tests
```

Runtime data is never bundled with the plugin:

```text
<target-repository>/.codex/orchestra/
├── workflows/                 reusable project workflows
├── runs/<run-id>/
│   ├── workflow.yaml          immutable run snapshot
│   ├── state.json             durable step state and workflow digest
│   ├── steps/                 inputs and structured attempt results
│   ├── evidence/              command and review evidence
│   └── summary.md             final or paused run summary
└── install/                   managed configuration lifecycle state
```

Configuration templates are separate from plugin installation. Users may install them per repository, as a selectable global profile, or deliberately reconcile them into their global default.
