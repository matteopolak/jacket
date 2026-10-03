//! Pure admission shares the real append path and the retained session state.
use super::*;
use crate::declaration_insertions::*;
use std::sync::Arc;

impl GraphDiscovery<'_> {
    /// Prove that the exact captured source can be registered while its
    /// producer still owns its effect transaction. This neither stages a
    /// response nor reads dependencies from the source provider.
    pub fn admit_insertion(
        &self,
        request: InsertionRequestId,
        code: &DeclarationInsertionCode,
    ) -> Result<InsertionAdmission, InsertionPublicationError> {
        if self.has_failed() {
            return Err(InsertionPublicationError::Response(
                InsertionResponseError::FailedDiscovery,
            ));
        }
        let generation = self
            .builder
            .insertion_requests
            .admission_generation(request)
            .map_err(InsertionPublicationError::Response)?;
        let source = self
            .builder
            .insertion_requests
            .request(request)
            .map_err(InsertionPublicationError::Response)?;
        self.builder
            .graph
            .validate_insertion_code(source.file, code)
            .map_err(InsertionPublicationError::Response)?;

        // Clone the actual identity counters, stores, namespace maps, and
        // pending work. Rebuilding from visible IDs loses allocator/session
        // state; a separate collision validator would drift from publication.
        // append_insertion registers declarations and queues original items,
        // but never advances discovery or invokes the provider.
        let code = Arc::new(code.clone());
        let mut frontier = self.builder.clone();
        frontier
            .append_insertion(source, Arc::clone(&code))
            .map_err(InsertionPublicationError::Graph)?;
        Ok(InsertionAdmission {
            request,
            generation,
            revision: self.insertion_revision,
            code,
        })
    }

    /// Consume the sealed payload after the semantic graph borrow retires.
    /// Public copies of the producer's result cannot alter admitted syntax.
    pub fn prepare_insertion_admitted(
        &mut self,
        request: InsertionRequestId,
        admission: InsertionAdmission,
    ) -> Result<InsertionTransaction, InsertionResponseError> {
        if self.has_failed() {
            return Err(InsertionResponseError::FailedDiscovery);
        }
        if admission.revision != self.insertion_revision {
            return Err(InsertionResponseError::StaleAdmission);
        }
        self.builder
            .insertion_requests
            .stage_admitted(request, admission)
    }
}

#[cfg(test)]
mod tests;
