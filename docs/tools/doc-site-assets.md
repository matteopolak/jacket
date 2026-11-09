# Doc site assets

The JavaScript and CSS of the [jacket doc](jacket-doc.md) site, `crates/jacket/src/doc/site/assets/{site.js,early.js,run-worker.js,site.css}`, are checked with development-only tools in `crates/jacket/src/doc/site-tools/`. Nothing there is built or shipped: `jacket` embeds the asset files as they are, and a shipped `jacket` never needs Node.

## Run the checks

```
just site-assets          # or: python3 tools/dev/dev.py site-assets
just site-assets --fix    # format the assets
```

This runs `pnpm run lint` in `site-tools/` (installing first when `node_modules` is missing). It needs pnpm at the version `package.json` names; without it the command fails and says so.

| Script | Tool | Checks |
| --- | --- | --- |
| `lint:js` | oxlint | correctness and suspicious rules, `--deny-warnings` |
| `lint:types` | tsc | `checkJs` in strict mode; the files are typed with JSDoc (`tsconfig.json`: `site.js`, `early.js`; `tsconfig.worker.json`: `run-worker.js`) |
| `lint:css` | stylelint | `stylelint-config-standard` |
| `fmt:check` | oxfmt | formatting (`fmt` rewrites) |

CI runs the same `pnpm run lint` in the `doc-site-assets` job, only when a file under `crates/jacket/src/doc/site/` or `site-tools/` changes.

## How to change it

- **Types.** Each script starts with `// @ts-check`. What the scripts put on `window` for each other (`jacketPage`, `jacketSide`, `jacketRestore`) and shared shapes are declared in `site-tools/globals.d.ts`. Prefer a typedef or a narrowing check to a cast; do not add `@ts-ignore`.
- **Rules.** `oxlint.config.ts`, `.stylelintrc.json` and `oxfmt.config.ts` (typed `defineConfig` exports; single quotes, 100 columns) live in `site-tools/`. Rules turned off, and why:
  - oxlint `unicorn/require-post-message-target-origin`: it asks for a target origin, which `Worker.postMessage` does not have.
  - stylelint `no-descending-specificity`: the sheet orders states and variants by hand, and the rule asks for reordering that would change the cascade.
  - stylelint `property-no-vendor-prefix` ignores `appearance`, `backdrop-filter` and `mask-image`, and `number-max-precision` is 5.
  - One inline `oxlint-disable no-control-regex` in `site.js`: the search index marks links with U+0001 to U+0003 on purpose.
- **Not under `site/`.** `crates/jacket/build.rs` embeds every file under `doc/site/` except dotfiles, so the tooling lives beside it in `site-tools/`.

## Dependencies

Exact versions in `package.json` as `devDependencies` only, `minimumReleaseAge` (14 days) in `pnpm-workspace.yaml`, and `pnpm install --frozen-lockfile --ignore-scripts` in CI, as for the [VS Code extension](vscode-extension.md). `editors/vscode/scripts/check-dependency-age.ts --dir <folder>` checks the lockfile and the pnpm release of another pnpm project. To update a pin, edit `package.json` and run `pnpm install` once to refresh `pnpm-lock.yaml`.
