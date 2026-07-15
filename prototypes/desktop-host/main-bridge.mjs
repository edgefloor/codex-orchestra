// PROTOTYPE ONLY. Models the retained Electron main seam: supervise one signed host,
// forward opaque data bytes, and keep fd 3 private from the renderer.

import { spawn } from "node:child_process";
import { MessageChannel } from "node:worker_threads";
import { createFrameDecoder, encodeFrame } from "./framing.mjs";

export class MainBridge {
  constructor(binaryPath, hostInstanceId) {
    this.child = spawn(binaryPath, [], {
      env: { ...process.env, ORCHESTRA_PROTOTYPE_HOST_INSTANCE: hostInstanceId },
      stdio: ["pipe", "pipe", "pipe", "pipe"],
    });
    this.diagnostics = [];
    this.confirmations = [];
    this.child.stderr.setEncoding("utf8");
    this.child.stderr.on("data", (text) => this.diagnostics.push(text.trim()));
    this.child.stdout.on("data", (chunk) => this.#forwardToRenderer(chunk));

    const decodeControl = createFrameDecoder((challenge) => {
      this.confirmations.push(challenge);
      this.child.stdio[3].write(
        encodeFrame({
          type: "confirmationResponse",
          challengeId: challenge.challengeId,
          decision: "accept",
        }),
      );
    });
    this.child.stdio[3].on("data", decodeControl);
    this.rendererPort = this.#newRendererPort();
  }

  #newRendererPort() {
    const { port1, port2 } = new MessageChannel();
    this.mainPort?.close();
    this.mainPort = port1;
    this.mainPort.on("message", (bytes) => this.child.stdin.write(Buffer.from(bytes)));
    this.mainPort.start();
    return port2;
  }

  #forwardToRenderer(chunk) {
    if (!this.mainPort) return;
    const bytes = Uint8Array.from(chunk);
    this.mainPort.postMessage(bytes, [bytes.buffer]);
  }

  reloadRenderer() {
    this.rendererPort?.close();
    this.rendererPort = this.#newRendererPort();
    return this.rendererPort;
  }

  async stop() {
    this.mainPort?.close();
    this.rendererPort?.close();
    if (this.child.exitCode !== null) return;
    this.child.stdin.end();
    await new Promise((resolve) => this.child.once("exit", resolve));
  }
}

