import assert from "node:assert/strict";
import test from "node:test";
import {
  applyNotification,
  applyOrchestraItem,
  createRendererState,
} from "./renderer-adapter.mjs";

const item = {
  id: "item-1",
  kind: "step",
  revision: 2,
  status: "running",
  childTaskId: "task-child",
};

test("lower revisions are ignored and same-revision divergence is rejected", () => {
  const state = createRendererState();
  applyOrchestraItem(state, "task-parent", item);
  applyOrchestraItem(state, "task-parent", { ...item, revision: 1, status: "queued" });
  assert.equal(state.orchestraItems.get("item-1").item.status, "running");
  assert.throws(
    () => applyOrchestraItem(state, "task-parent", { ...item, status: "completed" }),
    /integrity error/,
  );
});

test("unknown sequenced events remain visible as bounded diagnostics", () => {
  const state = createRendererState();
  for (let sequence = 1; sequence <= 20; sequence += 1) {
    applyNotification(state, {
      jsonrpc: "2.0",
      method: "codex/futureEvent",
      params: { bounded: true, sequence },
      orchestra: { taskId: "task-parent", sequence },
    });
  }
  assert.equal(state.unknownEvents.length, 16);
  assert.equal(state.unknownEvents[0].params.sequence, 5);
  assert.equal(state.cursors.get("task-parent"), 20);
});
