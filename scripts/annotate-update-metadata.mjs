#!/usr/bin/env node

import { readFile, writeFile } from "node:fs/promises";

const [manifestPath, metadataPath] = process.argv.slice(2);
if (!manifestPath || !metadataPath) {
  throw new Error("usage: annotate-update-metadata.mjs RELEASE_MANIFEST UPDATE_METADATA_YML");
}

const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
const snapshot = manifest.schemas?.snapshot;
const protocol = manifest.schemas?.protocol;
if (!manifest.manifestSha256 || !snapshot?.identity || !protocol?.identity || !protocol?.sha256) {
  throw new Error("release manifest has no exact Product compatibility tuple");
}

const original = await readFile(metadataPath, "utf8");
const keys = [
  "orchestraManifestSha256",
  "orchestraSnapshotSchema",
  "orchestraProjectionSchema",
];
if (keys.some((key) => new RegExp(`^${key}:`, "m").test(original))) {
  throw new Error("update metadata already contains Orchestra Product compatibility fields");
}

const annotated = `${original.trimEnd()}\n${keys[0]}: ${JSON.stringify(manifest.manifestSha256)}\n${keys[1]}: ${JSON.stringify(snapshot.identity)}\n${keys[2]}: ${JSON.stringify(`${protocol.identity}:${protocol.sha256}`)}\n`;
await writeFile(metadataPath, annotated);
