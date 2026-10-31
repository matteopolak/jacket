//! Every environment variable the Rust tools read, in one table.
//!
//! A variable is declared once in `vars!` below with its name, meaning, default and readers, and
//! is read through its constant (`env::STDLIB.os()`). Nothing else may call `std::env::var` or
//! `var_os`: clippy's `disallowed-methods` (root `clippy.toml`) rejects it outside this module,
//! so a misspelt name cannot silently do nothing. `docs/reference/environment-variables.md` is
//! generated from [`ALL`] (`tools/generators/generate_reference_docs.py`).
//!
//! The tests keep the table honest: every `"JAIC_*"`, `"JACKET_*"` and `"JAI_*"` string literal in
//! the Rust sources must be a declared name, and names are unique.
#![allow(clippy::disallowed_methods)]

use std::ffi::OsString;

/// A program that reads a variable. `Tests` covers the test harnesses and `Fuzz` the fuzz
/// harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reader {
    Jaic,
    Jacket,
    Jailint,
    Jailsp,
    Tests,
    Fuzz,
}

impl Reader {
    /// The name the reference page shows.
    pub fn label(self) -> &'static str {
        match self {
            Reader::Jaic => "jaic",
            Reader::Jacket => "jacket",
            Reader::Jailint => "jailint",
            Reader::Jailsp => "jailsp",
            Reader::Tests => "tests",
            Reader::Fuzz => "fuzzing",
        }
    }
}

/// Where a variable comes from, which sets its section on the reference page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// Set by a user to steer the compiler or the tools.
    Settings,
    /// Set by one tool for another (`jacket` for `jaic`).
    Handoff,
    /// Read by tests, test harnesses and the fuzz harness only.
    Testing,
    /// Not ours: set by the operating system, a shell, CI or a toolchain, and read by us.
    Standard,
}

impl Group {
    pub const ALL: [Group; 4] = [
        Group::Settings,
        Group::Handoff,
        Group::Testing,
        Group::Standard,
    ];

    /// The section title on the reference page.
    pub fn title(self) -> &'static str {
        match self {
            Group::Settings => "Settings",
            Group::Handoff => "Set by one tool for another",
            Group::Testing => "Tests and fuzzing",
            Group::Standard => "Standard variables",
        }
    }
}

/// One environment variable.
#[derive(Clone, Copy, Debug)]
pub struct Var {
    pub name: &'static str,
    pub group: Group,
    /// What it does, one or two sentences.
    pub summary: &'static str,
    /// What happens when it is unset.
    pub default: &'static str,
    pub readers: &'static [Reader],
}

impl Var {
    /// The value, when set and valid Unicode.
    pub fn get(&self) -> Option<String> {
        std::env::var(self.name).ok()
    }

    /// The value, when set.
    pub fn os(&self) -> Option<OsString> {
        std::env::var_os(self.name)
    }

    /// The value, when set and not empty. Shells such as PowerShell 7.5 pass `VAR=''` for an
    /// unset variable, so most settings treat empty as unset.
    pub fn non_empty(&self) -> Option<OsString> {
        self.os().filter(|value| !value.is_empty())
    }

    /// Whether the variable is set to anything, empty included.
    pub fn is_set(&self) -> bool {
        self.os().is_some()
    }
}

macro_rules! vars {
    ($($(#[$meta:meta])* $id:ident {
        name: $name:expr, group: $group:ident, readers: [$($reader:ident),* $(,)?],
        default: $default:literal, summary: $summary:literal $(,)?
    })*) => {
        $(
            $(#[$meta])*
            pub const $id: Var = Var {
                name: $name,
                group: Group::$group,
                summary: $summary,
                default: $default,
                readers: &[$(Reader::$reader),*],
            };
        )*
        /// Every declared variable, in declaration order.
        pub static ALL: &[Var] = &[$($id),*];
    };
}

vars! {
    STDLIB {
        name: "JAIC_STDLIB", group: Settings, readers: [Jaic, Jacket, Jailint, Jailsp, Tests],
        default: "the `stdlib/` next to the executable, else the checkout's",
        summary: "The standard library directory. `jacket test` also hands it to every `jaic` it starts.",
    }
    PRELUDE {
        name: "JAIC_PRELUDE", group: Settings, readers: [Jaic, Jailint, Jailsp],
        default: "the library's `prelude/module.jai`, else `Preload.jai`",
        summary: "The prelude file to load instead of the library's own, as a path (relative paths are taken from the library root).",
    }
    NATIVE_LIBS {
        name: "JAIC_NATIVE_LIBS", group: Settings, readers: [Jaic],
        default: "`native-libs/<os>-<cpu>` beside the library, else under `target/`",
        summary: "A path list of directories holding the prebuilt third-party native libraries (SDL3, FreeType, ...), replacing the default search.",
    }
    CROSS_LIBS {
        name: "JAIC_CROSS_LIBS", group: Settings, readers: [Jaic],
        default: "unset",
        summary: "A path list of directories with libraries cross-built for a MinGW target; a cross build links from these instead of the host's native libraries.",
    }
    LINKER {
        name: "JAIC_LINKER", group: Settings, readers: [Jaic],
        default: "the linker found for the target",
        summary: "The program that links executables and libraries. Empty counts as unset.",
    }
    AR {
        name: "JAIC_AR", group: Settings, readers: [Jaic],
        default: "the system `ar`, or `llvm-ar`, a MinGW `ar` or `lib.exe` where that is needed",
        summary: "The program that archives object files into a static library.",
    }
    SANITIZER_CC {
        name: "JAIC_SANITIZER_CC", group: Settings, readers: [Jaic],
        default: "the `clang` of the LLVM jaic was built with, else `cc`",
        summary: "The C driver that links a build made with a sanitizer.",
    }
    WASM_LD {
        name: "JAIC_WASM_LD", group: Settings, readers: [Jaic],
        default: "`wasm-ld` next to the build's LLVM, an LLD install, then `PATH`",
        summary: "The `wasm-ld` that links WebAssembly output.",
    }
    MEMORY_LIMIT {
        name: "JAIC_MEMORY_LIMIT", group: Settings, readers: [Jaic],
        default: "no limit",
        summary: "Stop with exit status 120 once this much memory is allocated: bytes, or a number with a `K`, `M` or `G` suffix. Child processes inherit it.",
    }
    DIAGNOSTICS {
        name: "JAIC_DIAGNOSTICS", group: Settings, readers: [Jaic, Jailint],
        default: "by terminal: `unicode` or `ascii` on one, `plain` otherwise",
        summary: "The diagnostic layout: `plain`, `ascii` or `unicode`.",
    }
    PROGRESS {
        name: "JAIC_PROGRESS", group: Settings, readers: [Jaic, Jacket],
        default: "by terminal: a live status line on one, nothing otherwise",
        summary: "The build status lines: `off`, `plain` or `live`, whatever stderr is (`-quiet` still wins).",
    }
    CACHE_DIR {
        name: "JAIC_CACHE_DIR", group: Settings, readers: [Jaic],
        default: "`$XDG_CACHE_HOME/jaic`, else the platform's user cache (`~/Library/Caches/jaic`, `~/.cache/jaic`)",
        summary: "Where `jaic` unpacks the helpers it embeds (the crash runtime).",
    }
    DSYM {
        name: "JAIC_DSYM", group: Settings, readers: [Jaic],
        default: "unset: macOS debug builds keep their object files for the debugger",
        summary: "When set, a macOS debug build writes a `.dSYM` bundle instead of keeping its object files.",
    }
    NO_HOST_TWIN {
        name: "JAIC_NO_HOST_TWIN", group: Settings, readers: [Jaic],
        default: "unset: compile-time code that needs the host's types gets a host twin",
        summary: "Any value but `0` turns the host twin off, which makes compile-time code with foreign calls fail the way it would without one.",
    }
    HOST_TWIN_DEBUG {
        name: "JAIC_HOST_TWIN_DEBUG", group: Settings, readers: [Jaic],
        default: "unset",
        summary: "When set, the host twin prints what it generates.",
    }
    SYM_SCRAMBLE {
        name: "JAIC_SYM_SCRAMBLE", group: Settings, readers: [Jaic],
        default: "unset",
        summary: "A seed that spreads the interner's symbol numbers apart, so output that depends on them rather than spellings changes (`tools/differential/determinism.py`).",
    }
    ID_SCRAMBLE {
        name: "JAIC_ID_SCRAMBLE", group: Settings, readers: [Jaic],
        default: "unset",
        summary: "The same as `JAIC_SYM_SCRAMBLE`, for AST ids.",
    }
    OUTPUT_DIR {
        name: "JACKET_OUTPUT_DIR", group: Settings, readers: [Jaic, Jacket],
        default: "`target/` in the project",
        summary: "Relocates the build output directory (the `native-libs/` lookup and `jacket test --coverage` reports follow it).",
    }
    CHECK_TIME_LIMIT {
        name: "JACKET_CHECK_TIME_LIMIT", group: Settings, readers: [Jailint, Jailsp],
        default: "10 seconds",
        summary: "Seconds one program check may take when the linter or language server runs compile-time code (`0`: no limit). CI sets it to `0`.",
    }
    TEST_DLLS {
        name: "JACKET_TEST_DLLS", group: Settings, readers: [Jacket],
        default: "unset",
        summary: "A path list of directories whose `.dll` files `jacket test` links next to each Windows test executable.",
    }
    COVERAGE_FILE {
        name: "JAIC_COVERAGE_FILE", group: Handoff, readers: [Jaic],
        default: "unset: the file is named for the program, in `JAIC_COVERAGE_DIR`",
        summary: "The file a `-interp` run writes its line coverage to.",
    }
    COVERAGE_DIR {
        name: "JAIC_COVERAGE_DIR", group: Handoff, readers: [Jaic],
        default: "the working directory",
        summary: "The directory a `-interp` run writes its line coverage to, one file per program. `jacket test --coverage` sets it.",
    }
    REPORT_EXE {
        name: "JAIC_REPORT_EXE", group: Handoff, readers: [Jaic],
        default: "unset",
        summary: "A file `jaic` appends the absolute path of each executable it writes to. `jacket run` and `jacket test` set it, because a metaprogram picks the name.",
    }
    PROJECT {
        name: "JACKET_PROJECT", group: Handoff, readers: [Jaic],
        default: "the main file's name",
        summary: "The project name `jaic` shows in its `Compiling` line; `jacket` sets it.",
    }
    REQUIRE_TOOLCHAIN {
        name: "JAIC_REQUIRE_TOOLCHAIN", group: Testing, readers: [Tests],
        default: "unset: a test whose tool is missing skips",
        summary: "`1`, or a comma list of tool names: a missing tool fails the test instead of skipping it. Always on under `CI`.",
    }
    TEST_TMPDIR {
        name: "JAIC_TEST_TMPDIR", group: Testing, readers: [Tests],
        default: "the system temporary directory",
        summary: "Where test scratch directories are made.",
    }
    KEEP_TEST_TMP {
        name: "JAIC_KEEP_TEST_TMP", group: Testing, readers: [Tests],
        default: "unset: scratch directories are removed",
        summary: "When set, test scratch directories are kept after the test.",
    }
    TEST_CODEGEN_UNITS {
        name: "JACKET_TEST_CODEGEN_UNITS", group: Testing, readers: [Tests],
        default: "unset",
        summary: "Passes `-codegen_units N` to the native build tests, to run them split into several modules.",
    }
    STDLIB_TARGETS_PART {
        name: "JACKET_STDLIB_TARGETS_PART", group: Testing, readers: [Tests],
        default: "`1/1`",
        summary: "`K/N`: the stdlib-by-target check takes every Nth module and target pair from the Kth, so CI can split it.",
    }
    DATABASE_URL {
        name: "JACKET_DATABASE_URL", group: Testing, readers: [Tests],
        default: "unset: the Postgres tests skip",
        summary: "A PostgreSQL connection URL for the typed-query tests.",
    }
    CORPUS_ROOT {
        name: "JAIC_CORPUS_ROOT", group: Testing, readers: [Tests],
        default: "the repository",
        summary: "The checkout whose `corpus/upstream` and `stdlib` the corpus sweep parses.",
    }
    CORPUS_STAGE {
        name: "JAIC_CORPUS_STAGE", group: Testing, readers: [Tests],
        default: "`parse`",
        summary: "How far the corpus sweep takes each file.",
    }
    ABI_CROSS {
        name: "JAIC_ABI_CROSS", group: Testing, readers: [Tests],
        default: "unset",
        summary: "Cross targets the C ABI tests also compile for, as a comma list of specs.",
    }
    ABI_CLANG {
        name: "JAIC_ABI_CLANG", group: Testing, readers: [Tests],
        default: "`clang`",
        summary: "The `clang` the C ABI tests compare against.",
    }
    PRELUDE_TEST_CHILD {
        name: "JAIC_PRELUDE_TEST_CHILD", group: Testing, readers: [Tests],
        default: "unset",
        summary: "Marks the child process of the prelude-selection tests, which run themselves again with `JAIC_PRELUDE` set.",
    }
    TEST_LIBRARY_MARKER {
        name: "JAIC_TEST_LIBRARY_MARKER", group: Testing, readers: [Tests],
        default: "unset",
        summary: "A file the native-link test's library writes to, to show it was loaded.",
    }
    WASM_TEST {
        name: "JAIC_WASM_TEST", group: Testing, readers: [Tests],
        default: "unset",
        summary: "A value the WebAssembly test passes to the program, which prints it back.",
    }
    ENFORCE_SCALE_BUDGET {
        name: "JAILSP_ENFORCE_SCALE_BUDGET", group: Testing, readers: [Tests],
        default: "unset: the budget is only reported",
        summary: "`1`: the language server scale test fails when a case is over its budget.",
    }
    TYPING_FUZZ_DUMP {
        name: "TYPING_FUZZ_DUMP", group: Testing, readers: [Tests],
        default: "unset",
        summary: "A directory the typing-session rig writes the failing document to.",
    }
    FUZZ_VERBOSE {
        name: "JAI_FUZZ_VERBOSE", group: Testing, readers: [Fuzz],
        default: "unset",
        summary: "When set, the fuzz harness prints each input it checks.",
    }
    EXPECTED_PRELUDE {
        name: "EXPECTED_PRELUDE", group: Testing, readers: [Tests],
        default: "unset",
        summary: "The prelude the child process of the prelude-selection tests must find, or `missing`.",
    }
    IDE_AUDIT_REVISION {
        name: "JAI_IDE_AUDIT_REVISION", group: Testing, readers: [Tests],
        default: "unset",
        summary: "Read at build time: the revision the language server coverage audit stamps into its report.",
    }

    PATH {
        name: "PATH", group: Standard, readers: [Jaic, Jacket, Jailsp, Tests],
        default: "the process's",
        summary: "Where tools such as the linker, `clang` and `wasm-ld` are searched for.",
    }
    PATHEXT {
        name: "PATHEXT", group: Standard, readers: [Tests],
        default: "the Windows default",
        summary: "The executable extensions tried when a test looks a tool up on Windows.",
    }
    HOME {
        name: "HOME", group: Standard, readers: [Jaic],
        default: "unset",
        summary: "Locates the user cache when neither `JAIC_CACHE_DIR` nor `XDG_CACHE_HOME` is set.",
    }
    XDG_CACHE_HOME {
        name: "XDG_CACHE_HOME", group: Standard, readers: [Jaic],
        default: "unset",
        summary: "The user cache root; `jaic` uses its `jaic` folder.",
    }
    CI {
        name: "CI", group: Standard, readers: [Jaic, Jacket, Tests],
        default: "unset",
        summary: "Set by CI services: the build status lines are plain, and a missing test tool fails instead of skipping.",
    }
    NO_COLOR {
        name: "NO_COLOR", group: Standard, readers: [Jaic, Jacket, Jailint],
        default: "unset",
        summary: "Any value turns colour off.",
    }
    FORCE_COLOR {
        name: "FORCE_COLOR", group: Standard, readers: [Jaic, Jailint],
        default: "unset",
        summary: "Anything but `0` turns colour on even when output is not a terminal.",
    }
    CLICOLOR_FORCE {
        name: "CLICOLOR_FORCE", group: Standard, readers: [Jaic, Jailint],
        default: "unset",
        summary: "The same as `FORCE_COLOR`.",
    }
    TERM {
        name: "TERM", group: Standard, readers: [Jaic, Jacket, Jailint],
        default: "unset",
        summary: "`dumb` turns colour, layout characters and the live status line off.",
    }
    CONEMU_ANSI {
        name: "ConEmuANSI", group: Standard, readers: [Jaic, Jacket, Jailint],
        default: "unset",
        summary: "`ON` marks a Windows console that takes ANSI control codes.",
    }
    ANSICON {
        name: "ANSICON", group: Standard, readers: [Jaic, Jacket],
        default: "unset",
        summary: "Set by ANSICON; marks a Windows console that takes ANSI control codes.",
    }
    WT_SESSION {
        name: "WT_SESSION", group: Standard, readers: [Jaic, Jailint],
        default: "unset",
        summary: "Set by Windows Terminal; it gets the Unicode diagnostic layout.",
    }
    LC_ALL {
        name: "LC_ALL", group: Standard, readers: [Jaic, Jailint],
        default: "unset",
        summary: "With `LC_CTYPE` and `LANG`: a UTF-8 locale selects the Unicode diagnostic layout.",
    }
    LC_CTYPE {
        name: "LC_CTYPE", group: Standard, readers: [Jaic, Jailint],
        default: "unset",
        summary: "See `LC_ALL`.",
    }
    LANG {
        name: "LANG", group: Standard, readers: [Jaic, Jailint],
        default: "unset",
        summary: "See `LC_ALL`.",
    }
    COLUMNS {
        name: "COLUMNS", group: Standard, readers: [Jaic],
        default: "the terminal's width",
        summary: "The width the live status line is fitted to.",
    }
    CC {
        name: "CC", group: Standard, readers: [Tests],
        default: "`cc`",
        summary: "The C compiler the native tests build their helper libraries with.",
    }
    LIB {
        name: "LIB", group: Standard, readers: [Jaic],
        default: "found from the Visual Studio and Windows SDK installs",
        summary: "On Windows, the library directories the MSVC linker searches; set by a developer prompt.",
    }
    LIBRARY_PATH {
        name: "LIBRARY_PATH", group: Standard, readers: [Jaic],
        default: "unset",
        summary: "Extra library directories compile-time code may load native libraries from on Windows.",
    }
    PROGRAM_FILES {
        name: "ProgramFiles", group: Standard, readers: [Jaic],
        default: "`C:\\Program Files`",
        summary: "Where Visual Studio and the Windows SDK are looked for.",
    }
    PROGRAM_FILES_X86 {
        name: "ProgramFiles(x86)", group: Standard, readers: [Jaic],
        default: "`C:\\Program Files (x86)`",
        summary: "See `ProgramFiles`.",
    }
    SYSTEM_ROOT {
        name: "SystemRoot", group: Standard, readers: [Jacket],
        default: "`C:\\Windows`",
        summary: "Locates the Windows system tools `jacket upgrade` uses.",
    }
    LLVM_SYS_PREFIX {
        name: crate::llvm::SYS_PREFIX_VAR, group: Standard, readers: [Jaic, Jacket],
        default: "unset",
        summary: "The LLVM install jaic is built against; its `bin/` supplies `clang`, `wasm-ld` and the LLVM tools. The name carries the LLVM version (`LLVM_SYS_<major><minor>_PREFIX`).",
    }
    LLVM_PREFIX {
        name: "LLVM_PREFIX", group: Standard, readers: [Jaic, Jacket],
        default: "`llvm-config --prefix`, then Homebrew's `llvm@<major>` or `/usr/lib/llvm-<major>`",
        summary: "The LLVM install to use when the `llvm-sys` variable above is unset. CI sets both.",
    }
    MACOSX_DEPLOYMENT_TARGET {
        name: "MACOSX_DEPLOYMENT_TARGET", group: Standard, readers: [Jaic],
        default: "the SDK's",
        summary: "The macOS version native output targets.",
    }
    DEVELOPER_DIR {
        name: "DEVELOPER_DIR", group: Standard, readers: [Jaic],
        default: "`xcode-select -p`",
        summary: "The Xcode installation whose SDK and libclang jaic uses.",
    }
    RUST_BACKTRACE {
        name: "RUST_BACKTRACE", group: Standard, readers: [Jaic],
        default: "unset",
        summary: "When set, an internal compiler error shows the Rust backtrace.",
    }
    GITHUB_TOKEN {
        name: "GITHUB_TOKEN", group: Standard, readers: [Jacket],
        default: "unset",
        summary: "Sent with `jacket upgrade`'s release lookups, which raises GitHub's rate limit.",
    }
    GITHUB_ACTIONS {
        name: "GITHUB_ACTIONS", group: Standard, readers: [Tests],
        default: "unset",
        summary: "Set by GitHub Actions: tests that need a capable filesystem stop skipping.",
    }
    INSTA_UPDATE {
        name: "INSTA_UPDATE", group: Standard, readers: [Tests],
        default: "unset",
        summary: "`always` rewrites the test snapshots instead of failing on a difference.",
    }
}

/// The declaration of the variable called `name`.
pub fn find(name: &str) -> Option<&'static Var> {
    ALL.iter().find(|var| var.name == name)
}

/// The value of a declared variable, by name: for code that is handed the name (a lookup passed
/// in so tests can supply their own environment). Panics on a name that is not in [`ALL`].
pub fn lookup(name: &str) -> Option<String> {
    let Some(var) = find(name) else {
        panic!("`{name}` is not declared in jacket_config::env");
    };
    var.get()
}

/// The value of a variable a program being compiled asks for (`getenv` in compile-time code):
/// any name.
pub fn for_program(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::Path;

    #[test]
    fn names_are_unique() {
        let mut seen = BTreeSet::new();
        for var in ALL {
            assert!(seen.insert(var.name), "`{}` is declared twice", var.name);
        }
    }

    #[test]
    fn every_variable_says_what_it_is() {
        for var in ALL {
            assert!(!var.readers.is_empty(), "`{}` has no readers", var.name);
            assert!(
                !var.summary.is_empty() && !var.default.is_empty(),
                "`{}`",
                var.name
            );
        }
    }

    fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            if path.is_dir() {
                if name != "target" && name != ".git" && name != "node_modules" {
                    rust_files(&path, out);
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                out.push(path);
            }
        }
    }

    /// The quoted `JAIC_*`, `JACKET_*` and `JAI_*` names in `text`, and any name handed straight
    /// to an `env("…")` or `lookup("…")` call, which panics at run time when it is undeclared.
    fn quoted_names(text: &str) -> Vec<&str> {
        let mut found = Vec::new();
        for (start, _) in text.match_indices('"') {
            let rest = &text[start + 1..];
            let Some(end) = rest.find('"') else {
                continue;
            };
            let word = &rest[..end];
            let before = &text[..start];
            // `.env("X", value)` sets a child's environment, which needs no declaration.
            let read = (before.ends_with("env(") && !before.ends_with(".env("))
                || before.ends_with("lookup(");
            let ours = ["JAIC_", "JACKET_", "JAI_"]
                .iter()
                .any(|prefix| word.starts_with(prefix));
            let shaped = !word.is_empty()
                && word.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
                && (read || !word.bytes().any(|b| b.is_ascii_lowercase()));
            if shaped && (read || (ours && word.len() > 5)) {
                found.push(word);
            }
        }
        found
    }

    /// A misspelt name in a `Command::env`, a comparison or a message would do nothing; this
    /// finds it.
    #[test]
    fn every_name_in_the_sources_is_declared() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut files = Vec::new();
        for dir in ["crates", "fuzz"] {
            rust_files(&root.join(dir), &mut files);
        }
        assert!(files.len() > 100, "found only {} sources", files.len());
        let mut unknown = Vec::new();
        for file in files {
            if file.ends_with("jacket-config/src/env.rs") {
                continue;
            }
            let text = std::fs::read_to_string(&file).unwrap_or_default();
            for name in quoted_names(&text) {
                if find(name).is_none() {
                    unknown.push(format!("{}: {name}", file.display()));
                }
            }
        }
        assert!(
            unknown.is_empty(),
            "not declared in env.rs:\n{}",
            unknown.join("\n")
        );
    }
}
