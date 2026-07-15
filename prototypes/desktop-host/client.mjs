import { createFrameDecoder, encodeFrame } from "./framing.mjs";

export class HostRpcError extends Error {
  constructor(error) {
    super(error.message);
    this.name = "HostRpcError";
    this.code = error.data?.code;
    this.retry = error.data?.retry;
    this.details = error.data?.details;
  }
}

export class RendererClient {
  constructor(port, onNotification) {
    this.port = port;
    this.onNotification = onNotification;
    this.nextId = 1;
    this.pending = new Map();
    const decode = createFrameDecoder((message) => this.#onMessage(message));
    port.on("message", decode);
    port.start();
  }

  request(method, params = {}) {
    const id = this.nextId++;
    const request = { jsonrpc: "2.0", id, method, params };
    const promise = new Promise((resolve, reject) => this.pending.set(id, { resolve, reject }));
    const frame = encodeFrame(request);
    const bytes = Uint8Array.from(frame);
    this.port.postMessage(bytes, [bytes.buffer]);
    return promise;
  }

  close() {
    this.port.close();
  }

  #onMessage(message) {
    if (message.id !== undefined) {
      const pending = this.pending.get(message.id);
      if (!pending) return;
      this.pending.delete(message.id);
      if (message.error) pending.reject(new HostRpcError(message.error));
      else pending.resolve(message.result);
      return;
    }
    this.onNotification(message);
  }
}

