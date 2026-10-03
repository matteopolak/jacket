# BuildCpp

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Build_Type :: enum`
- `build_cpp_static_lib :: #bake_arguments build_cpp(type = .STATIC_LIBRARY)`
- `build_cpp_dynamic_lib :: #bake_arguments build_cpp(type = .DYNAMIC_LIBRARY)`
- `build_cpp_executable :: #bake_arguments build_cpp(type = .EXECUTABLE)`
- `build_cpp :: (output_basename: string, files: ..string, type: Build_Type, debug := false, extra: [] string = .[], library_files: [] string = .[], target := OS, compiler_executable_path := "", ar_executable_path := "", working_directory := "", loc := #caller_location) -> bool`
- `build_universal_macos_binary :: (lib_path: string, files: ..string, type: Build_Type, debug := false, extra: [] string = .[]) -> bool`
- `enum_cpp_files :: (path: string, recursive:=false) -> [..] string`
- `free_cpp_files :: (files: [] string)`

## Public type members

#### `Build_Type` (enum)

```jai
Build_Type :: enum {
    STATIC_LIBRARY;
    DYNAMIC_LIBRARY;
    EXECUTABLE;
    OBJ_FILE;
}
```
## Behavioral contract and constraints

`build_cpp` compiles supplied source files into object files, static/dynamic libraries, or executables and returns success as a boolean. It is compile-time-only. On Windows it locates Visual Studio and Windows SDK tools; on non-Windows it infers `CC`/`CXX` or uses explicit compiler/archive paths. Missing toolchains are reported through compiler diagnostics. Intermediate object files are removed for linked outputs, and dynamic-library export files are cleaned up. Cross-target support is constrained by the selected host/toolchain; this is intentionally a small-build helper, not a complete build system.


Platform coverage note: exports for Nintendo targets are unspecified.
