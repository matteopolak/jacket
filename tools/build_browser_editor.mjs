import { build } from "esbuild";
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import path from "node:path";
const root = fileURLToPath(new URL("../", import.meta.url));
await build({ entryPoints: [path.join(root, "tools/browser-editor/editor-kit.mjs")], outfile: path.join(root, "web/scripting-runtime/editor.bundle.mjs"), bundle: true, format: "esm", platform: "browser", target: ["es2022"], minify: true, sourcemap: false, legalComments: "eof", metafile: true }).then(async result => {
  await writeFile(path.join(root, "editor-build.json"), JSON.stringify({ format: "esm", target: "es2022", inputs: Object.keys(result.metafile.inputs).sort(), output: "editor.bundle.mjs" }, null, 2) + "\n");
});
const lock = JSON.parse(await readFile(path.join(root, "package-lock.json"), "utf8"));
const notices = ["Jai playground editor — third-party notices\n"];
for (const name of Object.keys(lock.packages).filter(name => name.startsWith("node_modules/") && !name.includes("@esbuild/")).sort()) {
  const pkg = JSON.parse(await readFile(path.join(root, name, "package.json"), "utf8"));
  const license = await readFile(path.join(root, name, pkg.name === "esbuild" ? "LICENSE.md" : "LICENSE"), "utf8");
  notices.push(`${pkg.name} ${pkg.version}\n${license}\n`);
}
await writeFile(path.join(root, "web/scripting-runtime/THIRD-PARTY-NOTICES.txt"), notices.join("\n"));
