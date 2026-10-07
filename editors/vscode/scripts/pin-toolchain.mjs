// Pins the toolchain this build of the extension downloads: sets package.json's version and
// writes toolchain-checksums.json with the SHA-256 of each release archive, so a release asset
// that changes after packaging is rejected instead of trusted. The release workflow runs it on
// the archives it just built; run it by hand after a release to update the committed pins.
//
//   node scripts/pin-toolchain.mjs --version 0.4.0 --sums path/to/SHA256SUMS
//   node scripts/pin-toolchain.mjs --version 0.4.0 --archives dist/   # hash jaic-* archives
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const arg = (name) => {
  const index = process.argv.indexOf(name);
  return index > 0 ? process.argv[index + 1] : undefined;
};
const version = arg("--version")?.replace(/^v/, "");
if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("usage: pin-toolchain.mjs --version X.Y.Z (--sums SHA256SUMS | --archives DIR)");
  process.exit(2);
}

const sha256 = {};
const isArchive = (name) => /^jaic-[a-z0-9-]+\.(?:tar\.gz|zip)$/.test(name);
if (arg("--sums")) {
  for (const line of readFileSync(arg("--sums"), "utf8").split(/\r?\n/)) {
    const match = /^([0-9a-f]{64})\s+\*?(\S+)$/i.exec(line.trim());
    if (match && isArchive(match[2])) sha256[match[2]] = match[1].toLowerCase();
  }
} else if (arg("--archives")) {
  for (const name of readdirSync(arg("--archives")).toSorted()) {
    if (!isArchive(name)) continue;
    sha256[name] = createHash("sha256").update(readFileSync(join(arg("--archives"), name))).digest("hex");
  }
} else {
  console.error("pass --sums or --archives");
  process.exit(2);
}
if (Object.keys(sha256).length === 0) {
  console.error("no jaic-* archives found to pin");
  process.exit(1);
}

const packagePath = join(root, "package.json");
const manifest = JSON.parse(readFileSync(packagePath, "utf8"));
manifest.version = version;
writeFileSync(packagePath, JSON.stringify(manifest, null, 2) + "\n");
writeFileSync(join(root, "toolchain-checksums.json"), JSON.stringify({ version, sha256 }, null, 2) + "\n");
console.log(`pinned jaic ${version}: ${Object.keys(sha256).join(", ")}`);
