---
status: accepted
---

# Unify Codex tasks and Orchestra execution without conflating their state

A resident Workflow invocation belongs to one Codex task while Orchestra owns its parent-linked Run
Tree and execution graph; interruption suspends the Run rather than detaching it. Repository
checkpoints remain execution authority, Codex rollouts remain conversation and semantic history, and
Codex `StateRuntime` holds only rebuildable task replay and projections. Root models coordinate from
bounded Run Digests while raw child detail stays in child tasks, preserving native V2 behavior and
context economy without a separate Host store or copied histories.
