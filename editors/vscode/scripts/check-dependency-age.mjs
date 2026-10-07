// Every package in package-lock.json (direct and transitive) must have been published on the
// npm registry at least 14 days ago, the same policy as Cargo.lock (docs/tools/dependency-policy.md).
// Fails closed: an entry that is not a plain registry tarball, or whose publish time the
// registry does not report, is an error.
//
//   node scripts/check-dependency-age.mjs [--days 14]
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const daysArg = process.argv.indexOf("--days");
const days = daysArg > 0 ? Number(process.argv[daysArg + 1]) : 14;
const cutoff = Date.now() - days * 24 * 60 * 60 * 1000;
const REGISTRY = "https://registry.npmjs.org/";

const lock = JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8"));
const wanted = new Map(); // name -> Set of versions
const errors = [];
for (const [path, entry] of Object.entries(lock.packages ?? {})) {
  if (path === "") continue;
  const name = entry.name ?? path.slice(path.lastIndexOf("node_modules/") + "node_modules/".length);
  if (!entry.resolved?.startsWith(REGISTRY) || !entry.integrity) {
    errors.push(`${path}: not a registry package with an integrity hash (${entry.resolved ?? "no resolved URL"})`);
    continue;
  }
  if (!wanted.has(name)) wanted.set(name, new Set());
  wanted.get(name).add(entry.version);
}

const names = [...wanted.keys()].sort();
let checked = 0;
async function check(name) {
  const response = await fetch(REGISTRY + name.replace("/", "%2F"), {
    headers: { accept: "application/json" },
  });
  if (!response.ok) {
    errors.push(`${name}: registry answered ${response.status}`);
    return;
  }
  const meta = await response.json();
  for (const version of wanted.get(name)) {
    checked += 1;
    const published = Date.parse(meta.time?.[version] ?? "");
    if (Number.isNaN(published)) errors.push(`${name}@${version}: no publish time`);
    else if (published > cutoff)
      errors.push(`${name}@${version}: published ${new Date(published).toISOString()}, under ${days} days ago`);
    if (!meta.versions?.[version]) errors.push(`${name}@${version}: not on the registry (unpublished?)`);
  }
}
const queue = [...names];
await Promise.all(
  Array.from({ length: 8 }, async () => {
    while (queue.length) await check(queue.shift());
  }),
);
if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log(`npm dependency age check passed (${checked} packages, at least ${days} days old)`);
