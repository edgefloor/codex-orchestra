const compatibility = {
  rendererBuildId: "t3code-ecb35f75-orchestra-prototype",
  hostBuildId: "orchestra-host-issue-20-prototype",
  codexRevision: "f90e7deea6a715bbd153044af6f475eefa749177",
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

function encode(value) {
  const body = new TextEncoder().encode(JSON.stringify(value));
  const bytes = new Uint8Array(4 + body.length);
  new DataView(bytes.buffer).setUint32(0, body.length);
  bytes.set(body, 4);
  return bytes;
}

window.addEventListener("message", async (event) => {
  if (event.data?.type !== "orchestra-port" || event.ports.length !== 1) return;
  try {
    const port = event.ports[0];
    let buffered = new Uint8Array();
    const pending = new Map();
    port.onmessage = ({ data }) => {
      const chunk = new Uint8Array(data);
      const joined = new Uint8Array(buffered.length + chunk.length);
      joined.set(buffered);
      joined.set(chunk, buffered.length);
      buffered = joined;
      while (buffered.length >= 4) {
        const length = new DataView(buffered.buffer, buffered.byteOffset).getUint32(0);
        if (buffered.length < 4 + length) break;
        const message = JSON.parse(new TextDecoder().decode(buffered.slice(4, 4 + length)));
        buffered = buffered.slice(4 + length);
        if (message.id !== undefined && pending.has(message.id)) {
          const resolve = pending.get(message.id);
          pending.delete(message.id);
          resolve(message);
        }
      }
    };
    port.start();

    let nextId = 1;
    const request = (method, params) => new Promise((resolve, reject) => {
      const id = nextId++;
      pending.set(id, (message) => message.error ? reject(new Error(message.error.message)) : resolve(message.result));
      port.postMessage(encode({ id, method, params }));
    });

    await request("initialize", {
      clientInfo: { name: "retained-electron-prototype", version: "0.0.0" },
      compatibility,
    });
    const thread = await request("thread/read", { threadId: "task-parent" });
    const confirmation = await request("prototype/confirm", {});
    const isolated = typeof process === "undefined" && typeof require === "undefined";
    if (thread.thread.id !== "task-parent" || confirmation.channel !== "privilegedControlPipe" || !isolated) {
      throw new Error("unexpected bridge result");
    }
    window.postMessage({
      orchestraPrototypeResult: `ORCHESTRA_ELECTRON_PASS ${JSON.stringify({ isolated, taskId: thread.thread.id, confirmation: confirmation.channel })}`,
    }, "*");
  } catch (error) {
    window.postMessage({
      orchestraPrototypeResult: `ORCHESTRA_ELECTRON_FAIL ${error instanceof Error ? error.message : String(error)}`,
    }, "*");
  }
});
