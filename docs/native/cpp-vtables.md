# C++ vtables

`Bindings_Generator` emits C++ virtual dispatch tables for single-inheritance classes on 64-bit macOS, Linux and MSVC Windows (x86-64 or arm64). Each slot is a `#type (...) #cpp_method` function pointer, called like a C function with the instance pointer first.

## How it works

`stdlib/internal/developer-tools/generator-record-metadata.jai` builds the table from libclang declarations. It starts from the base's slots, replaces overridden ones, and appends new methods in declaration order (MSVC groups new same-name overloads in reverse). The object points at the first callable slot, after the Itanium offset-to-top and RTTI entries. Omitted methods keep opaque pointer slots so later offsets stay right.

- `<Record>_VTable` lists only the slots its record adds, after `using vtable: <Base>_VTable;`, so an overriding method is the base's slot.
- A record holds its base as `#as using,except(vtable) <base>: Base;` with `#overlay (<base>) vtable: *Record_VTable;`, so a derived pointer converts to a base pointer and the base's wrappers accept it.
- If the record's own fields start inside the base's tail padding, the base cannot be embedded (`gm_can_embed_parent`); the record repeats the base's fields and gets its own copy of each inherited wrapper.
- Only added slots get wrappers (`<Record>_<method>`). `print_struct_vtable` writes the table, the wrappers and `vtable(obj)`.

Supported signatures are scalars, pointers and small trivial records (up to 32 bytes, alignment at most eight; `generator-vtable-abi.jai`). Refused: unions, bitfields, packed or polymorphic record values, records with a user-provided constructor, copy or move constructor or destructor, extended floats, variadic methods, multiple inheritance and virtual bases. Destructors keep two opaque slots on Itanium and one on MSVC and are never callable. A record-returning slot keeps its natural signature on every target. MSVC returns every class or struct from an instance method through a hidden pointer after `this`, on x64 and arm64, whatever its size or shape (clang's IR for both shows `sret` after `this` for 4, 8, 16 and 24 byte records and float and double HFAs), and `#cpp_method` supplies that pointer itself.

What is verified:

- Generated text for a Windows target, on any host: `stdlib/Bindings_Generator/tests/generator-windows-vtable-slots.jai` (natural signatures, no hidden parameter, a constructor-bearing record refused).
- Runtime dispatch of 4, 8, 16 and 24 byte records and float and double HFAs, on the host running the checker. CI runs it on `windows-native` for x64 whenever that job runs and for arm64 in the full tier (`tools/ci/ci_changes.py`). Nothing here has run it on arm64 Windows by hand.

The target triple and the module's `TARGET_OS` must agree.

## How to change it

Change metadata construction and its tests together. Declarations come from the constructed table, and optional helpers call the same slots. Whether pure virtual methods get `virtual_` foreign declarations is decided in `binding_prints_virtual_function`. Add native C++ fixtures under `stdlib/Bindings_Generator/tests/driven/` that call the generated slots, not only check the text.

## Configuration and dependencies

The checker uses `JAIC`, `DEVELOPER_TOOLS_LLVM_PREFIX` (or `LLVM_PREFIX`) and `JACKET_OUTPUT_DIR`; use a release `JAIC` on macOS. It needs libclang and the checkout's binding helpers (`tools/developer_checks/build_generator_platform.py`, `build_developer_bindings.py`). On Windows it builds an import archive with `llvm-readobj` and `llvm-dlltool`.

```
PYTHONPATH=tools python3 tools/developer_checks/check_developer_tools.py ManifestChecks.test_check_developer_vtables
```

It generates for Linux and Windows on every host and runs real C++/Jai dispatch on the host at O0, O2 and in the interpreter. Cross-target generation does not exercise the ABI, so run it on each target host too.
