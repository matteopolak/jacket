//! Every procedure with a body that the files of some folders declare, read from the parsed
//! files without compiling them. `jaic -procedures DIRS` prints them for `jacket test
//! --coverage`, which counts the ones no program was built with as code that never ran.
//!
//! The files are the `.jai` files below each folder, without hidden folders and folders named
//! `tests`. A top-level or nested `#if` on `OS` or `CPU` keeps only the branch that holds for the
//! target; a condition of any other kind keeps both branches.
use crate::ast::{self, BinOp, ExprKind, StmtKind, UnOp};
use crate::parser::parse_file_ide;
use crate::sema::{TargetCpu, TargetOs};
use crate::source::FileId;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// One procedure with a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Procedure {
    pub file: PathBuf,
    pub name: String,
    /// The line it is declared on and the line its body ends on.
    pub line: u32,
    pub end: u32,
    /// The lines of the statements in the body (the declaration's too), without those of
    /// the procedures declared inside it. They are what a call of the procedure would run.
    pub lines: Vec<u32>,
}

/// The names `OS` and `CPU` have for the target in `#if` conditions.
struct Target {
    os: &'static str,
    cpu: &'static str,
}

impl Target {
    fn new(os: TargetOs, cpu: TargetCpu) -> Target {
        Target {
            os: match os {
                TargetOs::Windows => "WINDOWS",
                TargetOs::Linux => "LINUX",
                TargetOs::MacOS => "MACOS",
                TargetOs::Wasm => "WASM",
            },
            cpu: match cpu {
                TargetCpu::X64 => "X64",
                TargetCpu::Arm64 => "ARM64",
                TargetCpu::Wasm => "CUSTOM",
            },
        }
    }

    /// Whether `cond` is true for the target, or `None` when it asks about anything else.
    fn holds(&self, cond: &ast::Expr) -> Option<bool> {
        match &cond.kind {
            ExprKind::Bool(value) => Some(*value),
            ExprKind::Unary(UnOp::Not, inner) => self.holds(inner).map(|b| !b),
            ExprKind::Binary(op @ (BinOp::And | BinOp::Or), left, right) => {
                let (left, right) = (self.holds(left), self.holds(right));
                // The value that decides the result alone: false for `&&`, true for `||`.
                let decides = *op == BinOp::Or;
                if left == Some(decides) || right == Some(decides) {
                    Some(decides)
                } else {
                    left.zip(right).map(|_| !decides)
                }
            }
            ExprKind::Binary(op @ (BinOp::Eq | BinOp::Ne), left, right) => {
                let (ExprKind::Ident(name), ExprKind::InferredMember(member)) =
                    (&left.kind, &right.kind)
                else {
                    return None;
                };
                let current = match name.as_str() {
                    "OS" => self.os,
                    "CPU" => self.cpu,
                    _ => return None,
                };
                Some((member.name.as_str() == current) == (*op == BinOp::Eq))
            }
            _ => None,
        }
    }
}

fn jai_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    paths.sort();
    for path in paths {
        if crate::paths::is_hidden(&path) {
            continue;
        }
        if path.is_dir() {
            if path.file_name().is_none_or(|n| n != "tests") {
                jai_files(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "jai") {
            out.push(path);
        }
    }
}

/// The procedures of every file below `dirs`, in file order, for the target.
pub fn library_procedures(dirs: &[PathBuf], os: TargetOs, cpu: TargetCpu) -> Vec<Procedure> {
    let target = Target::new(os, cpu);
    let mut files = Vec::new();
    for dir in dirs {
        jai_files(dir, &mut files);
    }
    let mut out = Vec::new();
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let file = file.canonicalize().unwrap_or(file);
        let parsed = parse_file_ide(FileId(0), &text);
        let lines = Lines::new(&text);
        Walk {
            target: &target,
            lines: &lines,
            file: &file,
            out: &mut out,
        }
        .statements(&parsed.file.stmts);
    }
    out
}

/// The line of each byte offset of a file.
struct Lines(Vec<u32>);

impl Lines {
    fn new(text: &str) -> Lines {
        let starts = std::iter::once(0)
            .chain(text.match_indices('\n').map(|(i, _)| (i + 1) as u32))
            .collect();
        Lines(starts)
    }

    /// The 1-based line `offset` is on.
    fn of(&self, offset: u32) -> u32 {
        self.0.partition_point(|&start| start <= offset) as u32
    }
}

struct Walk<'a> {
    target: &'a Target,
    lines: &'a Lines,
    file: &'a Path,
    out: &'a mut Vec<Procedure>,
}

impl Walk<'_> {
    /// The statements of a file, a struct or a body: the procedures declared in them.
    fn statements(&mut self, stmts: &[ast::Stmt]) {
        for stmt in stmts {
            match &stmt.kind {
                StmtKind::StaticIf(branches) => match self.target.holds(&branches.cond) {
                    Some(true) => self.statements(&branches.then_branch),
                    Some(false) => self.statements(&branches.else_branch),
                    None => {
                        self.statements(&branches.then_branch);
                        self.statements(&branches.else_branch);
                    }
                },
                StmtKind::Decl(decl) => self.declaration(decl, stmt.span.start),
                other => self.nested(other),
            }
        }
    }

    /// The procedures declared inside a statement that is not itself a declaration.
    fn nested(&mut self, kind: &StmtKind) {
        let mut bodies: Vec<&ast::Stmt> = Vec::new();
        match kind {
            StmtKind::Block(block) => return self.statements(&block.stmts),
            StmtKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                bodies.push(then_branch);
                bodies.extend(else_branch.as_deref());
            }
            StmtKind::Switch(switch) | StmtKind::StaticSwitch(switch) => {
                for case in &switch.cases {
                    self.statements(&case.body);
                }
            }
            StmtKind::While(w) => bodies.push(&w.body),
            StmtKind::For(f) => bodies.push(&f.body),
            StmtKind::Defer {
                body, ..
            } => bodies.push(body),
            StmtKind::PushContext(push) => bodies.push(&push.body),
            _ => {}
        }
        for body in bodies {
            self.statements(std::slice::from_ref(body));
        }
    }

    fn declaration(&mut self, decl: &ast::Decl, start: u32) {
        match decl.value.as_ref().map(|v| &v.kind) {
            Some(ExprKind::Proc(lit)) => {
                let Some(body) = &lit.body else {
                    return;
                };
                let Some(name) = decl.names.first() else {
                    return;
                };
                let mut lines = vec![self.lines.of(start)];
                self.body_lines(&body.stmts, &mut lines);
                lines.sort_unstable();
                lines.dedup();
                self.out.push(Procedure {
                    file: self.file.to_path_buf(),
                    name: name.name.as_str().to_string(),
                    line: self.lines.of(start),
                    end: self.lines.of(body.span.end.saturating_sub(1)),
                    lines,
                });
                self.statements(&body.stmts);
            }
            Some(ExprKind::Struct(lit)) => self.statements(&lit.body),
            _ => {}
        }
    }

    /// The lines of the statements in `stmts` that run, not those of nested procedures.
    fn body_lines(&self, stmts: &[ast::Stmt], out: &mut Vec<u32>) {
        for stmt in stmts {
            let line = self.lines.of(stmt.span.start);
            match &stmt.kind {
                StmtKind::Decl(decl) => {
                    if decl.kind != ast::DeclKind::Const {
                        out.push(line);
                    }
                }
                StmtKind::Empty | StmtKind::Scope(_) | StmtKind::Placeholder(_) => {}
                StmtKind::Block(block) => self.body_lines(&block.stmts, out),
                StmtKind::StaticIf(branches) => match self.target.holds(&branches.cond) {
                    Some(true) => self.body_lines(&branches.then_branch, out),
                    Some(false) => self.body_lines(&branches.else_branch, out),
                    None => {
                        self.body_lines(&branches.then_branch, out);
                        self.body_lines(&branches.else_branch, out);
                    }
                },
                StmtKind::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    out.push(line);
                    self.body_lines(std::slice::from_ref(then_branch), out);
                    if let Some(other) = else_branch {
                        self.body_lines(std::slice::from_ref(other), out);
                    }
                }
                StmtKind::Switch(switch) => {
                    out.push(line);
                    for case in &switch.cases {
                        self.body_lines(&case.body, out);
                    }
                }
                StmtKind::StaticSwitch(switch) => {
                    for case in &switch.cases {
                        self.body_lines(&case.body, out);
                    }
                }
                StmtKind::While(w) => {
                    out.push(line);
                    self.body_lines(std::slice::from_ref(&w.body), out);
                }
                StmtKind::For(f) => {
                    out.push(line);
                    self.body_lines(std::slice::from_ref(&f.body), out);
                }
                StmtKind::Defer {
                    body, ..
                } => self.body_lines(std::slice::from_ref(body), out),
                StmtKind::PushContext(push) => {
                    out.push(line);
                    self.body_lines(std::slice::from_ref(&push.body), out);
                }
                _ => out.push(line),
            }
        }
    }
}

/// The procedures as records, one per line: `procedure`, file, name, first line, last line and
/// the lines that run (comma-separated), separated by tabs.
pub fn records(procedures: &[Procedure]) -> String {
    let mut text = String::new();
    for p in procedures {
        let lines: Vec<String> = p.lines.iter().map(u32::to_string).collect();
        let _ = writeln!(
            text,
            "procedure\t{}\t{}\t{}\t{}\t{}",
            p.file.display(),
            p.name,
            p.line,
            p.end,
            lines.join(",")
        );
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(source: &str, os: TargetOs) -> Vec<Procedure> {
        let dir = std::env::temp_dir().join(format!(
            "jaic-procedures-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        std::fs::write(dir.join("a.jai"), source).unwrap();
        std::fs::write(dir.join("tests/skipped.jai"), "t :: () { x := 1; }\n").unwrap();
        let found = library_procedures(std::slice::from_ref(&dir), os, TargetCpu::X64);
        let _ = std::fs::remove_dir_all(&dir);
        found
    }

    #[test]
    fn procedures_have_their_name_span_and_running_lines() {
        let found = read(
            "add :: (a: int, b: int) -> int {\n    c := a + b;\n    // note\n    return c;\n}\n\nforeign_one :: (x: int) #foreign lib;\n",
            TargetOs::Linux,
        );
        assert_eq!(found.len(), 1, "{found:?}");
        let add = &found[0];
        assert_eq!((add.name.as_str(), add.line, add.end), ("add", 1, 5));
        assert_eq!(add.lines, [1, 2, 4]);
        assert!(add.file.ends_with("a.jai"));
    }

    #[test]
    fn nested_procedures_are_separate_and_not_part_of_the_outer_lines() {
        let found = read(
            "outer :: () {\n    inner :: () {\n        y := 2;\n    }\n    x := 1;\n}\nS :: struct {\n    method :: (s: S) { z := 3; }\n}\n",
            TargetOs::Linux,
        );
        let names: Vec<_> = found.iter().map(|p| p.name.as_str()).collect();
        assert_eq!(names, ["outer", "inner", "method"]);
        assert_eq!(found[0].lines, [1, 5]);
        assert_eq!(found[1].lines, [2, 3]);
    }

    #[test]
    fn an_if_on_os_keeps_the_branch_of_the_target() {
        let source = "#if OS == .WINDOWS {\n    win :: () { a := 1; }\n} else {\n    other :: () { a := 1; }\n}\n#if OS == .WINDOWS || OS == .LINUX {\n    both :: () { a := 1; }\n}\n#if SOMETHING {\n    maybe :: () { a := 1; }\n}\n";
        let linux: Vec<_> = read(source, TargetOs::Linux)
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert_eq!(linux, ["other", "both", "maybe"]);
        let windows: Vec<_> = read(source, TargetOs::Windows)
            .into_iter()
            .map(|p| p.name)
            .collect();
        assert_eq!(windows, ["win", "both", "maybe"]);
    }

    #[test]
    fn records_are_tab_separated() {
        let p = Procedure {
            file: PathBuf::from("/m/a.jai"),
            name: "f".into(),
            line: 3,
            end: 9,
            lines: vec![3, 4, 8],
        };
        assert_eq!(records(&[p]), "procedure\t/m/a.jai\tf\t3\t9\t3,4,8\n");
    }
}
