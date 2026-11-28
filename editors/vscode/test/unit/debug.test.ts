import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { buildArguments, programPath } from "../../src/debugBuild";

describe("debug build", () => {
  it("builds at -O0 into the program, before the extra arguments", () => {
    assert.deepEqual(buildArguments("/p/main.jai", "/p/main", ["-O2"]), ["/p/main.jai", "-O0", "-exe", "/p/main", "-O2"]);
  });

  it("puts the program next to the file, with .exe on Windows", () => {
    assert.equal(programPath("/p/main.jai", "linux"), "/p/main");
    assert.equal(programPath("/p/main.jai", "win32"), "/p/main.exe");
  });
});
