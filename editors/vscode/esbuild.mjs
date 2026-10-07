// Bundles the extension into dist/extension.js (vscode-languageclient included, so the .vsix
// ships no node_modules). `--tests` instead transpiles src/ and test/ to out/ file by file for
// the unit and integration tests. `--production` minifies; `--watch` rebuilds on change.
import * as esbuild from "esbuild";
import { readdirSync } from "node:fs";
import { join } from "node:path";

const production = process.argv.includes("--production");
const watch = process.argv.includes("--watch");

function files(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? files(join(dir, entry.name)) : entry.name.endsWith(".ts") ? [join(dir, entry.name)] : [],
  );
}

const options = process.argv.includes("--tests")
  ? {
      entryPoints: [...files("src"), ...files("test")],
      outdir: "out",
      outbase: ".",
      format: "cjs",
      platform: "node",
      target: "node20",
      sourcemap: "inline",
      logLevel: "warning",
    }
  : {
      entryPoints: ["src/extension.ts"],
      outfile: "dist/extension.js",
      bundle: true,
      external: ["vscode"],
      format: "cjs",
      platform: "node",
      target: "node20",
      minify: production,
      sourcemap: !production,
      logLevel: "warning",
    };

if (watch) {
  const context = await esbuild.context(options);
  await context.watch();
} else {
  await esbuild.build(options);
}
