//! Help for names that a module provides but the code reaches the wrong way: `sqrt` written
//! bare while `M :: #import "Math"` only offers `M.sqrt`, or `Basic.printt` for a name `Basic`
//! spells differently or does not export. Like the other name help it is worked out when the
//! error is rendered (see `suggestions.rs`).
use super::scope::{EntityKind, ScopeId};
use super::{Compiler, ModuleId, Sym};
use crate::ast::ImportSource;
use crate::source::{Applicability, Diagnostic, ImportSuggestion, Span, Suggestion};

/// A namespaced import (`M :: #import "Math";`) in reach of an unknown name that exports it.
pub(super) struct Namespace {
    /// The name the import binds (`M`).
    binding: String,
    /// The imported module's name (`Math`).
    pub(super) module: String,
    /// The `#import` declaration.
    import: Span,
}

impl Compiler {
    /// Whether `module`, or a module it re-exports, declares `name` for its importers.
    fn module_exports(&self, module: ModuleId, name: Sym, seen: &mut Vec<ModuleId>) -> bool {
        if seen.contains(&module) {
            return false;
        }
        seen.push(module);
        let data = &self.ns.modules[module.0 as usize];
        let scope = self.scope(data.scope);
        let declared = scope.names.get(&name).is_some_and(|ids| {
            ids.iter().any(|&id| {
                let e = self.entity(id);
                e.exported && !e.file_private
            })
        });
        declared
            || data
                .exported_using_imports
                .iter()
                .filter_map(|&i| scope.imports.get(i)?.module)
                .any(|inner| self.module_exports(inner, name, seen))
    }

    /// The names `module` and the modules it re-exports declare for importers.
    fn module_export_names(
        &self,
        module: ModuleId,
        seen: &mut Vec<ModuleId>,
        out: &mut Vec<&'static str>,
    ) {
        if seen.contains(&module) {
            return;
        }
        seen.push(module);
        let data = &self.ns.modules[module.0 as usize];
        let scope = self.scope(data.scope);
        for (name, ids) in &scope.names {
            if ids.iter().any(|&id| {
                let e = self.entity(id);
                e.exported && !e.file_private
            }) {
                out.push(name.as_str());
            }
        }
        for inner in data
            .exported_using_imports
            .iter()
            .filter_map(|&i| scope.imports.get(i)?.module)
        {
            self.module_export_names(inner, seen, out);
        }
    }

    /// The namespaced imports visible from `scope` whose module declares `name`: found in the
    /// loaded module, or, for a module not settled yet, because `declaring` (the modules whose
    /// source declares the name) lists it.
    pub(super) fn namespaces_exporting(
        &self,
        scope: ScopeId,
        name: &str,
        declaring: &[ImportSuggestion],
    ) -> Vec<Namespace> {
        let sym = Sym::intern(name);
        let mut found: Vec<Namespace> = Vec::new();
        let mut current = Some(scope);
        while let Some(sid) = current {
            let s = self.scope(sid);
            for &id in s.names.values().flatten() {
                let EntityKind::Import(import) = &self.entity(id).kind else {
                    continue;
                };
                let (Some(binding), ImportSource::Module(module_name)) =
                    (&import.name, &import.source)
                else {
                    continue;
                };
                let loaded = self
                    .ns
                    .modules
                    .iter()
                    .position(|m| m.name == **module_name)
                    .is_some_and(|i| self.module_exports(ModuleId(i as u32), sym, &mut Vec::new()));
                let exports = loaded
                    || declaring
                        .iter()
                        .any(|i| i.name.is_none() && i.module == **module_name);
                if exports && !found.iter().any(|n| n.binding == binding.name.as_str()) {
                    found.push(Namespace {
                        binding: binding.name.to_string(),
                        module: module_name.to_string(),
                        import: import.span,
                    });
                }
            }
            current = s.parent;
        }
        found
    }

    /// `d` (an unknown `wanted` at `span`) with a fix that writes the name through each of the
    /// namespaces, and one that brings the whole namespace in with `using`.
    pub(super) fn with_namespace_suggestions(
        &self,
        mut d: Diagnostic,
        wanted: &str,
        span: Span,
        namespaces: &[Namespace],
    ) -> Diagnostic {
        let certain = if namespaces.len() == 1 {
            Applicability::MachineApplicable
        } else {
            Applicability::MaybeIncorrect
        };
        for n in namespaces {
            let binding = &n.binding;
            d = d.with_suggestion(
                Suggestion::insert(
                    format!(
                        "`{wanted}` is in the namespaced import `{binding}`; write `{binding}.{wanted}`"
                    ),
                    span,
                    format!("{binding}."),
                    certain,
                )
                .with_action(format!("Write `{binding}.{wanted}`")),
            );
            if let Some((at, text)) = self.using_after(n) {
                d = d.with_suggestion(
                    Suggestion::insert(
                        format!(
                            "or bring every name of `{}` into scope with `using {binding};`",
                            n.module
                        ),
                        at,
                        text,
                        Applicability::MaybeIncorrect,
                    )
                    .with_action(format!("Add `using {binding};`")),
                );
            }
        }
        d
    }

    /// Where `using M;` for the namespaced import `n` goes (on a line of its own after the
    /// import) and the text to insert.
    fn using_after(&self, n: &Namespace) -> Option<(Span, String)> {
        let text = &self.src.sources.try_get(n.import.file)?.text;
        let end = n.import.end as usize;
        let rest = text.get(end..)?;
        let trimmed = rest.trim_start();
        if !trimmed.starts_with(';') {
            return None;
        }
        let at = end + (rest.len() - trimmed.len()) + 1;
        let line_start = text
            .get(..n.import.start as usize)?
            .rfind('\n')
            .map_or(0, |i| i + 1);
        let indent: String = text[line_start..]
            .chars()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .collect();
        let point = Span {
            file: n.import.file,
            start: at as u32,
            end: at as u32,
        };
        Some((point, format!("\n{indent}using {};", n.binding)))
    }

    /// "module `M` has no exported member `name`": the closest exported name, or that the
    /// module declares `name` without exporting it.
    pub(in crate::sema) fn no_module_member(
        &self,
        module: ModuleId,
        name: Sym,
        span: Span,
    ) -> Diagnostic {
        let module_name = &self.ns.modules[module.0 as usize].name;
        let mut d = Diagnostic::error(
            span,
            format!("module `{module_name}` has no exported member `{name}`"),
        )
        .with_kind(crate::source::DiagnosticKind::UnknownModuleMember {
            module: module.0,
        });
        // `#scope_file` names live in the scope of their file, the others in the module's.
        let in_file = self
            .src
            .files
            .iter()
            .filter(|f| f.module == module)
            .any(|f| self.scope(f.scope).names.contains_key(&name));
        let hidden = in_file
            || self
                .scope(self.ns.modules[module.0 as usize].scope)
                .names
                .get(&name)
                .is_some_and(|ids| {
                    ids.iter().any(|&id| {
                        let e = self.entity(id);
                        !e.exported || e.file_private
                    })
                });
        if hidden {
            return d.with_help(format!(
                "`{name}` is declared in `{module_name}` but is not exported: it is private to the module"
            ));
        }
        let mut names = Vec::new();
        self.module_export_names(module, &mut Vec::new(), &mut names);
        names.retain(|n| !n.starts_with("__"));
        if let Some(near) = crate::suggest::closest(name.as_str(), names.iter().copied()) {
            d = d.with_replacement(
                format!("a member with a similar name exists: `{near}`"),
                span,
                near,
            );
        }
        d
    }
}
