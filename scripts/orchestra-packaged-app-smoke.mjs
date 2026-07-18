import { spawn } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, rmSync, statSync } from "node:fs";
import { connect } from "node:net";
import { tmpdir } from "node:os";
import { basename, join, resolve } from "node:path";
import { pathToFileURL } from "node:url";

const STARTUP_TIMEOUT_MS = 30_000;
const SHUTDOWN_TIMEOUT_MS = 10_000;
const ORPHAN_TIMEOUT_MS = 5_000;
const REQUIRED_MARKERS = ["backend ready", "main window created"];

const delay = (milliseconds) =>
  new Promise((resolveDelay) => setTimeout(resolveDelay, milliseconds));

const processGroupExists = (pid) => {
  try {
    process.kill(-pid, 0);
    return true;
  } catch (error) {
    if (error?.code === "ESRCH") return false;
    if (error?.code === "EPERM") return true;
    throw error;
  }
};

const waitFor = async (predicate, timeoutMs) => {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await predicate()) return true;
    await delay(100);
  }
  return predicate();
};

const loopbackUrl = (text) => {
  const backendMarker = text.lastIndexOf("backend ready");
  const backendOutput = backendMarker === -1 ? text : text.slice(backendMarker);
  const matches = backendOutput.matchAll(
    /https?:\/\/(?:127\.0\.0\.1|localhost|\[::1\]):\d+(?:\/[^\s"']*)?/giu,
  );
  let latest;
  for (const match of matches) latest = match[0];
  return latest;
};

const listenerAcceptsConnections = (url) =>
  new Promise((resolveConnection) => {
    const parsed = new URL(url);
    const socket = connect({
      host: parsed.hostname.replace(/^\[(.*)\]$/u, "$1"),
      port: Number(parsed.port),
    });
    const finish = (accepting) => {
      socket.destroy();
      resolveConnection(accepting);
    };
    socket.setTimeout(250, () => finish(false));
    socket.once("connect", () => finish(true));
    socket.once("error", () => finish(false));
  });

export const resolvePackagedApp = (appArgument) => {
  if (!appArgument) {
    throw new Error("usage: node scripts/orchestra-packaged-app-smoke.mjs EXTRACTED_ORCHESTRA_APP");
  }

  const app = resolve(appArgument);
  if (!existsSync(app) || !statSync(app).isDirectory() || !app.endsWith(".app")) {
    throw new Error(`packaged Orchestra app is not an extracted .app directory: ${app}`);
  }

  const executable = join(app, "Contents", "MacOS", "Orchestra");
  const resources = join(app, "Contents", "Resources", "orchestra");
  const requiredFiles = [
    executable,
    join(resources, "codex"),
    join(resources, "orchestra-product"),
    join(resources, "orchestra-validate-worker"),
    join(resources, "release-manifest.json"),
    join(resources, "release.toml"),
  ];
  for (const path of requiredFiles) {
    if (!existsSync(path) || !statSync(path).isFile()) {
      throw new Error(`packaged Orchestra app is missing required file: ${path}`);
    }
  }
  for (const path of requiredFiles.slice(0, 4)) {
    if ((statSync(path).mode & 0o111) === 0) {
      throw new Error(`packaged Orchestra executable is not executable: ${path}`);
    }
  }
  JSON.parse(readFileSync(join(resources, "release-manifest.json"), "utf8"));
  return { app, executable };
};

export const launchOnce = (
  executable,
  launchNumber,
  environment,
  {
    startupTimeoutMs = STARTUP_TIMEOUT_MS,
    shutdownTimeoutMs = SHUTDOWN_TIMEOUT_MS,
    orphanTimeoutMs = ORPHAN_TIMEOUT_MS,
    settleMs = 1_000,
  } = {},
) =>
  new Promise((resolveLaunch, rejectLaunch) => {
    const child = spawn(executable, [], {
      cwd: environment.T3CODE_HOME,
      detached: true,
      env: environment,
      stdio: ["ignore", "pipe", "pipe"],
    });
    const pid = child.pid;
    let output = "";
    let settled = false;
    let stopScheduled = false;
    let shutdownTimeout;

    const forceStop = () => {
      if (!pid) return;
      try {
        process.kill(-pid, "SIGTERM");
      } catch {
        child.kill("SIGTERM");
      }
    };

    const finish = (error, result) => {
      if (settled) return;
      settled = true;
      clearTimeout(startupTimeout);
      clearTimeout(shutdownTimeout);
      if (error) rejectLaunch(error);
      else resolveLaunch(result);
    };

    const requestGracefulStop = () => {
      clearTimeout(startupTimeout);
      child.kill("SIGINT");
      shutdownTimeout = setTimeout(() => {
        forceStop();
        finish(
          new Error(`packaged app launch ${launchNumber} did not shut down cleanly after startup`),
        );
      }, shutdownTimeoutMs);
    };

    const accept = (chunk) => {
      const text = chunk.toString();
      output = (output + text).slice(-256 * 1024);
      process.stdout.write(text);
      if (!stopScheduled && REQUIRED_MARKERS.every((marker) => output.includes(marker))) {
        stopScheduled = true;
        setTimeout(requestGracefulStop, settleMs);
      }
    };

    child.stdout.on("data", accept);
    child.stderr.on("data", accept);
    child.on("error", (error) =>
      finish(new Error(`packaged app launch ${launchNumber} failed: ${error.message}`)),
    );
    child.on("exit", async (code, signal) => {
      if (!REQUIRED_MARKERS.every((marker) => output.includes(marker))) {
        finish(
          new Error(
            `packaged app launch ${launchNumber} exited before backend readiness and main window creation (${code ?? signal})`,
          ),
        );
        return;
      }
      if (code !== 0 || signal !== null) {
        finish(
          new Error(
            `packaged app launch ${launchNumber} did not exit cleanly (${code ?? signal})`,
          ),
        );
        return;
      }
      if (!pid) {
        finish(new Error(`packaged app launch ${launchNumber} did not report a process id`));
        return;
      }

      const backendUrl = loopbackUrl(output);
      if (!backendUrl) {
        finish(
          new Error(`packaged app launch ${launchNumber} did not report its loopback backend URL`),
        );
        return;
      }
      const groupExited = await waitFor(() => !processGroupExists(pid), orphanTimeoutMs);
      if (!groupExited) {
        forceStop();
        finish(new Error(`packaged app launch ${launchNumber} left an orphan process group`));
        return;
      }
      const listenerClosed = await waitFor(
        async () => !(await listenerAcceptsConnections(backendUrl)),
        orphanTimeoutMs,
      );
      if (!listenerClosed) {
        finish(
          new Error(
            `packaged app launch ${launchNumber} left its backend listener active at ${backendUrl}`,
          ),
        );
        return;
      }
      finish(undefined, { backendUrl });
    });

    const startupTimeout = setTimeout(() => {
      forceStop();
      finish(
        new Error(
          `packaged app launch ${launchNumber} did not reach backend readiness and main window creation within ${startupTimeoutMs} ms`,
        ),
      );
    }, startupTimeoutMs);
  });

export const runPackagedAppSmoke = async (appArgument, options) => {
  const { app, executable } = resolvePackagedApp(appArgument);
  const t3Home = mkdtempSync(join(tmpdir(), "orchestra-packaged-smoke-t3-"));
  const codexHome = mkdtempSync(join(tmpdir(), "orchestra-packaged-smoke-codex-"));
  try {
    const environment = {
      ...process.env,
      T3CODE_HOME: t3Home,
      CODEX_HOME: codexHome,
      ELECTRON_ENABLE_LOGGING: "1",
    };
    const launches = [];
    for (const launchNumber of [1, 2]) {
      launches.push(await launchOnce(executable, launchNumber, environment, options));
    }
    return { app, codexHome: basename(codexHome), launches, t3Home: basename(t3Home) };
  } finally {
    rmSync(t3Home, { force: true, recursive: true });
    rmSync(codexHome, { force: true, recursive: true });
  }
};

const main = async () => {
  const result = await runPackagedAppSmoke(process.argv[2]);
  console.log(
    `Packaged Orchestra smoke passed: ${result.launches.length} launches, isolated T3Code and Codex homes, no orphan process or listener.`,
  );
};

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) {
  main().catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  });
}
