---
status: superseded by ADR-0009
---

# Preserve checkpoints and version-to-version self-hosting

Runs persist enough workflow, evidence, approval, and Git state to resume after interruption, and
installed version N validates candidate N+1 without modifying its own cache. ADR 0009 superseded
only this decision's earlier assumption that Orchestra should not generate a first-class workflow.
