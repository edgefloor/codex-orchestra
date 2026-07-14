# Codex Orchestra

**Run reviewable, recoverable multi-agent workflows inside native Codex—without a daemon, sidecar, MCP server, or model-authored state.**

Codex Orchestra turns a declared `.workflow.ts` plan into a Rust-owned execution run: agents receive only the context you declare, independent steps run concurrently, writes are isolated in worktrees, checks and approvals gate promotion, and every outcome is checkpointed in the target repository.

> **Status:** experimental, source-first infrastructure for an Orchestra-enabled Codex build. The plugin skills can install on stock Codex, but native workflow tools require the pinned integration described below.

## Try the native vertical slice

From a checkout of this repository, these commands build and verify the exact Codex integration Orchestra targets:

```bash
# Verify the Rust runtime itself
cargo test --workspace

# Check plugin and configuration capabilities
cargo run -p codex-orchestra-lifecycle -- doctor

# Clone the pinned Codex revision (if needed), apply the overlay, and verify it
scripts/codex-integration.sh /tmp/codex-orchestra-codex verify
```

To prepare a repository for a real run, preview the configuration first; add `--apply` only after reviewing the proposed changes:

```bash
cargo run -p codex-orchestra-lifecycle -- project --target /path/to/your/repo
cargo run -p codex-orchestra-lifecycle -- project --target /path/to/your/repo --apply
```

The resulting Codex build exposes native `orchestra_validate`, `orchestra_run`, `orchestra_resume`, `orchestra_status`, and `orchestra_cancel` tools. Runtime-owned artifacts stay in the target repository at `.codex/orchestra/runs/`.

## What a workflow looks like

Workflows are a small, restricted TypeScript-shaped data language. Rust parses and lowers them to an execution plan; it never evaluates JavaScript.

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

## Why Orchestra

| Need | Orchestra's default |
| --- | --- |
| Keep agents on the active Codex task | Parent-linked V2 `AgentControl`, canonical task paths, and completion watchers |
| Make context auditable | Exact declared bytes and dependency outputs, materialized and SHA-256 hashed |
| Avoid concurrent-write surprises | Sandbox-aware checks and isolated Git worktrees for conflicting writes |
| Recover from interruption | Atomic checkpoints, resumable approvals, cancellation, and a terminal run summary |
| Keep execution predictable | Bounded retries and repeats; child delegation is disabled by default |

Every run records its plan, attempts, context hashes, validated outputs, evidence, decisions, and summary independently of the parent transcript.

## Requirements and honest boundaries

Orchestra currently has two deliberate delivery pieces:

1. This repository’s Rust runtime, lifecycle tooling, and installable plugin skills.
2. A small, temporary integration patch for the Codex revision pinned in [integration/codex/UPSTREAM_REVISION](integration/codex/UPSTREAM_REVISION).

Stock plugin packages cannot dynamically register arbitrary Rust extensions. Consequently, stock Codex may load the authoring skills but cannot run native Orchestra tools. There is intentionally no fallback through SDK threads, `codex exec`, an App Server client, an MCP server, a daemon, or a sidecar.

The lifecycle tool is preview-first and hash-managed. It preserves modified files, supports upgrade/rollback/uninstall, and never removes run artifacts. To install a selectable profile rather than project configuration:

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

Before structural changes, run:

```bash
cargo test --workspace
cargo run -p codex-orchestra-lifecycle -- doctor
scripts/codex-integration.sh /tmp/codex-orchestra-codex verify
```

The integration command requires a clean checkout at the pinned revision. It applies the overlay, tests the Orchestra crates, and checks `codex-app-server`.

## License

[MIT](LICENSE)
