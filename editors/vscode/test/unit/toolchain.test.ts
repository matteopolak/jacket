import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdtempSync, promises as fs, readFileSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import * as path from "node:path";
import { describe, it } from "node:test";
import {
  type Fetch,
  RELEASE_ASSETS,
  INSTALLED_MARKER,
  ToolchainError,
  ToolchainStore,
  archiveRoot,
  assetFor,
  compareVersions,
  downloadUrl,
  install,
  parseSha256Sums,
  sha256,
  unsupportedMessage,
  verifyChecksum,
} from "../../src/toolchain";

const repository = path.join(__dirname, "..", "..", "..", "..", "..");

const offline: Fetch = async () => {
  throw new TypeError("fetch failed", { cause: { code: "ENOTFOUND" } });
};

describe("platform to release archive", () => {
  it("maps each released platform", () => {
    assert.equal(assetFor("darwin", "arm64"), "jacket-macos-arm64.tar.gz");
    assert.equal(assetFor("darwin", "x64"), "jacket-macos-x64.tar.gz");
    assert.equal(assetFor("linux", "x64"), "jacket-linux-x64.tar.gz");
    assert.equal(assetFor("linux", "arm64"), "jacket-linux-arm64.tar.gz");
    assert.equal(assetFor("win32", "x64"), "jacket-windows-x64.zip");
    assert.equal(assetFor("win32", "arm64"), "jacket-windows-arm64.zip");
  });

  it("has no archive for other systems", () => {
    assert.equal(assetFor("freebsd", "x64"), undefined);
    assert.equal(assetFor("linux", "ia32"), undefined);
    assert.equal(assetFor("win32", "ia32"), undefined);
    assert.match(unsupportedMessage("freebsd", "x64"), /no prebuilt Jai toolchain for freebsd x64/);
    assert.match(unsupportedMessage("darwin", "x64"), /jai\.server\.path/);
  });

  it("matches the archives release.yml builds", () => {
    const workflow = readFileSync(path.join(repository, ".github", "workflows", "release.yml"), "utf8");
    const matrixStart = workflow.indexOf("include:");
    assert.ok(matrixStart >= 0, "release.yml declares a platform matrix");
    const matrixEnd = workflow.indexOf("runs-on:", matrixStart);
    assert.ok(matrixEnd > matrixStart, "the build runner follows its platform matrix");
    const matrix = workflow.slice(matrixStart, matrixEnd);
    const built = [...matrix.matchAll(/- name: ([a-z0-9-]+)/g)].map((m) => m[1]);
    assert.ok(built.length >= 4, `found ${built.join(", ")}`);
    assert.match(workflow, /dir="jacket-\$\{\{ matrix\.name \}\}"/, "release.yml packages jacket-<platform>");
    for (const name of built) {
      const asset = `jacket-${name}.${name.startsWith("windows") ? "zip" : "tar.gz"}`;
      assert.ok(Object.values(RELEASE_ASSETS).includes(asset), `release.yml builds ${asset}; add it to RELEASE_ASSETS`);
    }
    for (const asset of Object.values(RELEASE_ASSETS)) {
      assert.ok(built.includes(archiveRoot(asset).replace(/^jacket-/, "")), `${asset} is not built by release.yml`);
    }
  });

  it("names the download URL by tag", () => {
    assert.equal(
      downloadUrl("0.4.0", "jacket-linux-x64.tar.gz"),
      "https://github.com/matteopolak/jacket/releases/download/v0.4.0/jacket-linux-x64.tar.gz",
    );
  });
});

describe("checksums", () => {
  it("parses sha256sum output", () => {
    const text = `${"a".repeat(64)}  jacket-linux-x64.tar.gz\n${"B".repeat(64)} *jacket-windows-x64.zip\nnot a line\n`;
    assert.deepEqual(parseSha256Sums(text), {
      "jacket-linux-x64.tar.gz": "a".repeat(64),
      "jacket-windows-x64.zip": "b".repeat(64),
    });
  });

  it("accepts the right data and rejects anything else", () => {
    const data = new TextEncoder().encode("archive bytes");
    const good = sha256(data);
    assert.equal(verifyChecksum("x.tar.gz", data, good.toUpperCase()), good);
    assert.throws(() => verifyChecksum("x.tar.gz", data, "0".repeat(64)), (e: ToolchainError) => e.kind === "integrity");
    assert.throws(() => verifyChecksum("x.tar.gz", data, undefined), (e: ToolchainError) => e.kind === "integrity");
  });
});

describe("versions", () => {
  it("orders semantic versions", () => {
    assert.ok(compareVersions("0.10.0", "0.9.9") > 0);
    assert.ok(compareVersions("1.0.0", "1.0.0") === 0);
    assert.deepEqual(["0.2.0", "0.10.1", "0.3.0"].toSorted(compareVersions), ["0.2.0", "0.3.0", "0.10.1"]);
  });

  it("does not reuse a cached toolchain when the pinned archive checksum changed", async () => {
    const storage = await fs.mkdtemp(path.join(tmpdir(), "jai-toolchain-pin-"));
    const store = new ToolchainStore(storage, "linux");
    const asset = assetFor("linux", "x64")!;
    const bin = store.binDir("0.17.0", asset);
    await fs.mkdir(bin, { recursive: true });
    await fs.writeFile(store.toolPath("0.17.0", asset, "jailsp"), "old binary");
    await fs.writeFile(path.join(store.versionDir("0.17.0"), INSTALLED_MARKER), JSON.stringify({ version: "0.17.0", asset, sha256: "a".repeat(64) }));
    assert.ok(store.isInstalled("0.17.0", asset));
    assert.ok(!store.isInstalled("0.17.0", asset, "b".repeat(64)));
    assert.ok(store.isInstalled("0.17.0", asset, "a".repeat(64)));
    await fs.rm(storage, { recursive: true, force: true });
  });
});

// A fake release: archives are directories written by `unpack`, so no tar is needed.
function fakeRelease(files: Record<string, Uint8Array | string>) {
  const requests: string[] = [];
  const fetcher: Fetch = async (url) => {
    requests.push(url);
    const name = url.slice(url.lastIndexOf("/") + 1);
    const body = files[name];
    if (body === undefined) return new Response("Not Found", { status: 404, statusText: "Not Found" });
    const bytes = typeof body === "string" ? new TextEncoder().encode(body) : body;
    return new Response(bytes as unknown as BodyInit, { status: 200, headers: { "content-length": String(bytes.length) } });
  };
  return { fetcher, requests };
}

function fakeUnpack(asset: string, tools = ["jaic", "jailsp", "jailint", "jaifmt"]) {
  return async (_archive: string, into: string) => {
    const dir = path.join(into, archiveRoot(asset));
    await fs.mkdir(path.join(dir, "stdlib", "prelude"), { recursive: true });
    await fs.writeFile(path.join(dir, "stdlib", "prelude", "module.jai"), "");
    for (const tool of tools) await fs.writeFile(path.join(dir, tool), "#!/bin/sh\n");
  };
}

describe("install", () => {
  const asset = "jacket-linux-x64.tar.gz";
  const archive = new TextEncoder().encode("pretend this is a tarball");
  const pinned = { version: "0.4.0", sha256: { [asset]: sha256(archive) } };

  it("downloads, verifies, unpacks into a versioned folder and removes older versions", async () => {
    const storage = mkdtempSync(path.join(tmpdir(), "jai-toolchain-"));
    const store = new ToolchainStore(storage, "linux");
    await fs.mkdir(path.join(store.versionDir("0.3.0")), { recursive: true });
    await fs.writeFile(path.join(store.versionDir("0.3.0"), "installed.json"), "{}");
    await fs.mkdir(path.join(store.root, ".staging-0.4.0-dead"), { recursive: true });
    const { fetcher, requests } = fakeRelease({ [asset]: archive });
    const progress: number[] = [];
    const dir = await install({
      version: "0.4.0",
      asset,
      store,
      fetcher,
      pinned,
      log: () => {},
      progress: (received) => progress.push(received),
      unpack: fakeUnpack(asset),
    });
    assert.equal(dir, path.join(storage, "toolchain", "0.4.0", "jacket-linux-x64"));
    assert.ok(store.isInstalled("0.4.0", asset));
    assert.ok(existsSync(path.join(dir, "stdlib", "prelude", "module.jai")), "the stdlib stays next to the binaries");
    assert.equal(statSync(path.join(dir, "jailsp")).mode & 0o111, 0o111, "executable bits are set");
    assert.deepEqual(await fs.readdir(store.root), ["0.4.0"], "older versions and staging leftovers are removed");
    assert.deepEqual(requests, [downloadUrl("0.4.0", asset)], "pinned checksums need no SHA256SUMS request");
    assert.equal(progress.at(-1), archive.length);
    assert.deepEqual(await store.installedVersions(), ["0.4.0"]);
  });

  it("rejects an archive that does not match the pinned checksum, leaving nothing behind", async () => {
    const storage = mkdtempSync(path.join(tmpdir(), "jai-toolchain-"));
    const store = new ToolchainStore(storage, "linux");
    const { fetcher } = fakeRelease({ [asset]: new TextEncoder().encode("tampered") });
    await assert.rejects(
      install({ version: "0.4.0", asset, store, fetcher, pinned, log: () => {}, unpack: fakeUnpack(asset) }),
      (e: ToolchainError) => e.kind === "integrity",
    );
    assert.ok(!store.isInstalled("0.4.0", asset));
    assert.deepEqual(existsSync(store.root) ? await fs.readdir(store.root) : [], []);
  });

  it("uses the release's SHA256SUMS when no checksums are pinned for the version", async () => {
    const storage = mkdtempSync(path.join(tmpdir(), "jai-toolchain-"));
    const store = new ToolchainStore(storage, "linux");
    const sums = `${sha256(archive)}  ${asset}\n`;
    const { fetcher, requests } = fakeRelease({ [asset]: archive, SHA256SUMS: sums });
    const logged: string[] = [];
    await install({
      version: "0.5.0",
      asset,
      store,
      fetcher,
      pinned, // pins are for 0.4.0
      log: (line) => logged.push(line),
      unpack: fakeUnpack(asset),
    });
    assert.ok(requests[0].endsWith("/v0.5.0/SHA256SUMS"));
    assert.ok(logged.some((line) => line.includes("no checksums are pinned")));
    assert.ok(store.isInstalled("0.5.0", asset));
  });

  it("refuses an archive SHA256SUMS does not list", async () => {
    const store = new ToolchainStore(mkdtempSync(path.join(tmpdir(), "jai-toolchain-")), "linux");
    const { fetcher } = fakeRelease({ [asset]: archive, SHA256SUMS: "" });
    await assert.rejects(
      install({ version: "0.5.0", asset, store, fetcher, log: () => {}, unpack: fakeUnpack(asset) }),
      (e: ToolchainError) => e.kind === "integrity",
    );
  });

  it("reports a pinned version without an archive for the platform as unsupported", async () => {
    const store = new ToolchainStore(mkdtempSync(path.join(tmpdir(), "jai-toolchain-")), "linux");
    const { fetcher, requests } = fakeRelease({});
    await assert.rejects(
      install({ version: "0.4.0", asset: "jacket-windows-arm64.zip", store, fetcher, pinned, log: () => {} }),
      (e: ToolchainError) => e.kind === "unsupported",
    );
    assert.deepEqual(requests, []);
  });

  it("turns network failures into an offline error", async () => {
    const store = new ToolchainStore(mkdtempSync(path.join(tmpdir(), "jai-toolchain-")), "linux");
    await assert.rejects(
      install({ version: "0.4.0", asset, store, fetcher: offline, pinned, log: () => {} }),
      (e: ToolchainError) => e.kind === "offline" && /ENOTFOUND/.test(e.message),
    );
  });

  it("reports a missing release asset", async () => {
    const store = new ToolchainStore(mkdtempSync(path.join(tmpdir(), "jai-toolchain-")), "linux");
    const { fetcher } = fakeRelease({});
    await assert.rejects(
      install({ version: "0.4.0", asset, store, fetcher, pinned, log: () => {} }),
      (e: ToolchainError) => e.kind === "missing",
    );
  });

  it("rejects an archive without the expected executables", async () => {
    const store = new ToolchainStore(mkdtempSync(path.join(tmpdir(), "jai-toolchain-")), "linux");
    const { fetcher } = fakeRelease({ [asset]: archive });
    await assert.rejects(
      install({ version: "0.4.0", asset, store, fetcher, pinned, log: () => {}, unpack: fakeUnpack(asset, ["jaic"]) }),
      (e: ToolchainError) => e.kind === "extract",
    );
    assert.ok(!store.isInstalled("0.4.0", asset));
  });

  it("unpacks a real tar.gz with the system tar", { skip: process.platform === "win32" && "tar.gz is not a Windows archive" }, async () => {
    const work = mkdtempSync(path.join(tmpdir(), "jai-archive-"));
    await fakeUnpack(asset)("", work);
    execFileSync("tar", ["-czf", path.join(work, asset), "-C", work, archiveRoot(asset)]);
    const bytes = new Uint8Array(readFileSync(path.join(work, asset)));
    const store = new ToolchainStore(mkdtempSync(path.join(tmpdir(), "jai-toolchain-")), "linux");
    const { fetcher } = fakeRelease({ [asset]: bytes });
    const dir = await install({
      version: "0.4.0",
      asset,
      store,
      fetcher,
      pinned: { version: "0.4.0", sha256: { [asset]: sha256(bytes) } },
      log: () => {},
    });
    assert.ok(existsSync(path.join(dir, "stdlib", "prelude", "module.jai")));
    assert.ok(store.isInstalled("0.4.0", asset));
  });

  it("uses .exe names in Windows archives", async () => {
    const zip = "jacket-windows-x64.zip";
    const store = new ToolchainStore(mkdtempSync(path.join(tmpdir(), "jai-toolchain-")), "win32");
    const { fetcher } = fakeRelease({ [zip]: archive });
    const dir = await install({
      version: "0.4.0",
      asset: zip,
      store,
      fetcher,
      pinned: { version: "0.4.0", sha256: { [zip]: sha256(archive) } },
      log: () => {},
      unpack: fakeUnpack(zip, ["jaic.exe", "jailsp.exe", "jailint.exe", "jaifmt.exe"]),
    });
    assert.equal(store.toolPath("0.4.0", zip, "jailsp"), path.join(dir, "jailsp.exe"));
    assert.ok(store.isInstalled("0.4.0", zip));
  });
});
