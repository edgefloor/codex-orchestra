import { createHash } from "node:crypto";

type Outcome = {
  name: string;
  exitCode: number | null;
  signalCode: string | null;
  stdout: string;
  stderr: string;
  elapsedMs: number;
  killedFor: string | null;
};

const worker = process.argv[2];
if (!worker) throw new Error("worker path is required");

async function run(
  name: string,
  request: unknown,
  options: { timeoutMs?: number } = {},
): Promise<Outcome> {
  const started = performance.now();
  const child = Bun.spawn([worker], {
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
    env: { PATH: "/usr/bin:/bin", LANG: "C", TMPDIR: "/tmp" },
  });
  child.stdin.write(JSON.stringify(request));
  child.stdin.end();

  let killedFor: string | null = null;
  const timeout = setTimeout(() => {
    if (!killedFor) {
      killedFor = "wall-time";
      child.kill("SIGKILL");
    }
  }, options.timeoutMs ?? 5_000);

  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  clearTimeout(timeout);
  return {
    name,
    exitCode,
    signalCode: child.signalCode,
    stdout: stdout.trim(),
    stderr: stderr.trim(),
    elapsedMs: Math.round(performance.now() - started),
    killedFor,
  };
}

const schema = `({
  output: z.object({
    count: z.number().int().min(0).transform((value) => value + 1),
    label: z.string().trim().min(1)
  }).refine((value) => value.count < 10, { path: ["count"], message: "count is too large" })
})`;
const bundleHash = createHash("sha256").update(schema).digest("hex");

const deterministic: Outcome[] = [];
for (let index = 0; index < 5; index += 1) {
  deterministic.push(await run(`determinism-${index}`, {
    op: "validate",
    bundleSource: schema,
    bundleHash,
    schemaId: "output",
    value: { label: " result ", count: 2 },
  }));
}

const rejection = await run("normalized-rejection", {
  op: "validate",
  bundleSource: schema,
  bundleHash,
  schemaId: "output",
  value: { label: "", count: -1 },
});

const provenanceMismatch = await run("provenance-mismatch", {
  op: "validate",
  bundleSource: schema,
  bundleHash: "0".repeat(64),
  schemaId: "output",
  value: { label: "result", count: 2 },
});

const oversizedRequest = await run("oversized-request", {
  op: "validate",
  bundleSource: schema,
  bundleHash,
  schemaId: "output",
  value: { text: "x".repeat(2 * 1024 * 1024) },
});

const timeout = await run("wall-time-kill", { op: "burn" }, { timeoutMs: 250 });
const crash = await run("crash", { op: "crash" });

const hashes = deterministic.map((outcome) => createHash("sha256").update(outcome.stdout).digest("hex"));
const report = {
  worker,
  architecture: process.arch,
  bun: Bun.version,
  deterministic: {
    attempts: deterministic.length,
    uniqueHashes: [...new Set(hashes)],
    outcomes: deterministic,
  },
  rejection,
  provenanceMismatch,
  oversizedRequest,
  timeout,
  crash,
};

process.stdout.write(`${JSON.stringify(report, null, 2)}\n`);
