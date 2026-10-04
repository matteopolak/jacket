# External data

## What it is

`x: T #elsewhere lib "symbol";` declares a variable whose storage lives in a foreign library or the process, with no initializer.

## How it works

```jai
libc :: #library,system "libSystem";
out: *void #elsewhere libc "__stdoutp";
main :: () { print("%\n", out != null); }   // true (macOS)
```

- The symbol defaults to the variable name; a string after the library overrides it.
- The declaration becomes an `ir::Foreign` with `is_data: true` and `Storage::Foreign` (`foreign_global` in `crates/jaic/src/sema/decls.rs`); the interpreter and the LLVM backend resolve it by library and symbol (`crates/jaic/src/interp/mod.rs`).
- Without a library, only the compiler's own `__runtime_info` is recognised.

## How to change it

Parse side: `DECL_FLAGS` and `parse_elsewhere_library` in `crates/jaic/src/parser/decl.rs`. Sema: `foreign_global` and `resolve_library` (`sema/procs.rs`).

## Configuration

`JAIC_NATIVE_LIBS` (path list) and the stdlib directory feed native library search; `JAIC_STDLIB` overrides the latter.

## Dependencies

Native libraries at run time; `crates/jaic/src/ir.rs` (`Foreign`).
