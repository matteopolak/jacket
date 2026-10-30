//! Build outputs a checkout needs before some tests can run. A fresh checkout (a git worktree
//! above all) has none of them, and each missing one fails every test that uses it with a link
//! error or a failed `file_open`; this names them once, with the command that builds each.
//!
//! The two that are plain files of the checkout, the fixture data and the Debug helper, are also
//! built before the run, when a selected test needs one that is missing ([`prepare`]).
use super::Outcome;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A build output a test needs: the script that makes it, and how to tell a test needs it.
struct Setup {
    /// What the output is, for the message.
    what: &'static str,
    /// The script below the checkout root.
    script: &'static str,
    args: &'static [&'static str],
    /// The test needs this when its source names one of these paths below `target/`.
    mentions: &'static [&'static str],
    /// The test needs this when its file is below this folder of the standard library.
    below: Option<&'static str>,
    /// Paths (below the checkout root, with a trailing `*` meaning any file of that prefix) that
    /// exist once it is built.
    outputs: &'static [&'static str],
}

const SETUPS: [Setup; 2] = [
    Setup {
        what: "the fixture data",
        script: "tools/portable/build_portable.py",
        args: &["data"],
        mentions: &[
            "portable-tests/zip/",
            "portable-tests/archive-fixtures/",
            "portable-tests/mpeg/",
        ],
        below: None,
        outputs: &[
            "target/portable-tests/zip",
            "target/portable-tests/archive-fixtures",
            "target/portable-tests/mpeg",
        ],
    },
    Setup {
        what: "the Debug native helper",
        script: "tools/developer_checks/build_developer_debug.py",
        args: &[],
        mentions: &[],
        below: Some("Debug"),
        outputs: &["stdlib/internal/developer-tools/debug-native/libdeveloper_tools_debug.*"],
    },
];

impl Setup {
    fn needed_by(&self, file: &Path, source: &str, stdlib: &Path) -> bool {
        self.mentions.iter().any(|m| source.contains(m))
            || self
                .below
                .is_some_and(|dir| file.starts_with(stdlib.join(dir)))
    }

    /// Whether every output is there below `root`.
    fn built(&self, root: &Path) -> bool {
        self.outputs
            .iter()
            .all(|output| match output.strip_suffix('*') {
                Some(prefix) => {
                    let path = root.join(prefix);
                    let (Some(folder), Some(stem)) = (path.parent(), path.file_name()) else {
                        return false;
                    };
                    std::fs::read_dir(folder).is_ok_and(|entries| {
                        entries.flatten().any(|e| {
                            e.file_name()
                                .to_string_lossy()
                                .starts_with(&*stem.to_string_lossy())
                        })
                    })
                }
                None => root.join(output).exists(),
            })
    }
}

/// The setups the `files` need whose outputs are missing below `root`, the checkout the
/// standard library `stdlib` belongs to. Nothing in a package without the build scripts.
fn needed(files: &[PathBuf], stdlib: &Path, root: &Path) -> Vec<&'static Setup> {
    let mut found: Vec<&'static Setup> = Vec::new();
    for setup in &SETUPS {
        if !root.join(setup.script).is_file() || setup.built(root) {
            continue;
        }
        let wanted = files.iter().any(|file| {
            let source = std::fs::read_to_string(file).unwrap_or_default();
            setup.needed_by(file, &source, stdlib)
        });
        if wanted {
            found.push(setup);
        }
    }
    found
}

/// Builds the outputs that tests among `files` read and a fresh checkout lacks, once, before
/// anything runs. It costs a few file checks when they are there. A script that cannot run is a
/// warning; the tests then fail as they would have, and the list of missing build outputs at the
/// end says what to run.
pub fn prepare(files: &[PathBuf], stdlib: &Path) {
    let Some(root) = stdlib.parent() else {
        return;
    };
    for setup in needed(files, stdlib, root) {
        eprintln!(
            "jacket: building {} (first run in this checkout)",
            setup.what
        );
        let status = Command::new("python3")
            .arg(root.join(setup.script))
            .args(setup.args)
            .env("PYTHONPATH", root.join("tools"))
            .current_dir(root)
            .output();
        match status {
            Ok(output) if output.status.success() => {}
            Ok(output) => eprintln!(
                "jacket: warning: could not build {}: {}",
                setup.what,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
            Err(error) => eprintln!(
                "jacket: warning: could not start python3 for {}: {error}",
                setup.what
            ),
        }
    }
}

const DEBUG_HELPER: &str = "developer_tools_debug";

/// Libraries the window and thread test drivers link; `window_render_native.py` builds them.
const WINDOW_DRIVER_LIBRARIES: [&str; 4] = [
    "window_clipboard_probe",
    "window_gl_probe",
    "unmapping_protection",
    "thread_sdk_identity",
];

/// Where a missing native library comes from: the stdlib's own libraries, or the probes that
/// test drivers link, which `build_native_libs.py` does not build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Source {
    Stdlib,
    WindowDrivers,
    BindingProbes,
}

impl Source {
    /// The group of library `name`, with any directory in front of it ignored.
    fn of(name: &str) -> Self {
        let name = name.rsplit('/').next().unwrap_or(name);
        if WINDOW_DRIVER_LIBRARIES.contains(&name) {
            Source::WindowDrivers
        } else if name.ends_with("_probe") {
            Source::BindingProbes
        } else {
            Source::Stdlib
        }
    }

    /// The hint line for the `names` of this group.
    fn hint(self, names: &[&str]) -> String {
        let list = names.join(", ");
        match self {
            Source::Stdlib => format!(
                "native libraries not built ({list}): PYTHONPATH=tools python3 tools/native/build_native_libs.py"
            ),
            Source::WindowDrivers => format!(
                "test-driver probe libraries not built ({list}): PYTHONPATH=tools python3 tools/window_render/window_render_native.py, then add target/window-native to JAIC_NATIVE_LIBS"
            ),
            Source::BindingProbes => format!(
                "test-driver probe libraries not built ({list}): PYTHONPATH=tools python3 tools/native/build_ci_native_bindings.py --output target/production-bindings/native, then add that directory to JAIC_NATIVE_LIBS"
            ),
        }
    }
}

/// One hint line for each group of missing libraries, each with the command that builds it.
fn library_hints(libraries: &BTreeSet<String>) -> Vec<String> {
    [Source::Stdlib, Source::WindowDrivers, Source::BindingProbes]
        .into_iter()
        .filter_map(|source| {
            let names: Vec<&str> = libraries
                .iter()
                .map(String::as_str)
                .filter(|name| Source::of(name) == source)
                .collect();
            (!names.is_empty()).then(|| source.hint(&names))
        })
        .collect()
}

/// One line for each build output the failed tests were missing, in a stable order.
pub fn missing(outcomes: &[&Outcome]) -> Vec<String> {
    let mut libraries = BTreeSet::new();
    let (mut debug_helper, mut fixtures) = (false, false);
    for outcome in outcomes {
        let texts = outcome
            .problems
            .iter()
            .cloned()
            .chain([String::from_utf8_lossy(&outcome.stderr).into_owned()]);
        for text in texts {
            for name in missing_libraries(&text) {
                if name.ends_with(DEBUG_HELPER) {
                    debug_helper = true;
                } else {
                    libraries.insert(name);
                }
            }
            fixtures |= text.contains("file_open failed");
        }
    }
    let mut lines = library_hints(&libraries);
    if debug_helper {
        lines.push(format!(
            "`{DEBUG_HELPER}` not built: PYTHONPATH=tools python3 tools/developer_checks/build_developer_debug.py"
        ));
    }
    if fixtures {
        lines.push(
            "a test could not open a file; if it reads generated fixtures, build them: PYTHONPATH=tools python3 tools/portable/build_portable.py data"
                .into(),
        );
    }
    lines
}

/// The names in `library 'NAME' not found` (the linker's and jaic's own wording).
fn missing_libraries(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("library '") {
        rest = &rest[at + "library '".len()..];
        let Some(end) = rest.find('\'') else {
            break;
        };
        if rest[end + 1..].starts_with(" not found") {
            names.push(rest[..end].to_string());
        }
        rest = &rest[end..];
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_come_from_both_wordings() {
        let text = "ld: library 'mbedtls' not found\nerror: library 'debug-native/developer_tools_debug' not found: looked";
        assert_eq!(
            missing_libraries(text),
            ["mbedtls", "debug-native/developer_tools_debug"]
        );
        assert!(missing_libraries("library 'x' is fine").is_empty());
    }

    #[test]
    fn a_missing_fixture_or_helper_is_built_only_for_the_tests_that_read_it() {
        let root = std::env::temp_dir().join(format!("jacket-setup-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let stdlib = root.join("stdlib");
        std::fs::create_dir_all(stdlib.join("Debug/tests")).unwrap();
        std::fs::create_dir_all(stdlib.join("Zip/tests")).unwrap();
        std::fs::create_dir_all(root.join("tools/portable")).unwrap();
        std::fs::create_dir_all(root.join("tools/developer_checks")).unwrap();
        std::fs::write(root.join("tools/portable/build_portable.py"), "").unwrap();
        std::fs::write(
            root.join("tools/developer_checks/build_developer_debug.py"),
            "",
        )
        .unwrap();
        let reads = stdlib.join("Zip/tests/reads.jai");
        std::fs::write(
            &reads,
            "x := \"%/../../../target/portable-tests/zip/a.zip\";",
        )
        .unwrap();
        let plain = stdlib.join("Zip/tests/plain.jai");
        std::fs::write(&plain, "main :: () {}").unwrap();
        let debug = stdlib.join("Debug/tests/ownership.jai");
        std::fs::write(&debug, "main :: () {}").unwrap();

        let names = |files: &[PathBuf]| -> Vec<&'static str> {
            needed(files, &stdlib, &root)
                .iter()
                .map(|s| s.what)
                .collect()
        };
        assert!(names(std::slice::from_ref(&plain)).is_empty());
        assert_eq!(names(std::slice::from_ref(&reads)), ["the fixture data"]);
        assert_eq!(
            names(std::slice::from_ref(&debug)),
            ["the Debug native helper"]
        );
        assert_eq!(names(&[reads, debug]).len(), 2);

        // Built outputs, or no build scripts (a package), need nothing.
        std::fs::create_dir_all(root.join("target/portable-tests/zip")).unwrap();
        std::fs::create_dir_all(root.join("target/portable-tests/archive-fixtures")).unwrap();
        std::fs::create_dir_all(root.join("target/portable-tests/mpeg")).unwrap();
        assert!(names(&[stdlib.join("Zip/tests/reads.jai")]).is_empty());
        std::fs::remove_dir_all(root.join("tools")).unwrap();
        assert!(names(&[stdlib.join("Debug/tests/ownership.jai")]).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn probe_libraries_name_the_command_that_builds_them() {
        let libraries: BTreeSet<String> = [
            "mbedtls",
            "window_gl_probe",
            "thread_sdk_identity",
            "rpmalloc_adaptive_probe",
            "sdl_abi_probe",
        ]
        .map(String::from)
        .into();
        let lines = library_hints(&libraries);
        assert_eq!(lines.len(), 3, "{lines:?}");
        assert!(lines[0].starts_with("native libraries not built (mbedtls):"));
        assert!(lines[0].ends_with("tools/native/build_native_libs.py"));
        assert!(lines[1].contains("(thread_sdk_identity, window_gl_probe)"));
        assert!(lines[1].contains("tools/window_render/window_render_native.py"));
        assert!(lines[2].contains("(rpmalloc_adaptive_probe, sdl_abi_probe)"));
        assert!(lines[2].contains("tools/native/build_ci_native_bindings.py"));
        assert!(
            lines[1..]
                .iter()
                .all(|line| !line.contains("build_native_libs"))
        );
    }
}
