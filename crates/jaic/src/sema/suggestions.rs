//! "Did you mean" help for unknown names. Working out the closest visible name costs a walk
//! over every scope in reach, so it happens when an error is rendered for the user, not each
//! time a lookup fails (failed lookups are routine while checking overloads and `#if`s).
use super::scope::{ScopeId, UsingEntry};
use super::{Compiler, Sym};
use crate::ast::{Expr, ExprKind, StmtKind};
use crate::lexer::{P, Tok};
use crate::source::{Diagnostic, DiagnosticKind, FileId, Span};
use std::path::{Path, PathBuf};

/// Modules searched first for an unknown name, most used first.
const COMMON_MODULES: &[&str] = &[
    "Basic",
    "String",
    "Math",
    "File",
    "File_Utilities",
    "Hash_Table",
    "Sort",
    "Random",
    "Process",
    "Thread",
    "System",
    "Compiler",
    "Bucket_Array",
    "Hash",
    "Unicode",
    "Reflection",
    "Program_Print",
    "Window_Creation",
    "Input",
    "Simp",
    "GetRect",
    "Sound_Player",
    "Calendar",
];

/// Modules the compiler loads itself; they are never imported by name.
const COMPILER_INTERNAL_MODULES: &[&str] = &["Preload", "Runtime_Support"];

/// Whether `text` declares `name` at the start of a line, outside a `#scope_file` section.
fn exports_name(text: &str, name: &str) -> bool {
    let mut file_scope = false;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("#scope_file") {
            file_scope = true;
        } else if trimmed.starts_with("#scope_export") || trimmed.starts_with("#scope_module") {
            file_scope = trimmed.starts_with("#scope_module");
        } else if !file_scope && line.len() - trimmed.len() <= 4 && declares_name(trimmed, name) {
            return true;
        }
    }
    false
}

/// Whether `code` starts with a declaration of `name` (`name ::`, `name :=` or `name: T`).
fn declares_name(code: &str, name: &str) -> bool {
    code.trim_start()
        .strip_prefix(name)
        .is_some_and(|rest| rest.trim_start().starts_with(':'))
}

/// Whether `code`, parsed as top-level Jai, declares `name`.
fn code_declares(code: &[u8], name: &str) -> bool {
    let Ok(code) = std::str::from_utf8(code) else {
        return false;
    };
    let Ok(file) = crate::parser::parse_file(FileId(u32::MAX), code) else {
        return false;
    };
    file.stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Decl(decl) => decl.names.iter().any(|n| n.name.as_str() == name),
        _ => false,
    })
}

/// Whether the metaprogram `text` calls `add_build_string` with a string literal whose code
/// declares `name`.
fn adds_declaration(text: &str, name: &str) -> bool {
    let Ok(tokens) = crate::lexer::lex(FileId(u32::MAX), text) else {
        return false;
    };
    tokens.windows(3).any(|w| {
        matches!(&w[0].tok, Tok::Ident(f) if f.as_str() == "add_build_string")
            && w[1].tok == Tok::Punct(P::LParen)
            && matches!(&w[2].tok, Tok::Str(code) if code_declares(code, name))
    })
}

/// Whether `cond` compares the compile target: it names `OS` or `CPU`.
fn mentions_target(cond: &Expr) -> bool {
    match &cond.kind {
        ExprKind::Ident(name) => matches!(name.as_str(), "OS" | "CPU"),
        ExprKind::Binary(_, a, b) => mentions_target(a) || mentions_target(b),
        ExprKind::Unary(_, a) => mentions_target(a),
        _ => false,
    }
}

impl Compiler {
    /// `d` with a `help: a similar name exists` line when it reports an unknown identifier
    /// and a visible name is close to it.
    pub(crate) fn with_name_suggestion(&self, d: &Diagnostic) -> Option<Diagnostic> {
        let DiagnosticKind::UnknownIdentifier { scope: Some(scope) } = d.kind else {
            return None;
        };
        let span = d.span;
        let wanted = self.sources.snippet_or_empty(span);
        if wanted.is_empty() {
            return None;
        }
        let names = self.names_visible_from(ScopeId(scope));
        let mut d = d.clone().with_label("not found in this scope");
        // An exact declaration elsewhere is a better lead than a similar name in scope.
        if let Some(builder) = self.metaprogram_defining(wanted, span) {
            return Some(d.with_help(format!(
                "`{wanted}` is added by `{builder}` (with `add_build_string`) when it builds this file: build through it, as in `jaic build {builder}`"
            )));
        }
        if let Some(found) = crate::suggest::closest(wanted, names.iter().copied()) {
            d = d.with_fix(format!("a similar name exists: `{found}`"), span, found);
        }
        if let Some(module) = self.module_declaring(wanted) {
            d = d.with_help(format!(
                "`{wanted}` is declared in the `{module}` module: add `#import \"{module}\";` to this file"
            ));
        }
        Some(d)
    }

    /// The metaprogram next to the file of `span` (in its directory or the one above) that
    /// adds a declaration of `name` with `add_build_string`, as a path shown to the user.
    fn metaprogram_defining(&self, name: &str, span: Span) -> Option<String> {
        if span.file.0 as usize >= self.sources.len() {
            return None;
        }
        let file = PathBuf::from(&self.sources.get(span.file).path);
        let dir = file.parent()?;
        for candidate_dir in [Some(dir), dir.parent()].into_iter().flatten() {
            let mut entries = self.fs.list_dir(candidate_dir);
            entries.sort();
            for (entry, is_dir) in entries {
                let path = candidate_dir.join(&entry);
                if is_dir || !entry.ends_with(".jai") || path == file {
                    continue;
                }
                let Some(bytes) = self.fs.read(&path) else {
                    continue;
                };
                if adds_declaration(&String::from_utf8_lossy(&bytes), name) {
                    return Some(crate::display_path(&path));
                }
            }
        }
        None
    }

    /// The standard-library module that declares `name` at its top level (outside
    /// `#scope_file`), common modules first.
    fn module_declaring(&self, name: &str) -> Option<String> {
        if name.len() < 2 || name.starts_with("__") {
            return None;
        }
        let stdlib = self.options.preload.as_deref()?.parent()?;
        let mut modules: Vec<(String, bool)> = self
            .fs
            .list_dir(stdlib)
            .into_iter()
            .filter_map(|(entry, is_dir)| {
                if is_dir {
                    self.fs
                        .is_file(&stdlib.join(&entry).join("module.jai"))
                        .then_some((entry, true))
                } else {
                    entry.strip_suffix(".jai").map(|m| (m.to_string(), false))
                }
            })
            .filter(|(m, _)| !COMPILER_INTERNAL_MODULES.contains(&m.as_str()))
            .collect();
        let rank = |m: &str| {
            COMMON_MODULES
                .iter()
                .position(|c| *c == m)
                .unwrap_or(usize::MAX)
        };
        modules.sort_by(|a, b| (rank(&a.0), &a.0).cmp(&(rank(&b.0), &b.0)));
        for (module, is_dir) in modules {
            let mut files = Vec::new();
            if is_dir {
                self.module_files(&stdlib.join(&module), &mut files, 0);
            } else {
                files.push(stdlib.join(format!("{module}.jai")));
            }
            for path in files {
                let Some(bytes) = self.fs.read(&path) else {
                    continue;
                };
                if exports_name(&String::from_utf8_lossy(&bytes), name) {
                    return Some(module);
                }
            }
        }
        None
    }

    /// The `.jai` files of a module directory, leaving out tests and examples.
    fn module_files(&self, dir: &Path, files: &mut Vec<PathBuf>, depth: usize) {
        if depth > 3 {
            return;
        }
        for (entry, is_dir) in self.fs.list_dir(dir) {
            let path = dir.join(&entry);
            if is_dir {
                if !matches!(entry.as_str(), "tests" | "examples" | "modules") {
                    self.module_files(&path, files, depth + 1);
                }
            } else if entry.ends_with(".jai") {
                files.push(path);
            }
        }
    }

    /// The names a lookup from `scope` can reach without loading anything new: each enclosing
    /// scope's own names, the modules it imports or `using`s, and Preload / Runtime_Support.
    fn names_visible_from(&self, scope: ScopeId) -> Vec<&'static str> {
        let mut names = Vec::new();
        let mut module_scopes = Vec::new();
        let mut current = Some(scope);
        while let Some(sid) = current {
            let s = self.scope(sid);
            names.extend(s.names.keys().map(|n| n.as_str()));
            for entry in &s.usings {
                if let UsingEntry::Module(m, _) = entry {
                    module_scopes.push(self.modules[m.0 as usize].scope);
                }
            }
            for import in &s.imports {
                if let Some(m) = import.module {
                    module_scopes.push(self.modules[m.0 as usize].scope);
                }
            }
            current = s.parent;
        }
        for m in [self.preload, self.runtime_support].into_iter().flatten() {
            module_scopes.push(self.modules[m.0 as usize].scope);
        }
        for sid in module_scopes {
            names.extend(self.scope(sid).names.keys().map(|n| n.as_str()));
        }
        names.retain(|n| !n.starts_with("__"));
        names.sort_unstable();
        names.dedup();
        names
    }

    /// A failed `#assert`: its message, or the condition when it has none.
    pub(super) fn static_assert_failed(
        &self,
        cond: &crate::ast::Expr,
        span: Span,
        message: String,
    ) -> Diagnostic {
        let text = self.sources.snippet_or_empty(cond.span).trim();
        let shown = (!text.is_empty() && !text.contains('\n') && text.len() <= 80).then_some(text);
        let literal_false = matches!(cond.kind, ExprKind::Bool(false));
        let mut d = match (message.is_empty(), shown) {
            (false, _) => Diagnostic::error(span, format!("#assert failed: {message}")),
            (true, _) if literal_false => {
                Diagnostic::error(span, "#assert failed: this `#assert(false)` was compiled")
            }
            (true, Some(text)) => {
                Diagnostic::error(span, format!("#assert failed: `{text}` is false"))
            }
            (true, None) => Diagnostic::error(span, "#assert failed: its condition is false"),
        }
        .with_kind(DiagnosticKind::StaticAssert);
        if !message.is_empty()
            && let Some(text) = shown
        {
            d = d.with_label(format!("`{text}` is false"));
        }
        // A condition on the target: say which one this compile is for.
        if mentions_target(cond) {
            let os = match self.options.os {
                super::TargetOs::Windows => ".WINDOWS",
                super::TargetOs::Linux => ".LINUX",
                super::TargetOs::MacOS => ".MACOS",
                super::TargetOs::Wasm => ".WASM",
            };
            let cpu = match self.options.cpu {
                super::TargetCpu::X64 => ".X64",
                super::TargetCpu::Arm64 => ".ARM64",
                super::TargetCpu::Wasm => ".WASM",
            };
            d = d.with_note(
                Span::NONE,
                format!("this compile targets `OS == {os}` and `CPU == {cpu}`; the code is written for other targets"),
            );
        }
        if literal_false {
            d = d.with_note(
                Span::NONE,
                "`#assert(false)` marks code that does not support this configuration (OS, CPU or build options); the code around it says which",
            );
        }
        d
    }

    /// `type `T` has no member `x``, with the closest member name or the members there are.
    pub(super) fn no_member(&self, ty: crate::types::TypeId, name: Sym, span: Span) -> Diagnostic {
        use crate::types::TypeKind;
        let shown = self.types.name(ty);
        let (what, members): (&str, Vec<String>) = match self.types.kind(ty) {
            TypeKind::Struct(id) => (
                "type",
                self.types
                    .struct_info(*id)
                    .fields
                    .iter()
                    .filter_map(|f| f.name.map(|n| n.to_string()))
                    .collect(),
            ),
            TypeKind::Enum(id) => (
                "enum",
                self.types
                    .enum_info(*id)
                    .members
                    .iter()
                    .map(|(n, _)| n.to_string())
                    .collect(),
            ),
            TypeKind::Pointer(inner) => {
                if matches!(self.types.kind(*inner), TypeKind::Pointer(_)) {
                    return Diagnostic::error(
                        span,
                        format!("type `{shown}` has no member `{name}`"),
                    )
                    .with_help(
                        "this is a pointer to a pointer: dereference it once with `.*` first",
                    );
                }
                ("type", Vec::new())
            }
            _ => ("type", Vec::new()),
        };
        let mut d = Diagnostic::error(span, format!("{what} `{shown}` has no member `{name}`"));
        if let Some(near) =
            crate::suggest::closest(name.as_str(), members.iter().map(String::as_str))
        {
            let near = near.to_string();
            d = d.with_fix(
                format!("a member with a similar name exists: `{near}`"),
                span,
                near,
            );
        } else if !members.is_empty() && members.len() <= 12 {
            let list: Vec<String> = members.iter().map(|m| format!("`{m}`")).collect();
            d = d.with_note(Span::NONE, format!("its members are {}", list.join(", ")));
        } else if members.is_empty()
            && matches!(
                self.types.kind(ty),
                TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::Bool
            )
        {
            d = d.with_note(
                Span::NONE,
                format!("`{shown}` is a plain value without members"),
            );
        }
        d
    }
}
