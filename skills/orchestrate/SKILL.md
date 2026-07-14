---
name: orchestrate
description: Conduct a bounded Codex-native delivery engagement using repository artifacts and native collaboration tools.
---

# Orchestra conductor

Act as the Operator-facing conductor. Durable state is `.codex/orchestra/`; plugin files are read-only resources.

1. Run the lifecycle `doctor`, then inspect the earliest incomplete artifact in charter, plans, tasks, results, verification, or recovery.
2. Ground intent and create a charter before planning implementation. Record reversible defaults; stop only for material authority, scope, safety, budget, or irreversible choices.
3. Create a delivery plan with bounded workstreams, dependencies, write domains, acceptance, review, and verification expectations.
4. Form the smallest taskforce. Use native `spawn_agent`, `send_message`, `followup_task`, `wait_agent`, and worktrees when available. Never invent an external scheduler.
5. Give each child a self-contained context capsule and explicit output path. The parent that spawns a child owns its wait, review, and join.
6. Require independent review for implementation and independent milestone verification when risk warrants it.
7. Persist decisions, results, evidence, and recovery checkpoints before handoff or close.

For a new run, route through `$codex-orchestra:ground-project`, `$codex-orchestra:create-charter`, and `$codex-orchestra:plan-delivery`. For interrupted work, use `$codex-orchestra:recover-run` first.

Complete when the next operator checkpoint is decision-ready or the accepted engagement is closed with evidence.
