---
status: accepted
---

# Compile restricted TypeScript into a native V2 Rust runtime

Workflow authors use a restricted TypeScript-shaped data language for editor support and composable `agent`, `parallel`, `pipeline`, `check`, `approval`, `worktree`, and bounded `repeat` calls. Orchestra lexes and parses this language itself; no JavaScript engine executes it. Functions, dynamic imports, methods, process/filesystem/network APIs, `eval`, and side effects are rejected.

The Rust runtime owns validation, DAG scheduling, exact context materialization and hashing, retries, repeat bounds, write conflicts, sandbox-aware checks, worktrees, approvals, atomic checkpoints, recovery, cancellation, validated outputs, evidence, and summaries.

Agent work uses an injected native host backed by the active parent task's V2 `AgentControl`. Each step declares its model and optional reasoning/service tier. `fork_turns` defaults to `none`, child delegation defaults off, and full-history override restrictions are enforced.

The decisions in ADRs 0004 through 0008 remain applicable where they describe bounded work, exact inputs, isolated writers, risk-derived checks, and transcript-independent recovery. Fixed role topology and model-authored logging do not.

