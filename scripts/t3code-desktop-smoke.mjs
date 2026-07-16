import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const [electron, desktopMain, releaseManifest, codexCli] = process.argv.slice(2);

if (!electron || !desktopMain || !releaseManifest || !codexCli) {
  console.error(
    "usage: node scripts/t3code-desktop-smoke.mjs ELECTRON MAIN MANIFEST CODEX_CLI",
  );
  process.exit(2);
}

const t3Home = mkdtempSync(join(tmpdir(), "orchestra-t3code-smoke-"));
const child = spawn(electron, [desktopMain], {
  detached: true,
  env: {
    ...process.env,
    T3CODE_HOME: t3Home,
    ORCHESTRA_CODEX_PATH: codexCli,
    ORCHESTRA_RELEASE_MANIFEST: releaseManifest,
  },
  stdio: ["ignore", "pipe", "pipe"],
});

let output = "";
let settled = false;
let passed = false;
let stopScheduled = false;
let shutdownTimeout;

const forceStop = () => {
  try {
    process.kill(-child.pid, "SIGTERM");
  } catch {
    child.kill("SIGTERM");
  }
};

const finish = (code, message) => {
  if (settled) return;
  settled = true;
  clearTimeout(timeout);
  clearTimeout(shutdownTimeout);
  rmSync(t3Home, { force: true, recursive: true });
  if (message) console.error(message);
  process.exitCode = code;
};

const requestGracefulStop = () => {
  if (passed) return;
  passed = true;
  clearTimeout(timeout);
  child.kill("SIGINT");
  shutdownTimeout = setTimeout(() => {
    forceStop();
    finish(1, "desktop did not shut down cleanly after its startup smoke");
  }, 10_000);
};

const accept = (chunk) => {
  const text = chunk.toString();
  output = (output + text).slice(-64 * 1024);
  process.stdout.write(text);
  if (output.includes("main window created") && !stopScheduled) {
    stopScheduled = true;
    setTimeout(requestGracefulStop, 1_000);
  }
};

child.stdout.on("data", accept);
child.stderr.on("data", accept);
child.on("error", (error) => finish(1, `desktop failed to start: ${error.message}`));
child.on("exit", (code, signal) => {
  if (passed) {
    finish(0);
    return;
  }
  if (!settled) {
    finish(1, `desktop exited before creating its main window (${code ?? signal})`);
  }
});

const timeout = setTimeout(() => {
  forceStop();
  finish(1, "desktop did not create its main window within 30 seconds");
}, 30_000);
