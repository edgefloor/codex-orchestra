import { readdir, stat } from "node:fs/promises";
import { extname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const dist = fileURLToPath(new URL("../dist", import.meta.url));
const limits = new Map([
  [".html", 32 * 1024],
  [".css", 32 * 1024],
  [".js", 16 * 1024],
  [".avif", 128 * 1024],
  [".webp", 192 * 1024],
  [".png", 2_500 * 1024],
]);

async function files(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map((entry) => {
      const path = join(directory, entry.name);
      return entry.isDirectory() ? files(path) : [path];
    }),
  );
  return nested.flat();
}

const failures = [];
for (const path of await files(dist)) {
  const limit = limits.get(extname(path));
  if (!limit) continue;
  const { size } = await stat(path);
  if (size > limit) {
    failures.push(`${relative(dist, path)} is ${size} bytes; limit is ${limit}`);
  }
}

if (failures.length > 0) {
  throw new Error(`Landing build budget exceeded:\n${failures.join("\n")}`);
}
