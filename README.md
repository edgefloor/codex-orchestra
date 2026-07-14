# Codex Orchestra

**Dynamic, reviewable multi-agent workflows for Codex.**

Codex Orchestra adds a dynamic workflow layer to Codex's native agent runtime. Instead of relying on a fixed team of agents or a hard-coded sequence, each `.workflow.ts` file defines the agents, models, context, dependencies, parallel stages, checks, approvals, and repeat policies needed for a particular task.

Rust parses that workflow source into an execution plan and owns its scheduling and durable state. Each agent receives an exact context bundle, concurrent writers use isolated worktrees, checks and approvals control progress, and atomic checkpoints make the run recoverable from the target repository.

## Verify the native runtime

From a checkout of this repository, these commands test Orchestra and verify it against the exact Codex revision it targets:

```bash
# Verify the Rust runtime itself
cargo test --workspace

# Check plugin and configuration capabilities
cargo run -p codex-orchestra-lifecycle -- doctor

# Clone the pinned Codex revision (if needed), apply the integration patch, and verify it
scripts/codex-integration.sh /tmp/codex-orchestra-codex verify
```

To configure a target repository, preview the proposed changes first. Add `--apply` only after reviewing them:

```bash
cargo run -p codex-orchestra-lifecycle -- project --target /path/to/your/repo
cargo run -p codex-orchestra-lifecycle -- project --target /path/to/your/repo --apply
```

The integrated Codex build exposes `orchestra_validate`, `orchestra_run`, `orchestra_resume`, `orchestra_status`, and `orchestra_cancel` as native tools. Run artifacts remain in the target repository under `.codex/orchestra/runs/`.

## Define a workflow

Workflow source uses a small, restricted TypeScript-shaped data language. It is expressive enough to compose task-specific workflows while remaining declarative and statically validated.

```ts
import { agent, check, pipeline, workflow } from "@codex-orchestra/workflow";

export default workflow({
  name: "native-slice",
  max_parallel: 2,
  steps: [
    pipeline([
      agent({
        id: "implement",
        prompt: "Implement the requested bounded change.",
        model: "gpt-5.4",
        reasoning_effort: "high",
        fork_turns: "none",
        context: [{ type: "file", path: "CONTEXT.md" }],
        outputs: ["summary", "changed_files"],
        write_scope: ["src/"],
      }),
      check({
        id: "tests",
        command: ["cargo", "test", "--workspace"],
        timeout_ms: 300000,
      }),
    ]),
  ],
});
```

See the complete runnable template in [assets/templates/WORKFLOW.workflow.ts](assets/templates/WORKFLOW.workflow.ts).

## Why dynamic workflows

| Requirement | Orchestra behavior |
| --- | --- |
| Fit the workflow to the task | Each workflow chooses its own agents, models, context, dependencies, checks, approvals, and repeat policies |
| Keep agent execution native to the active Codex task | The native host uses parent-linked V2 `AgentControl`, canonical task paths, and completion watchers |
| Make agent context auditable | Context bundles contain exact declared bytes and dependency outputs and carry a SHA-256 digest |
| Control concurrent writes | Dependency-ready writers run concurrently only with disjoint write scopes and isolated Git worktrees |
| Recover after interruption | Atomic checkpoints preserve run state; approvals resume explicitly; terminal state is recorded in the run summary |
| Bound execution | Attempts and repeat rounds have explicit limits; child delegation is disabled by default |

Every run records its execution plan, attempts, context hashes, validated outputs, evidence, decisions, and summary independently of the parent transcript.

## Integration boundary

Orchestra currently consists of two parts:

1. This repository’s Rust runtime, lifecycle tooling, and installable plugin skills.
2. A temporary integration patch for the Codex revision pinned in [integration/codex/UPSTREAM_REVISION](integration/codex/UPSTREAM_REVISION).

Stock Codex plugin packages cannot dynamically register arbitrary Rust extensions. Stock Codex can therefore load Orchestra's authoring skills, but it cannot expose the native workflow tools. Orchestra deliberately provides no alternate execution path through SDK threads, `codex exec`.

The lifecycle tool previews changes by default and tracks managed files by hash. It preserves user-modified files, supports upgrades, rollbacks, and uninstall, and never removes run artifacts. To install a selectable profile instead of project-local configuration:

```bash
cargo run -p codex-orchestra-lifecycle -- profile
cargo run -p codex-orchestra-lifecycle -- profile --apply
```

## Learn more

- [Domain language and runtime invariants](CONTEXT.md)
- [Repository structure](docs/REPOSITORY-STRUCTURE.md)
- [Configuration and lifecycle](docs/CONFIGURATION.md)
- [Self-hosting the pinned Codex build](docs/SELF-HOSTING.md)
- [Verification layers and current human-only checks](docs/VALIDATION.md)
- [Architecture decisions](docs/adr/)

## Development checks

After structural changes, run:

```bash
cargo test --workspace
cargo run -p codex-orchestra-lifecycle -- doctor
scripts/codex-integration.sh /tmp/codex-orchestra-codex verify
```

The integration command requires a clean checkout at the pinned revision. It applies the integration patch, tests the Orchestra crates, and checks `codex-app-server`.

## License

[MIT](LICENSE)
