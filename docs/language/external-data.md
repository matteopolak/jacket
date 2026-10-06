# External data

## What it is

`x: T #elsewhere lib "symbol";` declares a variable whose storage lives in a foreign library or the process. It has no initializer.

## How it works

```jai
libc :: #library,system "libSystem";
out: *void #elsewhere libc "__stdoutp";
main :: () { print("%\n", out != null); }   // true on macOS
```

The symbol defaults to the variable name. `foreign_global` in `sema/decls.rs` turns the declaration into an `ir::Foreign` with `is_data: true` and `Storage::Foreign`; the interpreter (`interp/mod.rs`) and the LLVM backend resolve it by library and symbol. Without a library, only the compiler's own `__runtime_info` is recognised.

## How to change it

Parsing: `DECL_FLAGS` and `parse_elsewhere_library` in `parser/decl.rs`. Sema: `foreign_global`, and `resolve_library` in `sema/procs.rs`.

## Configuration

Library search uses `JAIC_NATIVE_LIBS` (a path list) and the stdlib directory, which `JAIC_STDLIB` overrides.

## Dependencies

The native libraries at run time; `Foreign` in `crates/jaic/src/ir.rs`.
