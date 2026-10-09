// Every package in pnpm-lock.yaml (direct and transitive), and the pnpm release named by
// package.json's packageManager, must have been published on the npm registry at least 14 days
// ago, the same policy as Cargo.lock (docs/tools/dependency-policy.md). Runs before any install,
// so it reads the lockfile with a few regular expressions instead of a YAML library.
// Fails closed: an entry that is not a plain registry package with an integrity hash, or whose
// publish time the registry does not report, is an error.
//
//   node scripts/check-dependency-age.ts [--days 14]
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const daysArg = process.argv.indexOf("--days");
const days = daysArg > 0 ? Number(process.argv[daysArg + 1]) : 14;
const cutoff = Date.now() - days * 24 * 60 * 60 * 1000;
const REGISTRY = "https://registry.npmjs.org/";

const wanted = new Map(); // name -> Set of versions
const errors = [];
function want(name: string, version: string) {
  if (!wanted.has(name)) wanted.set(name, new Set());
  wanted.get(name).add(version);
}

const lock = readFileSync(join(root, "pnpm-lock.yaml"), "utf8");
if (!/^lockfileVersion: '9\.\d+'$/m.test(lock)) errors.push("pnpm-lock.yaml: expected lockfileVersion 9");
// The `packages:` section lists each resolved package once: a two-space-indented
// `name@version:` key (quoted when it starts with @), then its fields four spaces in.
const section = lock.split(/^packages:$/m)[1]?.split(/^\S/m)[0] ?? "";
const entries = section.split(/^(?=  \S)/m).filter((entry) => entry.trim());
for (const entry of entries) {
  const key = /^  '?((@[^/@\s]+\/)?[^@\s']+)@([^'(\s]+)'?:\s*$/m.exec(entry);
  if (!key) {
    errors.push(`pnpm-lock.yaml: cannot read the package entry ${JSON.stringify(entry.split("\n")[0])}`);
    continue;
  }
  const [, name, , version] = key;
  // A registry package is `resolution: {integrity: sha512-...}` and nothing else; tarball URLs,
  // git commits and local directories add other fields.
  if (!/^    resolution: \{integrity: sha512-[A-Za-z0-9+/]+={0,2}\}$/m.test(entry)) {
    errors.push(`${name}@${version}: not a registry package with an integrity hash`);
    continue;
  }
  want(name, version);
}
if (!wanted.size) errors.push("pnpm-lock.yaml: no packages found");

const packageManager = JSON.parse(readFileSync(join(root, "package.json"), "utf8")).packageManager;
const pnpm = /^pnpm@(\d+\.\d+\.\d+)$/.exec(packageManager ?? "");
if (pnpm) want("pnpm", pnpm[1]);
else errors.push(`package.json: packageManager should be pnpm@<exact version>, not ${packageManager}`);

const names = [...wanted.keys()].toSorted();
let checked = 0;
async function check(name: string) {
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
