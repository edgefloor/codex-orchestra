# Issue #20 prototype verdict

Question: can the retained T3Code-derived renderer boundary consume a direct extended Codex App
Server host seam while preserving task hydration and exposing native Orchestra lifecycle state?

Provisional answer: **keep with changes**. The retained snapshot/reducer concepts work with one
task-local replay cursor, typed lifecycle items, lazy child attachment, and replaceable World State.
Replace Effect RPC/WebSocket and Node authority with generated App Server bindings over the opaque
MessagePort bridge. Keep Electron main limited to process/OS duties and the private confirmation
pipe.

This fixture proves protocol/reducer behavior and the process boundary. It does not prove React
rendering, real Codex `StateRuntime` persistence, provider-backed turns, app signing, or native macOS
confirmation UI. Those must remain pending until the prototype decisions are absorbed into the two
product forks.

