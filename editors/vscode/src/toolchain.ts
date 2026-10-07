// The downloadable toolchain: which release archive fits this machine, verifying it, and the
// versioned folders it is installed into. Nothing here imports `vscode`, so the unit tests run
// it under plain node with a fake `fetch`.
import { createHash, randomBytes } from "node:crypto";
import { execFile } from "node:child_process";
import { existsSync, promises as fs } from "node:fs";
import * as path from "node:path";

export const REPOSITORY = "matteopolak/jai";
export const RELEASES_URL = `https://github.com/${REPOSITORY}/releases`;
export const TOOLS = ["jaic", "jailsp", "jailint", "jaifmt"] as const;
export type Tool = (typeof TOOLS)[number];

/**
 * The release archive per `${process.platform}-${process.arch}`, without its `jai-`/`jaic-`
 * prefix (see `archivePrefix`). A platform without a row has no prebuilt toolchain: add a row
 * here when release.yml starts building it. Releases up to 0.4.1 have no `macos-x64` or
 * `linux-arm64` archive; their pinned checksums then lack the asset (an "unsupported" error).
 */
export const RELEASE_ASSETS: Readonly<Record<string, string>> = {
  "darwin-arm64": "macos-arm64.tar.gz",
  "darwin-x64": "macos-x64.tar.gz",
  "linux-x64": "linux-x64.tar.gz",
  "linux-arm64": "linux-arm64.tar.gz",
  "win32-x64": "windows-x64.zip",
  "win32-arm64": "windows-arm64.zip",
};

/** The last release whose archives are named `jaic-<platform>`; later ones are `jai-<platform>`. */
export const LAST_JAIC_NAMED_RELEASE = "0.4.0";

/** The archive name prefix of a release: `jaic` up to 0.4.0, `jai` from 0.4.1 on. */
export function archivePrefix(version: string): "jai" | "jaic" {
  const core = /^\d+\.\d+\.\d+/.exec(version)?.[0] ?? version;
  return compareVersions(core, LAST_JAIC_NAMED_RELEASE) <= 0 ? "jaic" : "jai";
}

/** The release archive of `version` for a platform, or `undefined` when no release is built for it. */
export function assetFor(platform: NodeJS.Platform, arch: string, version: string): string | undefined {
  const asset = RELEASE_ASSETS[`${platform}-${arch}`];
  return asset && `${archivePrefix(version)}-${asset}`;
}

/** Why there is no download for a platform, and what to do instead. */
export function unsupportedMessage(platform: NodeJS.Platform, arch: string): string {
  return (
    `There is no prebuilt Jai toolchain for ${platformName(platform, arch)}. Build jaic, jailsp and jailint from source ` +
    `(https://github.com/${REPOSITORY}#install) and put them on PATH, or set jai.server.path and jai.compiler.path.`
  );
}

/** A human name for the platform, for messages. */
export function platformName(platform: NodeJS.Platform, arch: string): string {
  const os = { darwin: "macOS", linux: "Linux", win32: "Windows" }[platform as string] ?? platform;
  return `${os} ${arch}`;
}

/** The folder the archive unpacks to: `jai-linux-x64.tar.gz` holds `jai-linux-x64/`. */
export function archiveRoot(asset: string): string {
  return asset.replace(/\.(tar\.gz|zip)$/, "");
}

export function executableName(tool: Tool, platform: NodeJS.Platform): string {
  return platform === "win32" ? `${tool}.exe` : tool;
}

export function downloadUrl(version: string, asset: string): string {
  return `https://github.com/${REPOSITORY}/releases/download/v${version}/${asset}`;
}

/** Checksums written into the extension when it is packaged (`toolchain-checksums.json`). */
export interface PinnedChecksums {
  version: string;
  sha256: Record<string, string>;
}

/** Parses `sha256sum` output: `<hex>  <name>` (or `<hex> *<name>`) per line. */
export function parseSha256Sums(text: string): Record<string, string> {
  const sums: Record<string, string> = {};
  for (const line of text.split(/\r?\n/)) {
    const match = /^([0-9a-fA-F]{64})\s+\*?(.+?)\s*$/.exec(line);
    if (match) sums[match[2]] = match[1].toLowerCase();
  }
  return sums;
}

export function sha256(data: Uint8Array): string {
  return createHash("sha256").update(data).digest("hex");
}

export class ToolchainError extends Error {
  constructor(
    message: string,
    /** `offline`: the network failed; `missing`: no such release asset; `integrity`: bad checksum. */
    readonly kind: "offline" | "missing" | "integrity" | "unsupported" | "extract" | "cancelled",
  ) {
    super(message);
  }
}

/** Throws unless `data` has the expected checksum. */
export function verifyChecksum(asset: string, data: Uint8Array, expected: string | undefined): string {
  if (!expected) {
    throw new ToolchainError(`no SHA-256 checksum is known for ${asset}, so it was not installed`, "integrity");
  }
  const actual = sha256(data);
  if (actual !== expected.toLowerCase()) {
    throw new ToolchainError(
      `${asset} does not match its checksum (expected ${expected}, got ${actual}); it was not installed`,
      "integrity",
    );
  }
  return actual;
}

export type Fetch = (url: string, init?: { signal?: AbortSignal }) => Promise<Response>;

function describeNetworkError(error: unknown): string {
  const cause = (error as { cause?: { code?: string; message?: string } })?.cause;
  return cause?.code ?? cause?.message ?? (error as Error)?.message ?? String(error);
}

async function get(fetcher: Fetch, url: string, signal?: AbortSignal): Promise<Response> {
  let response: Response;
  try {
    response = await fetcher(url, { signal });
  } catch (error) {
    if (signal?.aborted) throw new ToolchainError("the download was cancelled", "cancelled");
    throw new ToolchainError(
      `could not reach github.com (${describeNetworkError(error)}); check the connection, or install the toolchain by hand from ${RELEASES_URL}`,
      "offline",
    );
  }
  if (response.status === 404) {
    throw new ToolchainError(`${url} does not exist`, "missing");
  }
  if (!response.ok) {
    throw new ToolchainError(`${url} answered ${response.status} ${response.statusText}`, "offline");
  }
  return response;
}

/** Downloads `url` into memory, reporting `(received, total)` as it goes. */
export async function download(
  fetcher: Fetch,
  url: string,
  progress?: (received: number, total: number | undefined) => void,
  signal?: AbortSignal,
): Promise<Uint8Array> {
  const response = await get(fetcher, url, signal);
  const total = Number(response.headers.get("content-length")) || undefined;
  if (!response.body) return new Uint8Array(await response.arrayBuffer());
  const chunks: Uint8Array[] = [];
  let received = 0;
  const reader = response.body.getReader();
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      chunks.push(value);
      received += value.length;
      progress?.(received, total);
    }
  } catch (error) {
    if (signal?.aborted) throw new ToolchainError("the download was cancelled", "cancelled");
    throw new ToolchainError(`the download of ${url} failed (${describeNetworkError(error)})`, "offline");
  }
  const data = new Uint8Array(received);
  let offset = 0;
  for (const chunk of chunks) {
    data.set(chunk, offset);
    offset += chunk.length;
  }
  return data;
}

/**
 * The checksum to hold `asset` to: the one pinned into the extension when it was packaged for
 * this version, else (a development build) the release's own `SHA256SUMS`.
 */
export async function expectedChecksum(
  fetcher: Fetch,
  version: string,
  asset: string,
  pinned: PinnedChecksums | undefined,
  log: (line: string) => void,
  signal?: AbortSignal,
): Promise<string | undefined> {
  if (pinned && pinned.version === version) {
    const sum = pinned.sha256[asset];
    if (!sum) {
      throw new ToolchainError(`jaic ${version} has no ${asset} release archive`, "unsupported");
    }
    return sum;
  }
  log(`no checksums are pinned for ${version} in this build of the extension; using the release's SHA256SUMS`);
  const text = new TextDecoder().decode(await download(fetcher, downloadUrl(version, "SHA256SUMS"), undefined, signal));
  return parseSha256Sums(text)[asset];
}

/** The folders under the extension's storage: `<storage>/toolchain/<version>/<archive root>/`. */
export class ToolchainStore {
  constructor(
    readonly storage: string,
    readonly platform: NodeJS.Platform = process.platform,
  ) {}

  get root(): string {
    return path.join(this.storage, "toolchain");
  }

  versionDir(version: string): string {
    return path.join(this.root, version);
  }

  /** Where `tool` lives in an installed version, if that version is installed. */
  binDir(version: string, asset: string): string {
    return path.join(this.versionDir(version), archiveRoot(asset));
  }

  isInstalled(version: string, asset: string): boolean {
    return existsSync(path.join(this.versionDir(version), INSTALLED_MARKER)) && existsSync(this.toolPath(version, asset, "jailsp"));
  }

  toolPath(version: string, asset: string, tool: Tool): string {
    return path.join(this.binDir(version, asset), executableName(tool, this.platform));
  }

  /** Installed versions, newest first (by semantic version). */
  async installedVersions(): Promise<string[]> {
    let entries: string[];
    try {
      entries = await fs.readdir(this.root);
    } catch {
      return [];
    }
    const versions = entries.filter((e) => /^\d+\.\d+\.\d+/.test(e) && existsSync(path.join(this.versionDir(e), INSTALLED_MARKER)));
    return versions.toSorted((a, b) => compareVersions(b, a));
  }

  /** Removes every version but `keep`, and leftovers of interrupted downloads. */
  async removeAllBut(keep: string): Promise<string[]> {
    let entries: string[];
    try {
      entries = await fs.readdir(this.root);
    } catch {
      return [];
    }
    const removed: string[] = [];
    for (const entry of entries) {
      if (entry === keep) continue;
      await fs.rm(path.join(this.root, entry), { recursive: true, force: true });
      removed.push(entry);
    }
    return removed;
  }

  /** A fresh staging folder in the same file system, so the final move is a rename. */
  async stagingDir(version: string): Promise<string> {
    await fs.mkdir(this.root, { recursive: true });
    const dir = path.join(this.root, `.staging-${version}-${randomBytes(4).toString("hex")}`);
    await fs.mkdir(dir);
    return dir;
  }
}

export const INSTALLED_MARKER = "installed.json";

function versionParts(v: string): (number | string)[] {
  return v.split(/[.-]/).map((p) => (/^\d+$/.test(p) ? Number(p) : p));
}

export function compareVersions(a: string, b: string): number {
  const x = versionParts(a);
  const y = versionParts(b);
  for (let i = 0; i < Math.max(x.length, y.length); i++) {
    if (x[i] === y[i]) continue;
    if (x[i] === undefined) return -1;
    if (y[i] === undefined) return 1;
    if (typeof x[i] === "number" && typeof y[i] === "number") return (x[i] as number) - (y[i] as number);
    return String(x[i]) < String(y[i]) ? -1 : 1;
  }
  return 0;
}

/** Unpacks an archive with the system `tar` (bsdtar reads zip files too; Windows 10+ ships it). */
export function extract(archive: string, into: string, platform: NodeJS.Platform = process.platform): Promise<void> {
  const tar =
    platform === "win32" ? path.join(process.env.SystemRoot ?? "C:\\Windows", "System32", "tar.exe") : "tar";
  const args = archive.endsWith(".zip") ? ["-xf", archive, "-C", into] : ["-xzf", archive, "-C", into];
  return new Promise((resolve, reject) => {
    execFile(tar, args, { windowsHide: true }, (error, _stdout, stderr) => {
      if (error) reject(new ToolchainError(`could not unpack ${path.basename(archive)}: ${stderr || error.message}`, "extract"));
      else resolve();
    });
  });
}

export interface InstallOptions {
  version: string;
  asset: string;
  store: ToolchainStore;
  fetcher: Fetch;
  pinned?: PinnedChecksums;
  log: (line: string) => void;
  progress?: (received: number, total: number | undefined) => void;
  signal?: AbortSignal;
  /** Replaceable in tests. */
  unpack?: (archive: string, into: string) => Promise<void>;
}

/**
 * Downloads, verifies and unpacks the toolchain into `<storage>/toolchain/<version>`, then
 * removes older versions. Returns the folder holding the executables.
 */
export async function install(options: InstallOptions): Promise<string> {
  const { version, asset, store, fetcher, log, signal } = options;
  const expected = await expectedChecksum(fetcher, version, asset, options.pinned, log, signal);
  if (!expected) {
    throw new ToolchainError(`the release's SHA256SUMS does not list ${asset}`, "integrity");
  }
  const url = downloadUrl(version, asset);
  log(`downloading ${url}`);
  const data = await download(fetcher, url, options.progress, signal);
  const actual = verifyChecksum(asset, data, expected);
  log(`${asset}: ${data.length} bytes, SHA-256 ${actual} (verified)`);

  const staging = await store.stagingDir(version);
  try {
    const archive = path.join(staging, asset);
    await fs.writeFile(archive, data);
    const unpacked = path.join(staging, version);
    await fs.mkdir(unpacked);
    await (options.unpack ?? ((a, i) => extract(a, i, store.platform)))(archive, unpacked);
    const bin = path.join(unpacked, archiveRoot(asset));
    for (const tool of TOOLS) {
      const file = path.join(bin, executableName(tool, store.platform));
      if (!existsSync(file)) {
        throw new ToolchainError(`${asset} has no ${path.basename(file)}`, "extract");
      }
      if (store.platform !== "win32") await fs.chmod(file, 0o755);
    }
    const marker = { version, asset, sha256: actual, installed: new Date().toISOString() };
    await fs.writeFile(path.join(unpacked, INSTALLED_MARKER), JSON.stringify(marker, null, 2));
    const target = store.versionDir(version);
    await fs.rm(target, { recursive: true, force: true });
    await fs.rename(unpacked, target);
  } finally {
    await fs.rm(staging, { recursive: true, force: true });
  }
  const removed = await store.removeAllBut(version);
  if (removed.length) log(`removed older toolchain folders: ${removed.join(", ")}`);
  return store.binDir(version, asset);
}
