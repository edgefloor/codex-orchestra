---
status: accepted
---

# Extend Codex behind the retained T3Code application

The MVP keeps T3Code's normal Electron, React, and local-server application and extends its existing
Codex provider seam to launch the exact Orchestra-enabled CLI. Codex App Server remains the sole
native agent/workflow protocol and Rust remains the authorization authority; the T3Code server is a
presentation adapter, not a second agent backend. The direct MessagePort/framed-host bridge remains
prototype evidence rather than the product UI architecture. Add a renderer-inaccessible native
confirmation channel only when Orchestra privileged-confirmation UX is implemented.
