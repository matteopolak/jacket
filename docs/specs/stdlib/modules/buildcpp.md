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

`build_cpp` compiles C/C++ sources into object files, static/dynamic libraries, or executables at compile time. It supports native macOS, Linux, and Windows toolchains. Cross-OS builds remain refused; the selected Jai CPU controls the Windows target architecture.

On Windows the default is the installed MSVC `cl.exe`/`lib.exe` toolset, discovered through `VCToolsInstallDir` or component-filtered `vswhere` queries for the selected target CPU. The helper supplies MSVC headers/library paths explicitly, so a developer prompt is not required. Windows SDK discovery uses `WindowsSdkDir` or the SDK registry installation root and selects a version with both headers and libraries for the target CPU. Native host tools are preferred on arm64; x64 tools can also run there through Windows emulation.

When MSVC is absent, discovery falls back to `clang-cl.exe` on `PATH`, in the LLVM installation, or in Visual Studio's optional Clang component. That component's directory is chosen by the host CPU (`HOST_ARM64`), not the target, so an x64 host targeting arm64 runs the x64 `clang-cl` with `--target=aarch64-pc-windows-msvc`. `cl` and `clang-cl` receive MSVC-style options, including caller-supplied `/D` definitions unchanged. Explicit GNU-driver Clang paths retain GNU-style options. `CC`, `CXX`, and `AR` override defaults; `compiler_executable_path` and `ar_executable_path` override those variables. Windows archive tools must accept Microsoft librarian arguments. Compiler and linker arguments are separate array entries and never pass through a command shell.

Windows outputs use `.obj`, `.lib`, `.dll`, and `.exe`. A DLL's generated import library is published beside it as `.lib`; a DLL without exports need not generate one. Debug links also publish `.pdb` symbols. DLL exports are controlled by the C/C++ source and compiler flags. COFF machine validation (`coff_object_matches`, a pure function over the first 32 bytes) rejects objects for another CPU before linking; it reads the machine from ordinary headers and from `/bigobj` headers (`ANON_OBJECT_HEADER_BIGOBJ` with its class GUID). `tests/coff-headers.jai` covers both on any host. Header decoding must group each widening cast before a shift: Jai prefix casts otherwise include the shift in their operand. Temporary directories use random names and atomic creation through Unicode Windows APIs; linker side products remain inside the staging directory and cleanup runs on both success and failure. macOS/Linux retain their existing POSIX directory handling, `CC`/`CXX` and `AR` tools, and ELF/Mach-O validation.

To change build orchestration, edit `stdlib/internal/build/cpp.jai`; platform filesystem and discovery operations live in `cpp-windows.jai` and `cpp-unix.jai`. Keep tool arguments as separate array entries, stage outputs before replacing destinations, and extend the Windows regression when adding output kinds or toolchain behavior. `stdlib/BuildCpp/tests/windows.jai` exercises C/C++, paths with spaces, commas and Unicode, all output kinds, explicit compiler paths, repeated archives, and staging cleanup on an actual Windows host. Negative compile/tool/target tests remain required there. Dependencies are the Visual Studio C++ tools and Windows SDK (or compatible Clang tools), `File`, `File_Utilities`, `Process`, `String`, a C/C++ toolchain, and the native platform APIs (`kernel32`/`bcrypt`/`advapi32` on Windows).
