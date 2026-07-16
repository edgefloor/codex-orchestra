---
status: accepted
---

# Ship the product forks as one pinned macOS release

Orchestra ships the Codex and desktop forks as one exact signed and notarized macOS release whose
native components update and roll back together; the Authoring plugin remains independently
versioned and cannot install native execution. Full-app updates use an exactly pinned updater,
preserve the user-owned Codex home, and may replace only rebuildable projections—never canonical
rollouts or repository Run checkpoints—because partial compatibility and history-rewriting recovery
would violate the pinned Product boundary.
