# jaic browser compiler

The jaic Jai compiler and interpreter, built for `wasm32-unknown-unknown`. The standard library is embedded in the module, so it needs no network or file access. A hosted playground built on it is at https://matteopolak.com/playground/jai.

| File | Contents |
| --- | --- |
| `jai_wasm.wasm` | Compiler, interpreter and language server. It takes no host imports. |
| `engine.mjs` | Optional JavaScript glue: `createEngine(bytes)` returns `{ play(files, main, { budget }), lsp(message) }`. |
| `jaifmt-playground.jai` | Formatter driver. Run it with `play` to format `/workspace/main.jai`. |
| `build-metadata.json` | Compiler commit, Rust toolchain and the module's SHA-256. |

```js
import { createEngine } from "./engine.mjs";
const engine = await createEngine(await (await fetch("jai_wasm.wasm")).arrayBuffer());
const result = engine.play({ "main.jai": "main :: () -> int { return 42; }" }, "main.jai");
// { exitCode: 42, stdout, stderr, output: [{ stream, text }], rendered, diagnostics }
```

`play` blocks until the program finishes, so call it from a Web Worker. Use `budget` to bound runaway programs. If it throws an error with `compilerCrashed` set, create a new engine.
