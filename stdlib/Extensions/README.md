# jaic extensions

The modules in this folder exist only in jaic, not in the official Jai distribution. Import them by
their path, so the import says the code is jaic-only:

| Import | What it is for |
| --- | --- |
| `#import "Extensions/Long_Double";` | C's `long double` in the target's format, for C functions that take or return one |
| `#import "Extensions/Jai_Format";` | The Jai source formatter behind `jaifmt` (text in, text out) |
| `#import "Extensions/WebGPU";` | The standard WebGPU C API (`webgpu.h`): wgpu-native natively, the page's WebGPU in the browser |
| `#import "Extensions/Wasi_Runtime";` | The C library and `_start` a wasm64 program needs as a WASI command (added by `jaic build -os wasm`) |

They may change between jaic releases. Details: `docs/stdlib/extensions.md` in the jaic repository.
