//! Linking on a Mac without the `cc` driver.
//!
//! `cc` is a shim that finds Xcode, starts `clang`, and `clang` starts `ld` with the SDK, the
//! platform version and the runtime library filled in: about 8 ms for a small program, a sixth of
//! its whole build. For a plain executable link this module writes that `ld` command itself and
//! [`super::link`] runs it; when `ld` fails, its own message is the error.
use jacket_config::os_text;
use jaic_toolchain::apple::{self, Xcode};
use std::ffi::OsString;
use std::process::Command;

/// The arguments `cc` would pass `ld` for `args`. `Ok(None)` when `args` hold something other
/// than the forms a native link uses (`-o`, `-l`, `-L`, `-framework`, `-Wl,`): only the driver
/// understands those, so that link goes through `cc`. An error when the selected developer
/// folder lacks a piece, or `SDKROOT` or `MACOSX_DEPLOYMENT_TARGET` is wrong.
pub(super) fn direct_command(args: &[OsString]) -> Result<Option<Command>, String> {
    let mut linker_args: Vec<OsString> = Vec::new();
    let mut rest = args.iter();

    while let Some(arg) = rest.next() {
        let text = arg.to_str();

        if matches!(text, Some("-o" | "-framework")) {
            let operand = rest
                .next()
                .ok_or_else(|| format!("`{}` has no operand", arg.display()))?;

            linker_args.push(arg.clone());
            linker_args.push(operand.clone());
        } else if let Some(wl) = os_text::strip_prefix(arg, "-Wl,") {
            linker_args.extend(os_text::split(&wl, b','));
        } else if os_text::strip_prefix(arg, "-l").is_some()
            || os_text::strip_prefix(arg, "-L").is_some()
        {
            linker_args.push(arg.clone());
        } else if os_text::strip_prefix(arg, "-").is_some() {
            return Ok(None);
        } else {
            linker_args.push(arg.clone());
        }
    }

    let host = crate::toolchain::system();
    let xcode = Xcode::find(host).map_err(|missing| missing.to_string())?;
    let sdk_version = xcode.sdk_version(host).ok_or_else(|| {
        format!(
            "cannot read the SDK version from {}",
            xcode.sdk.join("SDKSettings.json").display()
        )
    })?;
    let runtime = xcode
        .compiler_rt(host)
        .ok_or_else(|| format!("no libclang_rt.osx.a for {}", xcode.ld.display()))?;
    let mut cmd = Command::new(&xcode.ld);

    cmd.args(["-dynamic", "-arch", arch()?, "-platform_version", "macos"])
        .arg(apple::deployment_target(host).map_err(|missing| missing.to_string())?)
        .arg(sdk_version)
        .arg("-syslibroot")
        .arg(&xcode.sdk)
        .arg("-L/usr/local/lib")
        .args(linker_args)
        .arg("-lSystem")
        .arg(runtime);
    Ok(Some(cmd))
}

fn arch() -> Result<&'static str, String> {
    match std::env::consts::ARCH {
        "aarch64" => Ok("arm64"),
        "x86_64" => Ok("x86_64"),
        other => Err(format!("`ld` has no name for {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_the_driver_forms_and_leaves_the_rest_to_cc() {
        let ok = [
            "a.o",
            "-o",
            "out",
            "-Wl,-rpath,/x",
            "-framework",
            "AppKit",
            "-lm",
            "-L/y",
        ]
        .map(OsString::from);
        let cmd = direct_command(&ok).unwrap().unwrap();
        let args: Vec<String> = cmd
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let tail = args.join(" ");

        assert!(tail.contains("-rpath /x"), "{tail}");
        assert!(tail.contains("-framework AppKit"), "{tail}");
        assert!(tail.ends_with(".a") && tail.contains("-lSystem"), "{tail}");
        assert!(matches!(
            direct_command(&["a.o".into(), "-shared".into()]),
            Ok(None)
        ));
    }
}
