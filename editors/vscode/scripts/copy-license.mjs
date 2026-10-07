// vsce wants a LICENSE next to package.json; the extension is under the repository's license.
import { copyFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
copyFileSync(join(root, "..", "..", "LICENSE"), join(root, "LICENSE"));
