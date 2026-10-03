//! Specialized lexical wrappers project through the retained original header.
use super::*;

impl LocalGenericProcedures {
    pub(crate) fn baked_source_formals(&self, procedure: ProcedureId) -> Option<Vec<usize>> {
        let (key, signature) = self
            .signatures
            .iter()
            .find(|(_, signature)| signature.id == procedure)?;
        let source = &self.definitions.get(&key.declaration)?.source;
        signature
            .parameters
            .iter()
            .map(|parameter| {
                source
                    .parameters
                    .iter()
                    .position(|source| source.name == parameter.name)
            })
            .collect()
    }
}
