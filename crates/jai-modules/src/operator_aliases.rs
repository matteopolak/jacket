//! Operator alias edges resolve to exported original declarations.
use super::*;
use jai_syntax::OperatorKind;
use std::collections::HashSet;

impl ModuleGraph {
    pub(super) fn operator_alias_target(
        &self,
        declaration: &Declaration,
    ) -> Result<ModuleId, LocatedDiagnostic> {
        let FileDeclarationKind::OperatorAlias(alias) = &declaration.syntax().kind else {
            unreachable!("operator alias lookup requires an alias declaration");
        };
        let fail = |message: String| self.diagnostic(declaration.location(), message);
        match self.lookup(declaration.file(), &alias.target_namespace) {
            Ok(Binding::Module(module)) => Ok(module),
            Ok(_) | Err(LookupError::NotNamespace(_)) => Err(fail(
                "operator alias target must identify a source namespace".into(),
            )),
            Err(LookupError::PrivateMember { name, .. }) => Err(fail(format!(
                "operator alias namespace member '{}' is private",
                self.symbols.name(name)
            ))),
            Err(LookupError::UnknownName(name) | LookupError::UnknownMember { name, .. }) => {
                Err(fail(format!(
                    "unknown operator alias namespace '{}'",
                    self.symbols.name(name)
                )))
            }
            Err(LookupError::InvalidFile) => Err(fail(
                "operator alias defining source file is unavailable".into(),
            )),
            Err(LookupError::UnfilledPlaceholder(id)) => {
                let name = self
                    .placeholder(id)
                    .map(|marker| self.symbols.name(marker.name()));
                Err(fail(match name {
                    Some(name) => format!(
                        "operator alias namespace '{name}' is an unfilled source placeholder"
                    ),
                    None => "operator alias namespace placeholder metadata is unavailable".into(),
                }))
            }
        }
    }

    pub(super) fn validate_operator_aliases(&self) -> Result<(), LocatedDiagnostic> {
        let kinds = self
            .declarations
            .iter()
            .filter_map(super::operator_scopes::operator_kind)
            .collect::<HashSet<_>>();
        for declaration in &self.declarations {
            let FileDeclarationKind::OperatorAlias(alias) = &declaration.syntax().kind else {
                continue;
            };
            let module = self.operator_alias_target(declaration)?;
            let mut exported = false;
            for kind in kinds.iter().copied().chain([alias.target_kind]) {
                if !kind.shares_token(alias.target_kind) {
                    continue;
                }
                exported |= self.validate_exported_operator_paths(
                    module,
                    kind,
                    &mut HashSet::new(),
                    declaration.location(),
                )?;
            }
            if !exported {
                return Err(self.diagnostic(
                    declaration.location(),
                    "operator alias target has no exported declarations for this token",
                ));
            }
        }
        Ok(())
    }

    fn validate_exported_operator_paths(
        &self,
        module: ModuleId,
        kind: OperatorKind,
        active: &mut HashSet<(ModuleId, OperatorKind)>,
        origin: SourceSpan,
    ) -> Result<bool, LocatedDiagnostic> {
        if !active.insert((module, kind)) {
            return Err(self.diagnostic(origin, "cyclic operator alias namespace edge"));
        }
        let mut exported = false;
        for declaration in self.declarations.iter().filter(|declaration| {
            self.file(declaration.file())
                .is_some_and(|file| file.module() == module)
                && declaration.syntax().visibility == Visibility::Export
        }) {
            match &declaration.syntax().kind {
                FileDeclarationKind::Procedure(procedure) => {
                    exported |= procedure
                        .operator
                        .is_some_and(|operator| operator.kind == kind);
                }
                FileDeclarationKind::OperatorAlias(alias) if alias.kind.shares_token(kind) => {
                    exported |= self.validate_exported_operator_paths(
                        self.operator_alias_target(declaration)?,
                        kind,
                        active,
                        declaration.location(),
                    )?;
                }
                _ => {}
            }
        }
        exported |= self
            .published_operators(module, kind, None)
            .next()
            .is_some();
        for edge in self.imports.iter().filter(|edge| {
            self.file(edge.file())
                .is_some_and(|file| file.module() == module)
                && super::operator_scopes::import_declaration(self, **edge).is_some_and(|import| {
                    import.visibility == Visibility::Export
                        && (import.namespace.is_none() || import.using)
                })
        }) {
            exported |=
                self.validate_exported_operator_paths(edge.module(), kind, active, origin)?;
        }
        active.remove(&(module, kind));
        Ok(exported)
    }
}
