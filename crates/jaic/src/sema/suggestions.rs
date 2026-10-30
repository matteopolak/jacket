//! "Did you mean" help for unknown names. Working out the closest visible name costs a walk
//! over every scope in reach, so it happens when an error is rendered for the user, not each
//! time a lookup fails (failed lookups are routine while checking overloads and `#if`s).
use super::module_rank::ModuleRanking;
use super::name_caches::DeclaringKey;
use super::scope::{ScopeId, UsingEntry};
use super::stdlib_index::{StdlibIndex, StdlibView};
use super::{Compiler, ModuleId, Sym, TypeId};
use crate::ast::{Expr, ExprKind, StmtKind};
use crate::diagnostics::Diagnostic;
use crate::diagnostics::{Code, Data};
use crate::fxhash::HashMap;
use crate::lexer::{P, Tok};
use crate::sema::SourceState;
use crate::sema::TypeState;
use crate::source::{Applicability, FileId, ImportSuggestion, Span, Suggestion};
use std::path::Path;
use std::rc::Rc;

/// The most modules an unknown name's help offers to import.
const MAX_IMPORTS: usize = 4;

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

/// Whether `cond` compares the compile target: it names `OS` or `CPU`.
fn mentions_target(cond: &Expr) -> bool {
    mentions(cond, &["OS", "CPU"])
}

/// Whether `cond` is about the operating system: it names `OS`, or is the literal `false` (what
/// the unsupported branch of a target check asserts).
fn mentions_os(cond: &Expr) -> bool {
    matches!(cond.kind, ExprKind::Bool(false)) || mentions(cond, &["OS"])
}

fn mentions(cond: &Expr, names: &[&str]) -> bool {
    match &cond.kind {
        ExprKind::Ident(name) => names.contains(&name.as_str()),
        ExprKind::Binary(_, a, b) => mentions(a, names) || mentions(b, names),
        ExprKind::Unary(_, a) => mentions(a, names),
        _ => false,
    }
}

impl Compiler {
    /// `d` with a `help: a similar name exists` line when it reports an unknown identifier
    /// and a visible name is close to it, the namespaced import that provides it, and the
    /// modules that declare it; the same modules for an unknown member of a module.
    pub fn with_name_suggestion(&self, d: &Diagnostic) -> Option<Diagnostic> {
        match (d.code, d.data) {
            (Code::UnknownIdentifier, Data::Scope(Some(scope))) => {
                self.suggest_unknown_identifier(d, ScopeId(scope))
            }
            (Code::UnknownModuleMember, Data::Module(module)) => {
                self.suggest_module_member(d, ModuleId(module))
            }
            _ => None,
        }
    }

    fn suggest_unknown_identifier(&self, d: &Diagnostic, scope: ScopeId) -> Option<Diagnostic> {
        let span = d.span;
        let wanted = self.src.sources.snippet(span);
        if wanted.is_empty() {
            return None;
        }
        let mut d = d.clone().with_label("not found in this scope");
        // An exact declaration elsewhere is a better lead than a similar name in scope.
        if let Some(builder) = self.src.metaprogram_defining(wanted, span) {
            return Some(d.with_help(format!(
                "`{wanted}` is added by `{builder}` (with `add_build_string`) when it builds this file: build through it, as in `jaic {builder}`"
            )));
        }
        let mut imports = self.imports_declaring(wanted);
        let namespaces = self.namespaces_exporting(scope, wanted, &imports);
        if namespaces.is_empty() {
            let names = self.names_visible_from(scope);
            if let Some(found) = crate::suggest::closest(wanted, names.iter().copied()) {
                d = d.with_replacement(format!("a similar name exists: `{found}`"), span, found);
            }
        } else {
            // The name is one `import` away already: other modules that declare it are noise.
            imports.clear();
            d = self.with_namespace_suggestions(d, wanted, span, &namespaces);
        }
        Some(self.with_import_suggestions(d, wanted, &imports, None))
    }

    /// An unknown `Module.name`: the modules that declare `name` (other than `Module`).
    fn suggest_module_member(&self, d: &Diagnostic, module: ModuleId) -> Option<Diagnostic> {
        let wanted = self.src.sources.snippet(d.span);
        if wanted.is_empty() {
            return None;
        }
        let module_name = self.ns.modules[module.0 as usize].name.as_str();
        let mut imports = self.imports_declaring(wanted);
        imports.retain(|i| i.module != module_name);
        Some(self.with_import_suggestions(d.clone(), wanted, &imports, Some(module_name)))
    }

    /// `d` with one suggestion per import that would declare `wanted`: the first is the help
    /// line, the others are named together in one line below it. `qualifier` is the module a
    /// member was looked up in, when it is a member the code wrote as `Qualifier.wanted`.
    fn with_import_suggestions(
        &self,
        mut d: Diagnostic,
        wanted: &str,
        imports: &[ImportSuggestion],
        qualifier: Option<&str>,
    ) -> Diagnostic {
        for (i, import) in imports.iter().enumerate() {
            let module = &import.module;
            let title = match (&import.name, qualifier) {
                (Some(name), _) => format!(
                    "`{name}` is the name of a module: import it under that name with `{}`",
                    import.statement()
                ),
                (None, Some(qualifier)) => format!(
                    "`{wanted}` is declared in the `{module}` module: add `{}` to this file and write `{wanted}` without the `{qualifier}` qualifier",
                    import.statement()
                ),
                (None, None) => format!(
                    "`{wanted}` is declared in the `{module}` module: add `{}` to this file",
                    import.statement()
                ),
            };
            let suggestion = Suggestion::import(title, import.clone())
                .with_action(format!("Add `{}`", import.statement()));
            // The only module that declares the name is the one to import (but a member written
            // with a qualifier needs the code changed too).
            let suggestion = if imports.len() == 1 && qualifier.is_none() {
                suggestion.with_applicability(Applicability::MachineApplicable)
            } else {
                suggestion
            };
            d = d.with_suggestion(if i == 0 {
                suggestion
            } else {
                suggestion.unlisted()
            });
        }
        if imports.len() > 1 {
            let others: Vec<String> = imports[1..]
                .iter()
                .map(|i| format!("`{}`", i.module))
                .collect();
            d = d.with_help(format!(
                "other modules that declare `{wanted}`: {}",
                others.join(", ")
            ));
        }
        d
    }

    /// The `#import`s that would declare `name`, best first: the standard-library module of
    /// that name (bound to it, for `Name.member`), then the modules that declare it at their top
    /// level (outside `#scope_file`), best ranked first (`ModuleRanking`) and jaic's `Extensions/` modules
    /// (named by that path) last. At most [`MAX_IMPORTS`].
    fn imports_declaring(&self, name: &str) -> Vec<ImportSuggestion> {
        let key = DeclaringKey::Name(name.to_string());
        if let Some(found) = self.caches.declaring_imports.borrow().get(&key) {
            return found.clone();
        }
        let found = self.imports_declaring_name(name);
        self.caches
            .declaring_imports
            .borrow_mut()
            .insert(key, found.clone());
        found
    }

    /// The modules whose source declares `operator <text>` for operands of types `tys` (as
    /// [`Self::imports_declaring`] ranks and limits them). A module qualifies when it declares
    /// the struct of an operand or an overload whose parameters name every struct operand; with
    /// no struct operand any module that declares the operator does.
    pub(super) fn imports_declaring_operator(
        &self,
        text: &str,
        tys: &[TypeId],
    ) -> Vec<ImportSuggestion> {
        let mut names = Vec::new();
        let mut homes = Vec::new();
        for &ty in tys {
            let Some(home) = self.ty.struct_home_scope(ty) else {
                continue;
            };
            let pointee = self.ty.types.pointee(ty).unwrap_or(ty);
            let full = self.ty.types.name(pointee);
            let base = full.rsplit('.').next().unwrap_or(&full);
            let base: String = base
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            names.push(base);
            // Prelude scopes belong to no module.
            homes.extend(
                self.ns
                    .modules
                    .get(self.scope(home).module.0 as usize)
                    .map(|m| m.name.clone()),
            );
        }
        let key = DeclaringKey::Operator(text.to_string(), names.clone(), homes.clone());
        if let Some(found) = self.caches.declaring_imports.borrow().get(&key) {
            return found.clone();
        }
        let found = match self.stdlib_view() {
            Some(view) => {
                let modules = view.modules_exporting_operator(text, &names, &homes);
                self.ranked_imports(&view, Vec::new(), modules)
            }
            None => Vec::new(),
        };
        self.caches
            .declaring_imports
            .borrow_mut()
            .insert(key, found.clone());
        found
    }

    fn imports_declaring_name(&self, name: &str) -> Vec<ImportSuggestion> {
        if name.len() < 2 || name.starts_with("__") {
            return Vec::new();
        }
        let Some(view) = self.stdlib_view() else {
            return Vec::new();
        };
        let named = if view.has_module(name) {
            vec![ImportSuggestion {
                module: name.to_string(),
                name: Some(name.to_string()),
            }]
        } else {
            Vec::new()
        };
        let modules = view.modules_exporting_name(name, &*self.src.fs);
        self.ranked_imports(&view, named, modules)
    }

    /// `found` followed by the modules `modules` (indexes into the standard library), best
    /// ranked first, up to [`MAX_IMPORTS`] in all.
    fn ranked_imports(
        &self,
        view: &StdlibView,
        mut found: Vec<ImportSuggestion>,
        mut modules: Vec<usize>,
    ) -> Vec<ImportSuggestion> {
        if modules.is_empty() {
            return found;
        }
        let ranking = ModuleRanking::new(HashMap::default(), (*view.importers).clone())
            .with_project(self.project_import_counts());
        modules.sort_by_cached_key(|&m| {
            let name = view.module_name(m);
            (ranking.rank(name), name.to_string(), m)
        });
        for m in modules {
            if found.len() >= MAX_IMPORTS {
                break;
            }
            found.push(ImportSuggestion {
                module: view.module_name(m).to_string(),
                name: None,
            });
        }
        found
    }

    /// The standard library's index as this compile sees it (without the module it preloads).
    fn stdlib_view(&self) -> Option<Rc<StdlibView>> {
        if let Some(view) = self.caches.stdlib_view.borrow().as_ref() {
            return view.clone();
        }
        let view = self.options.stdlib_root().map(|stdlib| {
            let index = StdlibIndex::shared(&*self.src.fs, stdlib);
            Rc::new(StdlibView::new(
                index,
                self.options.preload.as_deref(),
                &*self.src.fs,
            ))
        });
        *self.caches.stdlib_view.borrow_mut() = Some(view.clone());
        view
    }

    /// How often the program's own files import each module.
    fn project_import_counts(&self) -> HashMap<String, u32> {
        let mut counts: HashMap<String, u32> = HashMap::default();
        let Some(main) = self.ns.main_module else {
            return counts;
        };
        for file in self.src.files.iter().filter(|f| f.module == main) {
            for import in &self.scope(file.scope).imports {
                if let Some(module) = import.module {
                    *counts
                        .entry(self.ns.modules[module.0 as usize].name.clone())
                        .or_insert(0) += 1;
                }
            }
        }
        counts
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
                    module_scopes.push(self.ns.modules[m.0 as usize].scope);
                }
            }
            for import in &s.imports {
                if let Some(m) = import.module {
                    module_scopes.push(self.ns.modules[m.0 as usize].scope);
                }
            }
            current = s.parent;
        }
        for m in [self.ns.preload, self.ns.runtime_support]
            .into_iter()
            .flatten()
        {
            module_scopes.push(self.ns.modules[m.0 as usize].scope);
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
            (false, _) => Diagnostic::new(
                Code::StaticAssert,
                span,
                format!("#assert failed: {message}"),
            ),
            (true, _) if literal_false => Diagnostic::new(
                Code::StaticAssert,
                span,
                "#assert failed: this `#assert(false)` was compiled",
            ),
            (true, Some(text)) => Diagnostic::new(
                Code::StaticAssert,
                span,
                format!("#assert failed: `{text}` is false"),
            ),
            (true, None) => Diagnostic::new(
                Code::StaticAssert,
                span,
                "#assert failed: its condition is false",
            ),
        }
        .with_code(Code::StaticAssert);
        self.src
            .import_trace
            .failure(span, &d.message, mentions_os(cond));
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
}

impl TypeState {
    /// The names of the fields of struct `id` and of the structs its `using` fields hold.
    fn field_names(&self, id: crate::types::StructId, depth: usize, out: &mut Vec<String>) {
        for f in self.types.struct_info(id).fields.iter() {
            if let Some(n) = f.name {
                out.push(n.to_string());
            }
            if f.using
                && depth < 3
                && let Some(inner) = self.types.as_struct(self.types.repr_struct(f.ty))
            {
                self.field_names(inner, depth + 1, out);
            }
        }
    }

    /// "type `T` has no member `x`", with the closest member name or the members there are.
    /// `extra` names members that are not fields (constants and procedures declared in the
    /// struct); the members of `using` fields count too.
    fn no_member_with(
        &self,
        ty: crate::types::TypeId,
        name: Sym,
        span: Span,
        extra: Vec<String>,
    ) -> Diagnostic {
        use crate::types::TypeKind;
        let shown = self.types.name(ty);
        let (what, members): (&str, Vec<String>) = match self.types.kind(ty) {
            TypeKind::Struct(id) => {
                let mut members = Vec::new();
                self.field_names(*id, 0, &mut members);
                members.extend(extra);
                members.sort();
                members.dedup();
                ("type", members)
            }
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
                    return Diagnostic::new(
                        Code::TypeNoMember,
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
        let mut d = Diagnostic::new(
            Code::NoMember,
            span,
            format!("{what} `{shown}` has no member `{name}`"),
        );
        if let Some(near) =
            crate::suggest::closest(name.as_str(), members.iter().map(String::as_str))
        {
            let near = near.to_string();
            d = d.with_replacement(
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

impl SourceState {
    /// The metaprogram next to the file of `span` (in its directory or the one above) that
    /// adds a declaration of `name` with `add_build_string`, as a path shown to the user.
    fn metaprogram_defining(&self, name: &str, span: Span) -> Option<String> {
        let file = self.sources.try_get(span.file)?.path.as_file()?;
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

    /// The modules in `stdlib/folder` as (import name, is a folder): `Name` for the stdlib
    /// itself (`folder` empty), `folder/Name` below it.
    pub(crate) fn stdlib_modules_under(&self, stdlib: &Path, folder: &str) -> Vec<(String, bool)> {
        super::stdlib_index::modules_under(&*self.fs, stdlib, folder)
    }
}

impl Compiler {
    /// "type `T` has no member `x`", with the closest member name: a field, a member of a
    /// `using` field, or a constant or procedure declared in the struct.
    pub(super) fn no_member(&self, ty: crate::types::TypeId, name: Sym, span: Span) -> Diagnostic {
        let declared = self
            .ty
            .types
            .as_struct(ty)
            .and_then(|id| self.ty.struct_asts.get(&id))
            .map(|src| {
                self.scope(src.scope)
                    .names
                    .keys()
                    .map(ToString::to_string)
                    .filter(|n| !n.starts_with("__"))
                    .collect()
            })
            .unwrap_or_default();
        self.ty.no_member_with(ty, name, span, declared)
    }
}
