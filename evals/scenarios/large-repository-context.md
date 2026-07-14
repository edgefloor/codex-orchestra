# Large-repository bounded context

- Behavior: delegate effectively without copying a large repository or transcript into every prompt.
- Setup: a fixture with at least three unrelated subsystems, generated/vendor noise, and a change confined to one subsystem plus one shared contract.
- Prompt: investigate and plan the confined change.
- Perturbation: seed plausible but irrelevant matches in the unrelated subsystems.
- Observe: search scope, binding references, capsule size/content, revision pins, duplicated context, and out-of-scope reads/writes.
- Pass: grounding cites the shared contract and relevant subsystem; the assignment capsule contains done condition, authority, boundary, validation, and only necessary references; unrelated content is excluded with rationale.
- Fail: full transcripts or broad dumps are delegated, irrelevant matches drive work, or the Worker must rediscover core scope.
