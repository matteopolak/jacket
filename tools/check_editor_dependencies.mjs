import { readFile, writeFile } from "node:fs/promises";
const root = new URL("../", import.meta.url);
const lock = JSON.parse(await readFile(new URL("package-lock.json", root), "utf8"));
const cutoff = Date.parse("2026-09-19T00:00:00Z");
const names = new Map();
for (const [name, pkg] of Object.entries(lock.packages)) if (name.startsWith("node_modules/")) names.set(name.slice(name.lastIndexOf("node_modules/") + "node_modules/".length), pkg);
const packages = [];
for (const [name, pkg] of names) {
  const response = await fetch(`https://registry.npmjs.org/${encodeURIComponent(name)}`);
  if (!response.ok) throw new Error(`Registry metadata unavailable for ${name}`);
  const metadata = await response.json(); const published = metadata.time?.[pkg.version];
  if (!published || Date.parse(published) > cutoff) throw new Error(`${name}@${pkg.version} does not meet the 14-day minimum at 2026-10-03.`);
  if (!pkg.integrity || !pkg.resolved?.startsWith("https://registry.npmjs.org/")) throw new Error(`Lock identity missing for ${name}`);
  packages.push({ name, version: pkg.version, published, integrity: pkg.integrity });
}
await writeFile(new URL("editor-dependency-age.json", root), JSON.stringify({ checkedFor: "2026-10-03", minimumAgeDays: 14, cutoff: "2026-09-19T00:00:00Z", packages }, null, 2) + "\n");
console.log(`Verified ${packages.length} exact locked package releases.`);
