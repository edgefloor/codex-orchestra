# Disposable desktop host prototype

This crate answers one question from issue #20: can a T3Code-derived renderer adapter hydrate and
recover normal Codex task state while consuming typed Orchestra lifecycle state over the direct,
extended App Server JSON-RPC seam?

It is deliberately a deterministic in-memory fixture, not a production host or a new authority.
The protocol shapes model the intended pinned `app-server-protocol` additions; fixture `thread/read`
data stands in for the pinned App Server handler, and fixture projections stand in for Codex
`StateRuntime`. Delete this crate and the sibling `prototypes/desktop-host/` harness after the
verdict is absorbed into the Codex and desktop product forks.

Run the complete gate from the repository root:

```sh
scripts/desktop-host-prototype.sh
```

