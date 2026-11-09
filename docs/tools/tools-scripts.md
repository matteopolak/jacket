# Tools scripts

The Node scripts under `tools/` (WASI runners, browser and playground checks, the doc-site probes, the WebGPU host test) are TypeScript files that `node` runs directly. Nothing is built or shipped.

## Run the checks

```
just tools-scripts          # or: python3 tools/dev/dev.py tools-scripts
just tools-scripts --fix    # format the scripts
```

This runs `pnpm run lint` in `tools/` (installing first when `node_modules` is missing). It needs pnpm at the version `tools/package.json` names; without it the command fails and says so.

| Script | Tool | Checks |
| --- | --- | --- |
| `lint:js` | oxlint | correctness and suspicious rules, `--deny-warnings` |
| `lint:types` | tsc | `tsc --noEmit`, strict |
| `fmt:check` | oxfmt | formatting of `**/*.ts` (`fmt` rewrites) |

CI runs the same `pnpm run lint` in the `tools-scripts` job, only when a `.ts` file or the tooling under `tools/` changes.

## How to change it

- **Erasable syntax only.** Node strips types and does not transform them, so no `enum`, `namespace`, constructor parameter properties or `import x = require`. `tsconfig.json` enforces this (`erasableSyntaxOnly`, `verbatimModuleSyntax`).
- **Imports.** Relative imports name the `.ts` file; type-only imports use `import type`.
- **Types.** The shipped JavaScript (`crates/jai-wasm/js/*`, the doc engine) is read as plain JS and its shapes are described once in `tools/browser/play_types.ts`. Use `unknown` and narrow at JSON boundaries instead of `any`. `tools/types/roughjs.d.ts` declares the `roughjs` module.
- **Rules.** `oxlint.config.ts` and `oxfmt.config.ts` are typed `defineConfig` exports (single quotes, 100 columns). `unicorn/require-post-message-target-origin` is off: a worker's `parentPort.postMessage` has no target origin.
- **Tests.** `node --test tools/browser/test_*.ts tools/dev/test_*.ts tools/bindings/test_webgpu_host.ts`.

## Dependencies

`oxlint`, `oxfmt`, `typescript` and `@types/node` at exact versions in `package.json`, `minimumReleaseAge` (14 days) in `pnpm-workspace.yaml`, and `pnpm install --frozen-lockfile --ignore-scripts` in CI, as for the [doc site assets](doc-site-assets.md). `editors/vscode/scripts/check-dependency-age.ts --dir tools` checks the lockfile (`pnpm run check-ages`). To update a pin, edit `package.json` and run `pnpm install` once to refresh `pnpm-lock.yaml`. `node_modules` is gitignored.
