import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { acceptedKey, parseVersionOutput, staleMessage, staleToolchain, type StaleCheck } from "../../src/pathVersion";

function check(version: string | undefined, options: Partial<StaleCheck> = {}): StaleCheck {
  return {
    found: { path: "/opt/homebrew/bin/jailsp", source: "path" },
    expected: "0.25.0",
    accepted: [],
    preferDownloaded: false,
    probe: async () => version,
    ...options,
  };
}

describe("stale PATH toolchain", () => {
  it("parses version output", () => {
    assert.equal(parseVersionOutput("jailsp 0.23.0\n"), "0.23.0");
    assert.equal(parseVersionOutput("jaic 0.26.0-rc.1 (abc)"), "0.26.0-rc.1");
    assert.equal(parseVersionOutput("garbage"), undefined);
  });

  it("warns for an older PATH toolchain, naming path, versions and precedence", async () => {
    const stale = await staleToolchain(check("0.23.0"));
    assert.deepEqual(stale, { path: "/opt/homebrew/bin/jailsp", version: "0.23.0", expected: "0.25.0" });
    const message = staleMessage(stale!);
    for (const part of ["/opt/homebrew/bin/jailsp", "0.23.0", "0.25.0", "PATH takes precedence"]) {
      assert.ok(message.includes(part), part);
    }
  });

  it("does not warn for equal or newer versions or an unreadable one", async () => {
    assert.equal(await staleToolchain(check("0.25.0")), undefined);
    assert.equal(await staleToolchain(check("0.26.0")), undefined);
    assert.equal(await staleToolchain(check(undefined)), undefined);
  });

  it("does not warn for other sources or once the download is preferred", async () => {
    assert.equal(await staleToolchain(check("0.23.0", { found: { path: "/x/jailsp", source: "setting" } })), undefined);
    assert.equal(await staleToolchain(check("0.23.0", { preferDownloaded: true })), undefined);
  });

  it("stays quiet for an accepted path and version, warns again for a newer one", async () => {
    const accepted = [acceptedKey("/opt/homebrew/bin/jailsp", "0.23.0")];
    assert.equal(await staleToolchain(check("0.23.0", { accepted })), undefined);
    assert.equal((await staleToolchain(check("0.24.0", { accepted })))?.version, "0.24.0");
  });
});
