//! Emit only committed references to canonical source declarations.
mod sources;
use jai_source::{SourceWarning, SourceWarningKind, WarningLocation, WarningNote};
pub(crate) use sources::DeprecationKey;
pub(crate) use sources::procedure_extent;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Clone, Debug)]
pub(crate) struct DeprecatedDeclaration {
    pub(crate) name: String,
    pub(crate) location: WarningLocation,
    pub(crate) extent: WarningLocation,
    pub(crate) advice: Option<Vec<u8>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct SiteKey {
    allocation: usize,
    source: usize,
    start: usize,
    end: usize,
}
impl From<&WarningLocation> for SiteKey {
    fn from(location: &WarningLocation) -> Self {
        let span = location.span();
        Self {
            allocation: location.shared_text().as_ptr() as usize,
            source: span.source.index(),
            start: span.span.start,
            end: span.span.end,
        }
    }
}

pub(crate) struct DeprecationWarnings<K> {
    declarations: HashMap<K, DeprecatedDeclaration>,
    references: HashSet<(SiteKey, SiteKey)>,
    warnings: Vec<SourceWarning>,
}
impl<K> Default for DeprecationWarnings<K> {
    fn default() -> Self {
        Self {
            declarations: HashMap::new(),
            references: HashSet::new(),
            warnings: Vec::new(),
        }
    }
}
impl<K: Eq + Hash> DeprecationWarnings<K> {
    pub(crate) fn register(&mut self, identity: K, declaration: DeprecatedDeclaration) {
        self.declarations.insert(identity, declaration);
    }
    /// Call this only after the actual reference or selected call has been checked.
    pub(crate) fn contains(&self, identity: &K) -> bool {
        self.declarations.contains_key(identity)
    }
    pub(crate) fn reference(
        &mut self,
        identity: &K,
        location: WarningLocation,
    ) -> Result<(), &'static str> {
        if self.declarations.values().any(|declaration| {
            let extent = declaration.extent.span().span;
            let reference = location.span().span;
            declaration.extent.same_source(&location)
                && extent.start <= reference.start
                && reference.end <= extent.end
        }) {
            return Ok(());
        }
        let Some(declaration) = self.declarations.get(identity) else {
            return Ok(());
        };
        let key = (
            SiteKey::from(&location),
            SiteKey::from(&declaration.location),
        );
        if self.references.contains(&key) {
            return Ok(());
        }
        if self.warnings.len() >= jai_source::MAX_SOURCE_WARNINGS {
            return Err("source warning count exceeds compiler limit (16384)");
        }
        self.references.insert(key);
        let mut message = format!("procedure '{}' is deprecated", declaration.name);
        if let Some(advice) = &declaration.advice {
            message.push_str(": ");
            message.push_str(&String::from_utf8_lossy(advice));
        }
        self.warnings.push(SourceWarning::new(
            SourceWarningKind::DeprecatedProcedureReference,
            location,
            message,
            vec![WarningNote {
                location: declaration.location.clone(),
                message: "declared deprecated here".into(),
            }],
        ));
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn warnings(&self) -> &[SourceWarning] {
        &self.warnings
    }
    pub(crate) fn take(&mut self) -> Vec<SourceWarning> {
        let mut warnings = std::mem::take(&mut self.warnings);
        warnings.sort_by(|left, right| {
            left.location()
                .path()
                .cmp(right.location().path())
                .then_with(|| {
                    left.location()
                        .span()
                        .span
                        .start
                        .cmp(&right.location().span().span.start)
                })
        });
        warnings
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::{SourceMap, SourceSpan, Span};

    fn sites() -> (WarningLocation, WarningLocation, WarningLocation) {
        let mut sources = SourceMap::default();
        let id = sources.insert(
            "warnings.jai".into(),
            "old::() #deprecated {}\nold(); old();".into(),
        );
        let at = |start, end| {
            WarningLocation::new(
                sources.get(id).unwrap(),
                SourceSpan {
                    source: id,
                    span: Span::new(start, end),
                },
            )
            .unwrap()
        };
        (at(8, 19), at(23, 26), at(30, 33))
    }

    #[test]
    fn retried_and_specialized_references_share_the_actual_source_target() {
        let (declaration, first, second) = sites();
        let mut warnings = DeprecationWarnings::default();
        for identity in [1_u32, 2] {
            warnings.register(
                identity,
                DeprecatedDeclaration {
                    name: "old".into(),
                    location: declaration.clone(),
                    extent: declaration.clone(),
                    advice: Some(b"use replacement".to_vec()),
                },
            );
        }
        assert!(warnings.contains(&1));
        warnings.reference(&1, first.clone()).unwrap();
        warnings.reference(&1, first.clone()).unwrap();
        warnings.reference(&2, first).unwrap();
        warnings.reference(&2, second).unwrap();
        assert_eq!(warnings.warnings().len(), 2);
        assert_eq!(
            warnings.warnings()[0].message(),
            "procedure 'old' is deprecated: use replacement"
        );
        assert_eq!(
            warnings.warnings()[0].notes()[0].location.text(),
            "#deprecated"
        );
        assert_eq!(warnings.take().len(), 2);
        assert!(warnings.warnings().is_empty());
    }

    #[test]
    fn deprecated_callers_suppress_warnings_without_poisoning_other_references() {
        let (declaration, first, _) = sites();
        let mut warnings = DeprecationWarnings::default();
        warnings.register(
            1,
            DeprecatedDeclaration {
                name: "old".into(),
                location: declaration.clone(),
                extent: first.clone(),
                advice: None,
            },
        );
        warnings.reference(&1, first.clone()).unwrap();
        warnings.reference(&9, first.clone()).unwrap();
        assert!(warnings.warnings().is_empty());
        warnings.register(
            1,
            DeprecatedDeclaration {
                name: "old".into(),
                location: declaration.clone(),
                extent: declaration,
                advice: None,
            },
        );
        warnings.reference(&1, first).unwrap();
        assert_eq!(warnings.warnings().len(), 1);
    }

    #[test]
    fn equal_coordinates_from_another_source_session_remain_distinct() {
        let (declaration, first, _) = sites();
        let (_, foreign, _) = sites();
        let mut warnings = DeprecationWarnings::default();
        warnings.register(
            1,
            DeprecatedDeclaration {
                name: "old".into(),
                location: declaration.clone(),
                extent: declaration,
                advice: None,
            },
        );
        warnings.reference(&1, first).unwrap();
        warnings.reference(&1, foreign).unwrap();
        assert_eq!(warnings.warnings().len(), 2);
    }
}
