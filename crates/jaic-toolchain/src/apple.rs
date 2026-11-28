//! The selected Xcode (or Command Line Tools) on a Mac: its linker, SDK, compiler runtime and
//! `libclang.dylib`, found the way `xcode-select -p` finds the developer folder.
//!
//! No crate finds these without starting `xcrun`, and starting it costs what the direct `ld` link
//! saves (`docs/native/native-linking.md`), so the folders are read here, through [`Host`].
use std::path::{Path, PathBuf};

use jacket_config::env;

use crate::found::Missing;
use crate::host::Host;

/// Where `xcode-select` saves its selection.
const SELECTION_LINK: &str = "/var/db/xcode_select_link";

const XCODE_APP: &str = "/Applications/Xcode.app/Contents/Developer";
const COMMAND_LINE_TOOLS: &str = "/Library/Developer/CommandLineTools";

/// The developer folder, in the order `xcode-select -p` uses: `DEVELOPER_DIR`, the saved
/// selection, an Xcode in `/Applications`, then the Command Line Tools.
pub fn developer_dir(host: &dyn Host) -> PathBuf {
    host.var(&env::DEVELOPER_DIR)
        .map(PathBuf::from)
        .or_else(|| host.read_link(Path::new(SELECTION_LINK)))
        .or_else(|| {
            let xcode = PathBuf::from(XCODE_APP);
            host.is_dir(&xcode).then_some(xcode)
        })
        .unwrap_or_else(|| PathBuf::from(COMMAND_LINE_TOOLS))
}

/// The two kinds of developer folder. Each is one unit: the `usr` and the SDK of a folder always
/// come from the same kind, so a half-installed folder is reported rather than mixed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Layout {
    /// An Xcode: `Toolchains/XcodeDefault.xctoolchain/usr` and the `MacOSX.platform` SDK.
    Xcode,
    /// The Command Line Tools: `usr` and `SDKs/MacOSX.sdk` at the top.
    CommandLineTools,
}

impl Layout {
    /// An Xcode has a default toolchain folder; the Command Line Tools do not.
    fn of(host: &dyn Host, developer: &Path) -> Layout {
        if host.is_dir(&developer.join("Toolchains/XcodeDefault.xctoolchain")) {
            Layout::Xcode
        } else {
            Layout::CommandLineTools
        }
    }

    fn usr(self, developer: &Path) -> PathBuf {
        match self {
            Layout::Xcode => developer.join("Toolchains/XcodeDefault.xctoolchain/usr"),
            Layout::CommandLineTools => developer.join("usr"),
        }
    }

    fn sdk(self, developer: &Path) -> PathBuf {
        match self {
            Layout::Xcode => developer.join("Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk"),
            Layout::CommandLineTools => developer.join("SDKs/MacOSX.sdk"),
        }
    }
}

/// What to do when the selected developer folder lacks a piece.
fn select_note() -> String {
    format!(
        "select a complete Xcode with `xcode-select -s {XCODE_APP}`, install the Command Line \
         Tools with `xcode-select --install`, or set DEVELOPER_DIR"
    )
}

/// The selected toolchain's library folder, which must hold `libclang.dylib`. The
/// `Bindings_Generator` parses headers with it, and the release archive does not carry a second
/// copy (docs/tools/releases.md). Only the selected developer folder is searched, so the
/// toolchain's other libraries are never picked up by name and another Xcode is never used.
pub fn libclang_dir(host: &dyn Host) -> Result<PathBuf, Missing> {
    let developer = developer_dir(host);
    let lib = Layout::of(host, &developer).usr(&developer).join("lib");

    if host.is_file(&lib.join("libclang.dylib")) {
        Ok(lib)
    } else {
        Err(Missing::new(
            format!(
                "the selected developer folder {} has no {}",
                developer.display(),
                lib.join("libclang.dylib").display()
            ),
            vec![select_note()],
        ))
    }
}

/// The linker, SDK and runtime of the selected developer folder.
#[derive(Debug, PartialEq, Eq)]
pub struct Xcode {
    pub ld: PathBuf,
    pub sdk: PathBuf,
    /// `usr/lib/clang`, one folder per compiler version.
    clang_lib: PathBuf,
}

impl Xcode {
    /// The selected developer folder's tools, or what it lacks. `SDKROOT` names the SDK in place
    /// of the folder's own, as it does for `cc`.
    pub fn find(host: &dyn Host) -> Result<Xcode, Missing> {
        let developer = developer_dir(host);
        let layout = Layout::of(host, &developer);
        let usr = layout.usr(&developer);
        let ld = usr.join("bin/ld");
        let sdk = match host.var(&env::SDKROOT).filter(|v| !v.is_empty()) {
            Some(root) => {
                let root = PathBuf::from(root);

                if !host.is_dir(&root) {
                    return Err(Missing::new(
                        format!("SDKROOT names {}, which is not a folder", root.display()),
                        vec!["point SDKROOT at a MacOSX.sdk folder, or unset SDKROOT".into()],
                    ));
                }

                root
            }
            None => layout.sdk(&developer),
        };
        let lacking = [(host.is_file(&ld), &ld), (host.is_dir(&sdk), &sdk)]
            .into_iter()
            .filter(|(present, _)| !present)
            .map(|(_, path)| path.display().to_string())
            .collect::<Vec<_>>();

        if !lacking.is_empty() {
            return Err(Missing::new(
                format!(
                    "the selected developer folder {} has no {}",
                    developer.display(),
                    lacking.join(" and no ")
                ),
                vec![select_note()],
            ));
        }

        Ok(Xcode {
            ld,
            sdk,
            clang_lib: usr.join("lib/clang"),
        })
    }

    /// The SDK's own version as its `SDKSettings.json` spells it (`"Version":"27.0"`), which is
    /// what `cc` passes `ld`.
    pub fn sdk_version(&self, host: &dyn Host) -> Option<String> {
        let settings = host.read_to_string(&self.sdk.join("SDKSettings.json"))?;
        let version = settings.split_once("\"Version\":\"")?.1.split_once('"')?.0;

        version
            .bytes()
            .all(|b| b.is_ascii_digit() || b == b'.')
            .then(|| version.to_string())
    }

    /// `libclang_rt.osx.a` of the newest compiler version installed.
    pub fn compiler_rt(&self, host: &dyn Host) -> Option<PathBuf> {
        let mut found: Vec<(Vec<u32>, PathBuf)> = host
            .read_dir(&self.clang_lib)
            .into_iter()
            .filter_map(|dir| {
                let key = dir
                    .file_name()?
                    .to_str()?
                    .split('.')
                    .map(|n| n.parse().ok())
                    .collect::<Option<_>>()?;
                let lib = dir.join("lib/darwin/libclang_rt.osx.a");

                host.is_file(&lib).then_some((key, lib))
            })
            .collect();

        found.sort();
        found.pop().map(|(_, lib)| lib)
    }
}

/// The version of the running macOS, three numbers, as `cc` passes it to `ld`.
pub fn macos_version(host: &dyn Host) -> Option<String> {
    let plist = host.read_to_string(Path::new(
        "/System/Library/CoreServices/SystemVersion.plist",
    ))?;
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

/// The macOS version a program is linked for, as `cc` passes it to `ld`:
/// `MACOSX_DEPLOYMENT_TARGET`, else the running macOS.
pub fn deployment_target(host: &dyn Host) -> Result<String, Missing> {
    match host.var(&env::MACOSX_DEPLOYMENT_TARGET).filter(|v| !v.is_empty()) {
        Some(value) => {
            let text = value.to_string_lossy();

            three_numbers(&text).ok_or_else(|| {
                Missing::new(
                    format!("MACOSX_DEPLOYMENT_TARGET is `{text}`, which is not a macOS version"),
                    vec!["set it to a version such as `13.0`, or unset it".into()],
                )
            })
        }
        None => macos_version(host).ok_or_else(|| {
            Missing::new(
                "cannot read the macOS version from /System/Library/CoreServices/SystemVersion.plist",
                vec!["set MACOSX_DEPLOYMENT_TARGET to the version to link for".into()],
            )
        }),
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
    use crate::testing::FakeHost;

    const TOOLCHAIN: &str =
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr";

    #[test]
    fn the_developer_folder_follows_xcode_select() {
        let mut host = FakeHost::mac();

        assert_eq!(developer_dir(&host), PathBuf::from(COMMAND_LINE_TOOLS));
        host.dir(XCODE_APP);
        assert_eq!(developer_dir(&host), PathBuf::from(XCODE_APP));
        host.link(
            SELECTION_LINK,
            "/Applications/Xcode-beta.app/Contents/Developer",
        );
        assert_eq!(
            developer_dir(&host),
            PathBuf::from("/Applications/Xcode-beta.app/Contents/Developer")
        );
        host.set(&env::DEVELOPER_DIR, "/opt/xcode");
        assert_eq!(developer_dir(&host), PathBuf::from("/opt/xcode"));
    }

    #[test]
    fn an_xcode_has_its_toolchain_and_platform_sdk() {
        let mut host = FakeHost::mac();

        host.dir(XCODE_APP);
        host.file(&format!("{TOOLCHAIN}/bin/ld"));
        let missing = Xcode::find(&host).unwrap_err().to_string();

        assert!(missing.contains("MacOSX.platform"), "{missing}");
        assert!(missing.contains("xcode-select -s"), "{missing}");
        let sdk = format!("{XCODE_APP}/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk");

        host.text(
            &format!("{sdk}/SDKSettings.json"),
            r#"{"Version":"27.0","x":1}"#,
        );

        for version in ["17.0.0", "9.1", "18"] {
            host.file(&format!(
                "{TOOLCHAIN}/lib/clang/{version}/lib/darwin/libclang_rt.osx.a"
            ));
        }

        let xcode = Xcode::find(&host).unwrap();

        assert_eq!(xcode.ld, PathBuf::from(format!("{TOOLCHAIN}/bin/ld")));
        assert_eq!(xcode.sdk, PathBuf::from(&sdk));
        assert_eq!(xcode.sdk_version(&host).as_deref(), Some("27.0"));
        assert_eq!(
            xcode.compiler_rt(&host),
            Some(PathBuf::from(format!(
                "{TOOLCHAIN}/lib/clang/18/lib/darwin/libclang_rt.osx.a"
            )))
        );
    }

    #[test]
    fn a_half_installed_xcode_is_reported_not_mixed_with_the_other_layout() {
        // An Xcode whose toolchain lost `ld`, beside a `usr` and SDK in the Command Line Tools
        // places: the Command Line Tools parts are not borrowed.
        let mut host = FakeHost::mac();

        host.dir(&format!("{TOOLCHAIN}/bin"));
        host.file(&format!("{XCODE_APP}/usr/bin/ld"));
        host.dir(&format!("{XCODE_APP}/SDKs/MacOSX.sdk"));
        let missing = Xcode::find(&host).unwrap_err().to_string();

        assert!(
            missing.contains(&format!("{TOOLCHAIN}/bin/ld")),
            "{missing}"
        );
        assert!(missing.contains("MacOSX.platform"), "{missing}");
    }

    #[test]
    fn the_command_line_tools_have_their_own_usr_and_sdk() {
        let mut host = FakeHost::mac();

        host.file(&format!("{COMMAND_LINE_TOOLS}/usr/bin/ld"));
        host.dir(&format!("{COMMAND_LINE_TOOLS}/SDKs/MacOSX.sdk"));
        host.file(&format!("{COMMAND_LINE_TOOLS}/usr/lib/libclang.dylib"));
        let xcode = Xcode::find(&host).unwrap();

        assert_eq!(
            xcode.ld,
            PathBuf::from(format!("{COMMAND_LINE_TOOLS}/usr/bin/ld"))
        );
        assert_eq!(
            libclang_dir(&host).unwrap(),
            PathBuf::from(format!("{COMMAND_LINE_TOOLS}/usr/lib"))
        );
    }

    #[test]
    fn libclang_is_taken_only_from_the_selected_folder() {
        let mut host = FakeHost::mac();

        host.file(&format!("{TOOLCHAIN}/lib/libclang.dylib"));
        // The Command Line Tools are selected, so the Xcode copy is not used, and the error
        // names the selected folder and how to choose another.
        host.link(SELECTION_LINK, COMMAND_LINE_TOOLS);
        let missing = libclang_dir(&host).unwrap_err().to_string();

        assert!(
            missing.starts_with(&format!(
                "the selected developer folder {COMMAND_LINE_TOOLS} has no {COMMAND_LINE_TOOLS}/usr/lib/libclang.dylib"
            )),
            "{missing}"
        );
        assert!(
            missing.contains("note: select a complete Xcode"),
            "{missing}"
        );
        host.link(SELECTION_LINK, XCODE_APP);
        assert_eq!(
            libclang_dir(&host).unwrap(),
            PathBuf::from(format!("{TOOLCHAIN}/lib"))
        );
    }

    #[test]
    fn sdkroot_names_the_sdk_cc_would_use() {
        let mut host = FakeHost::mac();

        host.file(&format!("{COMMAND_LINE_TOOLS}/usr/bin/ld"));
        host.dir(&format!("{COMMAND_LINE_TOOLS}/SDKs/MacOSX.sdk"));
        host.dir("/sdks/MacOSX14.sdk");
        host.set(&env::SDKROOT, "/sdks/MacOSX14.sdk");
        assert_eq!(
            Xcode::find(&host).unwrap().sdk,
            PathBuf::from("/sdks/MacOSX14.sdk")
        );
        host.set(&env::SDKROOT, "/sdks/gone.sdk");
        let missing = Xcode::find(&host).unwrap_err().to_string();

        assert!(
            missing.starts_with("SDKROOT names /sdks/gone.sdk, which is not a folder"),
            "{missing}"
        );
        assert!(missing.contains("unset SDKROOT"), "{missing}");
    }

    #[test]
    fn the_deployment_target_is_the_variable_else_the_running_macos() {
        let mut host = FakeHost::mac();

        host.text(
            "/System/Library/CoreServices/SystemVersion.plist",
            "<key>ProductVersion</key>\n\t<string>26.1</string>",
        );
        assert_eq!(deployment_target(&host).unwrap(), "26.1.0");
        host.set(&env::MACOSX_DEPLOYMENT_TARGET, "13.3");
        assert_eq!(deployment_target(&host).unwrap(), "13.3.0");
        host.set(&env::MACOSX_DEPLOYMENT_TARGET, "thirteen");
        let missing = deployment_target(&host).unwrap_err().to_string();

        assert!(
            missing.starts_with("MACOSX_DEPLOYMENT_TARGET is `thirteen`"),
            "{missing}"
        );
    }

    #[test]
    fn versions_have_three_numbers() {
        assert_eq!(three_numbers("27.0").as_deref(), Some("27.0.0"));
        assert_eq!(three_numbers("15.6.1").as_deref(), Some("15.6.1"));
        assert_eq!(three_numbers("15"), None);
        assert_eq!(three_numbers("15.x"), None);
        let mut host = FakeHost::mac();

        host.text(
            "/System/Library/CoreServices/SystemVersion.plist",
            "<key>ProductVersion</key>\n\t<string>26.1</string>",
        );
        assert_eq!(macos_version(&host).as_deref(), Some("26.1.0"));
    }
}
