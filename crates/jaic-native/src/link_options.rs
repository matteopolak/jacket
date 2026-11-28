//! Build-option arguments cross two boundaries: compiler driver and target linker.
//!
//! Arguments are OS strings end to end: Clang and the linkers take any bytes. Only the ASCII
//! spellings below are recognised and rewritten; every other argument passes through as given.
use std::ffi::{OsStr, OsString};

use jacket_config::os_text;
use jaic::toolchain::{Driver, Linker};
pub(super) use jaic_toolchain::names::is_clang;

pub(super) fn driver_for_command(program: &str, args: &[OsString]) -> Driver {
    let mut driver = driver_for_program(program);

    if driver != Driver::MsvcLink && is_clang(program) {
        let mut args = args.iter();

        while let Some(arg) = args.next() {
            if takes_driver_operand(arg) {
                args.next();
                continue;
            }

            let Some(arg) = arg.to_str() else {
                continue;
            };

            if driver == Driver::ClangCl && arg.eq_ignore_ascii_case("/link") {
                break;
            }

            match arg {
                "--driver-mode=cl" => driver = Driver::ClangCl,
                "--driver-mode=gcc" | "--driver-mode=g++" => driver = Driver::C,
                _ => {}
            }
        }
    }

    driver
}

pub(super) fn driver_for_program(program: &str) -> Driver {
    let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
    let name = name.to_ascii_lowercase();
    let name = name.strip_suffix(".exe").unwrap_or(&name);

    match name {
        "link" | "lld-link" => Driver::MsvcLink,
        "clang-cl" => Driver::ClangCl,
        _ => Driver::C,
    }
}

fn takes_driver_operand(arg: &OsStr) -> bool {
    matches!(
        arg.to_str(),
        Some(
            "-Xlinker"
                | "-Xclang"
                | "-o"
                | "-L"
                | "-l"
                | "-target"
                | "--target"
                | "-isysroot"
                | "--sysroot"
                | "-B"
                | "/Fo"
                | "/Fe"
                | "/Fd"
                | "/I"
                | "/D"
                | "/FI"
        )
    )
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct ExtraArgs {
    pub driver: Vec<OsString>,
    pub linker: Vec<OsString>,
}

/// Rewrites the `ld.lld` spellings of the link settings jacket writes from a project's
/// `stack-size`, `entry` and `gc-sections` into what the linker in use takes. Anything else, and
/// arguments already in the linker's own spelling, pass through.
///
/// | `ld.lld` | `link.exe` | macOS `ld` (through `cc`) | GNU `ld` (MinGW) |
/// | --- | --- | --- | --- |
/// | `--gc-sections` | `/OPT:REF` | `-Wl,-dead_strip` | `-Wl,--gc-sections` |
/// | `--no-gc-sections` | `/OPT:NOREF` | (the default) | `-Wl,--no-gc-sections` |
/// | `--entry=SYM` | `/ENTRY:SYM` | `-Wl,-e,SYM` | `-Wl,--entry=SYM` |
/// | `-z stack-size=N` | `/STACK:N` | `-Wl,-stack_size,0xN` | `-Wl,--stack,N` |
///
/// Other Unix linkers get the `ld.lld` spelling behind `-Wl,`. A stack size that is not a
/// number of bytes is an error.
pub(super) fn translate(
    args: &[OsString],
    linker: Linker,
    driver: Driver,
    apple: bool,
) -> Result<Vec<OsString>, String> {
    let msvc = matches!(driver, Driver::MsvcLink | Driver::ClangCl) || linker == Linker::Msvc;
    // The linker is started directly (`ld.lld`, `link.exe`, `wasm-ld`) or through a C driver.
    let through_driver = driver == Driver::C && !msvc;
    let wl = |text: &str| -> OsString {
        if through_driver {
            format!("-Wl,{text}").into()
        } else {
            text.into()
        }
    };
    let mut out = Vec::new();
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        let Some(text) = arg.to_str() else {
            out.push(arg.clone());
            continue;
        };
        let stack = (text == "-z")
            .then(|| {
                args.clone()
                    .next()
                    .and_then(|v| v.to_str())
                    .and_then(|v| v.strip_prefix("stack-size="))
            })
            .flatten();

        if let Some(size) = stack {
            args.next();
            let bytes = size.parse::<u64>().map_err(|_| {
                format!("`-z stack-size={size}` is not a stack size\nhelp: give it in bytes, such as `-z stack-size=8388608`")
            })?;

            out.push(if msvc {
                format!("/STACK:{bytes}").into()
            } else if apple {
                format!("-Wl,-stack_size,{bytes:#x}").into()
            } else if linker == Linker::MinGw {
                wl(&format!("--stack,{bytes}"))
            } else {
                wl(&format!("-z,stack-size={bytes}"))
            });

            continue;
        }

        match text {
            "--gc-sections" if msvc => out.push("/OPT:REF".into()),
            "--no-gc-sections" if msvc => out.push("/OPT:NOREF".into()),
            "--gc-sections" if apple => out.push("-Wl,-dead_strip".into()),
            "--no-gc-sections" if apple => {}
            "--gc-sections" | "--no-gc-sections" => out.push(wl(text)),
            _ => match text.strip_prefix("--entry=") {
                Some(symbol) if msvc => out.push(format!("/ENTRY:{symbol}").into()),
                Some(symbol) if apple => out.push(format!("-Wl,-e,{symbol}").into()),
                Some(_) => out.push(wl(text)),
                None => out.push(arg.clone()),
            },
        }
    }

    Ok(out)
}

/// The operand of `option`, the next argument.
fn operand<'a>(
    option: &OsStr,
    args: &mut impl Iterator<Item = &'a OsString>,
) -> Result<OsString, String> {
    args.next().cloned().ok_or_else(|| {
        format!(
            "driver option `{}` requires an argument",
            option.to_string_lossy()
        )
    })
}

pub(super) fn render(
    args: &[OsString],
    linker: Linker,
    driver: Driver,
) -> Result<ExtraArgs, String> {
    if driver == Driver::MsvcLink {
        return Ok(ExtraArgs {
            driver: Vec::new(),
            linker: args.to_vec(),
        });
    }

    if driver == Driver::ClangCl && linker != Linker::Msvc {
        return Err("clang-cl linking requires an MSVC target; use a GNU-style compiler driver for MinGW targets".into());
    }

    let mut output = ExtraArgs::default();
    let mut explicit_link = false;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        let text = arg.to_str();

        if driver == Driver::ClangCl {
            if text.is_some_and(|t| t.eq_ignore_ascii_case("/link")) {
                explicit_link = true;
            } else if explicit_link || text.and_then(msvc_option).is_some() {
                output.linker.push(arg.clone());
            } else if text == Some("-Xlinker") {
                output
                    .linker
                    .push(args.next().ok_or("-Xlinker requires an argument")?.clone());
            } else if let Some(value) = os_text::strip_prefix(arg, "-Wl,") {
                output.linker.extend(os_text::split(&value, b','));
            } else {
                output.driver.push(arg.clone());

                if takes_driver_operand(arg) {
                    output.driver.push(operand(arg, &mut args)?);
                }
            }

            continue;
        }

        // Explicit driver forwarding is already tokenized. In particular, the
        // operand of -Xlinker is not another implicit build-option argument.
        if takes_driver_operand(arg) {
            output.driver.push(arg.clone());
            output.driver.push(operand(arg, &mut args)?);
        } else if linker.is_windows()
            && let Some(text) = text
            && let Some((name, value)) = msvc_option(text)
        {
            let value = if linker == Linker::MinGw {
                mingw_option(text, &name, value)?.into()
            } else {
                arg.clone()
            };

            output.driver.push("-Xlinker".into());
            output.driver.push(value);
        } else {
            output.driver.push(arg.clone());
        }
    }

    Ok(output)
}

/// Recognize LINK switches, not arbitrary slash-prefixed paths or CL compiler
/// options. Unknown inputs still reach the selected tool's normal diagnostics.
fn msvc_option(arg: &str) -> Option<(String, Option<&str>)> {
    let arg = arg.strip_prefix('/')?;
    let (name, value) = arg
        .split_once(':')
        .map_or((arg, None), |(k, v)| (k, Some(v)));
    let name = name.to_ascii_uppercase();

    matches!(
        name.as_str(),
        "SUBSYSTEM"
            | "ENTRY"
            | "STACK"
            | "HEAP"
            | "BASE"
            | "INCLUDE"
            | "LIBPATH"
            | "OUT"
            | "DLL"
            | "DEBUG"
            | "PDB"
            | "PDBALTPATH"
            | "IMPLIB"
            | "DEF"
            | "EXPORT"
            | "DEFAULTLIB"
            | "NODEFAULTLIB"
            | "NOLOGO"
            | "OPT"
            | "INCREMENTAL"
            | "MACHINE"
            | "MAP"
            | "MAPINFO"
            | "MANIFEST"
            | "MANIFESTFILE"
            | "MANIFESTINPUT"
            | "MANIFESTDEPENDENCY"
            | "MANIFESTUAC"
            | "NXCOMPAT"
            | "DYNAMICBASE"
            | "HIGHENTROPYVA"
            | "LARGEADDRESSAWARE"
            | "FIXED"
            | "RELEASE"
            | "LTCG"
            | "WHOLEARCHIVE"
            | "DELAYLOAD"
            | "IGNORE"
            | "VERSION"
            | "ALIGN"
            | "FILEALIGN"
            | "SECTION"
            | "MERGE"
            | "SAFESEH"
            | "GUARD"
            | "CETCOMPAT"
            | "BREPRO"
            | "VERBOSE"
            | "FORCE"
            | "ERRORREPORT"
            | "TLBID"
            | "TLBOUT"
            | "STUB"
            | "DRIVER"
            | "NOENTRY"
            | "PROFILE"
    )
    .then_some((name, value))
}

fn mingw_option(original: &str, name: &str, value: Option<&str>) -> Result<String, String> {
    let option = match name {
        "SUBSYSTEM" => "subsystem",
        "ENTRY" => "entry",
        "STACK" => "stack",
        "HEAP" => "heap",
        "BASE" => "image-base",
        "INCLUDE" => "undefined",
        "LIBPATH" => "library-path",
        "OUT" => "output",
        "DLL" if value.is_none() => return Ok("--dll".into()),
        _ => {
            return Err(format!(
                "MSVC linker option `{original}` has no supported MinGW translation; pass an explicit GNU linker option for this target"
            ));
        }
    };
    let value = value
        .filter(|v| !v.is_empty())
        .ok_or_else(|| format!("linker option `{original}` requires a value"))?;
    let value = if name == "SUBSYSTEM" {
        value.to_ascii_lowercase().replacen(',', ":", 1)
    } else {
        value.to_owned()
    };

    Ok(format!("--{option}={value}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<OsString> {
        values.iter().map(OsString::from).collect()
    }

    #[cfg(unix)]
    #[test]
    fn arguments_that_are_not_utf8_pass_through_with_their_bytes() {
        use std::os::unix::ffi::{OsStrExt, OsStringExt};
        let bytes = |b: &[u8]| OsString::from_vec(b.to_vec());
        let input = vec![
            bytes(b"/lib/\xff.a"),
            "--gc-sections".into(),
            "-o".into(),
            bytes(b"out\xfe"),
        ];
        let translated = translate(&input, Linker::Unix, Driver::C, false).unwrap();

        assert_eq!(translated[0].as_bytes(), b"/lib/\xff.a");
        assert_eq!(translated[1], "-Wl,--gc-sections");
        let rendered = render(&translated, Linker::Unix, Driver::C).unwrap();

        assert_eq!(rendered.driver, translated);
        // clang-cl hands `-Wl,` pieces to the linker, split at the commas, bytes kept.
        let input = vec![bytes(b"-Wl,/LIBPATH:\xff,/DEBUG")];
        let rendered = render(&input, Linker::Msvc, Driver::ClangCl).unwrap();

        assert_eq!(rendered.linker[0].as_bytes(), b"/LIBPATH:\xff");
        assert_eq!(rendered.linker[1], "/DEBUG");
        assert_eq!(
            driver_for_command("clang", &[bytes(b"\xff"), "--driver-mode=cl".into()]),
            Driver::ClangCl
        );
    }

    #[test]
    fn link_settings_are_written_for_the_linker_in_use() {
        let settings = args(&["--gc-sections", "--entry=start", "-z", "stack-size=65536"]);
        let wl = |list: &[&str]| args(list);

        assert_eq!(
            translate(&settings, Linker::Unix, Driver::C, false).unwrap(),
            wl(&[
                "-Wl,--gc-sections",
                "-Wl,--entry=start",
                "-Wl,-z,stack-size=65536"
            ])
        );
        assert_eq!(
            translate(&settings, Linker::Unix, Driver::C, true).unwrap(),
            wl(&["-Wl,-dead_strip", "-Wl,-e,start", "-Wl,-stack_size,0x10000"])
        );
        assert_eq!(
            translate(&settings, Linker::MinGw, Driver::C, false).unwrap(),
            wl(&[
                "-Wl,--gc-sections",
                "-Wl,--entry=start",
                "-Wl,--stack,65536"
            ])
        );
        assert_eq!(
            translate(&settings, Linker::Msvc, Driver::MsvcLink, false).unwrap(),
            wl(&["/OPT:REF", "/ENTRY:start", "/STACK:65536"])
        );
        assert_eq!(
            translate(&settings, Linker::Msvc, Driver::C, false).unwrap(),
            wl(&["/OPT:REF", "/ENTRY:start", "/STACK:65536"])
        );
    }

    #[test]
    fn a_stack_size_that_is_not_a_number_is_an_error() {
        let error = translate(
            &args(&["-z", "stack-size=8M"]),
            Linker::Unix,
            Driver::C,
            true,
        )
        .unwrap_err();

        assert!(
            error.starts_with("`-z stack-size=8M` is not a stack size"),
            "{error}"
        );
        assert!(error.contains("\nhelp: "), "{error}");
    }

    #[test]
    fn other_arguments_and_the_linkers_own_spellings_pass_through() {
        let input = args(&[
            "-flto",
            "-Wl,--gc-sections",
            "/SUBSYSTEM:windows",
            "--no-gc-sections",
        ]);

        assert_eq!(
            translate(&input, Linker::Unix, Driver::C, false).unwrap(),
            args(&[
                "-flto",
                "-Wl,--gc-sections",
                "/SUBSYSTEM:windows",
                "-Wl,--no-gc-sections"
            ])
        );
        // The default on macOS: nothing to say.
        assert!(
            translate(&args(&["--no-gc-sections"]), Linker::Unix, Driver::C, true)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn windows_c_driver_forwards_link_options_without_splitting() {
        let input = args(&[
            "-flto",
            "/SUBSYSTEM:windows",
            "/ENTRY:mainCRTStartup",
            "/LIBPATH:C:\\SDK files,one",
            "first object.obj",
            "second.obj",
        ]);

        assert_eq!(
            render(&input, Linker::Msvc, Driver::C).unwrap().driver,
            args(&[
                "-flto",
                "-Xlinker",
                "/SUBSYSTEM:windows",
                "-Xlinker",
                "/ENTRY:mainCRTStartup",
                "-Xlinker",
                "/LIBPATH:C:\\SDK files,one",
                "first object.obj",
                "second.obj"
            ])
        );
    }

    #[test]
    fn mingw_converts_pe_options_but_preserves_driver_options_and_order() {
        let input = args(&[
            "-fuse-ld=lld",
            "/SUBSYSTEM:WINDOWS,6.2",
            "/ENTRY:mainCRTStartup",
            "one object.o",
            "/STACK:1048576,4096",
            "/LIBPATH:C:\\SDK files,one",
            "two.o",
            "-Wl,--gc-sections",
            "-Xlinker",
            "--trace",
        ]);

        assert_eq!(
            render(&input, Linker::MinGw, Driver::C).unwrap().driver,
            args(&[
                "-fuse-ld=lld",
                "-Xlinker",
                "--subsystem=windows:6.2",
                "-Xlinker",
                "--entry=mainCRTStartup",
                "one object.o",
                "-Xlinker",
                "--stack=1048576,4096",
                "-Xlinker",
                "--library-path=C:\\SDK files,one",
                "two.o",
                "-Wl,--gc-sections",
                "-Xlinker",
                "--trace"
            ])
        );
    }

    #[test]
    fn clang_cl_separates_driver_flags_from_linker_arguments() {
        let input = args(&[
            "/MD",
            "-fuse-ld=lld",
            "/SUBSYSTEM:windows",
            "/ENTRY:mainCRTStartup",
            "extra object.obj",
            "/link",
            "/DEBUG",
            "/PDB:C:\\symbols with spaces\\app.pdb",
        ]);

        assert_eq!(
            render(&input, Linker::Msvc, Driver::ClangCl).unwrap(),
            ExtraArgs {
                driver: args(&["/MD", "-fuse-ld=lld", "extra object.obj"]),
                linker: args(&[
                    "/SUBSYSTEM:windows",
                    "/ENTRY:mainCRTStartup",
                    "/DEBUG",
                    "/PDB:C:\\symbols with spaces\\app.pdb"
                ]),
            }
        );
    }

    #[test]
    fn direct_link_and_unix_arguments_are_unchanged() {
        let input = args(&[
            "/SUBSYSTEM:windows",
            "/ENTRY:mainCRTStartup",
            "object with spaces.obj",
        ]);

        assert_eq!(
            render(&input, Linker::Msvc, Driver::MsvcLink)
                .unwrap()
                .linker,
            input
        );
        assert_eq!(
            render(&input, Linker::Unix, Driver::C).unwrap().driver,
            input
        );
    }

    #[test]
    fn unsupported_mingw_dialect_options_are_diagnosed() {
        let error = render(&args(&["/PDB:file.pdb"]), Linker::MinGw, Driver::C).unwrap_err();

        assert!(error.contains("/PDB:file.pdb") && error.contains("MinGW"));
    }

    #[test]
    fn driver_detection_handles_native_and_cross_host_paths() {
        for program in [
            "clang-cl",
            "C:\\LLVM tools\\CLANG-CL.EXE",
            "/LLVM tools/clang-cl.exe",
        ] {
            assert_eq!(driver_for_program(program), Driver::ClangCl);
        }

        assert_eq!(driver_for_program("lld-link.exe"), Driver::MsvcLink);
        assert_eq!(driver_for_program("aarch64-w64-mingw32-clang"), Driver::C);
        assert!(is_clang("C:\\LLVM tools\\clang.exe"));
        assert!(is_clang("aarch64-w64-mingw32-clang"));
        assert!(is_clang("clang-23"));
        assert!(!is_clang("x86_64-w64-mingw32-gcc"));
        assert!(!is_clang("lld-link"));
    }

    #[test]
    fn compiler_option_operands_and_slash_paths_are_not_reinterpreted() {
        let input = args(&[
            "-Xclang",
            "/DEBUG",
            "-o",
            "/OUT:literal",
            "/absolute dir/a.obj",
            "/input:piece.obj",
            "-Xlinker",
            "/ENTRY:explicit",
        ]);

        assert_eq!(
            render(&input, Linker::MinGw, Driver::C).unwrap().driver,
            input
        );
        assert!(render(&args(&["-Xlinker"]), Linker::Msvc, Driver::C).is_err());
    }

    #[test]
    fn explicit_clang_driver_mode_selects_the_actual_argument_boundary() {
        assert_eq!(
            driver_for_command("clang", &args(&["--driver-mode=cl"])),
            Driver::ClangCl
        );
        assert_eq!(
            driver_for_command("clang-cl", &args(&["--driver-mode=gcc"])),
            Driver::C
        );
        assert_eq!(
            driver_for_command("clang", &args(&["--driver-mode=cl", "--driver-mode=g++"])),
            Driver::C
        );
        assert_eq!(
            driver_for_command("lld-link", &args(&["--driver-mode=cl"])),
            Driver::MsvcLink
        );
        assert_eq!(
            driver_for_command("clang", &args(&["-Xlinker", "--driver-mode=cl"])),
            Driver::C
        );
        assert_eq!(
            driver_for_command("clang-cl", &args(&["/link", "--driver-mode=gcc"])),
            Driver::ClangCl
        );
    }
}
