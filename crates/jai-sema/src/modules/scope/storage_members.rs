//! Graph identities name source fields; only semantic lowering creates places.
use super::*;

impl FileScope<'_> {
    pub(crate) fn using_storage_source(
        &self,
        path: &NamePath,
    ) -> Option<(DeclarationId, Vec<Symbol>)> {
        for count in (0..=path.members.len()).rev() {
            let prefix = NamePath {
                root: path.root,
                members: path.members[..count].to_vec(),
            };
            let Ok(binding) = self.declarations.graph.lookup(self.file, &prefix) else {
                continue;
            };
            let (owner, mut fields) = match binding {
                GraphBinding::Declaration(owner)
                    if matches!(
                        self.declarations.graph.declaration(owner)?.syntax().kind,
                        FileDeclarationKind::Global(_)
                    ) =>
                {
                    (owner, Vec::new())
                }
                GraphBinding::StorageMember(id) => {
                    let member = self.declarations.graph.source_storage_member(id)?;
                    (member.owner(), member.path().to_vec())
                }
                _ => continue,
            };
            fields.extend_from_slice(&path.members[count..]);
            return Some((owner, fields));
        }
        None
    }

    pub(crate) fn imported_storage_member_source(
        &self,
        id: jai_modules::SourceStorageMemberId,
        span: Span,
    ) -> Result<(Binding, Vec<Symbol>), Diagnostic> {
        let member = self
            .declarations
            .graph
            .source_storage_member(id)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "imported storage member belongs to another source graph",
                )
            })?;
        let declaration = self
            .declarations
            .graph
            .declaration(member.owner())
            .ok_or_else(|| {
                Diagnostic::new(span, "imported storage member has an unavailable owner")
            })?;
        if !matches!(declaration.syntax().kind, FileDeclarationKind::Global(_)) {
            return Err(Diagnostic::new(
                span,
                "imported storage member owner is not global storage",
            ));
        }
        if member.path().is_empty()
            || member.path().len() > jai_modules::MAX_SOURCE_STORAGE_PATH_DEPTH
        {
            return Err(Diagnostic::new(
                span,
                "imported storage member has an invalid source path",
            ));
        }
        let value = self
            .declarations
            .values
            .get(&member.owner())
            .cloned()
            .ok_or_else(|| {
                Diagnostic::new(span, "imported global storage metadata is not ready")
            })?;
        if !matches!(value, Binding::Storage(_)) {
            return Err(Diagnostic::new(
                span,
                "imported storage member owner has no physical storage",
            ));
        }
        Ok((value, member.path().to_vec()))
    }
}
