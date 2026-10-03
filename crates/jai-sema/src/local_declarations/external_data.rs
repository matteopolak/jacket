//! Source declaration identity and dense storage slots are separate append-only ledgers.
use super::*;
use jai_ir::{ExternalData, ExternalDataId, LocalExternalDataIndex};
use jai_source::SourceSpan;
use jai_types::TypeView;
mod bindings;

#[derive(Default)]
pub(crate) struct ExternalGlobals {
    expected_prefix: Option<usize>,
    base_prefix: Option<Vec<Global>>,
    checked_prefix: Option<jai_ir::GlobalDefinitionsPrefix>,
    identities: HashMap<LocalDeclarationId, ExternalDataId>,
    next_indices: HashMap<ProcedureId, usize>,
    slots: HashMap<LocalDeclarationId, usize>,
    globals: Vec<Global>,
}

impl ExternalGlobals {
    pub(crate) fn has_owner(&self, procedure: ProcedureId) -> bool {
        self.globals.iter().any(|global| matches!(
            global.initializer(),
            GlobalInitializer::External(data)
                if matches!(data.id(), ExternalDataId::Local { procedure: owner, .. } if owner == procedure)
        ))
    }
    pub(crate) fn checked_file_prefix(
        &mut self,
        types: &dyn TypeView,
        signatures: &HashMap<ProcedureId, TypeId>,
    ) -> Result<jai_ir::GlobalDefinitionsPrefix, Diagnostic> {
        if let Some(prefix) = &self.checked_prefix {
            return Ok(prefix.clone());
        }
        let prefix = self.ready_file_prefix().ok_or_else(|| {
            Diagnostic::new(
                Span::default(),
                "source owner waits for the complete file-global prefix",
            )
        })?;
        let expected = self.expected_prefix.ok_or_else(|| {
            Diagnostic::new(Span::default(), "file-global prefix has not been reserved")
        })?;
        let prefix = jai_ir::GlobalDefinitionsPrefix::new(prefix, expected, types, signatures)
            .map_err(|error| Diagnostic::new(Span::default(), error.to_string()))?;
        self.checked_prefix = Some(prefix.clone());
        Ok(prefix)
    }
    pub(crate) fn complete_file_prefix(&mut self, base: &[Global]) -> Result<(), Diagnostic> {
        self.check_base(base, Span::default())?;
        self.base_prefix.get_or_insert_with(|| base.to_vec());
        Ok(())
    }

    pub(crate) fn ready_file_prefix(&self) -> Option<&[Global]> {
        match &self.base_prefix {
            Some(prefix) => Some(prefix),
            None if self.expected_prefix == Some(0) => Some(&[]),
            None => None,
        }
    }

    pub(crate) fn reserve_file_prefix(&mut self, count: usize) -> Result<(), Diagnostic> {
        if self
            .expected_prefix
            .is_some_and(|previous| previous != count)
            || self
                .base_prefix
                .as_ref()
                .is_some_and(|prefix| prefix.len() != count)
        {
            return Err(Diagnostic::new(
                Span::default(),
                "external global file prefix changed within one semantic graph",
            ));
        }
        self.expected_prefix = Some(count);
        Ok(())
    }

    /// Reservation survives body retries; no source hash or spelling supplies identity.
    pub(crate) fn local_identity(
        &mut self,
        declaration: LocalDeclarationId,
    ) -> Result<ExternalDataId, Diagnostic> {
        let location = location(declaration)?;
        let LexicalScopeOwner::Procedure(procedure) = declaration.scope.owner else {
            return Err(Diagnostic::at_source(
                location,
                "external data requires a concrete procedure declaration owner",
            ));
        };
        if let Some(&identity) = self.identities.get(&declaration) {
            return Ok(identity);
        }
        let next = self.next_indices.entry(procedure).or_default();
        let index = LocalExternalDataIndex::new(*next);
        *next = next.checked_add(1).ok_or_else(|| {
            Diagnostic::at_source(location, "local external data identity space exhausted")
        })?;
        let identity = ExternalDataId::Local { procedure, index };
        self.identities.insert(declaration, identity);
        Ok(identity)
    }

    pub(crate) fn publish(
        &mut self,
        declaration: LocalDeclarationId,
        base: &[Global],
        data: ExternalData,
        types: &dyn TypeView,
    ) -> Result<Global, Diagnostic> {
        let location = location(declaration)?;
        if data.location() != location || data.id() != self.local_identity(declaration)? {
            return Err(Diagnostic::at_source(
                location,
                "external data differs from its registered source declaration",
            ));
        }
        data.validate(types)
            .map_err(|error| Diagnostic::at_source(location, error.to_string()))?;
        self.check_base(base, location.span)?;
        if let Some(&slot) = self.slots.get(&declaration) {
            let global = &self.globals[slot];
            if global.initializer() != &GlobalInitializer::External(data) {
                return Err(Diagnostic::at_source(
                    location,
                    "external declaration metadata changed after publication",
                ));
            }
            return Ok(global.clone());
        }
        let index = base.len().checked_add(self.globals.len()).ok_or_else(|| {
            Diagnostic::at_source(location, "external global identity space exhausted")
        })?;
        let global = Global::new_external(index, data);
        self.base_prefix.get_or_insert_with(|| base.to_vec());
        self.slots.insert(declaration, self.globals.len());
        self.globals.push(global.clone());
        Ok(global)
    }

    /// Providers and final publication use the same complete dense storage view.
    pub(crate) fn snapshot(&self, base: &[Global]) -> Result<Vec<Global>, Diagnostic> {
        if self.globals.is_empty() && self.base_prefix.is_none() {
            // Pure header prerequisites keep their actual partial prefix. No
            // appended identities can collide with that prefix yet.
            self.check_dense(base, Span::default())?;
            if self.expected_prefix.is_some_and(|count| base.len() > count) {
                return Err(Diagnostic::new(
                    Span::default(),
                    "file-global prefix exceeds its reserved extent",
                ));
            }
        } else {
            self.check_base(base, Span::default())?;
        }
        let mut globals = Vec::with_capacity(base.len().saturating_add(self.globals.len()));
        globals.extend_from_slice(base);
        globals.extend_from_slice(&self.globals);
        Ok(globals)
    }

    #[cfg(test)]
    fn file_prefix_pending(&self, base: &[Global]) -> bool {
        self.expected_prefix.is_some_and(|count| base.len() < count)
    }

    fn check_base(&self, base: &[Global], span: Span) -> Result<(), Diagnostic> {
        if self.expected_prefix.is_none() {
            return Err(Diagnostic::new(
                span,
                "file-global prefix has not been reserved",
            ));
        }
        if self
            .expected_prefix
            .is_some_and(|count| count != base.len())
        {
            return Err(Diagnostic::new(
                span,
                "external data waits for the complete file-global prefix",
            ));
        }
        if self
            .base_prefix
            .as_ref()
            .is_some_and(|prefix| prefix.as_slice() != base)
            || base
                .iter()
                .enumerate()
                .any(|(index, global)| global.id().index() != index)
        {
            return Err(Diagnostic::new(
                span,
                "external global registry requires its original dense file-global prefix",
            ));
        }
        Ok(())
    }

    fn check_dense(&self, base: &[Global], span: Span) -> Result<(), Diagnostic> {
        if base
            .iter()
            .enumerate()
            .any(|(index, global)| global.id().index() != index)
        {
            return Err(Diagnostic::new(
                span,
                "external global registry requires a dense file-global prefix",
            ));
        }
        Ok(())
    }
}

fn location(declaration: LocalDeclarationId) -> Result<SourceSpan, Diagnostic> {
    Ok(SourceSpan {
        source: declaration.defining_source().ok_or_else(|| {
            Diagnostic::new(
                declaration.source_span(),
                "external data requires a retained source declaration location",
            )
        })?,
        span: declaration.source_span(),
    })
}

#[cfg(test)]
mod tests;
