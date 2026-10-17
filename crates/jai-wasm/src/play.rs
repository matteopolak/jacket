//! Playground entry point: compile and run a set of files with the `jaic` core.
use jaic::build::{BuildEnv, Workspaces};
use jaic::interp::{Host, SandboxHost, SharedHost};
use jaic::sema::{Compiler, FileSystem, Options, TargetCpu, TargetOs, VirtualFs};
use jaic::source::{Diagnostic, Severity, Span};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::rc::Rc;

include!(concat!(env!("OUT_DIR"), "/stdlib_files.rs"));

/// Virtual directory holding the user's files.
pub const WORKSPACE_ROOT: &str = "/workspace";

/// Virtual directory holding the bundled standard library.
pub const STDLIB_ROOT: &str = "/stdlib";

/// One compiler or runtime message with a position inside a workspace file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayDiagnostic {
    pub severity: &'static str,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    /// What it reports, for tools (`DiagnosticKind::code`); `None` for most messages.
    pub code: Option<&'static str>,
}

#[derive(Debug, Clone, Default)]
pub struct PlayResult {
    /// `None` when no program exit is available (input/compile error, runtime trap, or compile-only).
    pub exit_code: Option<i32>,
    /// Where a diagnosed failure occurred: input validation, compilation or program runtime.
    pub failure_phase: Option<&'static str>,
    pub stdout: String,
    pub stderr: String,
    /// Everything the program wrote, in order: `("stdout" | "stderr", text)` runs.
    pub output: Vec<(&'static str, String)>,
    pub diagnostics: Vec<PlayDiagnostic>,
    /// Compiler errors rendered with source snippets.
    pub rendered: String,
}

impl PlayResult {
    pub fn to_json(&self) -> String {
        let mut out = String::from("{\"exitCode\":");
        match self.exit_code {
            Some(code) => {
                let _ = write!(out, "{code}");
            }
            None => out.push_str("null"),
        }
        out.push_str(",\"failurePhase\":");
        if let Some(phase) = self.failure_phase {
            json_string(&mut out, phase);
        } else {
            out.push_str("null");
        }
        out.push_str(",\"stdout\":");
        json_string(&mut out, &self.stdout);
        out.push_str(",\"stderr\":");
        json_string(&mut out, &self.stderr);
        out.push_str(",\"output\":[");
        for (i, (stream, text)) in self.output.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(out, "{{\"stream\":\"{stream}\",\"text\":");
            json_string(&mut out, text);
            out.push('}');
        }
        out.push(']');
        out.push_str(",\"rendered\":");
        json_string(&mut out, &self.rendered);
        out.push_str(",\"diagnostics\":[");
        for (i, d) in self.diagnostics.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(out, "{{\"severity\":\"{}\",\"file\":", d.severity);
            json_string(&mut out, &d.file);
            let _ = write!(
                out,
                ",\"line\":{},\"column\":{},\"message\":",
                d.line, d.column
            );
            json_string(&mut out, &d.message);
            if let Some(code) = d.code {
                let _ = write!(out, ",\"code\":\"{code}\"");
            }
            out.push('}');
        }
        out.push_str("]}");
        out
    }
}

fn json_string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

pub(crate) fn virtual_fs(files: &BTreeMap<String, Vec<u8>>) -> VirtualFs {
    workspace_fs(files, true)
}

fn workspace_fs(files: &BTreeMap<String, Vec<u8>>, bundled: bool) -> VirtualFs {
    let mut fs = VirtualFs::default();
    if bundled {
        for (name, bytes) in BUNDLED {
            fs.insert(format!("/{name}"), bytes.to_vec());
        }
    }
    for (name, bytes) in files {
        fs.insert(
            format!("{WORKSPACE_ROOT}/{}", name.trim_start_matches('/')),
            bytes.clone(),
        );
    }
    fs
}

fn options(main: &str, fs: &dyn FileSystem, stdlib: &Path) -> Options {
    let mut options = Options::host();
    options.os = TargetOs::Wasm;
    options.cpu = TargetCpu::Wasm;
    // Like the command line: the `modules` folder next to the main file is searched before the
    // stdlib.
    let main_dir = PathBuf::from(format!("{WORKSPACE_ROOT}/{}", main.trim_start_matches('/')))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(WORKSPACE_ROOT));
    options.import_paths = vec![main_dir.join("modules"), stdlib.to_path_buf()];
    options.preload =
        Some(jaic::find_preload(fs, stdlib).unwrap_or_else(|| stdlib.join(jaic::PRELOAD_ENTRY)));
    options
}

/// Split a `path:line:col: message` runtime-error string.
fn parse_located(message: &str) -> Option<(&str, u32, u32, &str)> {
    let (path, rest) = message.split_once(':')?;
    let (line, rest) = rest.split_once(':')?;
    let (column, rest) = rest.split_once(": ")?;
    Some((path, line.parse().ok()?, column.parse().ok()?, rest))
}

fn workspace_name(path: &str) -> String {
    let prefix = format!("{WORKSPACE_ROOT}/");
    path.strip_prefix(&prefix)
        .map_or_else(|| path.to_string(), str::to_string)
}

fn convert(compiler: &Compiler, d: &Diagnostic) -> PlayDiagnostic {
    let severity = match d.severity {
        Severity::Error => "error",
        Severity::Warning => "warning",
        Severity::Note => "note",
    };
    if let Some((path, line, column, message)) = parse_located(&d.message) {
        return PlayDiagnostic {
            severity,
            file: workspace_name(path),
            line,
            column,
            message: message.to_string(),
            code: d.kind.code(),
        };
    }
    if (d.span.file.0 as usize) < compiler.sources.len() {
        let file = compiler.sources.get(d.span.file);
        let (line, column) = file.line_col(d.span.start);
        return PlayDiagnostic {
            severity,
            file: workspace_name(&file.path),
            line,
            column,
            message: d.message.clone(),
            code: d.kind.code(),
        };
    }
    PlayDiagnostic {
        severity,
        file: String::new(),
        line: 0,
        column: 0,
        message: d.message.clone(),
        code: d.kind.code(),
    }
}

/// Limits for one playground run.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlayOptions {
    /// Interpreter basic blocks the main compile (its `#run`s and the program) may execute
    /// before it traps with "execution budget exhausted". `None` runs unbounded.
    pub budget: Option<u64>,
    /// Stop after compilation (every `#run` has executed); `main` is not run. The fuzz harness
    /// uses this to exercise the front end alone.
    pub compile_only: bool,
    /// Render compiler and runtime errors with ANSI colour and box drawing (the browser's
    /// output pane draws them); otherwise plain text.
    pub styled: bool,
}

/// Compile `main` (a key of `files`) against the bundled stdlib and run it.
pub fn run(files: &BTreeMap<String, Vec<u8>>, main: &str) -> PlayResult {
    run_with(files, main, PlayOptions::default())
}

/// What a run gets besides its files: the program's arguments and a fixed standard input.
#[derive(Debug, Clone, Default)]
pub struct PlayIo {
    /// The program's `argv`, program name first; empty passes no arguments at all.
    pub args: Vec<String>,
    /// The whole of standard input, when it is fixed; `None` leaves it to the page
    /// (`jai_stdin_read`), or to end at once without one.
    pub stdin: Option<Vec<u8>>,
}

/// [`run`] with limits.
pub fn run_with(files: &BTreeMap<String, Vec<u8>>, main: &str, limits: PlayOptions) -> PlayResult {
    run_with_io(files, main, limits, &PlayIo::default())
}

/// [`run_with`] with arguments and standard input.
pub fn run_with_io(
    files: &BTreeMap<String, Vec<u8>>,
    main: &str,
    limits: PlayOptions,
    io: &PlayIo,
) -> PlayResult {
    run_with_library(files, main, limits, io, None)
}

/// Compile against an explicitly supplied library directory inside this workspace.
/// Library ownership and private-source access remain the compiler's normal path rules.
pub fn run_with_library(
    files: &BTreeMap<String, Vec<u8>>,
    main: &str,
    limits: PlayOptions,
    io: &PlayIo,
    library_root: Option<&str>,
) -> PlayResult {
    let style = if limits.styled {
        jaic::render::Style {
            layout: jaic::render::Layout::Unicode,
            color: true,
        }
    } else {
        jaic::render::Style::PLAIN
    };
    // Scoped to this thread: concurrent runs (parallel tests) keep their own style.
    jaic::render::with_style(style, || run_styled(files, main, limits, io, library_root))
}

fn run_styled(
    files: &BTreeMap<String, Vec<u8>>,
    main: &str,
    limits: PlayOptions,
    io: &PlayIo,
    library_root: Option<&str>,
) -> PlayResult {
    let mut result = PlayResult::default();
    if !files.contains_key(main) {
        result.failure_phase = Some("input");
        result.diagnostics.push(PlayDiagnostic {
            severity: "error",
            file: String::new(),
            line: 0,
            column: 0,
            message: format!("main file '{main}' was not supplied"),
            code: None,
        });
        return result;
    }
    let fs: Rc<dyn FileSystem> = Rc::new(workspace_fs(files, library_root.is_none()));
    let stdlib = if let Some(root) = library_root {
        let valid = !root.contains(['\\', ':', '\0'])
            && root
                .split('/')
                .all(|part| !part.is_empty() && part != "." && part != "..");
        let path = Path::new(WORKSPACE_ROOT).join(root);
        if !valid || jaic::find_preload(&*fs, &path).is_none() {
            result.failure_phase = Some("input");
            result.diagnostics.push(PlayDiagnostic {
                severity: "error", file: String::new(), line: 0, column: 0,
                message: "supplied library root must be a relative workspace directory containing its prelude".into(), code: None,
            });
            return result;
        }
        path
    } else {
        PathBuf::from(STDLIB_ROOT)
    };
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        WORKSPACE_ROOT,
    )));
    // Metaprogram workspaces are checked (no output backend in the browser).
    let workspace_host = host.clone();
    let reports = host.clone();
    if let Some(input) = &io.stdin {
        host.borrow_mut().set_stdin(input);
    }
    let workspaces = Workspaces::new(BuildEnv {
        unwritten_output_hint: None,
        fs: fs.clone(),
        options: options(main, &*fs, &stdlib),
        backend: None,
        command_line: Vec::new(),
        make_host: Box::new(move |_| Box::new(SharedHost(workspace_host.clone()))),
        report: Box::new(move |text| {
            reports
                .borrow_mut()
                .write(format!("{text}\n").as_bytes(), true)
        }),
        observer: None,
    });
    let mut compiler = Compiler::new(options(main, &*fs, &stdlib), fs);
    compiler.interp.host = Box::new(crate::host_bridge::PlayHost::new(
        SharedHost(host.clone()),
        limits.budget,
    ));
    compiler.interp.block_budget = limits.budget;
    compiler.attach_workspaces(workspaces.clone());
    let entry = PathBuf::from(format!("{WORKSPACE_ROOT}/{}", main.trim_start_matches('/')));
    workspaces.borrow_mut().main_file = entry.display().to_string();
    let mut ran_program = false;
    let outcome = match compiler.compile_program(&entry) {
        Ok(()) => match jaic::build::finish_all(&workspaces, &mut compiler.interp.block_budget) {
            Err(message) => Err(Box::new(Diagnostic::error(Span::NONE, message))),
            Ok(()) if workspaces.borrow().any_failed() => Err(Box::new(Diagnostic::error(
                Span::NONE,
                "compilation failed in a metaprogram workspace (see stderr for diagnostics)",
            ))),
            Ok(()) if limits.compile_only => Ok(None),
            Ok(()) => {
                ran_program = true;
                compiler.run_program_with_args(&io.args).map(Some)
            }
        },
        Err(d) => Err(d),
    };
    match outcome {
        Ok(code) => result.exit_code = code,
        Err(d) => {
            result.failure_phase = Some(if ran_program {
                "runtime"
            } else {
                "compile"
            });
            result.rendered = compiler.render(&d);
            result.diagnostics.push(convert(&compiler, &d));
        }
    }
    for warning in &compiler.warnings {
        result.diagnostics.push(convert(&compiler, warning));
    }
    let host = host.borrow();
    result.stdout = String::from_utf8_lossy(&host.stdout).into_owned();
    result.stderr = String::from_utf8_lossy(&host.stderr).into_owned();
    let (mut out, mut err) = (0, 0);
    for &(to_stderr, len) in &host.order {
        let (stream, bytes, at) = if to_stderr {
            ("stderr", &host.stderr, &mut err)
        } else {
            ("stdout", &host.stdout, &mut out)
        };
        let text = String::from_utf8_lossy(&bytes[*at..*at + len]).into_owned();
        *at += len;
        result.output.push((stream, text));
    }
    result
}

#[cfg(test)]
mod tests {
    fn supplied_library_fixture() -> BTreeMap<String, Vec<u8>> {
        let mut files: BTreeMap<_, _> = BUNDLED
            .iter()
            .map(|(name, bytes)| {
                (
                    format!("library/{}", name.strip_prefix("stdlib/").unwrap()),
                    bytes.to_vec(),
                )
            })
            .collect();
        for (name, text) in [
            (
                "library/internal/browser-fixture/module.jai",
                "answer :: 17;",
            ),
            (
                "library/Public_Fixture/module.jai",
                "Impl :: #import \"internal/browser-fixture\"; answer :: Impl.answer;",
            ),
            (
                "library/tests/entry.jai",
                "Impl :: #import \"internal/browser-fixture\"; main :: () { #assert Impl.answer == 17; }",
            ),
            (
                "external/main.jai",
                "Public :: #import \"Public_Fixture\"; main :: () { #assert Public.answer == 17; }",
            ),
            (
                "external/private.jai",
                "Impl :: #import \"internal/browser-fixture\"; main :: () {}",
            ),
        ] {
            files.insert(name.into(), text.as_bytes().to_vec());
        }
        files
    }

    #[test]
    fn supplied_library_preserves_owned_private_context_and_external_boundary() {
        let files = supplied_library_fixture();
        let limits = PlayOptions {
            compile_only: true,
            ..PlayOptions::default()
        };
        for main in ["library/tests/entry.jai", "external/main.jai"] {
            let result =
                run_with_library(&files, main, limits, &PlayIo::default(), Some("library"));
            assert!(
                result.diagnostics.iter().all(|d| d.severity != "error"),
                "{result:?}"
            );
        }
        let result = run_with_library(
            &files,
            "external/private.jai",
            limits,
            &PlayIo::default(),
            Some("library"),
        );
        assert!(
            result.diagnostics.iter().any(|d| d
                .message
                .contains("standard-library internal sources are private")),
            "{result:?}"
        );
        // Replacing the library never exposes the unselected bundled private tree.
        assert!(!workspace_fs(&files, false).is_dir(Path::new("/stdlib/internal")));
    }

    #[test]
    fn supplied_library_rejects_escape_missing_and_invalid_roots() {
        let files = BTreeMap::from([("main.jai".into(), b"main :: () {}".to_vec())]);
        for root in [
            "",
            "../stdlib",
            "/stdlib",
            "x/../stdlib",
            "x//y",
            "x\\y",
            "x:dir",
            "missing",
        ] {
            let result = run_with_library(
                &files,
                "main.jai",
                PlayOptions {
                    compile_only: true,
                    ..PlayOptions::default()
                },
                &PlayIo::default(),
                Some(root),
            );
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|d| d.message.contains("supplied library root")),
                "{root}: {result:?}"
            );
        }
    }

    use super::*;

    fn single(source: &str) -> PlayResult {
        let mut files = BTreeMap::new();
        files.insert("main.jai".to_string(), source.as_bytes().to_vec());
        run(&files, "main.jai")
    }

    #[test]
    fn failed_child_workspace_prevents_parent_execution_and_compile_only_success() {
        let source = r##"
            C :: #import "Compiler";
            #run {
                w := C.compiler_create_workspace("invalid child");
                options := C.get_build_options(w);
                options.output_type = .NO_OUTPUT;
                C.set_build_options(options, w);
                C.add_build_string("#assert false; main :: () {}", w);
            }
            main :: () -> int { return 42; }
        "##;
        let files = BTreeMap::from([("main.jai".into(), source.as_bytes().to_vec())]);
        for compile_only in [false, true] {
            let result = run_with(
                &files,
                "main.jai",
                PlayOptions {
                    compile_only,
                    ..PlayOptions::default()
                },
            );
            assert_eq!(result.exit_code, None);
            assert!(
                result
                    .diagnostics
                    .iter()
                    .any(|d| d.message.contains("workspace")),
                "{}",
                result.rendered
            );
            assert!(
                result.stderr.contains("#assert failed"),
                "{}",
                result.stderr
            );
        }
    }

    /// `styled` errors carry ANSI colour and box drawing for the browser's output pane; the
    /// default stays plain text.
    #[test]
    fn styled_errors_are_coloured() {
        let mut files = BTreeMap::new();
        files.insert(
            "main.jai".to_string(),
            b"main :: () { x: int = \"no\"; }\n".to_vec(),
        );
        let styled = PlayOptions {
            styled: true,
            ..PlayOptions::default()
        };
        let r = run_with(&files, "main.jai", styled);
        assert!(
            r.rendered.contains("\x1b[") && r.rendered.contains('│'),
            "{}",
            r.rendered
        );
        let r = run(&files, "main.jai");
        assert!(
            !r.rendered.contains('\x1b') && !r.rendered.is_empty(),
            "{}",
            r.rendered
        );
    }

    fn with_input(source: &str, args: &[&str], stdin: &str) -> PlayResult {
        let mut files = BTreeMap::new();
        files.insert("main.jai".to_string(), source.as_bytes().to_vec());
        let io = PlayIo {
            args: args.iter().map(|a| a.to_string()).collect(),
            stdin: Some(stdin.as_bytes().to_vec()),
        };
        run_with_io(&files, "main.jai", PlayOptions::default(), &io)
    }

    /// Lines come from `fgets` and `getline` on `stdin`, `getc` and `fread` read the rest, and
    /// the end of input reads as the end.
    #[test]
    fn standard_input_is_read_through_the_c_stream() {
        let r = with_input(
            concat!(
                "#import \"Basic\";\n",
                "libc :: #system_library \"libc\";\n",
                "stdin: *void #elsewhere libc;\n",
                "fgets :: (buffer: *u8, count: s32, stream: *void) -> *u8 #foreign libc;\n",
                "getline :: (buffer: **u8, count: *u64, stream: *void) -> s64 #foreign libc;\n",
                "getc :: (stream: *void) -> s32 #foreign libc;\n",
                "fread :: (buffer: *void, size: u64, count: u64, stream: *void) -> u64 #foreign libc;\n",
                "feof :: (stream: *void) -> s32 #foreign libc;\n",
                "main :: () {\n",
                "    line: [32] u8;\n",
                "    p := fgets(line.data, 32, stdin);\n",
                "    print(\"fgets=%|\", to_string(line.data));\n",
                "    text: *u8; cap: u64;\n",
                "    n := getline(*text, *cap, stdin);\n",
                "    print(\"getline=%:%|\", n, string.{ n, text });\n",
                "    c := getc(stdin);\n",
                "    print(\"getc=%|\", c);\n",
                "    rest: [8] u8;\n",
                "    m := fread(rest.data, 1, 8, stdin);\n",
                "    print(\"fread=%:%|\", m, string.{ xx m, rest.data });\n",
                "    print(\"eof=%|\", feof(stdin) != 0);\n",
                "    q := fgets(line.data, 32, stdin);\n",
                "    print(\"after=%\\n\", q == null);\n",
                "}\n",
            ),
            &[],
            "one\ntwo words\nxyz",
        );
        assert_eq!(r.exit_code, Some(0), "{}\n{:?}", r.rendered, r.stderr);
        assert_eq!(
            r.stdout,
            "fgets=one\n|getline=10:two words\n|getc=120|fread=2:yz|eof=true|after=true\n"
        );
    }

    #[test]
    fn arguments_reach_main() {
        let r = with_input(
            "#import \"Basic\";\nmain :: () { for get_command_line_arguments() print(\"<%>\", it); }\n",
            &["prog", "a b", ""],
            "",
        );
        assert_eq!(
            r.stdout, "<prog><a b><>",
            "{:?}\n{}",
            r.diagnostics, r.rendered
        );
    }

    #[test]
    fn hello_world_prints_through_bundled_basic() {
        let r = single("#import \"Basic\";\nmain :: () { print(\"Hello, %!\\n\", 42); }\n");
        assert!(
            r.diagnostics.is_empty(),
            "{:?}\n{}",
            r.diagnostics,
            r.rendered
        );
        assert_eq!(r.stdout, "Hello, 42!\n");
        assert_eq!(r.exit_code, Some(0));
        assert!(r.to_json().contains("\"stdout\":\"Hello, 42!\\n\""));
    }

    #[test]
    fn independent_compiler_records_and_registered_data_work_in_browser_host() {
        let r = single(
            r#"
            #import "Basic";
            C :: #import "Compiler";
            bytes :: #run C.add_global_data(u8.[17, 31], .WRITABLE);
            #run {
                root := C.compiler_get_nodes(#code 7 + 9);
                assert(root && root.kind == .BINARY_OPERATOR);
                info := C.get_runtime_info();
                assert(info.global_data_info != null);
            }
            main :: () {
                assert(bytes.count == 2 && bytes[0] == 17);
                assert(C.get_runtime_info().global_data_info != null);
                print("browser compiler ok\n");
            }
        "#,
        );
        assert!(
            r.diagnostics.is_empty(),
            "{:?}\n{}",
            r.diagnostics,
            r.rendered
        );
        assert_eq!(r.stdout, "browser compiler ok\n");
        assert_eq!(r.exit_code, Some(0));
    }

    #[test]
    fn output_keeps_the_order_of_stdout_and_stderr_writes() {
        let r = single(concat!(
            "#import \"Basic\";\n",
            "main :: () { print(\"a\"); print(\"b\\n\"); log_error(\"bad\\n\"); print(\"c\\n\"); }\n",
        ));
        assert_eq!(r.stdout, "ab\nc\n", "{}", r.rendered);
        assert_eq!(r.stderr, "bad\n");
        assert_eq!(
            r.output,
            [
                ("stdout", "ab\n".to_string()),
                ("stderr", "bad\n".to_string()),
                ("stdout", "c\n".to_string())
            ]
        );
        assert!(r.to_json().contains(
            "\"output\":[{\"stream\":\"stdout\",\"text\":\"ab\\n\"},{\"stream\":\"stderr\""
        ));
    }

    #[test]
    fn budget_stops_a_runaway_program() {
        let mut files = BTreeMap::new();
        files.insert(
            "main.jai".to_string(),
            b"#import \"Basic\";\nmain :: () { print(\"start\\n\"); while true {} }\n".to_vec(),
        );
        let r = run_with(
            &files,
            "main.jai",
            PlayOptions {
                budget: Some(100_000),
                ..PlayOptions::default()
            },
        );
        assert_eq!(r.stdout, "start\n");
        assert!(r.exit_code.is_none() || r.exit_code != Some(0));
        let text = format!("{:?}{}", r.diagnostics, r.rendered);
        assert!(text.contains("execution budget exhausted"), "{text}");
    }

    #[test]
    fn budget_covers_the_workspaces_a_metaprogram_compiles() {
        // A workspace's compiler had no budget of its own, so an endless `#run` in a program
        // that a metaprogram compiled hung the playground.
        let source = concat!(
            "#import \"Basic\";\n",
            "#import \"Compiler\";\n",
            "#run {\n",
            "    w := compiler_create_workspace();\n",
            "    options := get_build_options(w);\n",
            "    options.output_type = .NO_OUTPUT;\n",
            "    set_build_options(options, w);\n",
            "    compiler_begin_intercept(w);\n",
            "    add_build_string(\"work :: () { while true {} }\\n#run work();\\n\", w);\n",
            "    while true {\n",
            "        message := compiler_wait_for_message();\n",
            "        if message.kind == .COMPLETE break;\n",
            "    }\n",
            "    compiler_end_intercept(w);\n",
            "}\n",
            "main :: () {}\n",
        );
        let mut files = BTreeMap::new();
        files.insert("main.jai".to_string(), source.as_bytes().to_vec());
        let r = run_with(
            &files,
            "main.jai",
            PlayOptions {
                budget: Some(100_000),
                compile_only: true,
                ..PlayOptions::default()
            },
        );
        let text = format!("{:?}{}{}", r.diagnostics, r.rendered, r.stderr);
        assert!(text.contains("execution budget exhausted"), "{text}");
    }

    #[test]
    fn made_up_pointers_in_compile_time_values() {
        // Found by the `interp` fuzz target: freezing a `#run` result read the memory behind a
        // pointer made from an integer (80000000, above the 64 KiB that are always treated as a
        // number), and the compiler crashed. A pointer to memory that cannot be read stays a
        // number; a string whose data cannot be read is an error.
        let run = |source: &str| {
            let mut files = BTreeMap::new();
            files.insert("main.jai".to_string(), source.as_bytes().to_vec());
            run_with(&files, "main.jai", PlayOptions::default())
        };
        let r = run(concat!(
            "#import \"Basic\";\n",
            "Handle :: *struct { unused: u8; };\n",
            "H :: cast(Handle) 80000000;\n",
            "P :: #run cast(*int) 80000000;\n",
            "main :: () { assert(cast(u64) H == 80000000 && cast(u64) P == 80000000); }\n",
        ));
        assert!(
            r.diagnostics.is_empty() && r.exit_code == Some(0),
            "{}{}",
            r.rendered,
            r.stderr
        );
        let r = run(concat!(
            "S :: #run () -> string { s: string; s.count = 5; s.data = cast(*u8) 80000000; return s; }();\n",
            "main :: () {}\n",
        ));
        assert!(
            r.rendered.contains("cannot be read"),
            "{}{}",
            r.rendered,
            r.stderr
        );
    }

    #[test]
    fn code_added_every_round_is_expanded() {
        // Found by the `lsp_edits` fuzz target: a metaprogram that adds code at every
        // TYPECHECKED_ALL_WE_CAN made each round revisit every scope and file so far, so a
        // long run of rounds took quadratic time. Rounds now visit only what is unfinished; the
        // `#if`s and imports in each added string must still expand.
        let source = concat!(
            "#import \"Basic\";\n",
            "#import \"Compiler\";\n",
            "#run {\n",
            "    w := compiler_create_workspace();\n",
            "    options := get_build_options(w);\n",
            "    options.output_type = .NO_OUTPUT;\n",
            "    set_build_options(options, w);\n",
            "    compiler_begin_intercept(w);\n",
            "    add_build_string(\"#import \\\"Basic\\\";\\nmain :: () {}\\n\", w);\n",
            "    rounds := 0;\n",
            "    failed := true;\n",
            "    while true {\n",
            "        message := compiler_wait_for_message();\n",
            "        if message.kind == .PHASE {\n",
            "            phase := cast(*Message_Phase) message;\n",
            "            if phase.phase == .TYPECHECKED_ALL_WE_CAN && rounds < 300 {\n",
            "                add_build_string(tprint(\"#import \\\"Math\\\";\\n#if % >= 0 { X_% :: %; }\\n\", rounds, rounds, rounds), w);\n",
            "                rounds += 1;\n",
            "                if rounds == 300 add_build_string(\"#run assert(X_0 + X_299 == 299);\\n\", w);\n",
            "            }\n",
            "        }\n",
            "        if message.kind == .COMPLETE {\n",
            "            failed = (cast(*Message_Complete) message).error_code != .NONE;\n",
            "            break;\n",
            "        }\n",
            "    }\n",
            "    compiler_end_intercept(w);\n",
            "    assert(rounds == 300 && !failed);\n",
            "}\n",
            "main :: () {}\n",
        );
        let mut files = BTreeMap::new();
        files.insert("main.jai".to_string(), source.as_bytes().to_vec());
        let r = run_with(
            &files,
            "main.jai",
            PlayOptions {
                compile_only: true,
                ..PlayOptions::default()
            },
        );
        let text = format!("{:?}{}{}", r.diagnostics, r.rendered, r.stderr);
        assert!(r.diagnostics.is_empty(), "{text}");
    }

    #[test]
    fn unknown_escape_of_invalid_utf8() {
        // Found by the `check` fuzz target: the invalid byte reads as U+FFFD (three bytes), and
        // the error's span ended inside it, so rendering the error panicked.
        let mut files = BTreeMap::new();
        files.insert("main.jai".to_string(), b"x := \"\\\x8b\";\n".to_vec());
        let r = run(&files, "main.jai");
        assert!(
            r.rendered.contains("unknown escape sequence"),
            "{}",
            r.rendered
        );
    }

    #[test]
    fn allocations_the_host_cannot_make_fail_without_aborting() {
        // Found by fuzzing: an infallible host allocation aborted the whole compiler.
        let r = single(concat!(
            "#import \"Basic\";\n",
            "BIG: [1 << 48] u8;\n",
            "main :: () {\n",
            "    p := alloc(1 << 60);\n",
            "    q := alloc(0x7fffffffffffffff);\n",
            "    print(\"% %\\n\", p == null, q == null);\n",
            "    BIG[1] = 1;\n",
            "}\n",
        ));
        assert_eq!(
            r.stdout, "true true\n",
            "{:?}\n{}",
            r.diagnostics, r.rendered
        );
        let text = format!("{:?}", r.diagnostics);
        assert!(text.contains("cannot allocate"), "{text}");
    }

    #[test]
    fn negative_basic_allocation_reports_its_assertion_without_aborting_the_host() {
        let r = single("#import \"Basic\"; main :: () { alloc(-1); }\n");
        assert_eq!(r.exit_code, None);
        assert_eq!(r.diagnostics.len(), 1, "{}", r.rendered);
        assert_eq!(r.diagnostics[0].code, Some("runtime-error"));
        assert!(
            r.diagnostics[0]
                .message
                .contains("assertion failed: `size >= 0` is false"),
            "{}",
            r.rendered
        );
        assert!(r.rendered.contains("`alloc`"), "{}", r.rendered);
    }

    #[test]
    fn compile_only_skips_main() {
        let mut files = BTreeMap::new();
        files.insert(
            "main.jai".to_string(),
            b"#import \"Basic\";\n#run print(\"compile\\n\");\nmain :: () { print(\"run\\n\"); }\n"
                .to_vec(),
        );
        let options = PlayOptions {
            compile_only: true,
            ..PlayOptions::default()
        };
        let r = run_with(&files, "main.jai", options);
        assert_eq!(r.stdout, "compile\n", "{:?}\n{}", r.diagnostics, r.rendered);
        assert_eq!(r.exit_code, None);
    }

    #[test]
    fn loads_sibling_workspace_files() {
        let mut files = BTreeMap::new();
        files.insert(
            "main.jai".to_string(),
            concat!(
                "#import \"Basic\";\n",
                "#load \"lib/helper.jai\";\n",
                "main :: () { print(\"%\\n\", twice(21)); }\n",
            )
            .as_bytes()
            .to_vec(),
        );
        files.insert(
            "lib/helper.jai".to_string(),
            b"twice :: (x: int) -> int { return x * 2; }\n".to_vec(),
        );
        let r = run(&files, "main.jai");
        assert_eq!(r.stdout, "42\n", "{}", r.rendered);
    }

    #[test]
    fn reports_diagnostics_with_positions() {
        let r = single("main :: () {\n    x := missing;\n}\n");
        assert_eq!(r.exit_code, None);
        let d = &r.diagnostics[0];
        assert_eq!(
            (d.file.as_str(), d.line),
            ("main.jai", 2),
            "{:?}",
            r.diagnostics
        );
        assert!(d.message.contains("missing"));
        assert_eq!(d.code, Some("unknown-identifier"));
        assert!(r.to_json().contains("\"code\":\"unknown-identifier\""));
    }

    /// C stream input retains empty and unterminated lines. The test program
    /// trims CRLF for display; it does not depend on a stdlib helper.
    #[test]
    fn c_stream_lines_include_empty_and_unterminated_lines() {
        let r = with_input(
            concat!(
                "#import \"Basic\";\n",
                "libc :: #system_library \"libc\";\n",
                "stdin: *void #elsewhere libc;\n",
                "getline :: (buffer: **u8, count: *u64, stream: *void) -> s64 #foreign libc;\n",
                "main :: () {\n",
                "    line: *u8; capacity: u64;\n",
                "    defer free(line);\n",
                "    while true {\n",
                "        count := getline(*line, *capacity, stdin);\n",
                "        if count < 0 break;\n",
                "        if count > 0 && line[count - 1] == 10 count -= 1;\n",
                "        if count > 0 && line[count - 1] == 13 count -= 1;\n",
                "        print(\"[%]\", string.{count, line});\n",
                "    }\n",
                "    print(\"\\n\");\n",
                "}\n",
            ),
            &[],
            "one\r\n\ntwo words\nlast",
        );
        assert_eq!(r.exit_code, Some(0), "{}\n{:?}", r.rendered, r.stderr);
        assert_eq!(r.stdout, "[one][][two words][last]\n");
    }

    /// The C process-exit bridge retains output and returns the supplied status.
    /// Basic.exit is only supported on native POSIX/Windows targets.
    #[test]
    fn exit_ends_the_program_with_its_code() {
        let r = single(
            "#import \"Basic\";\nlibc :: #system_library \"libc\";\nc_exit :: (status: s32) #foreign libc \"exit\";\nmain :: () {\n    print(\"before\\n\");\n    c_exit(7);\n    print(\"after\\n\");\n}\n",
        );
        assert_eq!(r.stdout, "before\n", "{:?}\n{}", r.diagnostics, r.rendered);
        assert_eq!(r.exit_code, Some(7), "{}", r.rendered);
        assert!(r.diagnostics.is_empty(), "{:?}", r.diagnostics);
    }

    #[test]
    fn a_foreign_procedure_the_sandbox_lacks_has_a_code() {
        let r = single(
            "lib :: #library \"libnothere\";\nnothere :: () #foreign lib;\nmain :: () { nothere(); }\n",
        );
        let codes: Vec<_> = r.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(codes, [Some("unavailable")], "{:?}", r.diagnostics);
    }

    #[test]
    fn a_trap_in_the_running_program_is_a_runtime_error() {
        let r = single(
            "#import \"Basic\";\nmain :: () {\n    print(\"before\\n\");\n    a: [2] int;\n    i := 5;\n    a[i] = 1;\n}\n",
        );
        assert_eq!(r.stdout, "before\n");
        let codes: Vec<_> = r.diagnostics.iter().map(|d| d.code).collect();
        assert_eq!(codes, [Some("runtime-error")], "{:?}", r.diagnostics);
    }

    #[test]
    fn public_thread_module_rejects_browser_target() {
        let r = single("#import \"Thread\"; main :: () {}");
        assert_eq!(r.exit_code, None);
        assert!(
            r.diagnostics
                .iter()
                .any(|d| d.code == Some("static-assert")),
            "{}",
            r.rendered
        );
        assert!(r.rendered.contains("OS"), "{}", r.rendered);
    }

    #[test]
    fn sandbox_threads_run_cooperatively_and_join_actual_results() {
        for _ in 0..3 {
            let r = single(include_str!("../tests/fixtures/cooperative-runtime.jai"));
            assert_eq!(r.stdout, "ok\n", "{:?}\n{}", r.diagnostics, r.rendered);
            assert_eq!(r.exit_code, Some(0));
        }
    }

    #[test]
    fn tour_threads_use_the_private_runtime_and_join_real_workers() {
        let source = format!(
            "#import \"Basic\";\n{}\nmain :: () {{ tour_threads(); }}",
            include_str!("../../../examples/tour/threads/cooperative.jai")
        );
        let result = single(&source);
        assert_eq!(result.exit_code, Some(0), "{}", result.rendered);
        assert!(
            result.stdout.contains(
                "3 workers counted the primes in 8 slices of 1000: 168 135 127 120 119 114 117 107"
            ),
            "{}",
            result.stdout
        );
        assert!(
            result
                .stdout
                .contains("The consumer received: 1 4 9 16 25 36 49 64 81 100 (sum 385)"),
            "{}",
            result.stdout
        );
    }

    #[test]
    fn sandbox_files_live_in_memory_and_include_workspace_sources() {
        let r = single(include_str!("../tests/fixtures/sandbox-files.jai"));
        assert_eq!(r.stdout, "ok\n", "{:?}\n{}", r.diagnostics, r.rendered);
        assert_eq!(r.exit_code, Some(0));
        let isolated = single(
            "fopen :: (path:*u8,mode:*u8)->*void #foreign; main :: () -> int { if fopen(\"/tmp/x.txt\\0\".data,\"rb\\0\".data)!=null return 1; return 0; }",
        );
        assert_eq!(isolated.exit_code, Some(0), "{}", isolated.rendered);
    }

    #[test]
    fn sandbox_scheduler_preserves_errors_and_reports_deadlock() {
        let invalid = single(
            "pthread_join :: (handle:u64,result:**void)->s32 #foreign; main :: ()->int {if pthread_join(0,null)==0 return 1;return 0;}",
        );
        assert_eq!(invalid.exit_code, Some(0), "{}", invalid.rendered);
        let deadlocked = single(
            r#"
            pthread_create :: (thread:*u64,attributes:*void,entry:*void,data:*void)->s32 #foreign;
            pthread_join :: (thread:u64,result:**void)->s32 #foreign;
            pthread_mutex_init :: (token:*void,attributes:*void)->s32 #foreign;
            pthread_mutex_lock :: (token:*void)->s32 #foreign;
            mutex:u64;
            worker :: (argument:*void)->*void #c_call {pthread_mutex_lock(*mutex);return argument;}
            main :: ()->int {
                if pthread_mutex_init(*mutex,null)!=0 return 1;
                if pthread_mutex_lock(*mutex)!=0 return 2;
                thread:u64;
                if pthread_create(*thread,null,cast(*void)worker,null)!=0 return 3;
                return pthread_join(thread,null);
            }
        "#,
        );
        assert_eq!(deadlocked.exit_code, None);
        assert!(
            deadlocked
                .diagnostics
                .iter()
                .any(|d| d.message.contains("deadlock")),
            "{}",
            deadlocked.rendered
        );
    }

    #[test]
    fn modules_folder_next_to_main_is_searched() {
        let mut files = BTreeMap::new();
        files.insert(
            "main.jai".to_string(),
            concat!(
                "#import \"Basic\";\n",
                "#import \"Local\";\n",
                "main :: () { print(\"%\\n\", from_local()); }\n",
            )
            .as_bytes()
            .to_vec(),
        );
        files.insert(
            "modules/Local/module.jai".to_string(),
            b"from_local :: () -> int { return 7; }\n".to_vec(),
        );
        let r = run(&files, "main.jai");
        assert_eq!(r.stdout, "7\n", "{:?}\n{}", r.diagnostics, r.rendered);
    }
}
