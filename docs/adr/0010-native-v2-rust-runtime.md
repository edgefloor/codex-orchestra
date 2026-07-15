---
status: accepted
---

# Compile restricted TypeScript into a native V2 Rust runtime

Workflow authors use a restricted TypeScript-shaped data language for editor support and composable `agent`, `parallel`, `pipeline`, `check`, `approval`, `worktree`, and bounded `repeat` calls. Orchestra lexes and parses this language itself; no JavaScript engine executes it. Functions, dynamic imports, methods, process/filesystem/network APIs, `eval`, and side effects are rejected.

The Rust runtime owns validation, DAG scheduling, exact context materialization and hashing, retries, repeat bounds, write conflicts, sandbox-aware checks, worktrees, approvals, atomic checkpoints, recovery, cancellation, validated outputs, evidence, and summaries.

Agent work uses an injected native host backed by the active parent task's V2 `AgentControl`. Each step declares its model and optional reasoning/service tier. `fork_turns` defaults to `none`, child delegation defaults off, and full-history override restrictions are enforced.

A workflow begins as a native invocation inside its parent Codex task, normally through an
Orchestra skill/tool call made in that task. After invocation, Rust deterministically owns the Run
and scheduling; the model does not simulate the scheduler. A future UI shortcut may initiate the
same task-bound action, but there is no detached renderer- or host-created Run.

The native invocation remains resident until the Run terminates or reaches a durable suspension.
If its Codex turn is interrupted or the host is lost, Rust prevents further commits from active
Attempts, requests best-effort child cancellation, and records the Run for reconciliation and
task-native recovery. It does not continue active workflow work in the background.

The decisions in ADRs 0004 through 0008 remain applicable where they describe bounded work, exact inputs, isolated writers, risk-derived checks, and transcript-independent recovery. Fixed role topology and model-authored logging do not.
