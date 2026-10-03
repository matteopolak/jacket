//! Operators keep declaration identity while using a separate typed namespace.
use super::*;
use jai_syntax::{FileItem, OperatorKind};
use std::collections::HashSet;

impl ModuleGraph {
    pub fn operator_declarations(
        &self,
        file: FileInstanceId,
        kind: OperatorKind,
    ) -> Vec<DeclarationId> {
        let Some(scope) = self.file(file) else {
            return vec![];
        };
        let captured = self
            .insertion_publication(file)
            .filter(|publication| publication.scope == jai_syntax::InsertScope::Captured);
        let mut declarations = captured
            .map(|publication| self.operator_declarations(publication.code.file, kind))
            .unwrap_or_default();
        declarations.extend(self.declarations.iter().filter_map(|declaration| {
            let defining = self.file(declaration.file())?;
            (defining.module() == scope.module()
                && ((captured.is_none() && declaration.syntax().visibility != Visibility::File)
                    || self.operator_file_visible(declaration.file(), file))
                && operator_kind(declaration) == Some(kind))
            .then_some(declaration.id())
        }));
        declarations.extend(self.published_operators(scope.module(), kind, Some(file)));
        let mut visited = HashSet::new();
        for declaration in self.declarations.iter().filter(|declaration| {
            self.file(declaration.file())
                .is_some_and(|file| file.module() == scope.module())
                && ((captured.is_none() && declaration.syntax().visibility != Visibility::File)
                    || self.operator_file_visible(declaration.file(), file))
                && matches!(&declaration.syntax().kind,
                    FileDeclarationKind::OperatorAlias(alias) if alias.kind.shares_token(kind))
        }) {
            if let Ok(module) = self.operator_alias_target(declaration) {
                self.collect_exported_operators(module, kind, &mut visited, &mut declarations);
            }
        }
        for edge in self.imports.iter().filter(|edge| {
            self.file(edge.file())
                .is_some_and(|origin| origin.module() == scope.module())
                && import_declaration(self, **edge).is_some_and(|import| {
                    (import.namespace.is_none() || import.using)
                        && (self.operator_file_visible(edge.file(), file)
                            || (captured.is_none() && import.visibility != Visibility::File))
                })
        }) {
            self.collect_exported_operators(edge.module(), kind, &mut visited, &mut declarations);
        }
        for module in [self.prelude, self.runtime_support].into_iter().flatten() {
            self.collect_exported_operators(module, kind, &mut visited, &mut declarations);
        }
        declarations.sort_by_key(|id| id.index());
        declarations.dedup();
        declarations
    }

    pub fn exported_operator_declarations(
        &self,
        module: ModuleId,
        kind: OperatorKind,
    ) -> Vec<DeclarationId> {
        let mut declarations = vec![];
        self.collect_exported_operators(module, kind, &mut HashSet::new(), &mut declarations);
        declarations.sort_by_key(|id| id.index());
        declarations.dedup();
        declarations
    }

    fn collect_exported_operators(
        &self,
        module: ModuleId,
        kind: OperatorKind,
        visited: &mut HashSet<ModuleId>,
        output: &mut Vec<DeclarationId>,
    ) {
        if !visited.insert(module) {
            return;
        }
        output.extend(self.declarations.iter().filter_map(|declaration| {
            (self
                .file(declaration.file())
                .is_some_and(|file| file.module() == module)
                && declaration.syntax().visibility == Visibility::Export
                && operator_kind(declaration) == Some(kind))
            .then_some(declaration.id())
        }));
        output.extend(self.published_operators(module, kind, None));
        for declaration in self.declarations.iter().filter(|declaration| {
            self.file(declaration.file())
                .is_some_and(|file| file.module() == module)
                && declaration.syntax().visibility == Visibility::Export
                && matches!(&declaration.syntax().kind,
                    FileDeclarationKind::OperatorAlias(alias) if alias.kind.shares_token(kind))
        }) {
            if let Ok(target) = self.operator_alias_target(declaration) {
                self.collect_exported_operators(target, kind, visited, output);
            }
        }
        for edge in self.imports.iter().filter(|edge| {
            self.file(edge.file())
                .is_some_and(|file| file.module() == module)
                && import_declaration(self, **edge).is_some_and(|import| {
                    import.visibility == Visibility::Export
                        && (import.namespace.is_none() || import.using)
                })
        }) {
            self.collect_exported_operators(edge.module(), kind, visited, output);
        }
    }

    pub(super) fn published_operators(
        &self,
        module: ModuleId,
        kind: OperatorKind,
        visible_from: Option<FileInstanceId>,
    ) -> impl Iterator<Item = DeclarationId> + '_ {
        self.using_publications
            .iter()
            .filter(move |publication| {
                publication.owner.is_none()
                    && publication.specialization.is_none()
                    && self
                        .file(publication.file)
                        .is_some_and(|file| file.module() == module)
                    && match visible_from {
                        Some(file) => {
                            (self.insertion_publication(file).is_none_or(|insertion| {
                                insertion.scope != jai_syntax::InsertScope::Captured
                            }) && publication.visibility != Visibility::File)
                                || self.operator_file_visible(publication.file, file)
                        }
                        None => publication.visibility == Visibility::Export,
                    }
            })
            .flat_map(|publication| publication.selected_operator_declarations.iter().copied())
            .filter(move |&id| {
                self.declaration(id)
                    .is_some_and(|declaration| operator_kind(declaration) == Some(kind))
            })
    }

    fn operator_file_visible(&self, mut defining: FileInstanceId, from: FileInstanceId) -> bool {
        // A captured expansion keeps its original lexical namespace. Only its
        // own generated descendants can add file-private operators to it.
        if self
            .insertion_publication(from)
            .is_some_and(|publication| publication.scope == jai_syntax::InsertScope::Captured)
        {
            loop {
                if defining == from {
                    return true;
                }
                let Some(publication) = self.insertion_publication(defining) else {
                    return false;
                };
                defining = publication.destination;
            }
        }
        self.canonical_binding_file(defining) == self.canonical_binding_file(from)
    }
}

pub(super) fn operator_kind(declaration: &Declaration) -> Option<OperatorKind> {
    let FileDeclarationKind::Procedure(procedure) = &declaration.syntax().kind else {
        return None;
    };
    procedure.operator.map(|operator| operator.kind)
}

pub(super) fn import_declaration(
    graph: &ModuleGraph,
    edge: ImportEdge,
) -> Option<&jai_syntax::ImportDeclaration> {
    fn find(items: &[FileItem], location: SourceSpan) -> Option<&jai_syntax::ImportDeclaration> {
        items.iter().find_map(|item| match item {
            FileItem::Import(import) if import.location == location => Some(import),
            FileItem::Conditional {
                then_items,
                else_items,
                ..
            } => find(then_items, location).or_else(|| find(else_items, location)),
            FileItem::Parameters(parameters) => find(&parameters.declarations, location),
            FileItem::CompileTimeCases { cases, .. } => cases
                .arms
                .iter()
                .find_map(|arm| find(&arm.body, location))
                .or_else(|| {
                    cases
                        .default
                        .as_ref()
                        .and_then(|default| find(&default.body, location))
                }),
            _ => None,
        })
    }
    find(graph.file(edge.file())?.syntax().items(), edge.location())
}
