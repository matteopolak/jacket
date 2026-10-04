//! Publish the original accepted application without replaying its modifier.
use super::application_proofs::{
    ApplicationAnnotationOwner, ApplicationEnvironment, ApplicationProofKey, ApplicationProofSite,
    ReadyApplicationProof,
};
use super::*;

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    pub(super) fn application_proof_key(
        &self,
        file: FileInstanceId,
        source: &syntax::TypeApplicationSyntax,
        substitution: Option<&Substitution>,
    ) -> TypeResult<ApplicationProofKey> {
        let environment = if self.lexical_active {
            self.lexical
                .and_then(|lexical| lexical.environment.clone())
                .ok_or_else(|| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            source.span,
                            "type application has no retained lexical source environment",
                        ),
                    )
                })?
        } else {
            ApplicationEnvironment::Graph
        };
        let owner = match self.nominal_context {
            NominalAnnotationContext::None => ApplicationAnnotationOwner::None,
            NominalAnnotationContext::Record(owner) => ApplicationAnnotationOwner::Record(owner),
            NominalAnnotationContext::Forbidden => ApplicationAnnotationOwner::Forbidden,
        };
        Ok(ApplicationProofKey::new(
            ApplicationProofSite {
                file,
                source,
                environment,
                owner,
                enclosing_record: self.records.active_owner(),
                target: self.nominals.annotation_target(),
                substitution,
            },
            self.graph.symbols(),
        ))
    }

    pub(super) fn instantiate_application_at(
        &mut self,
        id: DeclarationId,
        substitution: Substitution,
        location: jai_source::SourceSpan,
        site: ApplicationProofKey,
    ) -> TypeResult<TypeId> {
        // The core returns its original normalized inputs alongside the accepted
        // canonical type. The accepted key cannot reconstruct pre-modifier inputs.
        let (ty, initial) = self.in_lexical_scope(false, |resolver| {
            resolver.instantiate_inner(id, substitution, location)
        })?;
        let proof =
            ReadyApplicationProof::checked(ty, initial, self.types, self.records, location.span)
                .map_err(|error| {
                    TypeFailure::Diagnostic(LocatedDiagnostic {
                        location,
                        message: error.message,
                    })
                })?;
        if let Some(proof) = proof {
            self.records
                .application_proofs
                .remember(site, proof, location.span)
                .map_err(|error| {
                    TypeFailure::Diagnostic(LocatedDiagnostic {
                        location,
                        message: error.message,
                    })
                })?;
        }
        Ok(ty)
    }
}
