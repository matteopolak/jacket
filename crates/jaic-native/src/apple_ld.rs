//! Linking on a Mac without the `cc` driver.
//!
//! `cc` is a shim that finds Xcode, starts `clang`, and `clang` starts `ld` with the SDK, the
//! platform version and the runtime library filled in: about 8 ms for a small program, a sixth of
//! its whole build. For the usual case this module writes that `ld` command itself. Anything it does
//! not recognise returns `None` (or fails in the linker), and [`super::link`] runs `cc` as before.
use std::path::PathBuf;
use std::process::Command;

/// The arguments `cc` would pass `ld` for `args`, or `None` when `args` hold anything but the
/// forms a native link uses (`-o`, `-l`, `-L`, `-framework`, `-Wl,`) or the toolchain is not where
/// Xcode puts it.
pub(super) fn direct_command(args: &[String]) -> Option<Command> {
    if jacket_config::env::SDKROOT.os().is_some()
        || jacket_config::env::MACOSX_DEPLOYMENT_TARGET.os().is_some()
    {
        return None;
    }
    let toolchain = Toolchain::find()?;
    let mut cmd = Command::new(&toolchain.ld);
    cmd.args(["-dynamic", "-arch", arch()?, "-platform_version", "macos"])
        .arg(toolchain.minimum_os()?)
        .arg(toolchain.sdk_version()?)
        .arg("-syslibroot")
        .arg(&toolchain.sdk)
        .arg("-L/usr/local/lib");
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        match arg.as_str() {
            "-o" | "-framework" => {
                cmd.arg(arg).arg(rest.next()?);
            }
            _ if arg.starts_with("-l") || arg.starts_with("-L") => {
                cmd.arg(arg);
            }
            _ if arg.starts_with("-Wl,") => {
                cmd.args(arg["-Wl,".len()..].split(','));
            }
            _ if arg.starts_with('-') => return None,
            _ => {
                cmd.arg(arg);
            }
        }
    }
    cmd.arg("-lSystem").arg(toolchain.compiler_rt()?);
    Some(cmd)
}

fn arch() -> Option<&'static str> {
    match std::env::consts::ARCH {
        "aarch64" => Some("arm64"),
        "x86_64" => Some("x86_64"),
        _ => None,
    }
}

/// Where the selected Xcode (or Command Line Tools) keeps the linker, the SDK and the runtime.
struct Toolchain {
    ld: PathBuf,
    sdk: PathBuf,
    /// `usr/lib/clang`, one folder per compiler version.
    clang_lib: PathBuf,
}

impl Toolchain {
    fn find() -> Option<Self> {
        // The order `xcode-select -p` uses: the environment, the selection it saved, then an
        // Xcode in /Applications, then the Command Line Tools.
        let developer = jacket_config::env::DEVELOPER_DIR
            .os()
            .map(PathBuf::from)
            .or_else(|| std::fs::read_link("/var/db/xcode_select_link").ok())
            .or_else(|| {
                let xcode = PathBuf::from("/Applications/Xcode.app/Contents/Developer");
                xcode.is_dir().then_some(xcode)
            })
            .unwrap_or_else(|| PathBuf::from("/Library/Developer/CommandLineTools"));
        let xcode = developer.join("Toolchains/XcodeDefault.xctoolchain/usr");
        let (usr, sdk) = if xcode.join("bin/ld").is_file() {
            (
                xcode,
                developer.join("Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"),
            )
        } else {
            (developer.join("usr"), developer.join("SDKs/MacOSX.sdk"))
        };
        let ld = usr.join("bin/ld");
        (ld.is_file() && sdk.is_dir()).then(|| Self {
            ld,
            sdk,
            clang_lib: usr.join("lib/clang"),
        })
    }

    /// The version of the running macOS, as `cc` passes it: three numbers.
    fn minimum_os(&self) -> Option<String> {
        let plist =
            std::fs::read_to_string("/System/Library/CoreServices/SystemVersion.plist").ok()?;
        let version = plist
            .split_once("<key>ProductVersion</key>")?
            .1
            .split_once("<string>")?
            .1
            .split_once("</string>")?
            .0
            .trim();
        three_numbers(version)
    }

    /// The SDK's own version, from its `SDKSettings.json` (`"Version":"27.0"`).
    fn sdk_version(&self) -> Option<String> {
        let settings = std::fs::read_to_string(self.sdk.join("SDKSettings.json")).ok()?;
        let version = settings.split_once("\"Version\":\"")?.1.split_once('"')?.0;
        version
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'.')
            .then(|| version.to_string())
    }

    /// `libclang_rt.osx.a` of the newest compiler version installed.
    fn compiler_rt(&self) -> Option<PathBuf> {
        let mut found: Vec<(Vec<u32>, PathBuf)> = std::fs::read_dir(&self.clang_lib)
            .ok()?
            .flatten()
            .filter_map(|entry| {
                let lib = entry.path().join("lib/darwin/libclang_rt.osx.a");
                let name = entry.file_name().to_string_lossy().into_owned();
                let key = name
                    .split('.')
                    .map(|n| n.parse().ok())
                    .collect::<Option<_>>()?;
                lib.is_file().then_some((key, lib))
            })
            .collect();
        found.sort();
        found.pop().map(|(_, lib)| lib)
    }
}

/// `27.0` as `27.0.0`; a version with two or three numbers.
fn three_numbers(version: &str) -> Option<String> {
    let parts: Vec<&str> = version.split('.').collect();
    if !(2..=3).contains(&parts.len()) || parts.iter().any(|p| p.parse::<u32>().is_err()) {
        return None;
    }
    let mut text = version.to_string();
    if parts.len() == 2 {
        text.push_str(".0");
    }
    Some(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versions_have_three_numbers() {
        assert_eq!(three_numbers("27.0").as_deref(), Some("27.0.0"));
        assert_eq!(three_numbers("15.6.1").as_deref(), Some("15.6.1"));
        assert_eq!(three_numbers("15"), None);
        assert_eq!(three_numbers("15.x"), None);
    }

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
        .map(String::from);
        if let Some(cmd) = direct_command(&ok) {
            let args: Vec<String> = cmd
                .get_args()
                .map(|a| a.to_string_lossy().into_owned())
                .collect();
            let tail = args.join(" ");
            assert!(tail.contains("-rpath /x"), "{tail}");
            assert!(tail.contains("-framework AppKit"), "{tail}");
            assert!(tail.contains("-lSystem"), "{tail}");
        }
        assert!(direct_command(&["a.o".into(), "-shared".into()]).is_none());
    }
}
