//! Warnings for code Jacket accepts and the official compiler rejects, and operators found
//! through the operand type's own module.
//!
//! Every leniency is reported through `leniency_warning`: one diagnostic kind
//! per [`Leniency`], one shared note, and silence for code that is not the user's (an imported
//! module, bundled or not). The rest of this file is the operator case: the help for
//! operators nothing reaches.
//!
//! A namespaced import (`Math :: #import "Math"`) does not bring `operator +` into scope, so
//! `u + v` on `Math.Vector2` does not compile where the portable spelling is
//! `Math.operator +(u, v)`. Like `for_expansion`, operator lookup also searches the module that
//! declares an operand's struct; using that overload is a warning that names the portable
//! spelling.

use super::operators::binop_text;
use super::{
    Compiler, Diagnostic, DiagnosticKind, Operand, ProcId, Result, ScopeId, Span, Sym, TypeId,
    TypeKind, scope,
};
use crate::leniency::{Leniency, NOTE};
use crate::sema::ModuleId;
use crate::sema::ast;
use crate::sema::binary::Binary;
use crate::source::{Applicability, Edit, ImportSuggestion, Suggestion};

/// An operator overload accepted through the module of an operand's type.
#[derive(Clone, Debug)]
pub(in crate::sema) struct LenientUse {
    /// The module that declares the operand's struct.
    pub module: ModuleId,
    pub ty: TypeId,
    /// The overloads found there.
    pub procs: Vec<ProcId>,
}

/// How a warning or error rewrites the expression: `{prefix}Alias.operator {text}({args})`
/// in place of `span`.
pub(in crate::sema) struct Rewrite {
    pub span: Span,
    pub prefix: String,
    pub args: String,
}

impl Compiler {
    /// A warning for a leniency at `span` in `module`, to add help to and `warn`; `None` where nothing is reported: outside the program's own files (an
    /// imported module, bundled or not, or text with no file), because the author of a
    /// library cannot act on it from a program that uses the library.
    pub(in crate::sema) fn leniency_warning(
        &self,
        module: ModuleId,
        span: Span,
        leniency: Leniency,
        message: impl Into<String>,
    ) -> Option<Diagnostic> {
        if Some(module) != self.ns.main_module || self.src.sources.try_get(span.file).is_none() {
            return None;
        }
        Some(
            Diagnostic::warning(span, message)
                .with_kind(DiagnosticKind::Lenient(leniency))
                .with_note(Span::NONE, NOTE),
        )
    }

    /// The overloads of `operator <text>` declared in the modules of the struct types `tys`
    /// that `scoped` does not already hold, with the module and type that supplied them.
    pub(in crate::sema) fn home_operator_candidates(
        &mut self,
        text: &str,
        tys: &[TypeId],
        scoped: &[ProcId],
    ) -> Result<(Vec<ProcId>, Option<LenientUse>)> {
        let mut extra: Vec<ProcId> = Vec::new();
        let mut source = None;
        let mut homes = Vec::new();
        for &ty in tys {
            let Some(home) = self.ty.struct_home_scope(ty) else {
                continue;
            };
            if homes.contains(&home) {
                continue;
            }
            homes.push(home);
            for p in self.operator_candidates_in(home, text, true)? {
                if !scoped.contains(&p) && !extra.contains(&p) {
                    extra.push(p);
                    source.get_or_insert((self.scope(home).module, ty));
                }
            }
        }
        let used = source.map(|(module, ty)| LenientUse {
            module,
            ty,
            procs: extra.clone(),
        });
        Ok((extra, used))
    }

    /// The overloads of `operator <text>` for a receiver of type `ty`: those in scope, or when
    /// none of them takes `ty`, those in its module as well.
    pub(in crate::sema) fn operator_candidates_for(
        &mut self,
        scope: ScopeId,
        text: &str,
        ty: TypeId,
    ) -> Result<(Vec<ProcId>, Option<LenientUse>)> {
        let mut procs = self.operator_candidates(scope, text)?;
        if procs.iter().any(|&p| self.first_param_accepts(p, ty)) {
            return Ok((procs, None));
        }
        let (extra, used) = self.home_operator_candidates(text, &[ty], &procs)?;
        let used = used.filter(|_| extra.iter().any(|&p| self.first_param_accepts(p, ty)));
        if used.is_some() {
            procs.extend(extra);
        }
        Ok((procs, used))
    }

    /// The names of the namespaced imports (`Name :: #import "M"`) in reach of `scope`, with the
    /// modules they bind, innermost first.
    pub(in crate::sema) fn namespace_aliases(&mut self, scope: ScopeId) -> Vec<(Sym, ModuleId)> {
        let mut found = Vec::new();
        let mut current = Some(scope);
        while let Some(sid) = current {
            let s = self.scope(sid);
            let mut ids: Vec<(Sym, scope::EntityId)> = s
                .names
                .iter()
                .flat_map(|(name, ids)| ids.iter().map(move |&id| (*name, id)))
                .collect();
            current = s.parent;
            ids.sort_by_key(|&(name, id)| (name.as_str(), id));
            for (name, id) in ids {
                if !matches!(self.entity(id).kind, scope::EntityKind::Import(_)) {
                    continue;
                }
                if let Ok(scope::Resolved::Module(m)) = self.resolve_entity(id)
                    && !found.iter().any(|&(n, _)| n == name)
                {
                    found.push((name, m));
                }
            }
        }
        found
    }

    pub(in crate::sema) fn module_name(&self, module: ModuleId) -> Option<String> {
        let name = &self.ns.modules[module.0 as usize].name;
        let mut chars = name.chars();
        let ident = chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
            && chars.all(|c| c.is_alphanumeric() || c == '_');
        ident.then(|| name.clone())
    }

    /// Warn that `operator <text>` was taken from the module of an operand type, naming the
    /// portable spelling. With a `rewrite`, an editor can apply it.
    pub(in crate::sema) fn warn_lenient_operator(
        &mut self,
        scope: ScopeId,
        span: Span,
        text: &str,
        used: LenientUse,
        rewrite: Option<Rewrite>,
    ) {
        let module = self.ns.modules[used.module.0 as usize].name.clone();
        let ty = self.ty.types.name(used.ty);
        // The alias that exports the overload (`Math` for the types of `Math`, though they are
        // declared in a module `Math` loads itself), else the declaring module by its name.
        let name = Sym::intern(&format!("operator{text}"));
        let mut alias = None;
        for (candidate, m) in self.namespace_aliases(scope) {
            let module_scope = self.ns.modules[m.0 as usize].scope;
            let exported = self.lookup(module_scope, name).unwrap_or_default();
            let found = exported
                .into_iter()
                .filter_map(|id| self.resolve_entity(id).ok())
                .any(|r| match r {
                    scope::Resolved::Proc(p) => used.procs.contains(&p),
                    scope::Resolved::ProcSet(set) => set.iter().any(|p| used.procs.contains(p)),
                    _ => false,
                });
            if found {
                alias = Some(candidate.as_str().to_string());
                break;
            }
        }
        let qualifier = alias.clone().or_else(|| self.module_name(used.module));
        let module_id = self.scope(scope).module;
        let Some(d) = self.leniency_warning(
            module_id,
            span,
            Leniency::OperatorOutOfScope,
            format!("`operator {text}` is not in scope here: using the one declared with `{ty}`"),
        ) else {
            return;
        };
        let mut d = d.with_label("found through the operand type's module");
        if let Some(q) = &qualifier {
            if let Some(Rewrite {
                span: at,
                prefix,
                args,
            }) = rewrite
            {
                let call = format!("{prefix}{q}.operator {text}({args})");
                let mut edits = vec![Edit::Replace {
                    span: at,
                    text: call.clone(),
                }];
                if alias.is_none() {
                    edits.push(Edit::Import(ImportSuggestion {
                        module: module.clone(),
                        name: Some(q.clone()),
                    }));
                }
                let applicability = if alias.is_some() && prefix.is_empty() {
                    Applicability::MachineApplicable
                } else {
                    Applicability::MaybeIncorrect
                };
                d = d.with_suggestion(
                    Suggestion::with_edits(
                        format!("name the module in the call so it works wherever it is imported: `{call}`"),
                        edits,
                        applicability,
                    )
                    .with_action(format!("Call `{q}.operator {text}` explicitly")),
                );
            } else if text != "*[]" {
                // `Alias.operator *[](..)` does not parse: only `using` reaches that one.
                d = d.with_help(format!("call it as `{q}.operator {text}(..)`"));
            }
            d = d.with_help(format!("or add `using {q};` to this scope"));
        }
        self.warn(d);
    }

    /// Warn when the program's entry point (`main`) returns a value.
    pub(in crate::sema) fn warn_main_returning_a_value(&mut self) {
        // Read the declaration as written: resolving the procedure here would check things in
        // an order that hides the errors of headers elsewhere in the program.
        let Some(module) = self.ns.main_module else {
            return;
        };
        let scope = self.ns.modules[module.0 as usize].scope;
        let entry = Sym::intern(&self.options.entry_point);
        let Ok(ids) = self.lookup(scope, entry) else {
            return;
        };
        let mut found = None;
        for id in ids {
            if let scope::EntityKind::Decl {
                decl, ..
            } = &self.entity(id).kind
                && let Some(ast::ExprKind::Proc(lit)) = decl.value.as_ref().map(|v| &v.kind)
            {
                found = Some((self.entity(id).scope, lit.header.clone()));
                break;
            }
        }
        let Some((scope, header)) = found else {
            return;
        };
        let Some(first) = header.returns.first() else {
            return;
        };
        let span = first.span;
        let name = self.options.entry_point.clone();
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::MainReturnsValue,
            format!("`{name}` returns a value"),
        ) else {
            return;
        };
        let d = d
            .with_label("the entry point returns nothing")
            .with_help(format!(
                "remove the `->` and the return type of `{name}`; `exit(code)` ends the program with a status"
            ));
        self.warn(d);
    }

    /// Warn about the constructs the parser accepted in `file` although the official compiler
    /// rejects them, when `module` is the program's own.
    pub(in crate::sema) fn report_parsed_leniencies(&mut self, module: ModuleId, file: &ast::File) {
        for found in &file.lenient {
            let text = self.src.sources.snippet(found.span).to_string();
            let (message, label, help, fix) = match found.kind {
                Leniency::PlaceDirective => (
                    "`#place` is a deprecated directive".to_string(),
                    "overlays the member after it on the one it names",
                    Some("write `#overlay(member)` before the member that shares the space".to_string()),
                    None,
                ),
                Leniency::ForeignLibrary => {
                    let portable = if text.contains("system") {
                        "#system_library"
                    } else {
                        "#library"
                    };
                    (
                        format!("`{text}` is not a directive here"),
                        "declares the library to link",
                        Some(format!("write `{portable}`")),
                        Some((portable.to_string(), Applicability::MachineApplicable)),
                    )
                }
                Leniency::UnnamedLibrary => (
                    format!("`{text}` needs a name here"),
                    "a library is declared with a name",
                    Some(format!(
                        "bind it to a name: `lib :: {text},link_always \"name\";`"
                    )),
                    None,
                ),
                Leniency::MustAfterReturnType => (
                    "`#must` after a return type is not a directive here".to_string(),
                    "requires every call to use this result",
                    Some(
                        "remove `#must` and use the result at every call: there is no portable spelling"
                            .to_string(),
                    ),
                    Some((String::new(), Applicability::MaybeIncorrect)),
                ),
                Leniency::IfxThenSemicolon => (
                    "`;` before `else` ends the `#ifx`".to_string(),
                    "the `else` has nothing to belong to",
                    Some("remove the `;`: `#ifx c then a else b;`".to_string()),
                    Some((String::new(), Applicability::MachineApplicable)),
                ),
                Leniency::AddressOfLiteral => {
                    let operand = text.strip_prefix('*').unwrap_or(&text);
                    (
                        format!("`{text}` reads as a pointer type followed by a literal"),
                        "this takes the address of the literal here",
                        Some(format!("write `*({operand})` to take the address of the literal")),
                        Some((format!("*({operand})"), Applicability::MachineApplicable)),
                    )
                }
                Leniency::FloatExponent => {
                    let portable = text.replacen(['e', 'E'], ".0e", 1);
                    (
                        format!("`{text}` has an exponent and no fraction"),
                        "a decimal float with an exponent needs a `.`",
                        Some(format!("write `{portable}`")),
                        Some((portable, Applicability::MachineApplicable)),
                    )
                }
                Leniency::StructAlign => (
                    "`#align` is not a directive of a struct as a whole".to_string(),
                    "aligns the struct",
                    Some(
                        "align the first member instead: `member: T #align N;`, which gives the struct the same alignment"
                            .to_string(),
                    ),
                    None,
                ),
                Leniency::ForOverArrayLiteral => (
                    "`for` over an array literal needs its element type".to_string(),
                    "this literal names no element type",
                    Some("write the type before the literal: `for x: T.[..]`".to_string()),
                    None,
                ),
                Leniency::TargetFeatures => (
                    "`#target_features` is not a directive of a procedure header".to_string(),
                    "names CPU features for this procedure",
                    Some("remove it: there is no portable spelling".to_string()),
                    Some((String::new(), Applicability::MaybeIncorrect)),
                ),
                Leniency::PushContextStatement => (
                    "`push_context` takes a block".to_string(),
                    "this statement runs in the pushed context",
                    Some("put the statement in braces: `push_context c { statement; }`".to_string()),
                    None,
                ),
                Leniency::MultiValueInCodeArgument => (
                    "a comma ends a `#code` block that is a call argument".to_string(),
                    "this list of values is split at the comma",
                    Some(
                        "give each value its own statement (`a := x; b := y;`), or bind the code to a name first: `c :: #code { .. }; f(c);`"
                            .to_string(),
                    ),
                    None,
                ),
                Leniency::ExistsOnExpression => (
                    "`#exists` takes a name or names joined by `.`".to_string(),
                    "this operand is an expression",
                    Some(
                        "ask about a name or a member chain (`#exists(S.member)`); there is no portable spelling for an expression"
                            .to_string(),
                    ),
                    None,
                ),
                _ => continue,
            };
            let Some(mut d) = self.leniency_warning(module, found.span, found.kind, message) else {
                continue;
            };
            d = d.with_label(label);
            if let Some(help) = help {
                d =
                    match fix {
                        Some((replacement, applicability)) => d.with_suggestion(
                            Suggestion::replace(help, found.span, replacement, applicability),
                        ),
                        None => d.with_help(help),
                    };
            }
            self.warn(d);
        }
    }

    /// Warn about `Enum.loose`, the type of an enum's values that is not one of its members.
    pub(in crate::sema) fn warn_lenient_loose_enum(
        &mut self,
        scope: ScopeId,
        span: Span,
        ty: TypeId,
    ) {
        let name = self.ty.types.name(ty);
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::LooseEnum,
            format!("`loose` is not a member of `{name}`"),
        ) else {
            return;
        };
        let repr = self.ty.types.name(self.ty.types.repr(ty));
        let d = d
            .with_label("an enum type that takes any value of its integer type")
            .with_help(format!(
                "write the integer type, `{repr}`, and cast to `{name}` where the enum is needed"
            ));
        self.warn(d);
    }

    /// Reject `id`, found for `name` at `span`, when it is a constant of a procedure body that
    /// the code declares further down: a body's constants, procedures and types are visible only
    /// below the line that declares them, in imported modules as in the program. A use inside a
    /// procedure nested in another local procedure is not checked (the official compiler reads
    /// such a body once the procedure around it is used). The use and the declaration may sit in
    /// text an `#insert` produced; each stands where its `#insert` is.
    pub(in crate::sema) fn note_use_before_declaration(
        &mut self,
        scope: ScopeId,
        id: scope::EntityId,
        name: Sym,
        span: Span,
    ) -> Result<()> {
        let home = self.entity(id).scope;
        let local = matches!(
            self.scope(home).kind,
            scope::ScopeKind::Proc | scope::ScopeKind::Block
        );
        let constant = matches!(
            &self.entity(id).kind,
            scope::EntityKind::Decl { decl, .. } if decl.kind == ast::DeclKind::Const
                // A `name :: #code ..` constant is spliced in where it is named, not read in order.
                && !decl.value.as_ref().is_some_and(|v| matches!(v.kind, ast::ExprKind::Code(_)))
        );
        if !local || !constant {
            return Ok(());
        }
        let declared = self.entity(id).span;
        let (used_at, declared_at) = (self.insert_origin(span), self.insert_origin(declared));
        if declared_at.file != used_at.file || used_at.start >= declared_at.start {
            return Ok(());
        }
        // Only a use the declaration's own scope encloses is read in order: a macro body or code
        // inserted into another scope reaches the name by another route.
        let mut procedures = 0;
        let mut current = Some(scope);
        loop {
            let Some(sid) = current else {
                return Ok(());
            };
            if sid == home {
                break;
            }
            if self.scope(sid).inserted_into.is_some() || !self.scope(sid).fallbacks.is_empty() {
                return Ok(());
            }
            match self.scope(sid).kind {
                scope::ScopeKind::Macro => return Ok(()),
                scope::ScopeKind::Proc if self.scope(sid).proc.is_some() => {
                    // Text outside the procedure's body (its header, written elsewhere than
                    // the code it is read in) has no reading order against the body.
                    if procedures == 0 && !self.in_body_of(sid, used_at) {
                        return Ok(());
                    }
                    procedures += 1;
                }
                _ => {}
            }
            current = self.scope(sid).parent;
        }
        if procedures >= 2 {
            return Ok(());
        }
        Err(Box::new(
            Diagnostic::error(span, format!("`{name}` is used above its declaration"))
                .with_kind(DiagnosticKind::UseBeforeDeclaration)
                .with_label("declared further down")
                .with_note(declared, "declared here")
                .with_help(format!(
                    "move the declaration of `{name}` above its first use"
                )),
        ))
    }

    /// Does the body of the procedure `sid` holds contain `at`?
    fn in_body_of(&self, sid: ScopeId, at: Span) -> bool {
        let Some(proc) = self.scope(sid).proc else {
            return false;
        };
        self.proc(proc).lit.body.as_ref().is_some_and(|body| {
            body.span.file == at.file && body.span.start <= at.start && at.start < body.span.end
        })
    }

    /// Where `span` stands in a real file: a span in text an `#insert` produced stands at the
    /// `#insert`.
    fn insert_origin(&self, mut span: Span) -> Span {
        while let Some(&site) = self.src.insert_sites.get(&span.file) {
            span = site;
        }
        span
    }

    /// Warn that `for` found the `for_expansion` of `ty` through the module of the type
    /// (`home`, the scope that declares it) because the import that names that module is a
    /// namespace (`HT :: #import "Hash_Table"`), which does not bring `for_expansion` into scope.
    pub(in crate::sema) fn warn_lenient_for_expansion(
        &mut self,
        scope: ScopeId,
        span: Span,
        ty: TypeId,
        home: ScopeId,
    ) {
        let home_module = self.scope(home).module;
        let full = self.ty.types.name(ty);
        let name = full
            .trim_start_matches('*')
            .split('(')
            .next()
            .unwrap_or(&full)
            .to_string();
        let Some(d) = self.leniency_warning(
            self.scope(scope).module,
            span,
            Leniency::NamespacedForExpansion,
            format!("`for_expansion` is not in scope here: using the one declared with `{name}`"),
        ) else {
            return;
        };
        let mut d = d.with_label("found through the type's module");
        let alias = self
            .namespace_aliases(scope)
            .into_iter()
            .find(|&(_, m)| m == home_module)
            .map(|(alias, _)| alias.as_str().to_string());
        if let Some(alias) = alias {
            d = d.with_help(format!("add `using {alias};` to this scope"));
        } else if let Some(module) = self.module_name(home_module) {
            d = d.with_help(format!(
                "import the module without a name (`#import \"{module}\";`) so its `for_expansion` is in scope"
            ));
        }
        self.warn(d);
    }

    /// A binary operator on a struct that no overload takes, with help to reach one.
    pub(in crate::sema) fn reject_missing_operator(
        &mut self,
        scope: ScopeId,
        node: Binary<'_>,
        lhs: &Operand,
        rhs: &Operand,
        span: Span,
    ) -> Result<()> {
        use crate::ast::BinOp;
        let is_struct = |s: &Self, t: TypeId| matches!(s.ty.types.kind(t), TypeKind::Struct(_));
        let has_struct = [lhs.ty(), rhs.ty()].into_iter().any(|t| is_struct(self, t));
        if !has_struct {
            return Ok(());
        }
        if matches!(node.op, BinOp::Eq | BinOp::Ne)
            || matches!(lhs, Operand::Type(_) | Operand::Procs(_))
            || matches!(rhs, Operand::Type(_) | Operand::Procs(_))
        {
            return Ok(());
        }
        let text = binop_text(node.op);
        let d = Diagnostic::error(
            span,
            format!(
                "no `operator {text}` for `{}` and `{}`",
                self.ty.types.name(lhs.ty()),
                self.ty.types.name(rhs.ty())
            ),
        );
        let rewrite = self.operator_rewrite(node, span, false, false);
        Err(Box::new(self.with_missing_operator_help(
            scope,
            d,
            text,
            &[lhs.ty(), rhs.ty()],
            Some(rewrite),
        )))
    }

    /// Help for `operator <text>` on `tys` that nothing reaches: a namespaced import that
    /// declares it (call it through the alias, or `using` the alias), else a module to import.
    pub(in crate::sema) fn with_missing_operator_help(
        &mut self,
        scope: ScopeId,
        mut d: Diagnostic,
        text: &str,
        tys: &[TypeId],
        rewrite: Option<Rewrite>,
    ) -> Diagnostic {
        let name = Sym::intern(&format!("operator{text}"));
        let mut aliases = Vec::new();
        for (alias, module) in self.namespace_aliases(scope) {
            let module_scope = self.ns.modules[module.0 as usize].scope;
            if self
                .lookup(module_scope, name)
                .is_ok_and(|ids| !ids.is_empty())
            {
                aliases.push(alias.as_str().to_string());
            }
        }
        if let Some(alias) = aliases.first() {
            if let Some(Rewrite {
                span,
                prefix,
                args,
            }) = rewrite
            {
                let call = format!("{prefix}{alias}.operator {text}({args})");
                d = d.with_suggestion(
                    Suggestion::replace(
                        format!("`{alias}` declares `operator {text}`: call `{call}`"),
                        span,
                        call,
                        Applicability::MaybeIncorrect,
                    )
                    .with_action(format!("Call `{alias}.operator {text}`")),
                );
            }
            d = d.with_help(format!(
                "`{alias}` declares `operator {text}`: call `{alias}.operator {text}(..)` or add `using {alias};`"
            ));
            return d;
        }
        let imports = self.imports_declaring_operator(text, tys);
        for (i, import) in imports.iter().enumerate() {
            let title = format!(
                "`operator {text}` is declared in the `{}` module: add `{}` to this file",
                import.module,
                import.statement()
            );
            let suggestion = Suggestion::import(title, import.clone())
                .with_action(format!("Add `{}`", import.statement()));
            d = d.with_suggestion(if i == 0 {
                suggestion
            } else {
                suggestion.unlisted()
            });
        }
        d
    }
}
