import { z } from "zod";

const MAX_REQUEST_BYTES = 2 * 1024 * 1024;
const MAX_RESPONSE_BYTES = 2 * 1024 * 1024;
const MAX_BUNDLE_BYTES = 1024 * 1024;
const MAX_VALUE_BYTES = 1024 * 1024;
const MAX_ISSUES = 128;
const MAX_ISSUE_TEXT_BYTES = 512;
const EVALUATOR_REVISION = "bun-1.3.14-zod-4.4.3-prototype-1";

type Request =
  | { op: "validate"; bundleSource: string; bundleHash: string; schemaId: string; value: unknown }
  | { op: "burn" }
  | { op: "crash" };

type ValidationIssue = {
  code: string;
  path: Array<string | number>;
  message: string;
};

function byteLength(value: string): number {
  return new TextEncoder().encode(value).byteLength;
}

function sha256(value: string): string {
  return new Bun.CryptoHasher("sha256").update(value).digest("hex");
}

function canonicalize(value: unknown): string {
  if (value === null || typeof value === "boolean" || typeof value === "string") {
    return JSON.stringify(value);
  }
  if (typeof value === "number") {
    if (!Number.isFinite(value)) throw new Error("non-finite number");
    return JSON.stringify(value);
  }
  if (Array.isArray(value)) {
    return `[${value.map(canonicalize).join(",")}]`;
  }
  if (typeof value === "object") {
    const record = value as Record<string, unknown>;
    const keys = Object.keys(record).sort();
    return `{${keys.map((key) => `${JSON.stringify(key)}:${canonicalize(record[key])}`).join(",")}}`;
  }
  throw new Error(`noncanonical value type: ${typeof value}`);
}

function boundedText(value: string): string {
  const bytes = new TextEncoder().encode(value);
  if (bytes.byteLength <= MAX_ISSUE_TEXT_BYTES) return value;
  return new TextDecoder().decode(bytes.slice(0, MAX_ISSUE_TEXT_BYTES));
}

function normalizeIssues(issues: ReadonlyArray<{ code?: string; path?: PropertyKey[]; message?: string }>): ValidationIssue[] {
  return issues
    .slice(0, MAX_ISSUES)
    .map((issue) => ({
      code: String(issue.code ?? "custom"),
      path: (issue.path ?? []).map((part) => typeof part === "number" ? part : String(part)),
      message: boundedText(String(issue.message ?? "invalid value")),
    }))
    .sort((left, right) => canonicalize(left).localeCompare(canonicalize(right)));
}

function rejectForbiddenSource(source: string): void {
  const forbidden = [
    /\basync\b/u,
    /\bawait\b/u,
    /\bimport\s*\(/u,
    /\b(?:Bun|process|require|globalThis|WebAssembly|Function|eval|Date)\b/u,
    /Math\s*\.\s*random/u,
  ];
  if (forbidden.some((pattern) => pattern.test(source))) {
    throw new Error("validation bundle contains a forbidden capability");
  }
}

function loadSchema(bundleSource: string, schemaId: string): z.ZodType {
  if (byteLength(bundleSource) > MAX_BUNDLE_BYTES) throw new Error("validation bundle exceeds byte limit");
  rejectForbiddenSource(bundleSource);
  const factory = new Function(
    "z",
    `"use strict";
     const Bun = undefined, process = undefined, require = undefined;
     const globalThis = undefined, WebAssembly = undefined, Date = undefined;
     return (${bundleSource});`,
  );
  const bundle = factory(z) as Record<string, unknown>;
  const schema = bundle[schemaId];
  if (!(schema instanceof z.ZodType)) throw new Error("unknown or invalid schema identity");
  return schema;
}

async function handle(request: Request): Promise<unknown> {
  switch (request.op) {
    case "validate": {
      if (sha256(request.bundleSource) !== request.bundleHash) throw new Error("validation bundle hash mismatch");
      const rawCanonical = canonicalize(request.value);
      if (byteLength(rawCanonical) > MAX_VALUE_BYTES) throw new Error("canonical value exceeds byte limit");
      const result = loadSchema(request.bundleSource, request.schemaId).safeParse(request.value);
      if (!result.success) {
        return {
          kind: "rejected",
          provenance: { bundleHash: request.bundleHash, evaluatorRevision: EVALUATOR_REVISION },
          rawCanonical,
          issues: normalizeIssues(result.error.issues),
        };
      }
      return {
        kind: "accepted",
        provenance: { bundleHash: request.bundleHash, evaluatorRevision: EVALUATOR_REVISION },
        rawCanonical,
        transformedCanonical: canonicalize(result.data),
      };
    }
    case "burn":
      for (;;) Math.imul(31, 17);
    case "crash":
      process.abort();
  }
}

async function main(): Promise<void> {
  const input = await Bun.stdin.text();
  if (byteLength(input) > MAX_REQUEST_BYTES) throw new Error("request exceeds byte limit");
  const request = JSON.parse(input) as Request;
  const response = canonicalize({ ok: true, value: await handle(request) });
  if (byteLength(response) > MAX_RESPONSE_BYTES) throw new Error("response exceeds byte limit");
  process.stdout.write(`${response}\n`);
}

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`${boundedText(message)}\n`);
  process.exit(70);
});
