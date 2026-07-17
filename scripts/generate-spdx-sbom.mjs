#!/usr/bin/env node

import { createHash } from "node:crypto";
import { readFile, writeFile } from "node:fs/promises";

function fail(message) {
  console.error(message);
  process.exit(2);
}

function parseArgs(argv) {
  const options = { cargo: [] };
  for (let index = 0; index < argv.length; index += 1) {
    const name = argv[index];
    const value = argv[index + 1];
    if (!value || !name?.startsWith("--")) fail(`missing value for ${name ?? "argument"}`);
    if (name === "--cargo") options.cargo.push(value);
    else if (["--version", "--created", "--pnpm", "--pins", "--output", "--notices"].includes(name)) {
      options[name.slice(2)] = value;
    } else fail(`unknown option: ${name}`);
    index += 1;
  }
  for (const required of ["version", "created", "pnpm", "pins", "output", "notices"]) {
    if (!options[required]) fail(`--${required} is required`);
  }
  if (options.cargo.length === 0) fail("at least one --cargo LABEL=PATH input is required");
  return options;
}

function sha(value) {
  return createHash("sha256").update(value).digest("hex");
}

function spdxId(kind, name, version, source) {
  const readable = name.replace(/[^A-Za-z0-9.-]+/g, "-").replace(/^-+|-+$/g, "").slice(0, 72);
  return `SPDXRef-${kind}-${readable || "package"}-${sha(`${source}:${name}:${version}`).slice(0, 12)}`;
}

function licenseExpression(value, extracted) {
  const normalized = value?.trim();
  if (!normalized || /^(unknown|unlicensed|see license)/i.test(normalized)) return "NOASSERTION";
  if (/^[A-Za-z0-9.+() -]+$/.test(normalized)) return normalized;
  const licenseId = `LicenseRef-${sha(normalized).slice(0, 16)}`;
  extracted.set(licenseId, normalized);
  return licenseId;
}

function downloadLocation(value) {
  if (!value) return "NOASSERTION";
  return value.replace(/^git\+/, "");
}

function packageEntry({ kind, name, version, source, download, license, homepage }, extracted) {
  const ecosystem = kind === "cargo" ? "cargo" : "npm";
  const encodedName = name.split("/").map(encodeURIComponent).join("/");
  return {
    SPDXID: spdxId(kind, name, version, source),
    name,
    versionInfo: version,
    downloadLocation: downloadLocation(download),
    filesAnalyzed: false,
    licenseConcluded: "NOASSERTION",
    licenseDeclared: licenseExpression(license, extracted),
    copyrightText: "NOASSERTION",
    ...(homepage ? { homepage } : {}),
    externalRefs: [
      {
        referenceCategory: "PACKAGE-MANAGER",
        referenceType: "purl",
        referenceLocator: `pkg:${ecosystem}/${encodedName}@${encodeURIComponent(version)}`,
      },
    ],
  };
}

function pinMap(source) {
  const result = new Map();
  for (const line of source.split("\n")) {
    const match = line.match(/^([a-z0-9_]+) = "([^"]*)"$/);
    if (match) result.set(match[1], match[2]);
  }
  return result;
}

function sourcePackage(name, repository, revision, license) {
  const githubPath = repository
    .replace(/^https:\/\/github\.com\//, "")
    .replace(/\.git$/, "");
  return {
    SPDXID: spdxId("source", name, revision, repository),
    name,
    versionInfo: revision,
    downloadLocation: `git+${repository}@${revision}`,
    filesAnalyzed: false,
    licenseConcluded: "NOASSERTION",
    licenseDeclared: license,
    copyrightText: "NOASSERTION",
    externalRefs: [
      {
        referenceCategory: "PACKAGE-MANAGER",
        referenceType: "purl",
        referenceLocator: `pkg:github/${githubPath}@${revision}`,
      },
    ],
  };
}

const options = parseArgs(process.argv.slice(2));
const created = new Date(options.created);
if (Number.isNaN(created.valueOf())) fail("--created must be an ISO-8601 timestamp");

const cargoInputs = [];
for (const input of options.cargo) {
  const separator = input.indexOf("=");
  if (separator <= 0) fail("--cargo must use LABEL=PATH syntax");
  const label = input.slice(0, separator);
  const path = input.slice(separator + 1);
  cargoInputs.push({ label, path, raw: await readFile(path, "utf8") });
}
const pnpmRaw = await readFile(options.pnpm, "utf8");
const pnpm = JSON.parse(pnpmRaw);
const pinsRaw = await readFile(options.pins, "utf8");
const pins = pinMap(pinsRaw);
const extracted = new Map();
const dependencies = [];

for (const input of cargoInputs) {
  const metadata = JSON.parse(input.raw);
  for (const pkg of metadata.packages ?? []) {
    dependencies.push(
      packageEntry(
        {
          kind: "cargo",
          name: pkg.name,
          version: pkg.version,
          source: input.label,
          download: pkg.repository ?? pkg.source,
          license: pkg.license,
          homepage: pkg.homepage,
        },
        extracted,
      ),
    );
  }
}

for (const [licenseGroup, entries] of Object.entries(pnpm)) {
  for (const entry of entries) {
    for (const version of entry.versions ?? []) {
      dependencies.push(
        packageEntry(
          {
            kind: "npm",
            name: entry.name,
            version,
            source: "t3code",
            download: entry.homepage,
            license: entry.license ?? licenseGroup,
            homepage: entry.homepage,
          },
          extracted,
        ),
      );
    }
  }
}

const byIdentity = new Map();
for (const dependency of dependencies) byIdentity.set(dependency.SPDXID, dependency);
const sourcePackages = [
  sourcePackage(
    "orchestra-codex",
    pins.get("orchestra_codex_repository"),
    pins.get("orchestra_codex"),
    "Apache-2.0",
  ),
  sourcePackage(
    "orchestra-desktop",
    pins.get("orchestra_desktop_repository"),
    pins.get("orchestra_desktop"),
    "MIT",
  ),
];
if (sourcePackages.some((pkg) => pkg.versionInfo === undefined || pkg.downloadLocation.includes("undefined"))) {
  fail("Product pins are missing direct fork repository or revision identities");
}
const packages = [...sourcePackages, ...byIdentity.values()].sort((left, right) =>
  `${left.name}@${left.versionInfo}`.localeCompare(`${right.name}@${right.versionInfo}`),
);
const rootPackage = {
  SPDXID: "SPDXRef-Package-Orchestra",
  name: "Orchestra",
  versionInfo: options.version,
  downloadLocation: "NOASSERTION",
  filesAnalyzed: false,
  licenseConcluded: "NOASSERTION",
  licenseDeclared: "Apache-2.0",
  copyrightText: "Copyright 2026 Edgefloor",
};
const inputIdentity = sha(
  cargoInputs.map(({ label, raw }) => `${label}\0${raw}`).join("\0") + pnpmRaw + pinsRaw,
);
const document = {
  spdxVersion: "SPDX-2.3",
  dataLicense: "CC0-1.0",
  SPDXID: "SPDXRef-DOCUMENT",
  name: `Orchestra-${options.version}`,
  documentNamespace: `https://edgefloor.com/spdx/orchestra/${encodeURIComponent(options.version)}/${inputIdentity}`,
  creationInfo: {
    created: created.toISOString(),
    creators: ["Tool: orchestra-generate-spdx-sbom-1"],
  },
  documentDescribes: [rootPackage.SPDXID],
  packages: [rootPackage, ...packages],
  relationships: packages.map((dependency) => ({
    spdxElementId: rootPackage.SPDXID,
    relationshipType: "DEPENDS_ON",
    relatedSpdxElement: dependency.SPDXID,
  })),
  annotations: [
    {
      annotationType: "OTHER",
      annotator: "Tool: orchestra-generate-spdx-sbom-1",
      annotationDate: created.toISOString(),
      comment: `Fork ancestry: orchestra-codex descends from ${pins.get("codex_upstream_repository")}@${pins.get("codex_upstream")}; orchestra-desktop descends from ${pins.get("t3code_upstream_repository")}@${pins.get("t3code_upstream")}.`,
    },
  ],
  ...(extracted.size > 0
    ? {
        hasExtractedLicensingInfos: [...extracted.entries()].map(([licenseId, extractedText]) => ({
          licenseId,
          extractedText,
        })),
      }
    : {}),
};

const notices = [
  `# Orchestra ${options.version} third-party notices`,
  "",
  "This inventory is generated from the locked Rust and production pnpm dependency graphs.",
  "The corresponding SPDX 2.3 document is `orchestra.spdx.json`.",
  "",
  "| Package | Version | Declared license | Homepage/source |",
  "| --- | --- | --- | --- |",
  ...packages.map(
    (pkg) =>
      `| ${pkg.name.replaceAll("|", "\\|")} | ${pkg.versionInfo} | ${pkg.licenseDeclared} | ${pkg.homepage ?? pkg.downloadLocation} |`,
  ),
  "",
].join("\n");

await writeFile(options.output, `${JSON.stringify(document, null, 2)}\n`);
await writeFile(options.notices, notices);
console.log(`wrote SPDX 2.3 SBOM with ${packages.length} dependency packages`);
