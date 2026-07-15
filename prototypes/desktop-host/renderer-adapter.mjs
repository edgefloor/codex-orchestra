// PROTOTYPE ONLY. This is the portable reducer/client seam adapted from T3Code's
// snapshot + pure event reducer concepts; it intentionally has no React or Effect dependency.

const PROJECTION_TAIL_LIMIT = 16;

export function createRendererState() {
  return {
    hostInstanceId: null,
    tasks: new Map(),
    orchestraItems: new Map(),
    worldState: new Map(),
    cursors: new Map(),
    childSubscriptions: new Set(),
    rawEventsByTask: new Map(),
    unknownEvents: [],
    reconnectRequired: false,
  };
}

export function applyThreadRead(state, result) {
  const thread = result.thread;
  state.tasks.set(thread.id, structuredClone(thread));
  for (const turn of thread.turns ?? []) {
    for (const item of turn.items ?? []) {
      if (item.type === "orchestra") applyOrchestraItem(state, thread.id, item.item);
    }
  }
}

export function applySnapshot(state, result) {
  const snapshot = result.snapshot;
  applyThreadRead(state, snapshot.threadRead);
  for (const item of snapshot.orchestra.items ?? []) {
    applyOrchestraItem(state, snapshot.taskId, item.item);
  }
  for (const [key, value] of Object.entries(snapshot.orchestra.worldState ?? {})) {
    state.worldState.set(key, structuredClone(value));
  }
  state.cursors.set(snapshot.taskId, snapshot.barrier);
}

export function applyNotification(state, message) {
  const taskId = message.orchestra?.taskId;
  const sequence = message.orchestra?.sequence;
  if (message.method === "host/reconnectRequired") {
    state.reconnectRequired = true;
    return;
  }
  if (taskId && Number.isInteger(sequence)) {
    const current = state.cursors.get(taskId) ?? 0;
    if (sequence <= current) return;
    if (sequence !== current + 1) {
      throw new Error(`gap for ${taskId}: expected ${current + 1}, received ${sequence}`);
    }
    state.cursors.set(taskId, sequence);
  }

  switch (message.method) {
    case "orchestra/lifecycle/updated":
      applyOrchestraItem(state, taskId, message.params.item.item ?? message.params.item);
      if (message.params.worldState?.operation === "replace") {
        state.worldState.set(
          message.params.worldState.key,
          structuredClone(message.params.worldState.value),
        );
      }
      break;
    case "item/completed":
    case "prototype/noisy": {
      const events = state.rawEventsByTask.get(taskId) ?? [];
      events.push(structuredClone(message));
      if (events.length > PROJECTION_TAIL_LIMIT) events.shift();
      state.rawEventsByTask.set(taskId, events);
      break;
    }
    default:
      state.unknownEvents.push(structuredClone(message));
      if (state.unknownEvents.length > PROJECTION_TAIL_LIMIT) state.unknownEvents.shift();
  }
}

export function markChildExpanded(state, taskId) {
  state.childSubscriptions.add(taskId);
}

export function applyOrchestraItem(state, taskId, item) {
  const current = state.orchestraItems.get(item.id);
  if (current && item.revision < current.item.revision) return;
  if (
    current &&
    item.revision === current.item.revision &&
    JSON.stringify(item) !== JSON.stringify(current.item)
  ) {
    throw new Error(`integrity error: item ${item.id} revision ${item.revision} changed`);
  }
  state.orchestraItems.set(item.id, { taskId, item: structuredClone(item) });
}

export function visibleState(state) {
  return {
    hostInstanceId: state.hostInstanceId,
    tasks: [...state.tasks.keys()],
    orchestraItems: Object.fromEntries(
      [...state.orchestraItems].map(([id, value]) => [id, value]),
    ),
    worldState: Object.fromEntries(state.worldState),
    cursors: Object.fromEntries(state.cursors),
    childSubscriptions: [...state.childSubscriptions],
    rawEventTasks: Object.fromEntries(
      [...state.rawEventsByTask].map(([taskId, events]) => [taskId, events.length]),
    ),
    reconnectRequired: state.reconnectRequired,
  };
}
