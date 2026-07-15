# Issue tracker operations

The primary tracker is GitHub repository `edgefloor/codex-orchestra`. Use native GitHub issue
parents, sub-issues, and `blocked by` relationships. Issue titles are the human-facing identity;
numbers and URLs travel with the title but never replace it in summaries or Wayfinder maps.

Run all commands from this checkout or pass `--repo edgefloor/codex-orchestra`. Confirm the target
and authentication before a mutation:

```bash
gh auth status
gh repo view --json nameWithOwner,url,hasIssuesEnabled
```

## Labels

- `ready-for-agent` — an approved implementation ticket.
- `wayfinder:map` — the canonical map issue.
- `wayfinder:research`, `wayfinder:prototype`, `wayfinder:grilling`, `wayfinder:task` — Wayfinder
  child ticket types.

Create a missing label before publishing the first issue that uses it. Do not silently repurpose an
existing label with a different description.

## Read and claim

Read the full issue and its comments, then claim it before work:

```bash
gh issue view <number-or-url> --comments --json number,title,body,state,assignees,labels,url
gh issue edit <number-or-url> --add-assignee @me
```

An open, unassigned issue is unclaimed. An assignment to another user is not available work. Do not
steal or clear a claim merely because it appears stale.

## Publish and relate issues

Draft all issues before mutation and obtain the workflow's required publication approval. Create
blockers before dependants so relationships can use stable numbers. Prefer `--body-file -` for
generated Markdown. Apply `ready-for-agent` only to approved implementation tickets.

```bash
gh issue create --title "<title>" --label ready-for-agent --body-file -
gh issue edit <blocked-issue> --add-blocked-by <blocking-issue>
gh issue edit <child-issue> --parent <parent-issue>
```

Create issues in the first pass and wire dependency or parent edges in a second pass when identities
were not known beforehand. Read the resulting graph back before reporting success:

```bash
gh api graphql \
  -f query='query($owner:String!,$name:String!){repository(owner:$owner,name:$name){issues(first:100,states:OPEN){nodes{number title url assignees(first:10){nodes{login}} blockedBy(first:100){nodes{number title state}} parent{number title}}}}}' \
  -F owner=edgefloor -F name=codex-orchestra
```

The implementation frontier is the open, unassigned set whose `blockedBy` nodes are all closed. A
ticket becomes frontier work only after graph readback proves those conditions.

## Wayfinding operations

Create the map with `wayfinder:map`. Create decision tickets with exactly one Wayfinder type label,
set the map as their parent, then add native blocking edges. Open, unassigned, unblocked child issues
form the map frontier; suspected questions that are not yet precise remain only in the map's
`Not yet specified` section.

Claim one non-research frontier ticket per run. Record its answer in a resolution comment, close it,
and append only a one-line named link to the map's `Decisions so far`. Create newly visible tickets
before wiring their edges. Close mis-scoped tickets and link them from `Out of scope`; do not list
them as decisions.

## Resolve

Publish a concise resolution comment with commit and verification evidence, then close the issue:

```bash
gh issue close <number-or-url> --comment "<resolution and evidence>"
```

Read the issue and dependency graph back after closing it. Closing a blocker, rather than editing a
dependant's body, advances the frontier.

## Git-backed local Markdown fallback

Tests, offline runs, and dry runs use `.scratch/<feature-slug>/issues/` in a temporary or explicitly
selected Git repository. One file represents one issue. Git history is the fallback tracker's source
of truth, while Orchestra stores the resulting commit identity in the run's external-effect receipt;
the commit does not replace checkpoint evidence. Do not fall back from a failed live GitHub mutation
automatically; select local mode before the run.

Use dependency order and this shape:

```markdown
---
schema_version: 1
id: "01"
status: ready-for-agent
assignee: null
blocked_by: []
---

# 01 — Ticket title

**What to build:** End-to-end behavior.

**Blocked by:** None — can start immediately.

- [ ] Acceptance criterion
```

Frontmatter is the canonical machine state and is validated before mutation. `schema_version` is
currently `1`; `id` is the quoted filename prefix; `status` is `ready-for-agent`, `claimed`, or
`closed`; `assignee` is null or a nonempty login; and `blocked_by` is an array of quoted local ids.
The human-readable `Blocked by` line names the same tickets and must agree with frontmatter.

A frontier file has status `ready-for-agent`, a null assignee, and every `blocked_by` file marked
`closed`. Claim by setting status `claimed` and the assignee, commit that single-file change before
work, and reject the claim if the branch no longer fast-forwards. Resolve by appending a resolution
section, setting status `closed`, committing it, and recording the commit id in run evidence. Parent
maps use the same directory with `map.md`; child filenames are their local identities because Git has
no issue graph.
