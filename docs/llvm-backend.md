# LLVM backend

## What it is

`jai-codegen` constructs modules through LLVM's typed Rust API using Inkwell. LLVM performs instruction construction, verification and serialization; the compiler does not concatenate instruction text.

## How it works

`lower(&Context, &Program)` declares all procedure signatures first, then lowers checked bodies into LLVM basic blocks and values. Forward and recursive calls target existing function handles. Local storage is allocated once in entry blocks. Branches implement conditionals, loops and short-circuit expressions. Phi nodes use the actual end blocks of nested expressions. Fully terminating conditionals do not create unused unterminated joins. Typed exit records emit scope cleanup before branching or returning, after saving any scalar return value.

Private integer/Boolean value and storage wrappers preserve language widths because LLVM uses one Rust value type for both. Enums select operations and comparison predicates. Strings name modules, functions and instructions for diagnostics; they do not select types or encode LLVM syntax.

The complete module is verified before returning. `emit` creates a context and uses LLVM to serialize it. The CLI surfaces structured backend errors before writing output or invoking Clang. `check` stops at semantic resolution. [Native target selection, optimization and object emission](native-build.md) use LLVM target machines; guarded installed Clang links newly generated host objects. [Workspace artifact scheduling](workspace-artifacts.md) applies source build settings and emits checked child builds.

## How to change it

Extend checked semantic types before adding lowering. Use typed LLVM operations and exhaustive enum matches. Add execution tests for behavior and preserve verification. Keep LLVM context lifetimes explicit; do not retain handles after their module or context is dropped. No unchecked text backend or unsafe Rust is enabled in workspace code.

`examples/sum.jai` is the README program. Keep its displayed source and exit status aligned when changing it.

## Configuration

Inkwell 0.10.0 uses `llvm22-1-prefer-dynamic` with independently installed LLVM 22. On macOS:

```sh
brew install llvm@22
export LLVM_SYS_221_PREFIX="$(brew --prefix llvm@22)"
export PATH="$LLVM_SYS_221_PREFIX/bin:$PATH"
python3 tools/check_dependency_age.py
cargo build -p jai-cli --locked
```


## Dependencies

Inkwell 0.10.0, llvm-sys 221.1.0, independently installed LLVM 22.1, and `jai-sema`/`jai-syntax`. Run the locked dependency-age guard before compilation, including after lockfile changes. Inkwell and llvm-sys use unsafe FFI internally; workspace code forbids unsafe Rust. See [Inkwell](https://github.com/TheDan64/inkwell) and [Homebrew LLVM 22](https://formulae.brew.sh/formula/llvm@22).
