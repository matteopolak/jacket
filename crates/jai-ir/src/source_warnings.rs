//! Warnings are checked against the original source map, independently of debug emission.
use crate::IrError;
use jai_source::{MAX_SOURCE_WARNINGS, SourceMap, SourceWarning};

#[derive(Debug, Default)]
pub struct SourceWarnings(Vec<SourceWarning>);
impl SourceWarnings {
    pub fn checked(warnings: Vec<SourceWarning>, sources: &SourceMap) -> Result<Self, IrError> {
        if warnings.len() > MAX_SOURCE_WARNINGS {
            return Err(IrError::Arity {
                kind: "maximum source warnings",
                expected: MAX_SOURCE_WARNINGS,
                actual: warnings.len(),
            });
        }
        for warning in &warnings {
            for location in std::iter::once(warning.location())
                .chain(warning.notes().iter().map(|note| &note.location))
            {
                if !sources
                    .get(location.span().source)
                    .is_some_and(|source| location.matches_source(source))
                {
                    return Err(IrError::UnknownIdentity {
                        kind: "warning source span",
                        index: location.span().source.index(),
                    });
                }
            }
        }
        Ok(Self(warnings))
    }
    pub fn as_slice(&self) -> &[SourceWarning] {
        &self.0
    }
}
