#!/usr/bin/env node

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";

const [worker, evaluatorRevision] = process.argv.slice(2);

const BUN_X64_AVX_WARNING =
  "warn: CPU lacks AVX support, strange crashes may occur. Reinstall Bun or use *-baseline build:\n" +
  "  https://github.com/oven-sh/bun/releases/download/bun-v1.3.14/bun-darwin-x64-baseline.zip\n";

if (!worker || !evaluatorRevision) {
  process.stderr.write("usage: node scripts/evaluator-smoke.mjs WORKER EVALUATOR_REVISION\n");
  process.exit(64);
}

const limits = Object.freeze({
  requestBytes: 2 * 1024 * 1024,
  responseBytes: 2 * 1024 * 1024,
  bundleBytes: 1024 * 1024,
  canonicalValueBytes: 1024 * 1024,
  wallTimeMs: 1_000,
  valueDepth: 64,
  valueNodes: 100_000,
  collectionEntries: 10_000,
  stringBytes: 256 * 1024,
  issueCount: 128,
  issueTextBytes: 512,
});

function sha256(value) {
  return createHash("sha256").update(value, "utf8").digest("hex");
}

function request(bundleSource, value, overrides = {}) {
  return {
    op: "validate",
    bundleSource,
    bundleHash: sha256(bundleSource),
    schemaId: "output",
    value,
    evaluatorRevision,
    limits: { ...limits },
    ...overrides,
  };
}

function invoke(input) {
  const result = spawnSync(worker, [], {
    input,
    encoding: "utf8",
    maxBuffer: 4 * 1024 * 1024,
    timeout: 10_000,
  });
  assert.equal(result.error, undefined, `could not invoke evaluator worker: ${result.error?.message}`);
  assert.equal(result.signal, null, `evaluator worker terminated with ${result.signal}`);
  return {
    ...result,
    stderr: result.stderr.startsWith(BUN_X64_AVX_WARNING)
      ? result.stderr.slice(BUN_X64_AVX_WARNING.length)
      : result.stderr,
  };
}

function expectInfrastructureFailure(name, input) {
  const result = invoke(input);
  assert.equal(result.status, 70, `${name}: expected exit 70; stderr=${JSON.stringify(result.stderr)}`);
  assert.equal(result.stdout, "", `${name}: infrastructure failure must not emit a response`);
  const diagnosticBytes = Buffer.byteLength(result.stderr, "utf8");
  assert.ok(result.stderr.trim().length > 0, `${name}: diagnostic must be non-empty`);
  assert.ok(diagnosticBytes <= 512, `${name}: diagnostic is ${diagnosticBytes} bytes, expected at most 512`);
}

const exactZodBundle = `({
  output: z.object({
    count: z.number().int().min(0).transform((value) => value + 1),
    label: z.string().trim().min(1)
  }).refine((value) => value.count < 10, {
    path: ["count"],
    message: "count is too large"
  })
})`;
const acceptedRequest = request(exactZodBundle, { label: " result ", count: 2 });
const accepted = invoke(JSON.stringify(acceptedRequest));
assert.equal(accepted.status, 0, `exact Zod request failed: ${accepted.stderr}`);
assert.equal(accepted.stderr, "", "accepted request unexpectedly emitted a diagnostic");

const acceptedResponse = JSON.parse(accepted.stdout);
assert.equal(acceptedResponse.ok, true);
assert.equal(acceptedResponse.value.kind, "accepted");
assert.deepEqual(acceptedResponse.value.provenance, {
  bundleHash: sha256(exactZodBundle),
  evaluatorRevision,
});
assert.equal(acceptedResponse.value.rawCanonical, '{"count":2,"label":" result "}');
assert.equal(acceptedResponse.value.transformedCanonical, '{"count":3,"label":"result"}');

expectInfrastructureFailure(
  "evaluator revision mismatch",
  JSON.stringify(request("({ output: z.string() })", "ok", {
    evaluatorRevision: `${evaluatorRevision}-mismatch`,
  })),
);

const forbiddenSources = new Map([
  ["Bun capability", "({ output: z.string().refine(() => Bun.version.length > 0) })"],
  ["process capability", "({ output: z.string().refine(() => process.pid > 0) })"],
  ["dynamic import capability", '({ output: z.string(), load: import("forbidden") })'],
  ["async capability", "({ output: z.string().refine(async () => true) })"],
]);
for (const [name, source] of forbiddenSources) {
  expectInfrastructureFailure(name, JSON.stringify(request(source, "ok")));
}

expectInfrastructureFailure("malformed JSON", "{");
expectInfrastructureFailure("malformed request", JSON.stringify({ op: "validate" }));

const limitedRequest = request("({ output: z.string() })", "request is larger than its effective limit");
limitedRequest.limits.requestBytes = 64;
const limitedInput = JSON.stringify(limitedRequest);
assert.ok(Buffer.byteLength(limitedInput, "utf8") > limitedRequest.limits.requestBytes);
expectInfrastructureFailure("effective request byte limit", limitedInput);

process.stdout.write(`Evaluator smoke passed for ${evaluatorRevision}\n`);
