import { app, BrowserWindow, MessageChannelMain, ipcMain } from "electron";
import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const hostBinary = process.argv[2];
if (!hostBinary) throw new Error("host binary path is required");

const child = spawn(hostBinary, [], {
  env: { ...process.env, ORCHESTRA_PROTOTYPE_HOST_INSTANCE: "electron-host" },
  stdio: ["pipe", "pipe", "pipe", "pipe"],
});
child.stderr.pipe(process.stderr);

let finished = false;
const finish = (code, message) => {
  if (finished) return;
  finished = true;
  process.stdout.write(`${message}\n`);
  child.stdin.end();
  setTimeout(() => app.exit(code), 50);
};

app.whenReady().then(async () => {
  globalThis.setTimeout(() => finish(1, "ORCHESTRA_ELECTRON_FAIL timeout"), 15_000);
  const window = new BrowserWindow({
    show: false,
    webPreferences: {
      preload: path.join(here, "preload.cjs"),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  const { port1, port2 } = new MessageChannelMain();
  port1.on("message", ({ data }) => child.stdin.write(Buffer.from(data)));
  port1.start();
  child.stdout.on("data", (chunk) => port1.postMessage(Uint8Array.from(chunk)));

  const control = child.stdio[3];
  let controlBytes = Buffer.alloc(0);
  control.on("data", (chunk) => {
    controlBytes = Buffer.concat([controlBytes, chunk]);
    if (controlBytes.length < 4) return;
    const length = controlBytes.readUInt32BE(0);
    if (controlBytes.length < 4 + length) return;
    const challenge = JSON.parse(controlBytes.subarray(4, 4 + length).toString("utf8"));
    const body = Buffer.from(JSON.stringify({
      type: "confirmationResponse",
      challengeId: challenge.challengeId,
      decision: "accept",
    }));
    const frame = Buffer.allocUnsafe(4 + body.length);
    frame.writeUInt32BE(body.length, 0);
    body.copy(frame, 4);
    control.write(frame);
  });

  ipcMain.on("orchestra-prototype-result", (_event, result) => {
    const message = String(result);
    finish(message.startsWith("ORCHESTRA_ELECTRON_PASS ") ? 0 : 1, message);
  });
  window.webContents.once("did-finish-load", () => {
    window.webContents.postMessage("orchestra-port", null, [port2]);
  });
  await window.loadFile(path.join(here, "renderer.html"));
}).catch((error) => finish(1, `ORCHESTRA_ELECTRON_FAIL ${error}`));
