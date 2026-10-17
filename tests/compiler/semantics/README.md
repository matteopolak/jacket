# Original compiler semantic fixtures

These positive, independently authored sources are shared by Rust core, CLI and
LLVM regressions. They exercise ordered scalar/spread packs, SIMD operand
forwarding, plugin lifecycle callbacks, and loaded-file constructor namespaces.
The Rust harnesses execute exported checks in interpreter/native/WASM64 modes;
these sources do not replace the production runtime or application corpus.

Format with repository jaifmt and lint with `jailint -v -D warnings`, selecting
the NEW independent library overlay. Verbose output must contain no incomplete
compilation report. The VFS strings for deliberate invalid-program tests remain
in the Rust harnesses because those cases must fail compilation; they are not
positive Jai lint inputs.
