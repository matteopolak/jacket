//! "Did you mean" help for unknown names. Working out the closest visible name costs a walk
//! over every scope in reach, so it happens when an error is rendered for the user, not each
//! time a lookup fails (failed lookups are routine while checking overloads and `#if`s).
use super::Compiler;
use super::scope::{ScopeId, UsingEntry};
use crate::source::{Diagnostic, Span};

impl Compiler {
    /// Remember where the last unknown identifier was looked up, for `render`.
    pub(crate) fn note_unknown_name(&mut self, span: Span, scope: ScopeId) {
        self.last_unknown_name = Some((span, scope));
    }

    /// `d` with a `help: a similar name exists` line when it reports the last unknown
    /// identifier and a visible name is close to it.
    pub(crate) fn with_name_suggestion(&self, d: &Diagnostic) -> Option<Diagnostic> {
        let (span, scope) = self.last_unknown_name?;
        if d.span != span || !d.message.starts_with("unknown identifier") {
            return None;
        }
        let wanted = self.sources.snippet(span);
        let names = self.names_visible_from(scope);
        let d = d.clone().with_label("not found in this scope");
        Some(match crate::suggest::closest(wanted, names.iter().copied()) {
            Some(found) => d.with_fix(format!("a similar name exists: `{found}`"), span, found),
            None => d,
        })
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
}
