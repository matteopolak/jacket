//! Linking on a Mac without the `cc` driver.
//!
//! `cc` is a shim that finds Xcode, starts `clang`, and `clang` starts `ld` with the SDK, the
//! platform version and the runtime library filled in: about 8 ms for a small program, a sixth of
//! its whole build. For the usual case this module writes that `ld` command itself. Anything it does
//! not recognise returns the reason (shown under `-verbose`), or fails in the linker, and
//! [`super::link`] runs `cc` as before, whose message and rules then decide what is an error.
use jacket_config::os_text;
use jaic_toolchain::Host;
use jaic_toolchain::apple::{self, Xcode};
use std::ffi::OsString;
use std::process::Command;

/// The arguments `cc` would pass `ld` for `args`, or why it cannot: `args` hold something other
/// than the forms a native link uses (`-o`, `-l`, `-L`, `-framework`, `-Wl,`), the environment
/// asks `cc` for a different SDK, or the selected developer folder lacks a piece.
pub(super) fn direct_command(args: &[OsString]) -> Result<Command, String> {
    let host = crate::toolchain::system();
    for var in [
        &jacket_config::env::SDKROOT,
        &jacket_config::env::MACOSX_DEPLOYMENT_TARGET,
    ] {
        if host.var(var).is_some() {
            return Err(format!("{} is set", var.name));
        }
    }
    let xcode = Xcode::find(host).map_err(|missing| missing.to_string())?;
    let unreadable = |what: &str| format!("cannot read {what}");
    let mut cmd = Command::new(&xcode.ld);
    cmd.args(["-dynamic", "-arch", arch()?, "-platform_version", "macos"])
        .arg(apple::macos_version(host).ok_or_else(|| unreadable("the macOS version"))?)
        .arg(
            xcode
                .sdk_version(host)
                .ok_or_else(|| unreadable(&format!("the version of {}", xcode.sdk.display())))?,
        )
        .arg("-syslibroot")
        .arg(&xcode.sdk)
        .arg("-L/usr/local/lib");
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        let text = arg.to_str();
        if matches!(text, Some("-o" | "-framework")) {
            let operand = rest
                .next()
                .ok_or_else(|| format!("`{}` has no operand", arg.display()))?;
            cmd.arg(arg).arg(operand);
        } else if let Some(linker_args) = os_text::strip_prefix(arg, "-Wl,") {
            cmd.args(os_text::split(&linker_args, b','));
        } else if os_text::strip_prefix(arg, "-l").is_some()
            || os_text::strip_prefix(arg, "-L").is_some()
        {
            cmd.arg(arg);
        } else if os_text::strip_prefix(arg, "-").is_some() {
            return Err(format!("`{}` is a driver option", arg.display()));
        } else {
            cmd.arg(arg);
        }
    }
    let runtime = xcode
        .compiler_rt(host)
        .ok_or_else(|| format!("no libclang_rt.osx.a for {}", xcode.ld.display()))?;
    cmd.arg("-lSystem").arg(runtime);
    Ok(cmd)
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
    fn rewrites_the_driver_forms_and_refuses_the_rest() {
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
        if let Ok(cmd) = direct_command(&ok) {
            let args: Vec<String> = cmd
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            let tail = args.join(" ");
            assert!(tail.contains("-rpath /x"), "{tail}");
            assert!(tail.contains("-framework AppKit"), "{tail}");
            assert!(tail.contains("-lSystem"), "{tail}");
        }
        assert!(direct_command(&["a.o".into(), "-shared".into()]).is_err());
    }
}
