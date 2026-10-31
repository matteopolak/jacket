//! The rendered link arguments are pinned to the exact strings the linker has always been given.
use super::*;

fn msvc() -> &'static Toolchain {
    Toolchain::for_target(Target::windows(crate::target::TargetCpu::X64, true))
}

fn mingw() -> &'static Toolchain {
    Toolchain::for_target(Target::windows(crate::target::TargetCpu::X64, false))
}

fn strings(args: &[&str]) -> Vec<String> {
    args.iter().map(ToString::to_string).collect()
}

#[test]
fn link_syntax_names_the_dll_runtime_by_default_library() {
    let args = msvc().runtime_args(Driver::MsvcLink);
    assert!(args.compile.is_empty());
    assert_eq!(
        args.link,
        strings(&[
            "/DEFAULTLIB:msvcrt",
            "/DEFAULTLIB:oldnames",
            "/DEFAULTLIB:legacy_stdio_definitions",
        ])
    );
}

#[test]
fn clang_cl_selects_the_dll_runtime_before_the_link_options() {
    let args = msvc().runtime_args(Driver::ClangCl);
    assert_eq!(args.compile, strings(&["/MD"]));
    assert_eq!(args.link, msvc().runtime_args(Driver::MsvcLink).link);
}

/// Clang's own link step adds the static runtime; it has to be excluded by name.
#[test]
fn the_cc_driver_drops_the_static_runtime() {
    let args = msvc().runtime_args(Driver::C);
    assert!(args.compile.is_empty());
    assert_eq!(
        args.link,
        strings(&[
            "-fms-runtime-lib=dll",
            "-Wl,/NODEFAULTLIB:libcmt",
            "-Wl,/DEFAULTLIB:msvcrt",
            "-llegacy_stdio_definitions",
        ])
    );
}

#[test]
fn only_the_msvc_runtime_adds_link_arguments() {
    for target in [
        "x86_64-pc-windows-gnu",
        "aarch64-pc-windows-gnu",
        "x86_64-unknown-linux-gnu",
        "x86_64-unknown-linux-musl",
        "aarch64-apple-darwin",
        "wasm32-wasi",
    ] {
        let toolchain = Toolchain::for_target(Target::parse(target).unwrap());
        for driver in [Driver::C, Driver::ClangCl, Driver::MsvcLink] {
            assert_eq!(
                toolchain.runtime_args(driver),
                RuntimeArgs::default(),
                "{target}"
            );
        }
    }
}

#[test]
fn the_main_stack_is_8_mib_on_windows_only() {
    assert_eq!(
        msvc().stack_arg(Driver::MsvcLink).unwrap(),
        "/STACK:8388608"
    );
    assert_eq!(msvc().stack_arg(Driver::ClangCl).unwrap(), "/STACK:8388608");
    assert_eq!(msvc().stack_arg(Driver::C).unwrap(), "-Wl,/STACK:8388608");
    assert_eq!(mingw().stack_arg(Driver::C).unwrap(), "-Wl,--stack,8388608");
    for target in ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin"] {
        let toolchain = Toolchain::for_target(Target::parse(target).unwrap());
        assert_eq!(toolchain.stack_arg(Driver::C), None, "{target}");
    }
}

#[test]
fn a_custom_link_command_names_every_runtime_library() {
    assert_eq!(
        msvc().runtime_inputs().collect::<Vec<_>>(),
        strings(&[
            "msvcrt.lib",
            "vcruntime.lib",
            "ucrt.lib",
            "oldnames.lib",
            "legacy_stdio_definitions.lib",
            "legacy_stdio_wide_specifiers.lib",
        ])
    );
    assert_eq!(mingw().runtime_inputs().count(), 0);
}

#[test]
fn windows_runtime_names_ignore_case_and_unix_ones_do_not() {
    for name in [
        "c",
        "libc",
        "crt",
        "msvcrt",
        "ucrt",
        "ucrtbase",
        "vcruntime",
        "libcmt",
        "m",
        "libm",
        "pthread",
        "libpthread",
        "dl",
        "libdl",
        "rt",
        "librt",
        "MSVCRT",
        "Libc",
    ] {
        assert!(msvc().is_runtime(name), "{name}");
        assert!(mingw().is_runtime(name), "{name}");
    }
    assert!(!msvc().is_runtime("kernel32"));
    let linux = Toolchain::for_target(Target::parse("x86_64-unknown-linux-gnu").unwrap());
    assert!(linux.is_runtime("c") && linux.is_runtime("libc"));
    assert!(!linux.is_runtime("C") && !linux.is_runtime("m") && !linux.is_runtime("pthread"));
}

#[test]
fn the_interpreter_loads_one_dll_per_runtime_name() {
    let toolchain = Toolchain::interpreter();
    for name in [
        "c",
        "libc",
        "m",
        "libm",
        "msvcrt",
        "crt",
        "libcmt",
        "pthread",
        "libpthread",
        "dl",
        "libdl",
        "rt",
        "librt",
        "LibM",
        "CRT",
        "LibCMT",
    ] {
        assert_eq!(
            toolchain.interpreter_dll(name),
            Some("msvcrt.dll"),
            "{name}"
        );
    }
    for name in ["ucrt", "ucrtbase", "vcruntime", "UCRT", "VCRuntime"] {
        assert_eq!(
            toolchain.interpreter_dll(name),
            Some("ucrtbase.dll"),
            "{name}"
        );
    }
    assert_eq!(toolchain.interpreter_dll("kernel32"), None);
    let dlls = toolchain.runtime.dlls.unwrap();
    assert_eq!((dlls.c, dlls.stdio), ("msvcrt.dll", "ucrtbase.dll"));
}

#[test]
fn every_windows_runtime_name_maps_to_a_dll_and_the_two_runtimes_stay_apart() {
    for toolchain in [msvc(), mingw()] {
        assert_eq!(toolchain.runtime_names.len(), 16);
        for row in toolchain.runtime_names {
            let dll = row.dll.unwrap_or_else(|| panic!("{} has no DLL", row.name));
            let universal = matches!(row.name, "ucrt" | "ucrtbase" | "vcruntime");
            assert_eq!(
                dll,
                if universal {
                    "ucrtbase.dll"
                } else {
                    "msvcrt.dll"
                },
                "{}",
                row.name
            );
        }
    }
    let linux = Toolchain::for_target(Target::parse("x86_64-unknown-linux-gnu").unwrap());
    assert_eq!(linux.interpreter_dll("c"), None);
}

#[test]
fn msvc_calls_the_double_function_for_long_double_math() {
    assert_eq!(msvc().rename("sqrtl"), Some("sqrt"));
    assert_eq!(msvc().rename("atan2l"), Some("atan2"));
    assert_eq!(msvc().rename("sqrt"), None);
    assert_eq!(msvc().rename("printfl"), None);
    assert_eq!(mingw().rename("sqrtl"), None);
}

#[test]
fn data_is_imported_from_a_dll_unless_the_runtime_is_static() {
    assert!(msvc().imports_data("kernel32", true));
    assert!(!msvc().imports_data("kernel32", false));
    assert!(msvc().imports_data("ucrt", true));
    assert!(msvc().imports_data("UCRTBASE", true));
    // The generic names also reach the static import library.
    assert!(!msvc().imports_data("libc", true));
    assert!(!msvc().imports_data("msvcrt", true));
    assert!(mingw().imports_data("kernel32", true));
    assert!(!mingw().imports_data("ucrt", true));
    let linux = Toolchain::for_target(Target::parse("x86_64-unknown-linux-gnu").unwrap());
    assert!(!linux.imports_data("SDL3", true));
}

#[test]
fn wasm_imports_the_runtime_from_env() {
    let wasm = Toolchain::for_target(Target::wasi());
    assert_eq!(wasm.import_module(""), Some("env"));
    for name in ["c", "libc", "crt", "msvcrt", "m", "libm", "dir/libm"] {
        assert_eq!(wasm.import_module(name), Some("env"), "{name}");
    }
    assert_eq!(wasm.import_module("host_graphics"), Some("host_graphics"));
    assert_eq!(wasm.import_module("a\\host.dll"), Some("host.dll"));
    assert_eq!(msvc().import_module("c"), None);
}

#[test]
fn every_target_has_a_toolchain() {
    assert_eq!(msvc().linker, Linker::Msvc);
    assert_eq!(mingw().linker, Linker::MinGw);
    for (triple, linker) in [
        ("aarch64-pc-windows-msvc", Linker::Msvc),
        ("aarch64-pc-windows-gnu", Linker::MinGw),
        ("x86_64-unknown-linux-gnu", Linker::Unix),
        ("x86_64-unknown-linux-musl", Linker::Unix),
        ("aarch64-apple-darwin", Linker::Unix),
        ("wasm32-wasi", Linker::Unix),
    ] {
        let target = Target::parse(triple).unwrap();
        assert_eq!(Toolchain::for_target(target).linker, linker, "{triple}");
    }
}
