import assert from "node:assert/strict";
import { chmodSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import { resolvePackagedApp, runPackagedAppSmoke } from "./orchestra-packaged-app-smoke.mjs";

const makeFixture = () => {
  const root = mkdtempSync(join(tmpdir(), "orchestra-packaged-smoke-test-"));
  const app = join(root, "Orchestra.app");
  const executable = join(app, "Contents", "MacOS", "Orchestra");
  const resources = join(app, "Contents", "Resources", "orchestra");
  mkdirSync(join(app, "Contents", "MacOS"), { recursive: true });
  mkdirSync(resources, { recursive: true });
  writeFileSync(
    executable,
    `#!/usr/bin/env node
const net = require("node:net");
const { realpathSync } = require("node:fs");
if (
  !process.env.T3CODE_HOME ||
  !process.env.CODEX_HOME ||
  realpathSync(process.cwd()) !== realpathSync(process.env.T3CODE_HOME)
) {
  process.exit(2);
}
const server = net.createServer();
server.listen(0, "127.0.0.1", () => {
  const { port } = server.address();
  console.log("backend ready http://127.0.0.1:" + port + "/");
  console.log("main window created");
});
process.once("SIGINT", () => server.close(() => process.exit(0)));
`,
  );
  chmodSync(executable, 0o755);
  for (const name of ["codex", "orchestra-product", "orchestra-validate-worker"]) {
    const path = join(resources, name);
    writeFileSync(path, "fixture\n");
    chmodSync(path, 0o755);
  }
  writeFileSync(join(resources, "release-manifest.json"), "{}\n");
  writeFileSync(join(resources, "release.toml"), "fixture = true\n");
  return { app, root };
};

test("rejects a directory that is not an extracted app bundle", () => {
  const root = mkdtempSync(join(tmpdir(), "orchestra-packaged-smoke-invalid-"));
  try {
    assert.throws(() => resolvePackagedApp(root), /extracted \.app directory/u);
  } finally {
    rmSync(root, { force: true, recursive: true });
  }
});

test("launches an extracted packaged app twice and observes listener cleanup", async () => {
  const fixture = makeFixture();
  try {
    const result = await runPackagedAppSmoke(fixture.app, {
      orphanTimeoutMs: 2_000,
      settleMs: 10,
      shutdownTimeoutMs: 2_000,
      startupTimeoutMs: 2_000,
    });
    assert.equal(result.launches.length, 2);
    assert.match(result.launches[0].backendUrl, /^http:\/\/127\.0\.0\.1:/u);
    assert.match(result.launches[1].backendUrl, /^http:\/\/127\.0\.0\.1:/u);
  } finally {
    rmSync(fixture.root, { force: true, recursive: true });
  }
});
