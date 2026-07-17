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

const launchOnce = (launchNumber) =>
  new Promise((resolve, reject) => {
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
    let sawWindow = false;
    let stopScheduled = false;
    let shutdownTimeout;

    const forceStop = () => {
      try {
        process.kill(-child.pid, "SIGTERM");
      } catch {
        child.kill("SIGTERM");
      }
    };

    const finish = (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(startupTimeout);
      clearTimeout(shutdownTimeout);
      if (error) reject(error);
      else resolve();
    };

    const requestGracefulStop = () => {
      clearTimeout(startupTimeout);
      child.kill("SIGINT");
      shutdownTimeout = setTimeout(() => {
        forceStop();
        finish(
          new Error(
            `desktop launch ${launchNumber} did not shut down cleanly after startup`,
          ),
        );
      }, 10_000);
    };

    const accept = (chunk) => {
      const text = chunk.toString();
      output = (output + text).slice(-64 * 1024);
      process.stdout.write(text);
      if (output.includes("main window created") && !stopScheduled) {
        sawWindow = true;
        stopScheduled = true;
        setTimeout(requestGracefulStop, 1_000);
      }
    };

    child.stdout.on("data", accept);
    child.stderr.on("data", accept);
    child.on("error", (error) =>
      finish(
        new Error(`desktop launch ${launchNumber} failed: ${error.message}`),
      ),
    );
    child.on("exit", (code, signal) => {
      if (sawWindow) {
        finish();
        return;
      }
      finish(
        new Error(
          `desktop launch ${launchNumber} exited before creating its main window (${code ?? signal})`,
        ),
      );
    });

    const startupTimeout = setTimeout(() => {
      forceStop();
      finish(
        new Error(
          `desktop launch ${launchNumber} did not create its main window within 30 seconds`,
        ),
      );
    }, 30_000);
  });

try {
  await launchOnce(1);
  await launchOnce(2);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
} finally {
  rmSync(t3Home, { force: true, recursive: true });
}
