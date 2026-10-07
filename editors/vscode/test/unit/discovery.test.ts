import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { type Environment, MissingConfiguredTool, expandPath, findCompiler, findFormatter, findServer, onPath } from "../../src/discovery";

function machine(files: string[], options: Partial<Environment> = {}): Environment {
  const set = new Set(files);
  return {
    platform: "linux",
    pathVariable: "/usr/bin:/opt/jai/bin",
    home: "/home/me",
    isFile: (file) => set.has(file),
    ...options,
  };
}

describe("binary discovery", () => {
  it("prefers jai.server.path over PATH", () => {
    const env = machine(["/custom/jailsp", "/usr/bin/jailsp"]);
    assert.deepEqual(findServer({ serverPath: "/custom/jailsp" }, env), { path: "/custom/jailsp", source: "setting" });
  });

  it("reports a configured path that does not exist instead of falling back", () => {
    const env = machine(["/usr/bin/jailsp"]);
    assert.throws(() => findServer({ serverPath: "/missing/jailsp" }, env), MissingConfiguredTool);
  });

  it("finds jailsp on PATH", () => {
    const env = machine(["/opt/jai/bin/jailsp"]);
    assert.deepEqual(findServer({}, env), { path: "/opt/jai/bin/jailsp", source: "path" });
  });

  it("finds jailsp next to jai.compiler.path", () => {
    const env = machine(["/opt/jaic-0.3/jaic", "/opt/jaic-0.3/jailsp"], { pathVariable: "/usr/bin" });
    assert.deepEqual(findServer({ compilerPath: "/opt/jaic-0.3/jaic" }, env), {
      path: "/opt/jaic-0.3/jailsp",
      source: "next to jaic",
    });
  });

  it("finds jailsp next to the jaic on PATH", () => {
    // Only jaic is linked into the bin folder... then jailsp must be beside it to count.
    const env = machine(["/opt/jai/bin/jaic", "/opt/jai/bin/jailsp"], { pathVariable: "/opt/jai/bin" });
    assert.equal(findServer({}, env)?.source, "path");
    const unlinked = machine(["/usr/local/jai/jaic", "/usr/local/jai/jailsp"], { pathVariable: "/usr/local/jai-bin" });
    assert.equal(findServer({ compilerPath: "/usr/local/jai/jaic" }, unlinked)?.source, "next to jaic");
  });

  it("falls back to the downloaded toolchain last", () => {
    const env = machine(["/storage/toolchain/0.3.0/jaic-linux-x64/jailsp"]);
    assert.deepEqual(findServer({}, env, "/storage/toolchain/0.3.0/jaic-linux-x64"), {
      path: "/storage/toolchain/0.3.0/jaic-linux-x64/jailsp",
      source: "toolchain",
    });
    const both = machine(["/usr/bin/jailsp", "/storage/t/jailsp"]);
    assert.equal(findServer({}, both, "/storage/t")?.source, "path");
  });

  it("returns undefined when nothing is installed", () => {
    assert.equal(findServer({}, machine([])), undefined);
    assert.equal(findCompiler({}, machine([])), undefined);
    assert.equal(findFormatter({}, machine([])), undefined);
  });

  it("finds jaifmt next to jaic or jailsp", () => {
    const env = machine(["/a/jaic", "/a/jaifmt"], { pathVariable: "" });
    assert.deepEqual(findFormatter({ compilerPath: "/a/jaic" }, env), { path: "/a/jaifmt", source: "next to jaic" });
    const server = machine(["/b/jailsp", "/b/jaifmt"], { pathVariable: "" });
    assert.deepEqual(findFormatter({ serverPath: "/b/jailsp" }, server), { path: "/b/jaifmt", source: "next to jailsp" });
  });

  it("uses .exe names and PATHEXT on Windows", () => {
    const env = machine(["C:\\jai\\jailsp.exe", "C:\\tools\\jaic.EXE", "C:\\tools\\jailsp.exe"], {
      platform: "win32",
      pathVariable: "C:\\nothing;C:\\jai",
      pathExt: ".COM;.EXE",
    });
    assert.equal(onPath("jailsp", env), "C:\\jai\\jailsp.exe");
    assert.deepEqual(findServer({ compilerPath: "C:\\tools\\jaic.EXE" }, { ...env, pathVariable: "" }), {
      path: "C:\\tools\\jailsp.exe",
      source: "next to jaic",
    });
  });

  it("expands ~ and ${env:NAME}", () => {
    const env = machine([]);
    assert.equal(expandPath("~/jai/jailsp", env), "/home/me/jai/jailsp");
    assert.equal(expandPath("${env:JAI_HOME}/jailsp", env, { JAI_HOME: "/opt/jai" }), "/opt/jai/jailsp");
  });
});
