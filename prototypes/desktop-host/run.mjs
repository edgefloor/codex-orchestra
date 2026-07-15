#!/usr/bin/env node
// PROTOTYPE ONLY. Deterministic end-to-end trace for GitHub issue #20.

import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { RendererClient, HostRpcError } from "./client.mjs";
import { MainBridge } from "./main-bridge.mjs";
import {
  applyNotification,
  applySnapshot,
  applyThreadRead,
  createRendererState,
  markChildExpanded,
  visibleState,
} from "./renderer-adapter.mjs";

const CODEX_REVISION = "f90e7deea6a715bbd153044af6f475eefa749177";
const T3CODE_REVISION = "ecb35f75839925dd1ac6f854efeef5c9e291d11b";
const compatibility = {
  rendererBuildId: "t3code-ecb35f75-orchestra-prototype",
  hostBuildId: "orchestra-host-issue-20-prototype",
  codexRevision: CODEX_REVISION,
  protocolSchemaHash: "e87ff11b6d0c02e466b3b225c67faff9c84c90b8d04b7f4ee5bcc086e89c20a5",
  snapshotSchemaId: "task-snapshot-prototype-v1",
  requiredCapabilities: [
    "thread/read",
    "host/snapshot",
    "host/replay",
    "orchestra/threadItem",
    "orchestra/worldState",
    "orchestra/query",
    "host/privilegedConfirmation",
  ],
};

const here = path.dirname(fileURLToPath(import.meta.url));
const binary = process.argv[2] ?? path.resolve(here, "../../target/debug/orchestra-host-prototype");
const trace = [];

function makeSession(instance) {
  const state = createRendererState();
  const bridge = new MainBridge(binary, instance);
  const client = new RendererClient(bridge.rendererPort, (message) => applyNotification(state, message));
  return { state, bridge, client };
}

async function initialize(session) {
  const result = await session.client.request("initialize", {
    clientInfo: { name: "t3code-derived-prototype", version: "0.0.0" },
    compatibility,
  });
  assert.equal(result.hostInstanceId.startsWith("host-"), true);
  assert.equal("taskCursors" in result, false);
  session.state.hostInstanceId = result.hostInstanceId;
  trace.push({ check: "exact bundle initialization", result: "passed", host: result.hostInstanceId });
}

const first = makeSession("host-a");
await initialize(first);

const listed = await first.client.request("thread/list", {});
assert.deepEqual(listed.data.map((entry) => entry.id), ["task-child", "task-parent"]);
const threadRead = await first.client.request("thread/read", { threadId: "task-parent" });
applyThreadRead(first.state, threadRead);
const snapshot = await first.client.request("host/snapshot/get", { taskId: "task-parent" });
assert.equal(
  createHash("sha256").update(JSON.stringify(snapshot.snapshot)).digest("hex"),
  snapshot.digest,
);
applySnapshot(first.state, snapshot);
assert.equal(snapshot.snapshot.barrier, 1);
trace.push({ check: "thread/read plus composed snapshot", result: "passed", barrier: 1 });

await first.client.request("host/replay/subscribe", {
  taskId: "task-parent",
  cursor: snapshot.snapshot.barrier,
});
await first.client.request("prototype/advance", {});
await new Promise(setImmediate);
assert.equal(first.state.cursors.get("task-parent"), 2);
assert.equal(first.state.orchestraItems.get("orchestra-step-1").item.revision, 2);
assert.equal(first.state.orchestraItems.get("orchestra-step-1").item.childTaskId, "task-child");
assert.equal(first.state.worldState.get("orchestra.runDigest").revision, 2);
trace.push({ check: "fixture replay-to-live plus lifecycle and digest reducer", result: "passed", cursor: 2 });

const parentText = JSON.stringify(first.state.tasks.get("task-parent"));
assert.equal(parentText.includes("task-child"), true);
assert.equal(parentText.includes("child detail stays here"), false);
const childRead = await first.client.request("thread/read", { threadId: "task-child" });
applyThreadRead(first.state, childRead);
markChildExpanded(first.state, "task-child");
await first.client.request("host/replay/subscribe", { taskId: "task-child", cursor: 0 });
await new Promise(setImmediate);
assert.equal(first.state.rawEventsByTask.get("task-child").length, 1);
const childSnapshot = await first.client.request("host/snapshot/get", { taskId: "task-child" });
applySnapshot(first.state, childSnapshot);
await first.client.request("host/replay/subscribe", {
  taskId: "task-child",
  cursor: childSnapshot.snapshot.barrier,
});
assert.equal(JSON.stringify(first.state.tasks.get("task-child")).includes("child detail stays here"), true);
trace.push({ check: "lazy parent/child fixture attachment", result: "passed", childTaskId: "task-child" });

const selector = { kind: "run", id: "run-1" };
const rendererQuery = await first.client.request("orchestra/query", { taskId: "task-parent", consumer: "renderer", selector });
const modelQuery = await first.client.request("orchestra/query", { taskId: "task-parent", consumer: "nativeTaskTool", selector });
assert.deepEqual(rendererQuery.selection, modelQuery.selection);
await assert.rejects(
  first.client.request("orchestra/query", { taskId: "task-child", consumer: "renderer", selector }),
  (error) => error instanceof HostRpcError && error.code === "authorization_denied",
);
const diagnostics = await first.client.request("host/diagnostics/export", {});
assert.equal(diagnostics.redacted, true);
assert.equal(diagnostics.events[0].token, "[REDACTED]");
trace.push({ check: "authorized bounded query adapter contract", result: "passed" });

const confirmation = await first.client.request("prototype/confirm", {});
assert.deepEqual(confirmation, { confirmed: true, channel: "privilegedControlPipe" });
assert.equal(first.bridge.confirmations.length, 1);
assert.equal("stdio" in first.client, false);
trace.push({ check: "separate privileged confirmation pipe plumbing", result: "passed" });

const oldClient = first.client;
const reloadedState = createRendererState();
const reloadedPort = first.bridge.reloadRenderer();
oldClient.close();
first.client = new RendererClient(reloadedPort, (message) => applyNotification(reloadedState, message));
first.state = reloadedState;
await initialize(first);
const reloadSnapshot = await first.client.request("host/snapshot/get", { taskId: "task-parent" });
applySnapshot(first.state, reloadSnapshot);
await first.client.request("host/replay/subscribe", { taskId: "task-parent", cursor: reloadSnapshot.snapshot.barrier });
assert.equal(first.state.orchestraItems.get("orchestra-step-1").item.revision, 2);
trace.push({ check: "renderer reload from snapshot plus cursor", result: "passed", barrier: reloadSnapshot.snapshot.barrier });

const flood = await first.client.request("prototype/flood", { count: 8 });
await new Promise(setImmediate);
assert.equal(flood.journaled, 8);
assert.equal(flood.delivered, 0);
assert.equal(first.state.reconnectRequired, true);
const staleCursor = first.state.cursors.get("task-parent");
trace.push({ check: "overload policy after bounded journaling", result: "passed", staleCursor });
await first.bridge.stop();

const restarted = makeSession("host-b");
await initialize(restarted);
await assert.rejects(
  restarted.client.request("host/replay/subscribe", { taskId: "task-parent", cursor: staleCursor }),
  (error) => error instanceof HostRpcError && error.code === "snapshot_required" && error.retry === "after_snapshot",
);
const recovered = await restarted.client.request("host/snapshot/get", { taskId: "task-parent" });
applySnapshot(restarted.state, recovered);
trace.push({ check: "host restart and expired cursor recovery", result: "passed", newBarrier: recovered.snapshot.barrier });
await restarted.bridge.stop();

const mismatch = makeSession("host-mismatch");
const badCompatibility = { ...compatibility, rendererBuildId: "wrong-renderer" };
await assert.rejects(
  mismatch.client.request("initialize", { compatibility: badCompatibility }),
  (error) => error instanceof HostRpcError && error.code === "incompatible_bundle" && error.retry === "never",
);
trace.push({ check: "exact bundle mismatch typed failure", result: "passed" });
await mismatch.bridge.stop();

console.log(JSON.stringify({
  prototype: "issue-20-desktop-host",
  codexRevision: CODEX_REVISION,
  t3codeRevision: T3CODE_REVISION,
  verdict: "keep-with-changes",
  checks: trace,
  finalRendererState: visibleState(first.state),
}, null, 2));
