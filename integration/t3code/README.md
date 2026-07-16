# Pinned T3Code Product fork

`UPSTREAM_REVISION` pins the complete T3Code product. `scripts/t3code-integration.sh` applies one
reviewed provider seam and copies generated protocol bindings from the matching Codex fork. The
normal Electron, React, local-server, task, chat, project, settings, and native-subagent surfaces stay
intact.

The MVP boundary is:

```text
sandboxed T3Code renderer -> retained T3Code local server -> pinned Orchestra-enabled Codex CLI
```

`ORCHESTRA_CODEX_PATH` overrides only the binary selected by T3Code's existing Codex driver. The
driver still owns provider process lifecycle and protocol adaptation; Codex owns tasks, subagents,
workflow execution, authorization, and canonical history. No workflow runner replaces the home or
chat screen, and the renderer cannot start detached Runs.

The product manifest seals the Codex CLI and the complete T3Code desktop/server/renderer tuple. The
build verifies the exact source patch, normal UI entrypoints, provider tests, task timeline tests,
typechecks, a real desktop startup, a native App Server handshake, and evaluator behavior.

Use the repository scripts rather than editing a prepared checkout:

```sh
scripts/product-source-prepare.sh /tmp/orchestra-product-sources
scripts/product-dev-build.sh /tmp/orchestra-product-sources
scripts/product-dogfood.sh /tmp/orchestra-product-sources
```
