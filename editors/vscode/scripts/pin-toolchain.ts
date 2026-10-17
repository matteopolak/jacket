// Pins the toolchain this build of the extension downloads and independently selects the
// extension's marketplace version. The checksums remain tied to the toolchain release.
//
//   node scripts/pin-toolchain.ts --toolchain-version 0.17.0 --extension-version 0.17.1 --sums path/to/SHA256SUMS
//   node scripts/pin-toolchain.ts --toolchain-version 0.17.0 --extension-version 0.17.1 --archives dist/
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const arg = (name: string) => {
  const index = process.argv.indexOf(name);
  return index > 0 ? process.argv[index + 1] : undefined;
};
const toolchainVersion = (arg("--toolchain-version") ?? arg("--version"))?.replace(/^v/, "");
const extensionVersion = (arg("--extension-version") ?? toolchainVersion)?.replace(/^v/, "");
const validVersion = (version: string | undefined) =>
  Boolean(version && /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version));
if (!validVersion(toolchainVersion) || !validVersion(extensionVersion)) {
  console.error("usage: pin-toolchain.ts --toolchain-version X.Y.Z [--extension-version X.Y.Z] (--sums SHA256SUMS | --archives DIR)");
  process.exit(2);
}

const sha256: Record<string, string> = {};
const isArchive = (name: string) => /^jacket-(?!vscode-)[a-z0-9-]+\.(?:tar\.gz|zip)$/.test(name);
const sums = arg("--sums");
const archives = arg("--archives");
if (sums) {
  for (const line of readFileSync(sums, "utf8").split(/\r?\n/)) {
    const match = /^([0-9a-f]{64})\s+\*?(\S+)$/i.exec(line.trim());
    if (match?.[1] && match[2] && isArchive(match[2])) sha256[match[2]] = match[1].toLowerCase();
  }
} else if (archives) {
  for (const name of readdirSync(archives).toSorted()) {
    if (!isArchive(name)) continue;
    sha256[name] = createHash("sha256").update(readFileSync(join(archives, name))).digest("hex");
  }
} else {
  console.error("pass --sums or --archives");
  process.exit(2);
}
if (Object.keys(sha256).length === 0) {
  console.error("no jacket-* archives found to pin");
  process.exit(1);
}

const packagePath = join(root, "package.json");
const manifest = JSON.parse(readFileSync(packagePath, "utf8"));
manifest.version = extensionVersion;
writeFileSync(packagePath, JSON.stringify(manifest, null, 2) + "\n");
writeFileSync(join(root, "toolchain-checksums.json"), JSON.stringify({ version: toolchainVersion, sha256 }, null, 2) + "\n");
console.log(`packaged extension ${extensionVersion} pinned to jaic ${toolchainVersion}: ${Object.keys(sha256).join(", ")}`);
