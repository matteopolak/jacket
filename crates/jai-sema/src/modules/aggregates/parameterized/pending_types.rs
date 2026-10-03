//! Preserve distinct type-preparation dependencies before diagnostic boundaries.
//!
//! Retained source preparation consumes these typed identities directly.
use super::PendingRecordModifier;
use crate::modules::placeholder_demands::PlaceholderDemand;
use jai_modules::{LookupError, ModuleGraph};
use jai_source::{DeclarationId, LocatedDiagnostic, SourceSpan};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PendingType {
    RecordModifier(PendingRecordModifier),
    Placeholder(PlaceholderDemand),
    ProcedureDefault {
        declaration: DeclarationId,
        parameter: usize,
        location: SourceSpan,
    },
    Constant {
        declaration: DeclarationId,
        location: SourceSpan,
    },
}

impl From<PendingRecordModifier> for PendingType {
    fn from(pending: PendingRecordModifier) -> Self {
        Self::RecordModifier(pending)
    }
}

impl PendingType {
    pub(crate) fn from_lookup(
        error: LookupError,
        location: SourceSpan,
    ) -> Result<Self, LookupError> {
        PlaceholderDemand::from_lookup(error, location).map(Self::Placeholder)
    }

    pub(crate) fn location(self) -> SourceSpan {
        match self {
            Self::RecordModifier(pending) => pending.location,
            Self::Placeholder(demand) => demand.location,
            Self::Constant {
                location, ..
            }
            | Self::ProcedureDefault {
                location, ..
            } => location,
        }
    }

    /// Only one-shot/diagnostic clients call this. Retained preparation uses
    /// the real cause and its identity directly, including the original demand.
    pub(crate) fn diagnostic(self, graph: &ModuleGraph) -> LocatedDiagnostic {
        match self {
            Self::RecordModifier(pending) => pending.diagnostic(),
            Self::ProcedureDefault {
                location, ..
            } => LocatedDiagnostic {
                location,
                message: "selected source parameter default is pending checked preparation".into(),
            },
            Self::Constant {
                location, ..
            } => LocatedDiagnostic {
                location,
                message: "constant requires checked compile-time execution".into(),
            },
            Self::Placeholder(demand) => {
                LocatedDiagnostic::new(demand.location.source, demand.diagnostic(graph))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_modules::{GraphOptions, SourceOverlay};

    #[test]
    fn placeholder_dependency_keeps_the_actual_demand_and_reservation_note() {
        let path = std::path::Path::new("/jai-pending-type/main.jai");
        let mut overlay = SourceOverlay::new();
        overlay
            .insert(path, b"#placeholder Later; Alias::#type *Later;".to_vec())
            .unwrap();
        let graph =
            ModuleGraph::load_with_provider(path, GraphOptions::default(), &overlay).unwrap();
        let marker = &graph.placeholders()[0];
        let alias = &graph.declarations()[0];
        let lookup = graph
            .lookup(
                alias.file(),
                &crate::syntax::NamePath {
                    root: marker.name(),
                    members: vec![],
                },
            )
            .unwrap_err();
        let pending = PendingType::from_lookup(lookup, alias.location()).unwrap();
        let PendingType::Placeholder(demand) = pending else {
            panic!("actual source placeholder dependency")
        };
        assert_eq!(demand.placeholder, marker.id());
        assert_eq!(pending.location(), alias.location());
        assert_ne!(pending.location(), marker.location());
        let diagnostic = pending.diagnostic(&graph);
        assert_eq!(diagnostic.location, alias.location());
        assert!(diagnostic.message.contains("#placeholder 'Later'"));
        assert!(diagnostic.message.contains("placeholder reserved here"));
        assert_eq!(
            PendingType::from_lookup(LookupError::UnknownName(marker.name()), alias.location())
                .unwrap_err(),
            LookupError::UnknownName(marker.name()),
        );
    }
}
