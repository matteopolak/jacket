//! Warnings for code Jacket accepts and the official compiler rejects, and operators found
//! through the operand type's own module.
//!
//! Every leniency is reported through `leniency_warning`/`warn_leniency`: one diagnostic kind
//! per [`Leniency`], one shared note, and silence for code that is not the user's (an imported
//! module, the standard library). The rest of this file is the operator case: the help for
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
    /// A warning for a leniency at `span` in `module`, to add help to and hand to
    /// `warn_leniency`; `None` where nothing is reported: outside the program's own files (an
    /// imported module, the standard library, text with no file), because the author of a
    /// library cannot act on it from a program that uses the library.
    pub(in crate::sema) fn leniency_warning(
        &self,
        module: ModuleId,
        span: Span,
        leniency: Leniency,
        message: impl Into<String>,
    ) -> Option<Diagnostic> {
        if Some(module) != self.ns.main_module || self.is_library_file(span.file) {
            return None;
        }
        Some(
            Diagnostic::warning(span, message)
                .with_kind(DiagnosticKind::Lenient(leniency))
                .with_note(Span::NONE, NOTE),
        )
    }

    /// Record the warning `leniency_warning` made, if it made one.
    pub(in crate::sema) fn warn_leniency(&mut self, warning: Option<Diagnostic>) {
        if let Some(warning) = warning {
            self.warn(warning);
        }
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
    fn namespace_aliases(&mut self, scope: ScopeId) -> Vec<(Sym, ModuleId)> {
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

    fn module_name(&self, module: ModuleId) -> Option<String> {
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
