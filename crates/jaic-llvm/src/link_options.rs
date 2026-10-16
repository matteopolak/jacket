//! Build-option arguments cross two boundaries: compiler driver and target linker.
use super::LinkFlavor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Driver {
    C,
    ClangCl,
    MsvcLink,
}

impl Driver {
    pub(super) fn for_command(program: &str, args: &[String]) -> Self {
        let mut driver = Self::for_program(program);
        if driver != Self::MsvcLink && is_clang(program) {
            let mut args = args.iter();
            while let Some(arg) = args.next() {
                if takes_driver_operand(arg) {
                    args.next();
                    continue;
                }
                if driver == Self::ClangCl && arg.eq_ignore_ascii_case("/link") {
                    break;
                }
                match arg.as_str() {
                    "--driver-mode=cl" => driver = Self::ClangCl,
                    "--driver-mode=gcc" | "--driver-mode=g++" => driver = Self::C,
                    _ => {}
                }
            }
        }
        driver
    }

    pub(super) fn for_program(program: &str) -> Self {
        let name = program.rsplit(['/', '\\']).next().unwrap_or(program);
        let name = name.to_ascii_lowercase();
        let name = name.strip_suffix(".exe").unwrap_or(&name);
        match name {
            "link" | "lld-link" => Self::MsvcLink,
            "clang-cl" => Self::ClangCl,
            _ => Self::C,
        }
    }
}

fn takes_driver_operand(arg: &str) -> bool {
    matches!(
        arg,
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
}

pub(super) fn is_clang(program: &str) -> bool {
    let name = program
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(program)
        .to_ascii_lowercase();
    let name = name.strip_suffix(".exe").unwrap_or(&name);
    matches!(name, "clang" | "clang++" | "clang-cl")
        || name.ends_with("-clang")
        || name.ends_with("-clang++")
        || name
            .strip_prefix("clang-")
            .is_some_and(|suffix| !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit()))
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct ExtraArgs {
    pub driver: Vec<String>,
    pub linker: Vec<String>,
}

pub(super) fn render(
    args: &[String],
    flavor: LinkFlavor,
    driver: Driver,
) -> Result<ExtraArgs, String> {
    if driver == Driver::MsvcLink {
        return Ok(ExtraArgs {
            driver: Vec::new(),
            linker: args.to_vec(),
        });
    }
    if driver == Driver::ClangCl && flavor != LinkFlavor::Msvc {
        return Err("clang-cl linking requires an MSVC target; use a GNU-style compiler driver for MinGW targets".into());
    }
    let mut output = ExtraArgs::default();
    let mut explicit_link = false;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if driver == Driver::ClangCl {
            if arg.eq_ignore_ascii_case("/link") {
                explicit_link = true;
            } else if explicit_link || msvc_option(arg).is_some() {
                output.linker.push(arg.clone());
            } else if arg == "-Xlinker" {
                output
                    .linker
                    .push(args.next().ok_or("-Xlinker requires an argument")?.clone());
            } else if let Some(value) = arg.strip_prefix("-Wl,") {
                output.linker.extend(value.split(',').map(str::to_owned));
            } else {
                output.driver.push(arg.clone());
                if takes_driver_operand(arg) {
                    output.driver.push(
                        args.next()
                            .ok_or_else(|| format!("driver option `{arg}` requires an argument"))?
                            .clone(),
                    );
                }
            }
            continue;
        }
        // Explicit driver forwarding is already tokenized. In particular, the
        // operand of -Xlinker is not another implicit build-option argument.
        if takes_driver_operand(arg) {
            output.driver.push(arg.clone());
            output.driver.push(
                args.next()
                    .ok_or_else(|| format!("driver option `{arg}` requires an argument"))?
                    .clone(),
            );
        } else if flavor.is_windows()
            && let Some((name, value)) = msvc_option(arg)
        {
            let value = if flavor == LinkFlavor::MinGw {
                mingw_option(arg, &name, value)?
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

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|v| (*v).into()).collect()
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
            render(&input, LinkFlavor::Msvc, Driver::C).unwrap().driver,
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
            render(&input, LinkFlavor::MinGw, Driver::C).unwrap().driver,
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
            render(&input, LinkFlavor::Msvc, Driver::ClangCl).unwrap(),
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
            render(&input, LinkFlavor::Msvc, Driver::MsvcLink)
                .unwrap()
                .linker,
            input
        );
        assert_eq!(
            render(&input, LinkFlavor::Unix, Driver::C).unwrap().driver,
            input
        );
    }

    #[test]
    fn unsupported_mingw_dialect_options_are_diagnosed() {
        let error = render(&args(&["/PDB:file.pdb"]), LinkFlavor::MinGw, Driver::C).unwrap_err();
        assert!(error.contains("/PDB:file.pdb") && error.contains("MinGW"));
    }

    #[test]
    fn driver_detection_handles_native_and_cross_host_paths() {
        for program in [
            "clang-cl",
            "C:\\LLVM tools\\CLANG-CL.EXE",
            "/LLVM tools/clang-cl.exe",
        ] {
            assert_eq!(Driver::for_program(program), Driver::ClangCl);
        }
        assert_eq!(Driver::for_program("lld-link.exe"), Driver::MsvcLink);
        assert_eq!(Driver::for_program("aarch64-w64-mingw32-clang"), Driver::C);
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
            render(&input, LinkFlavor::MinGw, Driver::C).unwrap().driver,
            input
        );
        assert!(render(&args(&["-Xlinker"]), LinkFlavor::Msvc, Driver::C).is_err());
    }

    #[test]
    fn explicit_clang_driver_mode_selects_the_actual_argument_boundary() {
        assert_eq!(
            Driver::for_command("clang", &args(&["--driver-mode=cl"])),
            Driver::ClangCl
        );
        assert_eq!(
            Driver::for_command("clang-cl", &args(&["--driver-mode=gcc"])),
            Driver::C
        );
        assert_eq!(
            Driver::for_command("clang", &args(&["--driver-mode=cl", "--driver-mode=g++"])),
            Driver::C
        );
        assert_eq!(
            Driver::for_command("lld-link", &args(&["--driver-mode=cl"])),
            Driver::MsvcLink
        );
        assert_eq!(
            Driver::for_command("clang", &args(&["-Xlinker", "--driver-mode=cl"])),
            Driver::C
        );
        assert_eq!(
            Driver::for_command("clang-cl", &args(&["/link", "--driver-mode=gcc"])),
            Driver::ClangCl
        );
    }
}
