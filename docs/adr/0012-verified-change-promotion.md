---
status: accepted
---

# Promote verified isolated changes into the target checkout

Agent writes remain isolated patches until their dependencies, deterministic checks, review, and
acceptance succeed. Promotion applies the verified aggregate conflict-safely and idempotently without
overwriting target changes; rejection or conflict preserves the target and durable candidate evidence
because isolated execution must not imply permission to mutate the user's checkout.
