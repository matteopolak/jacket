//! "Did you mean" help for unknown names. Working out the closest visible name costs a walk
//! over every scope in reach, so it happens when an error is rendered for the user, not each
//! time a lookup fails (failed lookups are routine while checking overloads and `#if`s).
use super::module_rank::ModuleRanking;
use super::name_caches::DeclaringKey;
use super::scope::{ScopeId, UsingEntry};
use super::stdlib_index::{StdlibIndex, StdlibView};
use super::{Compiler, FileSystem, ModuleId, Sym, TypeId};
use crate::ast::{Expr, ExprKind, StmtKind};
use crate::diagnostics::Diagnostic;
use crate::diagnostics::{Code, Data, OperatorImportHint};
use crate::fxhash::{HashMap, HashSet};
use crate::lexer::{P, Tok};
use crate::sema::SourceState;
use crate::sema::TypeState;
use crate::source::{Applicability, FileId, ImportSuggestion, Span, Suggestion};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

/// Independent render-time hint caches for one compiler, its options and unchanged facts.
/// The owner must validate filesystem responses AND invalidate after compiler/options/fact
/// mutation before reuse. A fresh context never borrows the process-wide stamp-based index.
/// For example, use one `HintContext::default()` for a batch of diagnostics, then discard it.
#[derive(Default)]
pub struct HintContext {
    declaring: RefCell<HashMap<DeclaringKey, Vec<ImportSuggestion>>>,
    view: RefCell<Option<Option<Rc<StdlibView>>>>,
}

struct HintInputs<'a> {
    fs: &'a dyn FileSystem,
    declaring: &'a RefCell<HashMap<DeclaringKey, Vec<ImportSuggestion>>>,
    view: &'a RefCell<Option<Option<Rc<StdlibView>>>>,
    fresh: bool,
}

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
    /// modules that declare it; the same modules for an unknown member of a module. Pending
    /// operator-import payloads are prepared here too, using the current import ranking.
    pub fn with_name_suggestion(&self, d: &Diagnostic) -> Option<Diagnostic> {
        self.name_suggestion(d, &self.default_hint_inputs())
    }

    /// Render help using caller-owned filesystem authority and independent caches.
    /// For example, a language server can pass an observed filesystem and retain the captured
    /// proof with the returned diagnostic, without adding hint reads to name-resolution inputs.
    pub fn with_name_suggestion_in(
        &self,
        d: &Diagnostic,
        context: &HintContext,
        fs: &dyn FileSystem,
    ) -> Option<Diagnostic> {
        self.name_suggestion(
            d,
            &HintInputs {
                fs,
                declaring: &context.declaring,
                view: &context.view,
                fresh: true,
            },
        )
    }

    fn default_hint_inputs(&self) -> HintInputs<'_> {
        HintInputs {
            fs: &*self.src.fs,
            declaring: &self.caches.declaring_imports,
            view: &self.caches.stdlib_view,
            fresh: self.options.hint_index_mode == super::HintIndexMode::PerCompiler,
        }
    }

    fn name_suggestion(&self, d: &Diagnostic, inputs: &HintInputs<'_>) -> Option<Diagnostic> {
        if let Some(hint) = d.operator_import_hint() {
            let imports = self.imports_declaring_operator(hint, inputs);
            let mut prepared = d.clone();
            prepared.take_operator_import_hint();
            for (i, import) in imports.into_iter().enumerate() {
                let title = format!(
                    "`operator {}` is declared in the `{}` module: add `{}` to this file",
                    hint.operator,
                    import.module,
                    import.statement()
                );
                let suggestion = Suggestion::import(title, import.clone())
                    .with_action(format!("Add `{}`", import.statement()));
                prepared = prepared.with_suggestion(if i == 0 {
                    suggestion
                } else {
                    suggestion.unlisted()
                });
            }
            return Some(prepared);
        }
        match (d.code, d.data) {
            (Code::UnknownIdentifier, Data::Scope(scope)) => {
                self.suggest_unknown_identifier(d, ScopeId(scope), inputs)
            }
            (Code::UnknownModuleMember, Data::Module(module)) => {
                self.suggest_module_member(d, ModuleId(module), inputs)
            }
            _ => None,
        }
    }

    fn suggest_unknown_identifier(
        &self,
        d: &Diagnostic,
        scope: ScopeId,
        inputs: &HintInputs<'_>,
    ) -> Option<Diagnostic> {
        let span = d.span;
        let wanted = self.src.sources.snippet(span);
        if wanted.is_empty() {
            return None;
        }
        let mut d = d.clone().with_label("not found in this scope");
        if let Some((declared, file)) = self.file_scoped_elsewhere(scope, wanted) {
            d.code = Code::ScopeFileName;
            return Some(
                d.with_note(
                    declared,
                    format!("`{wanted}` is declared here, under `#scope_file`"),
                )
                .with_help(format!(
                    "`#scope_file` names are only visible in {file}: declare `{wanted}` under `#scope_module` to share it with the module's other files"
                )),
            );
        }
        // An exact declaration elsewhere is a better lead than a similar name in scope.
        if let Some(builder) = self.src.metaprogram_defining(wanted, span, inputs.fs) {
            return Some(d.with_help(format!(
                "`{wanted}` is added by `{builder}` (with `add_build_string`) when it builds this file: build through it, as in `jaic {builder}`"
            )));
        }
        let mut imports = self.imports_declaring(wanted, inputs);
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

    /// Where another file of the module `scope` belongs to declares `wanted` under `#scope_file`,
    /// and the name of that file.
    fn file_scoped_elsewhere(&self, scope: ScopeId, wanted: &str) -> Option<(Span, String)> {
        let name = Sym::intern(wanted);
        let module = self.scope(scope).module;
        let here = self.scope_file(scope);
        self.src
            .files
            .iter()
            .filter(|f| f.module == module && f.id != here)
            .find_map(|f| {
                let ids = self.scope(f.scope).names.get(&name)?;
                let entity = ids
                    .iter()
                    .map(|&id| self.entity(id))
                    .find(|e| e.file_private)?;
                let file = f
                    .path
                    .as_file()
                    .and_then(|p| p.file_name())
                    .map_or_else(|| f.path.to_string(), |n| n.to_string_lossy().into_owned());
                Some((entity.span, format!("`{file}`")))
            })
    }

    /// An unknown `Module.name`: the modules that declare `name` (other than `Module`).
    fn suggest_module_member(
        &self,
        d: &Diagnostic,
        module: ModuleId,
        inputs: &HintInputs<'_>,
    ) -> Option<Diagnostic> {
        let wanted = self.src.sources.snippet(d.span);
        if wanted.is_empty() {
            return None;
        }
        let module_name = self.ns.modules[module.0 as usize].name.as_str();
        let mut imports = self.imports_declaring(wanted, inputs);
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
    fn imports_declaring(&self, name: &str, inputs: &HintInputs<'_>) -> Vec<ImportSuggestion> {
        let key = DeclaringKey::Name(name.to_string());
        if let Some(found) = inputs.declaring.borrow().get(&key) {
            return found.clone();
        }
        let found = self.imports_declaring_name(name, inputs);
        crate::sema::body_probe::write(crate::sema::body_probe::W::CachesDeclaringImports);
        inputs.declaring.borrow_mut().insert(key, found.clone());
        found
    }

    /// Capture operand names while their compiler identities are available, without searching
    /// external modules. For example, a discarded dead-branch error never needs import help.
    pub(super) fn operator_import_hint(&self, text: &str, tys: &[TypeId]) -> OperatorImportHint {
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
        OperatorImportHint {
            operator: text.to_string(),
            operands: names,
            homes,
        }
    }

    fn imports_declaring_operator(
        &self,
        hint: &OperatorImportHint,
        inputs: &HintInputs<'_>,
    ) -> Vec<ImportSuggestion> {
        let key = DeclaringKey::Operator(
            hint.operator.clone(),
            hint.operands.clone(),
            hint.homes.clone(),
        );
        if let Some(found) = inputs.declaring.borrow().get(&key) {
            return found.clone();
        }
        let found = match self.stdlib_view(inputs) {
            Some(view) => {
                let modules =
                    view.modules_exporting_operator(&hint.operator, &hint.operands, &hint.homes);
                self.ranked_imports(&view, Vec::new(), modules)
            }
            None => Vec::new(),
        };
        crate::sema::body_probe::write(crate::sema::body_probe::W::CachesDeclaringImports);
        inputs.declaring.borrow_mut().insert(key, found.clone());
        found
    }

    fn imports_declaring_name(&self, name: &str, inputs: &HintInputs<'_>) -> Vec<ImportSuggestion> {
        if name.len() < 2 || name.starts_with("__") {
            return Vec::new();
        }
        let Some(view) = self.stdlib_view(inputs) else {
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
        let modules = view.modules_exporting_name(name, inputs.fs);
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
    fn stdlib_view(&self, inputs: &HintInputs<'_>) -> Option<Rc<StdlibView>> {
        if let Some(view) = inputs.view.borrow().as_ref() {
            return view.clone();
        }
        let view = self.options.stdlib_root().map(|stdlib| {
            let index = if inputs.fresh {
                StdlibIndex::fresh(inputs.fs, stdlib)
            } else {
                StdlibIndex::shared(inputs.fs, stdlib)
            };
            Rc::new(StdlibView::new(
                index,
                self.options.preload.as_deref(),
                inputs.fs,
            ))
        });
        crate::sema::body_probe::write(crate::sema::body_probe::W::CachesStdlibView);
        *inputs.view.borrow_mut() = Some(view.clone());
        view
    }

    /// Resolved main-module/file-scope import sites, including named imports. A named
    /// `using` has two representations but counts once; ranking never resolves a new import.
    fn project_import_counts(&self) -> HashMap<String, u32> {
        use super::scope::{EntityKind, EntityState, Resolved};
        let mut counts: HashMap<String, u32> = HashMap::default();
        let Some(main) = self.ns.main_module else {
            return counts;
        };
        let files: HashSet<_> = self
            .src
            .files
            .iter()
            .filter(|file| file.module == main)
            .map(|file| file.scope)
            .collect();
        let mut scopes = files.clone();
        scopes.insert(self.ns.modules[main.0 as usize].scope);
        let mut seen = HashSet::default();
        let mut count = |span, module: ModuleId| {
            if seen.insert(span) {
                *counts
                    .entry(self.ns.modules[module.0 as usize].name.clone())
                    .or_insert(0) += 1;
            }
        };
        for scope in scopes {
            for import in &self.scope(scope).imports {
                if files.contains(&import.from_scope)
                    && let Some(module) = import.module
                {
                    count(import.import.span, module);
                }
            }
            for (id, _) in self.scope(scope).declared() {
                let entity = self.entity(id);
                if files.contains(&entity.home)
                    && let (EntityKind::Import(import), EntityState::Done(Resolved::Module(module))) =
                        (&entity.kind, &entity.state)
                {
                    count(import.span, *module);
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
        let text = self.src.sources.snippet(cond.extent()).trim();
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
    fn metaprogram_defining(&self, name: &str, span: Span, fs: &dyn FileSystem) -> Option<String> {
        let file = self.sources.try_get(span.file)?.path.as_file()?;
        let dir = file.parent()?;
        for candidate_dir in [Some(dir), dir.parent()].into_iter().flatten() {
            let mut entries = fs.list_dir(candidate_dir);
            entries.sort();
            for (entry, is_dir) in entries {
                let path = candidate_dir.join(&entry);
                if is_dir || !entry.ends_with(".jai") || path == file {
                    continue;
                }
                let Some(bytes) = fs.read(&path) else {
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

#[cfg(test)]
mod ranking_regressions {
    use super::*;
    use crate::sema::scope::{EntityKind, EntityState};
    use crate::sema::{Options, ProgramSource, VirtualFs};

    fn compiler(main: &str, extra: &str) -> Compiler {
        let mut fs = VirtualFs::default();
        fs.insert("/main.jai", main.as_bytes());
        fs.insert("/extra.jai", extra.as_bytes());
        fs.insert(
            "/modules/A/module.jai",
            b"#import \"Transitive\"; AValue :: 1;".as_slice(),
        );
        fs.insert("/modules/B/module.jai", b"BValue :: 2;".as_slice());
        fs.insert("/modules/C/module.jai", b"CValue :: 3;".as_slice());
        fs.insert(
            "/modules/Transitive/module.jai",
            b"TransitiveValue :: 4;".as_slice(),
        );
        let mut options = Options::host();
        options.runtime_support = false;
        options.import_paths = vec!["/modules".into()];
        Compiler::new(options, Rc::new(fs))
    }

    #[test]
    fn counts_resolved_main_module_import_sites_once_across_scopes() {
        let mut compiler = compiler(
            concat!(
                "#load \"extra.jai\"; #import \"A\"; using Named :: #import \"B\"; ",
                "Alias :: Named; #scope_file Private :: #import \"C\"; ",
                "main :: () { a := AValue; b := Named.BValue; c := Private.CValue; d := other(); }",
            ),
            "#scope_file #import \"A\"; Other :: #import \"B\"; #scope_export other :: () -> int { return Other.BValue; }",
        );
        compiler
            .compile_program(Path::new("/main.jai"))
            .expect("valid imports");
        let counts = compiler.project_import_counts();
        assert_eq!(counts.get("A"), Some(&2), "{counts:?}");
        assert_eq!(
            counts.get("B"),
            Some(&2),
            "named using counts once, alias constants never count: {counts:?}"
        );
        assert_eq!(
            counts.get("C"),
            Some(&1),
            "private named import: {counts:?}"
        );
        assert!(!counts.contains_key("Transitive"), "{counts:?}");
    }

    #[test]
    fn procedural_imports_are_not_project_import_sites() {
        let mut compiler = compiler(
            "#import \"A\"; main :: () { Local :: #import \"C\"; value := Local.CValue; }",
            "",
        );
        compiler
            .compile_program(Path::new("/main.jai"))
            .expect("local import");
        let counts = compiler.project_import_counts();
        assert_eq!(counts.get("A"), Some(&1));
        assert!(!counts.contains_key("C"), "{counts:?}");
    }

    #[test]
    fn ranking_does_not_resolve_unused_named_imports() {
        let mut compiler = compiler("Unused :: #import \"Missing\"; main :: () {}", "");
        compiler
            .prepare_sources(&[ProgramSource::File("/main.jai".into())])
            .expect("declarations only");
        let imports = || {
            compiler
                .ns
                .entities
                .iter()
                .filter(|entity| matches!(entity.kind, EntityKind::Import(_)))
                .map(|entity| matches!(entity.state, EntityState::Unresolved))
                .collect::<Vec<_>>()
        };
        assert_eq!(imports(), [true]);
        let files = compiler.src.files.len();
        let modules = compiler.ns.modules.len();
        assert!(compiler.project_import_counts().is_empty());
        assert_eq!(imports(), [true]);
        assert_eq!(compiler.src.files.len(), files);
        assert_eq!(compiler.ns.modules.len(), modules);
    }
}
