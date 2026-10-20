//! "Did you mean" help for unknown names. Working out the closest visible name costs a walk
//! over every scope in reach, so it happens when an error is rendered for the user, not each
//! time a lookup fails (failed lookups are routine while checking overloads and `#if`s).
use super::module_rank::{ModuleRanking, count_importers};
use super::scope::{ScopeId, UsingEntry};
use super::{Compiler, Sym};
use crate::ast::{Expr, ExprKind, ScopeKind, StmtKind};
use crate::fxhash::HashMap;
use crate::lexer::{P, Tok};
use crate::source::{Diagnostic, DiagnosticKind, FileId, ImportSuggestion, Span};
use std::cell::RefCell;
use std::path::{Path, PathBuf};

/// The most modules an unknown name's help offers to import.
const MAX_IMPORTS: usize = 4;

/// Modules the compiler loads itself; they are never imported by name.
pub const COMPILER_INTERNAL_MODULES: &[&str] = &["prelude", "Preload", "Runtime_Support"];

/// Whether the module file `text` declares `name` for its importers: at its top level (or in a
/// top-level `#if`), outside `#scope_file` and `#scope_module` sections. Only a file whose lines
/// look like such a declaration is parsed.
fn exports_name(text: &str, name: &str) -> bool {
    if !may_export_name(text, name) {
        return false;
    }
    let Ok(file) = crate::parser::parse_file(FileId(u32::MAX), text) else {
        return true;
    };
    fn declares(stmts: &[crate::ast::Stmt], name: &str, exported: &mut bool) -> bool {
        stmts.iter().any(|stmt| match &stmt.kind {
            StmtKind::Scope(kind) => {
                *exported = *kind == ScopeKind::Export;
                false
            }
            StmtKind::Decl(decl) => *exported && decl.names.iter().any(|n| n.name.as_str() == name),
            StmtKind::StaticIf(boxed) => {
                let crate::ast::StaticIfStmt {
                    then_branch,
                    else_branch,
                    ..
                } = &**boxed;
                let (mut then_exported, mut else_exported) = (*exported, *exported);
                declares(then_branch, name, &mut then_exported)
                    || declares(else_branch, name, &mut else_exported)
            }
            _ => false,
        })
    }
    declares(&file.stmts, name, &mut true)
}

/// Whether `text` has a line, indented at most four spaces and outside a `#scope_file` section,
/// that starts with a declaration of `name` (a quick test before [`exports_name`] parses).
fn may_export_name(text: &str, name: &str) -> bool {
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
/// declares `name`. The metaprogram is a neighbouring file that was never compiled, so there is
/// no entity to resolve the call to: this reads the tokens for the name `add_build_string`, the
/// documented way a metaprogram adds source.
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

/// The modules `#import "Name"` names in `text`, found from its tokens.
fn imported_modules(text: &str) -> Vec<String> {
    if !text.contains("#import") {
        return Vec::new();
    }
    let Ok(tokens) = crate::lexer::lex(FileId(u32::MAX), text) else {
        return Vec::new();
    };
    crate::project::scan_directives(&tokens)
        .into_iter()
        .filter_map(|link| match link.source {
            crate::project::LinkSource::Import(crate::ast::ImportSource::Module(name)) => {
                Some(name.to_string())
            }
            _ => None,
        })
        .collect()
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
    /// and a visible name is close to it, and the modules that declare it (`d.imports`).
    pub fn with_name_suggestion(&self, d: &Diagnostic) -> Option<Diagnostic> {
        let DiagnosticKind::UnknownIdentifier {
            scope: Some(scope),
        } = d.kind
        else {
            return None;
        };
        let span = d.span;
        let wanted = self.src.sources.snippet(span);
        if wanted.is_empty() {
            return None;
        }
        let names = self.names_visible_from(ScopeId(scope));
        let mut d = d.clone().with_label("not found in this scope");
        // An exact declaration elsewhere is a better lead than a similar name in scope.
        if let Some(builder) = self.metaprogram_defining(wanted, span) {
            return Some(d.with_help(format!(
                "`{wanted}` is added by `{builder}` (with `add_build_string`) when it builds this file: build through it, as in `jaic {builder}`"
            )));
        }
        if let Some(found) = crate::suggest::closest(wanted, names.iter().copied()) {
            d = d.with_fix(format!("a similar name exists: `{found}`"), span, found);
        }
        let imports = self.imports_declaring(wanted);
        if let Some((first, rest)) = imports.split_first() {
            let module = &first.module;
            d = d.with_help(match &first.name {
                Some(name) => format!(
                    "`{name}` is the name of a module: import it under that name with `{}`",
                    first.statement()
                ),
                None => format!(
                    "`{wanted}` is declared in the `{module}` module: add `{}` to this file",
                    first.statement()
                ),
            });
            if !rest.is_empty() {
                let others: Vec<String> = rest.iter().map(|i| format!("`{}`", i.module)).collect();
                d = d.with_help(format!(
                    "other modules that declare `{wanted}`: {}",
                    others.join(", ")
                ));
            }
            d.fixes.get_or_insert_default().imports = imports;
        }
        Some(d)
    }

    /// The metaprogram next to the file of `span` (in its directory or the one above) that
    /// adds a declaration of `name` with `add_build_string`, as a path shown to the user.
    fn metaprogram_defining(&self, name: &str, span: Span) -> Option<String> {
        if span.file.0 as usize >= self.src.sources.len() {
            return None;
        }
        let file = self.src.sources.get(span.file).path.as_file()?;
        let dir = file.parent()?;
        for candidate_dir in [Some(dir), dir.parent()].into_iter().flatten() {
            let mut entries = self.src.fs.list_dir(candidate_dir);
            entries.sort();
            for (entry, is_dir) in entries {
                let path = candidate_dir.join(&entry);
                if is_dir || !entry.ends_with(".jai") || path == file {
                    continue;
                }
                let Some(bytes) = self.src.fs.read(&path) else {
                    continue;
                };
                if adds_declaration(&String::from_utf8_lossy(&bytes), name) {
                    return Some(crate::display_path(&path));
                }
            }
        }
        None
    }

    /// The `#import`s that would declare `name`, best first: the standard-library module of
    /// that name (bound to it, for `Name.member`), then the modules that declare it at their top
    /// level (outside `#scope_file`), best ranked first (`ModuleRanking`) and jaic's `Extensions/` modules
    /// (named by that path) last. At most [`MAX_IMPORTS`].
    fn imports_declaring(&self, name: &str) -> Vec<ImportSuggestion> {
        let mut found = Vec::new();
        if name.len() < 2 || name.starts_with("__") {
            return found;
        }
        let Some(stdlib) = self.options.stdlib_root() else {
            return found;
        };
        let mut modules: Vec<(String, bool)> = self
            .stdlib_modules_under(stdlib, "")
            .into_iter()
            .chain(self.stdlib_modules_under(stdlib, crate::STDLIB_EXTENSIONS_DIR))
            .filter(|(m, _)| !COMPILER_INTERNAL_MODULES.contains(&m.as_str()))
            .collect();
        let ranking = ModuleRanking::clone(&self.stdlib_importers(stdlib, &modules))
            .with_project(self.project_import_counts());
        modules.sort_by_cached_key(|(m, _)| (ranking.rank(m), m.clone()));
        modules.dedup_by(|a, b| a.0 == b.0);
        if modules.iter().any(|(m, _)| m == name) {
            found.push(ImportSuggestion {
                module: name.to_string(),
                name: Some(name.to_string()),
            });
        }
        for (module, is_dir) in modules {
            if found.len() >= MAX_IMPORTS {
                break;
            }
            let mut files = Vec::new();
            if is_dir {
                self.module_files(&stdlib.join(&module), &mut files, 0);
            } else {
                files.push(stdlib.join(format!("{module}.jai")));
            }
            let declares = files.iter().any(|path| {
                self.src
                    .fs
                    .read(path)
                    .is_some_and(|bytes| exports_name(&String::from_utf8_lossy(&bytes), name))
            });
            if declares {
                found.push(ImportSuggestion {
                    module,
                    name: None,
                });
            }
        }
        found
    }

    /// How many other stdlib modules import each stdlib module, worked out once per thread and
    /// stdlib folder: help for several unknown names should not read the stdlib again.
    fn stdlib_importers(&self, stdlib: &Path, modules: &[(String, bool)]) -> ModuleRanking {
        thread_local! {
            static CACHE: RefCell<Option<(PathBuf, usize, ModuleRanking)>> = const { RefCell::new(None) };
        }
        if let Some(found) = CACHE.with(|c| {
            c.borrow()
                .as_ref()
                .filter(|(dir, count, _)| dir == stdlib && *count == modules.len())
                .map(|(_, _, ranking)| ranking.clone())
        }) {
            return found;
        }
        let imports: Vec<(&str, Vec<String>)> = modules
            .iter()
            .map(|(module, is_dir)| {
                let mut files = Vec::new();
                if *is_dir {
                    self.module_files(&stdlib.join(module), &mut files, 0);
                } else {
                    files.push(stdlib.join(format!("{module}.jai")));
                }
                let paths = files
                    .iter()
                    .filter_map(|path| self.src.fs.read(path))
                    .flat_map(|bytes| imported_modules(&String::from_utf8_lossy(&bytes)))
                    .collect();
                (module.as_str(), paths)
            })
            .collect();
        let importers = count_importers(
            imports
                .iter()
                .map(|(m, paths)| (*m, paths.iter().map(String::as_str))),
        );
        let ranking = ModuleRanking::new(HashMap::default(), importers);
        CACHE.with(|c| {
            *c.borrow_mut() = Some((stdlib.to_path_buf(), modules.len(), ranking.clone()));
        });
        ranking
    }

    /// How often the program's own files import each module.
    fn project_import_counts(&self) -> HashMap<String, u32> {
        let mut counts: HashMap<String, u32> = HashMap::default();
        let Some(main) = self.main_module else {
            return counts;
        };
        for file in self.src.files.iter().filter(|f| f.module == main) {
            for import in &self.scope(file.scope).imports {
                if let Some(module) = import.module {
                    *counts
                        .entry(self.modules[module.0 as usize].name.clone())
                        .or_insert(0) += 1;
                }
            }
        }
        counts
    }

    /// The modules in `stdlib/folder` as (import name, is a folder): `Name` for the stdlib
    /// itself (`folder` empty), `folder/Name` below it.
    pub(crate) fn stdlib_modules_under(&self, stdlib: &Path, folder: &str) -> Vec<(String, bool)> {
        let dir = stdlib.join(folder);
        let prefix = if folder.is_empty() {
            String::new()
        } else {
            format!("{folder}/")
        };
        self.src
            .fs
            .list_dir(&dir)
            .into_iter()
            .filter_map(|(entry, is_dir)| {
                if is_dir {
                    self.src
                        .fs
                        .is_file(&dir.join(&entry).join("module.jai"))
                        .then(|| (format!("{prefix}{entry}"), true))
                } else {
                    entry
                        .strip_suffix(".jai")
                        .map(|m| (format!("{prefix}{m}"), false))
                }
            })
            .collect()
    }

    /// The `.jai` files of a module directory, leaving out tests and examples.
    fn module_files(&self, dir: &Path, files: &mut Vec<PathBuf>, depth: usize) {
        if depth > 3 {
            return;
        }
        for (entry, is_dir) in self.src.fs.list_dir(dir) {
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
    /// scope's own names, the modules it imports or `using`s, and Preload / `Runtime_Support`.
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
        let text = self.src.sources.snippet(cond.span).trim();
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
            let os = crate::target::os_member(self.options.os);
            let cpu = crate::target::cpu_member(self.options.cpu);
            d = d.with_note(
                Span::NONE,
                format!("this compile targets `OS == .{os}` and `CPU == .{cpu}`; the code is written for other targets"),
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

    /// "type `T` has no member `x`", with the closest member name or the members there are.
    pub(super) fn no_member(&self, ty: crate::types::TypeId, name: Sym, span: Span) -> Diagnostic {
        use crate::types::TypeKind;
        let shown = self.ty.types.name(ty);
        let (what, members): (&str, Vec<String>) = match self.ty.types.kind(ty) {
            TypeKind::Struct(id) => (
                "type",
                self.ty
                    .types
                    .struct_info(*id)
                    .fields
                    .iter()
                    .filter_map(|f| f.name.map(|n| n.to_string()))
                    .collect(),
            ),
            TypeKind::Enum(id) => (
                "enum",
                self.ty
                    .types
                    .enum_info(*id)
                    .members
                    .iter()
                    .map(|(n, _)| n.to_string())
                    .collect(),
            ),
            TypeKind::Pointer(inner) => {
                if matches!(self.ty.types.kind(*inner), TypeKind::Pointer(_)) {
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
                self.ty.types.kind(ty),
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
