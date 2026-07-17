import { z } from "zod";

const MAX_REQUEST_BYTES = 2 * 1024 * 1024;
const MAX_DIAGNOSTIC_BYTES = 512;
const EVALUATOR_REVISION = "bun-1.3.14-zod-4.4.3-sealed-2";

type Limits = {
  requestBytes: number;
  responseBytes: number;
  bundleBytes: number;
  canonicalValueBytes: number;
  wallTimeMs: number;
  valueDepth: number;
  valueNodes: number;
  collectionEntries: number;
  stringBytes: number;
  issueCount: number;
  issueTextBytes: number;
};

type Request = {
  op: "validate";
  bundleSource: string;
  bundleHash: string;
  schemaId: string;
  value: unknown;
  evaluatorRevision: string;
  limits: Limits;
};

type ValidationIssue = {
  code: string;
  path: Array<string | number>;
  message: string;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

function byteLength(value: string): number {
  return encoder.encode(value).byteLength;
}

function sha256(value: string): string {
  return new Bun.CryptoHasher("sha256").update(value).digest("hex");
}

function canonicalize(value: unknown, limits: Limits): string {
  let nodes = 0;
  const active = new WeakSet<object>();

  function visit(current: unknown, depth: number): string {
    nodes += 1;
    if (nodes > limits.valueNodes) throw new Error("canonical value exceeds node limit");
    if (depth > limits.valueDepth) throw new Error("canonical value exceeds depth limit");
    if (current === null || typeof current === "boolean") return JSON.stringify(current);
    if (typeof current === "string") {
      if (byteLength(current) > limits.stringBytes) throw new Error("canonical string exceeds byte limit");
      return JSON.stringify(current);
    }
    if (typeof current === "number") {
      if (!Number.isFinite(current)) throw new Error("non-finite number");
      return JSON.stringify(current);
    }
    if (typeof current !== "object") {
      throw new Error(`noncanonical value type: ${typeof current}`);
    }
    if (active.has(current)) throw new Error("cyclic value");
    active.add(current);
    try {
      if (Array.isArray(current)) {
        if (current.length > limits.collectionEntries) throw new Error("array exceeds entry limit");
        for (let index = 0; index < current.length; index += 1) {
          if (!Object.hasOwn(current, index)) throw new Error("sparse arrays are not canonical");
        }
        return `[${current.map((entry) => visit(entry, depth + 1)).join(",")}]`;
      }
      const prototype = Object.getPrototypeOf(current);
      if (prototype !== Object.prototype && prototype !== null) {
        throw new Error("class instances are not canonical JSON");
      }
      const record = current as Record<string, unknown>;
      const keys = Object.keys(record).sort();
      if (keys.length > limits.collectionEntries) throw new Error("object exceeds key limit");
      return `{${keys
        .map((key) => `${JSON.stringify(key)}:${visit(record[key], depth + 1)}`)
        .join(",")}}`;
    } finally {
      active.delete(current);
    }
  }

  const canonical = visit(value, 0);
  if (byteLength(canonical) > limits.canonicalValueBytes) {
    throw new Error("canonical value exceeds byte limit");
  }
  return canonical;
}

function boundedText(value: string, bytesLimit: number): string {
  const bytes = encoder.encode(value);
  if (bytes.byteLength <= bytesLimit) return value;
  return decoder.decode(bytes.slice(0, bytesLimit));
}

function normalizeIssues(
  issues: ReadonlyArray<{ code?: string; path?: PropertyKey[]; message?: string }>,
  limits: Limits,
): ValidationIssue[] {
  return issues
    .slice(0, limits.issueCount)
    .map((issue) => ({
      code: String(issue.code ?? "custom"),
      path: (issue.path ?? []).map((part) => (typeof part === "number" ? part : String(part))),
      message: boundedText(String(issue.message ?? "invalid value"), limits.issueTextBytes),
    }))
    .map((issue) => ({ issue, identity: canonicalize(issue, limits) }))
    .sort((left, right) => (left.identity < right.identity ? -1 : left.identity > right.identity ? 1 : 0))
    .map(({ issue }) => issue);
}

function rejectForbiddenSource(source: string): void {
  const forbidden = [
    /\basync\b/u,
    /\bawait\b/u,
    /\bimport\s*\(/u,
    /\b(?:Bun|process|require|globalThis|WebAssembly|Function|eval|Date|Promise)\b/u,
    /Math\s*\.\s*random/u,
  ];
  if (forbidden.some((pattern) => pattern.test(source))) {
    throw new Error("validation bundle contains a forbidden capability");
  }
}

function loadSchema(bundleSource: string, schemaId: string, limits: Limits): z.ZodType {
  if (byteLength(bundleSource) > limits.bundleBytes) {
    throw new Error("validation bundle exceeds byte limit");
  }
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

async function readRequest(): Promise<string> {
  const chunks: Uint8Array[] = [];
  let length = 0;
  for await (const chunk of Bun.stdin.stream()) {
    length += chunk.byteLength;
    if (length > MAX_REQUEST_BYTES) throw new Error("request exceeds byte limit");
    chunks.push(chunk);
  }
  const bytes = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return decoder.decode(bytes);
}

function validateRequest(value: unknown): asserts value is Request {
  if (!value || typeof value !== "object") throw new Error("request must be an object");
  const request = value as Partial<Request>;
  if (
    request.op !== "validate" ||
    typeof request.bundleSource !== "string" ||
    typeof request.bundleHash !== "string" ||
    typeof request.schemaId !== "string" ||
    typeof request.evaluatorRevision !== "string" ||
    !request.limits ||
    typeof request.limits !== "object"
  ) {
    throw new Error("malformed validation request");
  }
}

async function main(): Promise<void> {
  const input = await readRequest();
  const parsed: unknown = JSON.parse(input);
  validateRequest(parsed);
  const request = parsed;
  if (byteLength(input) > request.limits.requestBytes) throw new Error("request exceeds effective byte limit");
  if (sha256(request.bundleSource) !== request.bundleHash) {
    throw new Error("validation bundle hash mismatch");
  }
  if (request.evaluatorRevision !== EVALUATOR_REVISION) {
    throw new Error("unsupported evaluator revision");
  }
  const rawCanonical = canonicalize(request.value, request.limits);
  const result = loadSchema(request.bundleSource, request.schemaId, request.limits).safeParse(request.value);
  const provenance = {
    bundleHash: request.bundleHash,
    evaluatorRevision: EVALUATOR_REVISION,
  };
  const value = result.success
    ? {
        kind: "accepted",
        provenance,
        rawCanonical,
        transformedCanonical: canonicalize(result.data, request.limits),
      }
    : {
        kind: "rejected",
        provenance,
        rawCanonical,
        issues: normalizeIssues(result.error.issues, request.limits),
      };
  const response = canonicalize({ ok: true, value }, request.limits);
  if (byteLength(response) > request.limits.responseBytes) {
    throw new Error("response exceeds byte limit");
  }
  process.stdout.write(response);
}

main().catch((error: unknown) => {
  const message = error instanceof Error ? error.message : String(error);
  process.stderr.write(`${boundedText(message, MAX_DIAGNOSTIC_BYTES - 1)}\n`);
  process.exit(70);
});
