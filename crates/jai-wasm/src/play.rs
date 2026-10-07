//! Playground entry point: compile and run a set of files with the `jaic` core.
use jaic::build::{BuildEnv, Workspaces};
use jaic::interp::{Host, SandboxHost, SharedHost};
use jaic::sema::{Compiler, FileSystem, Options, TargetCpu, TargetOs, VirtualFs};
use jaic::source::{Diagnostic, Severity};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
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
    /// `None` when compilation failed before the program could run.
    pub exit_code: Option<i32>,
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
    let mut fs = VirtualFs::default();
    for (name, bytes) in BUNDLED {
        fs.insert(format!("/{name}"), bytes.to_vec());
    }
    for (name, bytes) in files {
        fs.insert(
            format!("{WORKSPACE_ROOT}/{}", name.trim_start_matches('/')),
            bytes.clone(),
        );
    }
    fs
}

fn options(main: &str) -> Options {
    let mut options = Options::host();
    options.os = TargetOs::Wasm;
    options.cpu = TargetCpu::Wasm;
    // Like the command line: the `modules` folder next to the main file is searched before the
    // stdlib.
    let main_dir = PathBuf::from(format!("{WORKSPACE_ROOT}/{}", main.trim_start_matches('/')))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(WORKSPACE_ROOT));
    options.import_paths = vec![main_dir.join("modules"), PathBuf::from(STDLIB_ROOT)];
    options.preload = Some(PathBuf::from(format!("{STDLIB_ROOT}/module.jai")));
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

/// [`run`] with limits.
pub fn run_with(files: &BTreeMap<String, Vec<u8>>, main: &str, limits: PlayOptions) -> PlayResult {
    let style = if limits.styled {
        jaic::render::Style {
            layout: jaic::render::Layout::Unicode,
            color: true,
        }
    } else {
        jaic::render::Style::PLAIN
    };
    // Scoped to this thread: concurrent runs (parallel tests) keep their own style.
    jaic::render::with_style(style, || run_styled(files, main, limits))
}

fn run_styled(files: &BTreeMap<String, Vec<u8>>, main: &str, limits: PlayOptions) -> PlayResult {
    let mut result = PlayResult::default();
    if !files.contains_key(main) {
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
    let fs: Rc<dyn FileSystem> = Rc::new(virtual_fs(files));
    let host = Rc::new(RefCell::new(SandboxHost::with_files(
        fs.clone(),
        WORKSPACE_ROOT,
    )));
    // Metaprogram workspaces are checked (no output backend in the browser).
    let workspace_host = host.clone();
    let reports = host.clone();
    let workspaces = Workspaces::new(BuildEnv {
        unwritten_output_hint: None,
        fs: fs.clone(),
        options: options(main),
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
    let mut compiler = Compiler::new(options(main), fs);
    compiler.interp.host = Box::new(SharedHost(host.clone()));
    compiler.interp.block_budget = limits.budget;
    compiler.attach_workspaces(workspaces.clone());
    let entry = PathBuf::from(format!("{WORKSPACE_ROOT}/{}", main.trim_start_matches('/')));
    let outcome = match compiler.compile_program(&entry) {
        Ok(()) => {
            if let Err(message) = jaic::build::finish_all(&workspaces) {
                let text = format!("error: {message}\n");
                host.borrow_mut().write(text.as_bytes(), true);
            }
            if limits.compile_only {
                Ok(None)
            } else {
                compiler.run_program().map(Some)
            }
        }
        Err(d) => Err(d),
    };
    match outcome {
        Ok(code) => result.exit_code = code,
        Err(d) => {
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
    use super::*;

    fn single(source: &str) -> PlayResult {
        let mut files = BTreeMap::new();
        files.insert("main.jai".to_string(), source.as_bytes().to_vec());
        run(&files, "main.jai")
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
    fn output_keeps_the_order_of_stdout_and_stderr_writes() {
        let r = single(concat!(
            "#import \"Basic\";\n",
            "main :: () { print(\"a\"); print(\"b\\n\"); log_error(\"bad\"); print(\"c\\n\"); }\n",
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
            "    q := alloc(-1);\n",
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
    fn threads_run_cooperatively_and_files_live_in_memory() {
        let r = single(
            r##"#import "Basic";
#import "File";
#import "Thread";
total: s64;
mutex: Mutex;
worker :: (thread: *Thread) -> s64 {
    for 0..9 { lock(*mutex); total += 1; unlock(*mutex); }
    return 0;
}
main :: () {
    init(*mutex);
    threads: [3] Thread;
    for *threads { thread_init(it, worker); thread_start(it); }
    for *threads thread_deinit(it);
    assert(total == 30);
    assert(write_entire_file("/tmp/x.txt", "data"));
    text, ok := read_entire_file("/tmp/x.txt");
    assert(ok && text == "data");
    own, own_ok := read_entire_file("main.jai");
    assert(own_ok && own.count > 10);
    print("ok\n");
}
"##,
        );
        assert_eq!(r.stdout, "ok\n", "{:?}\n{}", r.diagnostics, r.rendered);
        assert_eq!(r.exit_code, Some(0));
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
