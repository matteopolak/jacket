import assert from "node:assert/strict";
import { it } from "node:test";
import { toolEnvironment } from "../../src/toolEnvironment";

it("maps stdlib settings, expands home and preserves inherited variables", () => {
  const settings: Record<string, string> = { "stdlib.path": " ~/lib ", "stdlib.prelude": "~/Base.jai" };
  assert.deepEqual(toolEnvironment((key) => settings[key], { PATH: "/bin", JAIC_PRELUDE: "old" }, "/home/me"), {
    PATH: "/bin", JAIC_STDLIB: "/home/me/lib", JAIC_PRELUDE: "/home/me/Base.jai",
  });
  assert.equal(toolEnvironment(() => "", { JAIC_PRELUDE: "inherited" }, "/home/me").JAIC_PRELUDE, "inherited");
  assert.equal(toolEnvironment((key) => key === "stdlib.prelude" ? "Base.jai" : undefined, {}, "/home/me").JAIC_PRELUDE, "Base.jai");
});
