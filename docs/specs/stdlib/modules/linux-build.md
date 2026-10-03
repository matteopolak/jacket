# linux_build

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `get_plugin :: () -> *Metaprogram_Plugin`
- `before_intercept :: (p: *Metaprogram_Plugin, flags: *Intercept_Flags)`
- `add_source :: (p: *Metaprogram_Plugin)`
- `message :: (p: *Metaprogram_Plugin, message: *Message)`
- `finish :: (p: *Metaprogram_Plugin)`
- `shutdown :: (p: *Metaprogram_Plugin)`

## Public type members

No public aggregate fields or enum members were identified.

## Behavioral contract and constraints

`get_plugin()` allocates and returns a `Metaprogram_Plugin` with `before_intercept`, `add_source`, `message`, `finish`, and `shutdown` callbacks. It does not install an `init` callback. `before_intercept` sets the workspace target OS to Linux, enables custom linking, and selects the X64 backend; the supplied intercept flags are not changed. `add_source` and `finish` have no effects. `shutdown` frees the plugin allocation.

This module is Windows-host-only: its compile-time assertion rejects other hosts. The default toolchain root is `C:/UnrealToolchains/v21_clang-15.0.1-centos7`; `LINUX_MULTIARCH_ROOT`, when nonempty, replaces that root. The root is normalized to forward slashes and trailing `/` bytes are removed. The SDK subdirectory is `x86_64-unknown-linux-gnu`; the plugin expects the Unreal Engine v21 Clang/LLD 15.0.1 distribution. It selects the X64 backend and does not provide an LLVM path. The helper also derives `clang++` and `llvm-ar` paths, but the plugin invokes only the LLD linker for the custom link phase.

At `PRE_WRITE_EXECUTABLE`, `message` prints the executable name. At `READY_FOR_CUSTOM_LINK_COMMAND`, it launches `ld.lld.exe` with the SDK sysroot; `--build-id`, `--gdb-index`, `-g`, `--nostdlib`; SDK `/usr/lib` and `/usr/lib64` search paths; `crt1.o`, `crti.o`, `crtn.o`; the Linux dynamic loader `/lib64/ld-linux-x86-64.so.2`; `--eh-frame-hdr`, `-export-dynamic`, and `-rpath='$ORIGIN'`; compiler-generated and support object files; then a start/end library group and `-o <output>`. The output path is `<options.output_path>/<options.output_executable_name>`, using `.` when output_path is empty.

User library names without an extension receive `.a`; names with an extension are passed as supplied. For system libraries with a `.dll` extension, a basename beginning with `lib` becomes `-l<basename-without-lib>`; other DLL basenames become `<basename>.a`. Other system-library entries are ignored except `-lc` and `-lpthread`, which are passed through. If the linker file is missing, the plugin logs an error and still attempts launch. A nonzero linker result logs failure details and marks the workspace failed; the plugin signals custom-link completion afterward and writes the output path into the phase message. These rules do not guarantee that the cross-toolchain is installed or that a Linux executable is produced.
