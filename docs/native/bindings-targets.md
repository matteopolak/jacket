# Bindings for another target

`Bindings_Generator` hands one target triple to clang, and clang decides everything about it.

## Choosing the triple

- Put `-target <triple>` or `--target=<triple>` in `Generate_Bindings_Options.extra_clang_arguments`. The last one wins, as in clang.
- Otherwise the generator derives a default from `options.os` and `options.cpu`. `internal/target-triple` holds that table and nothing else does; it names x64 and arm64 on Windows, Linux, macOS, iOS and Android. Any other combination needs an explicit triple, and the generator says so.
- The triple is always passed to clang, on every OS. Clang rejects a triple it does not know; the generator reports that error instead of checking against a list of its own.

```jai
options: Generate_Bindings_Options;
array_add(*options.source_files, "lvgl.h");
array_add(*options.extra_clang_arguments, "--target=thumbv7em-none-eabihf");
array_add(*options.system_include_paths, "arm-none-eabi/include");
```

## What clang supplies

The pointer width (2, 4 or 8 bytes), the Apple/Windows/Linux family, the Debian multiarch directory (`clang -print-multiarch`) and the layout of every record come from clang's view of the triple. `size_t` is `u32` for `i686-linux-gnu` and `u16` for `avr`.

Passing a triple does not find headers: set `system_include_paths` for a freestanding target, or let the generator ask the C compiler for the include search list of the triple.

## Changing it

- A new default OS/CPU pair goes in `default_target_triple` (`stdlib/internal/target-triple/module.jai`).
- Do not read `OS`, `CPU` or `size_of(*void)` to learn about the target; use `clang_getTranslationUnitTargetInfo` (`gm_pointer_size` in `generator-special-construction.jai`).
- The native vtable metadata (`generator-vtable-abi.jai`) is only produced for 8-byte pointers on Itanium and MSVC triples; other targets get no vtable metadata.

Tests: `stdlib/Bindings_Generator/tests/targets`.
