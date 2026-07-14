# Bounded candidate plan

1. Inventory the seed and validate version-sensitive assumptions.
2. Scaffold the plugin manifest, skills, assets, configuration kit, lifecycle helper, docs, and tests.
3. Independently review the resulting source and run manifest/config/test validation.
4. Leave marketplace installation and fresh-task invocation as an explicit Operator checkpoint so user-owned Codex state is not silently changed.

Topology: Operator -> Consultant (plan/charter) -> Team Leader (scaffold integration) -> Worker (implementation) -> Reviewer (independent review) -> Operator checkpoint.
