//! Playground entry point: compile and run a set of files with the `jaic` core.
use jaic::build::{BuildEnv, Workspaces};
use jaic::interp::{Host, SandboxHost};
use jaic::ir;
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
}

#[derive(Debug, Clone, Default)]
pub struct PlayResult {
    /// `None` when compilation failed before the program could run.
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
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

/// Shares a `SandboxHost` with the caller so output can be read after the run.
struct SharedHost(Rc<RefCell<SandboxHost>>);
impl Host for SharedHost {
    fn write(&mut self, bytes: &[u8], to_stderr: bool) {
        self.0.borrow_mut().write(bytes, to_stderr);
    }
    fn foreign(
        &mut self,
        symbol: &str,
        args: &[u64],
        sig: &ir::Sig,
    ) -> Option<Result<Vec<u64>, String>> {
        self.0.borrow_mut().foreign(symbol, args, sig)
    }
    fn native_linking(&self) -> bool {
        false
    }
}

fn virtual_fs(files: &BTreeMap<String, Vec<u8>>) -> VirtualFs {
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

fn options() -> Options {
    let mut options = Options::host();
    options.os = TargetOs::Wasm;
    options.cpu = TargetCpu::Wasm;
    options.import_paths = vec![PathBuf::from(STDLIB_ROOT)];
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
        };
    }
    let has_location = d.span.start != 0 || d.span.end != 0;
    if has_location && (d.span.file.0 as usize) < compiler.sources.len() {
        let file = compiler.sources.get(d.span.file);
        let (line, column) = file.line_col(d.span.start);
        return PlayDiagnostic {
            severity,
            file: workspace_name(&file.path),
            line,
            column,
            message: d.message.clone(),
        };
    }
    PlayDiagnostic {
        severity,
        file: String::new(),
        line: 0,
        column: 0,
        message: d.message.clone(),
    }
}

/// Compile `main` (a key of `files`) against the bundled stdlib and run it.
pub fn run(files: &BTreeMap<String, Vec<u8>>, main: &str) -> PlayResult {
    let mut result = PlayResult::default();
    if !files.contains_key(main) {
        result.diagnostics.push(PlayDiagnostic {
            severity: "error",
            file: String::new(),
            line: 0,
            column: 0,
            message: format!("main file '{main}' was not supplied"),
        });
        return result;
    }
    let host = Rc::new(RefCell::new(SandboxHost::default()));
    let fs: Rc<dyn FileSystem> = Rc::new(virtual_fs(files));
    // Metaprogram workspaces are checked (no output backend in the browser).
    let workspace_host = host.clone();
    let reports = host.clone();
    let workspaces = Workspaces::new(BuildEnv {
        fs: fs.clone(),
        options: options(),
        backend: None,
        command_line: Vec::new(),
        make_host: Box::new(move || Box::new(SharedHost(workspace_host.clone()))),
        report: Box::new(move |text| {
            reports
                .borrow_mut()
                .write(format!("{text}\n").as_bytes(), true)
        }),
    });
    let mut compiler = Compiler::new(options(), fs);
    compiler.interp.host = Box::new(SharedHost(host.clone()));
    compiler.attach_workspaces(workspaces.clone());
    let entry = PathBuf::from(format!("{WORKSPACE_ROOT}/{}", main.trim_start_matches('/')));
    let outcome = match compiler.compile_program(&entry) {
        Ok(()) => {
            if let Err(message) = jaic::build::finish_all(&workspaces) {
                let text = format!("error: {message}\n");
                host.borrow_mut().write(text.as_bytes(), true);
            }
            compiler.run_program().map(Some)
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
    fn loads_sibling_workspace_files() {
        let mut files = BTreeMap::new();
        files.insert(
            "main.jai".to_string(),
            b"#import \"Basic\";\n#load \"lib/helper.jai\";\nmain :: () { print(\"%\\n\", twice(21)); }\n"
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
    }
}
