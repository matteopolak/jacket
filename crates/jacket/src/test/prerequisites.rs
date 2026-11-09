//! Build outputs a checkout needs before some tests can run. A fresh checkout (a git worktree
//! above all) has none of them, and each missing one fails every test that uses it with a link
//! error or a failed `file_open`; this names them once, with the command that builds each.
//!
//! The ones the checkout can build itself, the fixture and conformance data and the Debug helper,
//! are also built before the run, when a selected test needs one that is missing ([`prepare`]).
use super::options::TestOptions;
use super::{Outcome, Status};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where a setup's outputs are.
#[derive(Clone, Copy)]
enum Under {
    /// The output directory the tools write to and the tests read from: `target/` of the checkout,
    /// or `JACKET_OUTPUT_DIR` ([`output_dir`]).
    Output,
    /// The checkout itself.
    Checkout,
}

/// A build output a test needs: the script that makes it, and how to tell a test needs it.
struct Setup {
    /// What the output is, for the message.
    what: &'static str,
    /// The script below the checkout root.
    script: &'static str,
    args: &'static [&'static str],
    /// The test needs this when its source names one of these.
    mentions: &'static [&'static str],
    /// The test needs this when its file is below this folder of the standard library.
    below: Option<&'static str>,
    under: Under,
    /// Paths (below `under`, with a trailing `*` meaning any file of that prefix) that exist once
    /// it is built.
    outputs: &'static [&'static str],
}

/// The fixtures `build_portable.py data` writes get a setup each. One run builds them all (the
/// others are skipped once it has run, [`prepare_with`]), but each is checked for the files its
/// tests read. A test of a new generated directory needs an entry here; the test
/// `every_generated_directory_the_stdlib_tests_read_has_a_setup` finds a missing one.
const SETUPS: [Setup; 8] = [
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
        under: Under::Output,
        outputs: &[
            "portable-tests/zip",
            "portable-tests/archive-fixtures",
            "portable-tests/mpeg",
        ],
    },
    Setup {
        what: "the machine fixture data",
        script: "tools/portable/build_portable.py",
        args: &["data"],
        mentions: &["portable-machine/"],
        below: None,
        under: Under::Output,
        outputs: &["portable-machine/cpuid.bin"],
    },
    Setup {
        what: "the time zone data",
        script: "tools/portable/build_portable.py",
        args: &["data"],
        mentions: &["time-conformance/"],
        below: None,
        under: Under::Output,
        outputs: &["time-conformance/vectors.bin", "time-conformance/zoneinfo"],
    },
    Setup {
        what: "the CSV conformance data",
        script: "tools/portable/build_portable.py",
        args: &["data"],
        mentions: &["csv-conformance/"],
        below: None,
        under: Under::Output,
        outputs: &["csv-conformance/csvs", "csv-conformance/json"],
    },
    Setup {
        what: "the Unicode text conformance data",
        script: "tools/portable/build_portable.py",
        args: &["data"],
        mentions: &["unicode-text-conformance/"],
        below: None,
        under: Under::Output,
        outputs: &[
            "unicode-text-conformance/NormalizationTest.txt",
            "unicode-text-conformance/GraphemeBreakTest.txt",
            "unicode-text-conformance/WordBreakTest.txt",
            "unicode-text-conformance/SentenceBreakTest.txt",
        ],
    },
    Setup {
        what: "the archive conformance data",
        script: "tools/portable/build_portable.py",
        args: &["data"],
        mentions: &["archive-conformance"],
        below: None,
        under: Under::Output,
        outputs: &["archive-conformance/zip", "archive-conformance/tar"],
    },
    // The sample models are a separate 1.1 GB download that only the tests tagged for them read.
    // `data` fetches the validator's data, and writes `.complete` last.
    Setup {
        what: "the glTF validator data",
        script: "tools/portable/build_portable.py",
        args: &["data"],
        mentions: &["gltf-conformance/validator"],
        below: None,
        under: Under::Output,
        outputs: &["gltf-conformance/validator/.complete"],
    },
    Setup {
        what: "the Debug native helper",
        script: "tools/developer_checks/build_developer_debug.py",
        args: &[],
        mentions: &[],
        below: Some("Debug"),
        under: Under::Checkout,
        outputs: &["stdlib/internal/developer-tools/debug-native/libdeveloper_tools_debug.*"],
    },
];

/// The directory the repository's tools write to and the tests read from: `target/` of the
/// checkout `root`, or `JACKET_OUTPUT_DIR`.
fn output_dir(root: &Path) -> PathBuf {
    output_in(root, jacket_config::env::OUTPUT_DIR.non_empty())
}

/// [`output_dir`] for a given `JACKET_OUTPUT_DIR`. Empty is unset, and a relative one is below
/// the checkout, as `tools/repo_output.py` reads it.
fn output_in(root: &Path, over: Option<std::ffi::OsString>) -> PathBuf {
    over.filter(|dir| !dir.is_empty())
        .map_or_else(|| root.join("target"), |dir| root.join(dir))
}

/// `source` with its `//` and `/* */` comments (which nest) blanked, leaving strings alone.
fn without_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut depth = 0usize;
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('*', Some('/')) if depth > 0 => {
                chars.next();
                depth -= 1;
            }
            ('/', Some('*')) => {
                chars.next();
                depth += 1;
            }
            _ if depth > 0 => {}
            ('/', Some('/')) => while chars.next_if(|&n| n != '\n').is_some() {},
            ('"', _) => {
                out.push(c);
                while let Some(s) = chars.next() {
                    out.push(s);
                    match s {
                        '\\' => out.extend(chars.next()),
                        '"' => break,
                        _ => {}
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

impl Setup {
    /// `code` is the source of `file` without its comments ([`without_comments`]): a doc example
    /// that spells a fixture path reads no data.
    fn needed_by(&self, file: &Path, code: &str, stdlib: &Path) -> bool {
        self.mentions.iter().any(|m| code.contains(m))
            || self
                .below
                .is_some_and(|dir| file.starts_with(stdlib.join(dir)))
    }

    /// Whether every output is there, below the output directory `output` or the checkout `root`.
    fn built(&self, root: &Path, output: &Path) -> bool {
        let base = match self.under {
            Under::Output => output,
            Under::Checkout => root,
        };
        self.outputs
            .iter()
            .all(|file| match file.strip_suffix('*') {
                Some(prefix) => {
                    let path = base.join(prefix);
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
                None => base.join(file).exists(),
            })
    }
}

/// The setups the `files` need whose outputs are missing below `root`, the checkout the
/// standard library `stdlib` belongs to, or in the output directory `output`. Nothing in a
/// package without the build scripts.
fn needed(files: &[PathBuf], stdlib: &Path, root: &Path, output: &Path) -> Vec<&'static Setup> {
    let mut found: Vec<&'static Setup> = Vec::new();
    for setup in &SETUPS {
        if !root.join(setup.script).is_file() || setup.built(root, output) {
            continue;
        }
        let wanted = files.iter().any(|file| {
            let source = std::fs::read_to_string(file).unwrap_or_default();
            setup.needed_by(file, &without_comments(&source), stdlib)
        });
        if wanted {
            found.push(setup);
        }
    }
    found
}

/// Builds the outputs that tests among `files` read and a fresh checkout lacks, once, before
/// anything runs. It costs a few file checks when they are there. A script that cannot run is an
/// error that names the command to run by hand: the tests would only fail later, less clearly.
pub fn prepare(files: &[PathBuf], stdlib: &Path) -> Result<(), String> {
    prepare_with("python3", files, stdlib)
}

fn prepare_with(python: &str, files: &[PathBuf], stdlib: &Path) -> Result<(), String> {
    let Some(root) = stdlib.parent() else {
        return Ok(());
    };
    let mut ran: Vec<(&str, &[&str])> = Vec::new();
    for setup in needed(files, stdlib, root, &output_dir(root)) {
        // One script builds several outputs; a run that was due for two of them is one run.
        if ran.contains(&(setup.script, setup.args)) {
            continue;
        }
        ran.push((setup.script, setup.args));
        eprintln!(
            "jacket: building {} (first run in this checkout)",
            setup.what
        );
        let by_hand = format!(
            "PYTHONPATH=tools python3 {} {}",
            setup.script,
            setup.args.join(" ")
        );
        let output = Command::new(python)
            .arg(root.join(setup.script))
            .args(setup.args)
            .env("PYTHONPATH", root.join("tools"))
            .current_dir(root)
            .output()
            .map_err(|error| {
                format!(
                    "could not start `{python}` to build {}: {error}\nhelp: install Python 3 and put it on PATH, or build it yourself: {}",
                    setup.what,
                    by_hand.trim_end()
                )
            })?;
        if !output.status.success() {
            return Err(format!(
                "could not build {}: {}\nhelp: run `{}` to see the failure",
                setup.what,
                String::from_utf8_lossy(&output.stderr).trim(),
                by_hand.trim_end()
            ));
        }
    }
    Ok(())
}

/// CI must diagnose a missing dependency rather than silently reduce its coverage.
pub fn in_ci() -> bool {
    jacket_config::env::CI.is_set() || jacket_config::env::GITHUB_ACTIONS.is_set()
}

/// Why a test that declared `optional(NAME)` is skipped: one of its `args` uses `${NAME}` and
/// the variable is not set. A test that does not declare it runs and fails, on every machine,
/// the way [`super::golden`] fails an argument with an unset variable.
pub fn optional_skip(
    options: &TestOptions,
    env: &dyn Fn(&str) -> Option<String>,
) -> Option<String> {
    options.optional.iter().find_map(|name| {
        let used = options
            .args
            .iter()
            .any(|arg| arg.contains(&format!("${{{name}}}")));
        (used && env(name).is_none_or(|value| value.is_empty()))
            .then(|| format!("missing optional test prerequisite: {name} is not set"))
    })
}

/// Only absent test-driver libraries qualify. Missing production libraries, assertions,
/// crashes and timeouts remain failures, as do all missing dependencies in CI.
pub fn skip_missing_probe(outcome: &mut Outcome, ci: bool) {
    if ci || outcome.status != Status::Fail || outcome.crashed || !outcome.failures.is_empty() {
        return;
    }
    let text = format!(
        "{}\n{}",
        outcome.problems.join("\n"),
        String::from_utf8_lossy(&outcome.stderr)
    );
    let names = missing_libraries(&text)
        .into_iter()
        .collect::<BTreeSet<_>>();
    if names.is_empty() || names.iter().any(|name| Source::of(name) == Source::Stdlib) {
        return;
    }
    let hints = library_hints(&names);
    outcome.status = Status::Skip;
    outcome.skip_reason = Some(format!("missing test prerequisite: {}", hints.join("; ")));
    outcome.problems.clear();
}

/// Build only probes that need system headers and the host C compiler. SDK providers and
/// the clipboard decoder need separately prepared dependencies and are left to CI's recipes.
/// The returned directory is added through the compiler API, preserving `JAIC_NATIVE_LIBS`.
pub fn prepare_probes(files: &[PathBuf], stdlib: &Path) -> Option<PathBuf> {
    if !cfg!(any(target_os = "macos", target_os = "linux")) {
        return None;
    }
    let root = stdlib.parent()?;
    if !stdlib.join("Compiler/module.jai").is_file() {
        return None;
    }
    let output = output_dir(root).join("jacket-test-probes");
    let sources: String = files
        .iter()
        .filter_map(|file| std::fs::read_to_string(file).ok())
        .collect();
    let mut probes = vec![(
        "thread_sdk_identity",
        "Thread/tests/sdk-identity.c",
        vec!["-pthread"],
    )];
    if cfg!(target_os = "macos") {
        probes.extend([
            (
                "unmapping_protection",
                "Unmapping_Allocator/tests/native/unmapping-protection.c",
                vec![],
            ),
            (
                "window_gl_probe",
                "GL/tests/native/context.m",
                vec!["-framework", "AppKit", "-framework", "OpenGL"],
            ),
        ]);
    }
    let suffix = if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    };
    let shared = if cfg!(target_os = "macos") {
        "-dynamiclib"
    } else {
        "-shared"
    };
    let mut available = false;
    for (name, source, flags) in probes {
        if !sources.contains(name) {
            continue;
        }
        let source = stdlib.join(source);
        if !source.is_file() {
            continue;
        }
        let library = output.join(format!("lib{name}.{suffix}"));
        let archive = output.join(format!("lib{name}.a"));
        let fresh = |path: &Path| -> bool {
            path.metadata()
                .and_then(|m| m.modified())
                .ok()
                .zip(source.metadata().and_then(|m| m.modified()).ok())
                .is_some_and(|(built, source)| built >= source)
        };
        if fresh(&library) && fresh(&archive) {
            available = true;
            continue;
        }
        if std::fs::create_dir_all(&output).is_err() {
            continue;
        }
        eprintln!("jacket: building test probe {name}");
        let object = output.join(format!("{name}.o"));
        let compiled = Command::new("cc")
            .args(["-O2", "-fPIC", "-Wall", "-Wextra", "-Werror", "-c"])
            .arg(&source)
            .arg("-o")
            .arg(&object)
            .output()
            .is_ok_and(|out| out.status.success());
        let linked = compiled
            && Command::new("cc")
                .arg(shared)
                .arg(&object)
                .args(&flags)
                .arg("-o")
                .arg(&library)
                .output()
                .is_ok_and(|out| out.status.success());
        let archived = linked
            && Command::new("ar")
                .arg("rcs")
                .arg(&archive)
                .arg(&object)
                .output()
                .is_ok_and(|out| out.status.success());
        if archived {
            available = true;
        } else {
            eprintln!(
                "jacket: could not build {name}; tests requiring it need a prepared probe library"
            );
        }
    }
    available.then_some(output)
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
        std::fs::create_dir_all(stdlib.join("Extensions/Time/tests")).unwrap();
        let zones = stdlib.join("Extensions/Time/tests/zones.jai");
        std::fs::write(&zones, "x := \"%/time-conformance/zoneinfo\";").unwrap();

        let output = root.join("target");
        let names = |files: &[PathBuf]| -> Vec<&'static str> {
            needed(files, &stdlib, &root, &output)
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
        assert_eq!(names(std::slice::from_ref(&zones)), ["the time zone data"]);

        // Built outputs, or no build scripts (a package), need nothing. The zone data is built
        // apart from the other fixtures, and one does not stand in for the other.
        std::fs::create_dir_all(root.join("target/portable-tests/zip")).unwrap();
        std::fs::create_dir_all(root.join("target/portable-tests/archive-fixtures")).unwrap();
        std::fs::create_dir_all(root.join("target/portable-tests/mpeg")).unwrap();
        assert!(names(&[stdlib.join("Zip/tests/reads.jai")]).is_empty());
        assert_eq!(names(std::slice::from_ref(&zones)), ["the time zone data"]);
        std::fs::create_dir_all(root.join("target/time-conformance/zoneinfo")).unwrap();
        assert_eq!(names(std::slice::from_ref(&zones)), ["the time zone data"]);
        std::fs::write(root.join("target/time-conformance/vectors.bin"), "").unwrap();
        assert!(names(std::slice::from_ref(&zones)).is_empty());

        // Every generated suite has its own setup, and is built until the files it reads are
        // there. Each source names a directory of the output directory, as the tests do.
        let suites: [(&str, &str, &str, &[&str]); 6] = [
            (
                "csv.jai",
                "%/csv-conformance/csvs/a.csv",
                "the CSV conformance data",
                &["csv-conformance/csvs", "csv-conformance/json"],
            ),
            (
                "unicode.jai",
                "%/unicode-text-conformance/%",
                "the Unicode text conformance data",
                &[
                    "unicode-text-conformance/NormalizationTest.txt",
                    "unicode-text-conformance/GraphemeBreakTest.txt",
                    "unicode-text-conformance/WordBreakTest.txt",
                    "unicode-text-conformance/SentenceBreakTest.txt",
                ],
            ),
            (
                "archive.jai",
                "%/archive-conformance",
                "the archive conformance data",
                &["archive-conformance/zip", "archive-conformance/tar"],
            ),
            (
                "gltf.jai",
                "%/gltf-conformance/validator/test/base/data",
                "the glTF validator data",
                &["gltf-conformance/validator/.complete"],
            ),
            (
                "cpuid.jai",
                "%/portable-machine/cpuid.bin",
                "the machine fixture data",
                &["portable-machine/cpuid.bin"],
            ),
            ("samples.jai", "%/gltf-conformance/samples/Models", "", &[]),
        ];
        std::fs::create_dir_all(stdlib.join("Extensions/Suites/tests")).unwrap();
        for (file, source, what, outputs) in suites {
            let test = stdlib.join("Extensions/Suites/tests").join(file);
            std::fs::write(&test, source).unwrap();
            let files = std::slice::from_ref(&test);
            if what.is_empty() {
                // The sample models are a separate download that `data` does not fetch.
                assert!(names(files).is_empty(), "{file}");
                continue;
            }
            assert_eq!(names(files), [what], "{file}");
            for (i, output_file) in outputs.iter().enumerate() {
                let path = output.join(output_file);
                std::fs::create_dir_all(path.parent().unwrap()).unwrap();
                std::fs::write(&path, "").unwrap();
                let built = i + 1 == outputs.len();
                assert_eq!(names(files).is_empty(), built, "{file} after {output_file}");
            }
        }

        // The tests read the relocated output directory, so that is where the setup looks: a
        // relocated one that has the data is built, and one that is missing it is not mistaken for
        // the checkout's.
        let moved = root.join("elsewhere");
        let names_in = |output: &Path, file: &PathBuf| -> Vec<&'static str> {
            needed(std::slice::from_ref(file), &stdlib, &root, output)
                .iter()
                .map(|s| s.what)
                .collect()
        };
        assert_eq!(names_in(&moved, &zones), ["the time zone data"]);
        std::fs::create_dir_all(moved.join("time-conformance/zoneinfo")).unwrap();
        std::fs::write(moved.join("time-conformance/vectors.bin"), "").unwrap();
        assert!(names_in(&moved, &zones).is_empty());
        std::fs::remove_dir_all(root.join("target/time-conformance")).unwrap();
        assert!(names_in(&moved, &zones).is_empty());
        assert_eq!(names_in(&output, &zones), ["the time zone data"]);
        // The Debug helper is a file of the checkout whatever the output directory is.
        std::fs::remove_dir_all(root.join("tools")).unwrap();
        assert!(names(&[stdlib.join("Debug/tests/ownership.jai")]).is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }

    /// The generated directories a test source names: a `*-conformance` directory (below `%/` or
    /// `target/`) with the folder below it, or `portable-machine/`.
    fn generated_directories(source: &str) -> Vec<String> {
        let is_name = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
        let mut found = Vec::new();
        for (at, _) in source.match_indices("-conformance") {
            let start = source[..at]
                .rfind(|c: char| !is_name(c))
                .map_or(0, |i| i + 1);
            // A path of the output directory, not a name in a comment.
            if !source[..start].ends_with("%/") && !source[..start].ends_with("target/") {
                continue;
            }
            let mut end = at + "-conformance".len();
            if source[end..].starts_with('/') {
                end += 1;
                end += source[end..].find(|c: char| !is_name(c)).unwrap_or(0);
            }
            found.push(source[start..end].to_string());
        }
        if source.contains("portable-machine/") {
            found.push("portable-machine/".into());
        }
        found
    }

    #[test]
    fn a_path_in_a_comment_needs_no_data() {
        let stdlib = Path::new("/checkout/stdlib");
        let fixtures = &SETUPS[0];
        let wants = |source: &str| {
            fixtures.needed_by(Path::new("a.jai"), &without_comments(source), stdlib)
        };
        assert!(!wants(
            "// output_path(\"portable-tests/zip/a.zip\");\nf :: () {}"
        ));
        assert!(!wants(
            "/* a /* nested */ \"portable-tests/zip/\" */ f :: () {}"
        ));
        assert!(wants("p :: \"%/portable-tests/zip/a.zip\"; // read it"));
        // `//` inside a string is not a comment.
        assert!(wants(
            "u :: \"http://x\"; p :: \"portable-tests/zip/\\\"\";"
        ));
    }

    #[test]
    fn the_output_directory_is_the_one_the_tools_write_to() {
        let root = Path::new("/checkout");
        assert_eq!(output_in(root, None), Path::new("/checkout/target"));
        assert_eq!(
            output_in(root, Some("".into())),
            Path::new("/checkout/target")
        );
        assert_eq!(
            output_in(root, Some("/scratch/out".into())),
            Path::new("/scratch/out")
        );
        assert_eq!(
            output_in(root, Some("out".into())),
            Path::new("/checkout/out")
        );
    }

    #[test]
    fn every_generated_directory_the_stdlib_tests_read_has_a_setup() {
        // The tests read what `build_portable.py data` writes, as `%/<directory>/...` below the
        // output directory or `target/<directory>/...` below the checkout.
        let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut unbuilt = BTreeSet::new();
        let mut folders = vec![stdlib.clone()];
        while let Some(folder) = folders.pop() {
            for entry in std::fs::read_dir(&folder).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    folders.push(path);
                } else if path.extension().is_some_and(|ext| ext == "jai")
                    && path.components().any(|c| c.as_os_str() == "tests")
                {
                    let source = std::fs::read_to_string(&path).unwrap_or_default();
                    for name in generated_directories(&source) {
                        let covered = SETUPS
                            .iter()
                            .any(|setup| setup.mentions.iter().any(|m| name.starts_with(m)));
                        // The sample models are fetched on request, by the tests that say so.
                        if !covered && name != "gltf-conformance/samples" {
                            unbuilt.insert(format!("{name} (read by {})", path.display()));
                        }
                    }
                }
            }
        }
        assert!(unbuilt.is_empty(), "no setup builds: {unbuilt:?}");
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

#[cfg(test)]
mod local_prerequisite_tests {
    use super::*;

    #[test]
    fn an_unset_variable_skips_only_a_test_that_declares_it_optional() {
        let mut options = TestOptions::default();
        options.apply("args(\"${JAIC_TEST_FONT_PATH}\")").unwrap();
        let unset = |_: &str| None;
        let set = |_: &str| Some("/fonts/a.ttf".to_string());
        // Not declared: it runs, and fails for the missing variable.
        assert!(optional_skip(&options, &unset).is_none());
        options.apply("optional(JAIC_TEST_FONT_PATH)").unwrap();
        let reason = optional_skip(&options, &unset).unwrap();
        assert!(reason.contains("JAIC_TEST_FONT_PATH"), "{reason}");
        assert!(optional_skip(&options, &set).is_none());
        assert!(optional_skip(&options, &|_: &str| Some(String::new())).is_some());
        // Declared but not used by the arguments: nothing to skip.
        options.args.clear();
        assert!(optional_skip(&options, &unset).is_none());
    }

    #[test]
    fn a_missing_python_or_a_failed_script_is_an_error_with_the_command_to_run() {
        let root = std::env::temp_dir().join(format!("jacket-prepare-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let stdlib = root.join("stdlib");
        std::fs::create_dir_all(stdlib.join("Debug/tests")).unwrap();
        std::fs::create_dir_all(root.join("tools/developer_checks")).unwrap();
        std::fs::write(
            root.join("tools/developer_checks/build_developer_debug.py"),
            "",
        )
        .unwrap();
        let test = stdlib.join("Debug/tests/ownership.jai");
        std::fs::write(&test, "main :: () {}").unwrap();
        let files = [test];

        let error = prepare_with("/nowhere/python3", &files, &stdlib).unwrap_err();
        assert!(
            error.contains("could not start `/nowhere/python3`"),
            "{error}"
        );
        assert!(error.contains("build_developer_debug.py"), "{error}");
        assert!(error.contains("install Python 3"), "{error}");
        #[cfg(unix)]
        {
            let error = prepare_with("false", &files, &stdlib).unwrap_err();
            assert!(
                error.contains("could not build the Debug native helper"),
                "{error}"
            );
            assert!(error.contains("PYTHONPATH=tools python3"), "{error}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_probes_skip_locally_but_real_failures_and_ci_stay_fatal() {
        let failed = |text: &str| {
            let mut result = Outcome::new(0, Status::Fail);
            result.stderr = text.as_bytes().to_vec();
            result
        };
        for (text, ci, expected) in [
            (
                "library 'rpmalloc_adaptive_probe' not found",
                false,
                Status::Skip,
            ),
            ("library 'window_gl_probe' not found", false, Status::Skip),
            (
                "library 'rpmalloc_adaptive_probe' not found",
                true,
                Status::Fail,
            ),
            ("library 'mbedtls' not found", false, Status::Fail),
            (
                "library 'abi_probe' not found; library 'SDL3' not found",
                false,
                Status::Fail,
            ),
            ("assertion failed", false, Status::Fail),
        ] {
            let mut result = failed(text);
            skip_missing_probe(&mut result, ci);
            assert_eq!(result.status, expected, "{text}, ci={ci}");
            if expected == Status::Skip {
                assert!(
                    result
                        .skip_reason
                        .unwrap()
                        .contains("missing test prerequisite")
                );
            }
        }
        let mut crash = failed("library 'abi_probe' not found");
        crash.crashed = true;
        skip_missing_probe(&mut crash, false);
        assert_eq!(crash.status, Status::Fail);
    }
}
