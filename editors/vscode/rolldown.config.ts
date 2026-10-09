// Bundles the extension into dist/extension.cjs (vscode-languageclient included, so the .vsix
// ships no node_modules). With `--environment TESTS` it instead transpiles src/ and test/ to
// out/ file by file (nothing bundled) for the unit, integration and screenshot runs.
// `--environment PRODUCTION` minifies; `--watch` rebuilds on change.
//
//   rolldown -c [--environment PRODUCTION|TESTS] [--watch]
import { readdirSync } from "node:fs";
import { join } from "node:path";
import { defineConfig, type RolldownOptions } from "rolldown";

function files(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
    entry.isDirectory() ? files(join(dir, entry.name)) : entry.name.endsWith(".ts") ? [join(dir, entry.name)] : [],
  );
}

const common: RolldownOptions = {
  platform: "node",
  transform: { target: "node20" },
  logLevel: "warn",
};

const options: RolldownOptions = process.env.TESTS
  ? {
      ...common,
      input: [...files("src"), ...files("test")],
      // Every package import stays a require() of the installed module.
      external: /^[^./]/,
      output: {
        dir: "out",
        format: "cjs",
        entryFileNames: "[name].cjs",
        preserveModules: true,
        sourcemap: "inline",
      },
    }
  : {
      ...common,
      input: "src/extension.ts",
      external: ["vscode"],
      output: {
        file: "dist/extension.cjs",
        format: "cjs",
        minify: Boolean(process.env.PRODUCTION),
        sourcemap: !process.env.PRODUCTION,
      },
    };

export default defineConfig(options);
