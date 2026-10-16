use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    BuiltinNoSource,
    InactiveTarget,
    UninstantiatedGeneric,
    GeneratedNoSource,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Missing {
    pub hover: bool,
    pub definition: bool,
    pub wrong_definition: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Target {
    pub uri: String,
    pub line: u32,
    pub character: u32,
}

pub fn definitions_match(expected: &[Target], actual: &[Target]) -> bool {
    let mut expected = expected.to_vec();
    expected.sort();
    expected.dedup();
    let mut actual = actual.to_vec();
    actual.sort();
    actual.dedup();
    expected == actual
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub case_id: String,
    pub input_id: String,
    pub fingerprint: String,
    pub line: u32,
    pub character: u32,
    pub missing: Missing,
    pub reason: Reason,
    pub evidence: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Oracle {
    pub case_id: String,
    pub input_id: String,
    pub fingerprint: String,
    pub line: u32,
    pub character: u32,
    pub definitions: Vec<Target>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(default)]
    pub reviews: Vec<Review>,
    #[serde(default)]
    pub oracles: Vec<Oracle>,
}

impl Manifest {
    pub fn validate(&self) -> Result<(), String> {
        let mut reviews = std::collections::HashSet::new();
        for review in &self.reviews {
            if review.evidence.trim().is_empty()
                || review.fingerprint.is_empty()
                || review.missing == Missing::default()
                || review.missing.wrong_definition
            {
                return Err(
                    "review needs fingerprint/evidence and cannot waive wrong definitions".into(),
                );
            }
            if review.reason == Reason::BuiltinNoSource
                && (review.missing.hover || !review.missing.definition)
            {
                return Err("builtin no-source reviews cannot waive missing hover".into());
            }
            if !reviews.insert((
                &review.case_id,
                &review.input_id,
                review.line,
                review.character,
            )) {
                return Err("duplicate gap review".into());
            }
        }
        let mut oracles = std::collections::HashSet::new();
        for oracle in &self.oracles {
            if oracle.fingerprint.is_empty()
                || !oracles.insert((
                    &oracle.case_id,
                    &oracle.input_id,
                    oracle.line,
                    oracle.character,
                ))
            {
                return Err("invalid or duplicate exact-location oracle".into());
            }
        }
        Ok(())
    }
}

#[derive(Serialize)]
pub struct Gap {
    pub line: u32,
    pub character: u32,
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub missing: Missing,
    pub classification: Option<Reason>,
    pub definitions: Vec<Target>,
}

#[derive(Serialize)]
pub struct Failure {
    pub kind: String,
    pub line: Option<u32>,
    pub character: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<String>,
}

#[derive(Serialize)]
pub struct File {
    pub case_id: String,
    pub input_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub fingerprint: Option<String>,
    pub status: String,
    pub identifiers: usize,
    pub syntax_tokens: usize,
    pub semantic: Option<jai_language_server::SemanticCoverage>,
    pub failures: Vec<Failure>,
    pub gaps: Vec<Gap>,
}

#[derive(Default, Serialize)]
pub struct Counts {
    pub files: usize,
    pub audited_files: usize,
    pub unexecuted_files: usize,
    pub unqueried_files: usize,
    pub typecheck_failed_files: usize,
    pub identifiers: usize,
    pub syntax_tokens: usize,
    pub gaps: usize,
    pub unexpected_gaps: usize,
    pub reviewed_gaps: usize,
    pub stale_reviews: usize,
    pub missing_oracles: usize,
    pub failures: usize,
    pub by_role: BTreeMap<String, usize>,
    pub by_missing: BTreeMap<String, usize>,
    pub by_failure: BTreeMap<String, usize>,
    pub gaps_in_failed_files: usize,
    pub semantic_unreached_files: usize,
    pub incomplete_context_files: usize,
}

#[derive(Serialize)]
pub struct Report {
    pub schema: u32,
    pub build_id: String,
    pub stdlib_id: String,
    pub target: String,
    pub runtime_support: bool,
    pub redacted: bool,
    pub scope: String,
    pub accepted: bool,
    pub counts: Counts,
    pub files: Vec<File>,
    pub expected_inputs: usize,
    pub query_selection: Option<Vec<String>>,
}

pub fn fingerprint(text: &str) -> String {
    // Stable change detection; release identity uses SHA256 of binaries and input manifests.
    let hash = text.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
    });
    format!("fnv1a64:{hash:016x}")
}

impl Report {
    pub fn finish(&mut self, reviews: &[Review], used: &[bool], missing_oracles: usize) {
        let mut counts = Counts {
            stale_reviews: used.iter().filter(|used| !**used).count(),
            missing_oracles,
            ..Counts::default()
        };
        for file in &self.files {
            counts.files += 1;
            counts.identifiers += file.identifiers;
            counts.syntax_tokens += file.syntax_tokens;
            counts.failures += file.failures.len();
            if file.semantic.as_ref().is_none_or(|s| !s.source_loaded) {
                counts.semantic_unreached_files += 1;
            }
            if file
                .semantic
                .as_ref()
                .is_some_and(|s| !s.program_complete || s.program_errors > 0)
            {
                counts.incomplete_context_files += 1;
            }
            for failure in &file.failures {
                *counts.by_failure.entry(failure.kind.clone()).or_default() += 1;
            }
            if file.status == "unexecuted" {
                counts.unexecuted_files += 1;
            } else {
                counts.audited_files += 1;
            }
            if file.failures.iter().any(|f| f.kind == "check") {
                counts.typecheck_failed_files += 1;
            }
            for gap in &file.gaps {
                counts.gaps += 1;
                *counts.by_role.entry(gap.role.clone()).or_default() += 1;
                let flags = format!(
                    "hover:{} definition:{} wrong:{}",
                    gap.missing.hover, gap.missing.definition, gap.missing.wrong_definition
                );
                *counts.by_missing.entry(flags).or_default() += 1;
                if !file.failures.is_empty() {
                    counts.gaps_in_failed_files += 1;
                }
                if gap.classification.is_some() {
                    counts.reviewed_gaps += 1;
                } else {
                    counts.unexpected_gaps += 1;
                }
            }
        }
        counts.unqueried_files = self.expected_inputs.saturating_sub(counts.files);
        counts.unexecuted_files += counts.unqueried_files;
        self.accepted = counts.files > 0
            && counts.files == self.expected_inputs
            && counts.unexpected_gaps == 0
            && counts.failures == 0
            && counts.unexecuted_files == 0
            && counts.semantic_unreached_files == 0
            && counts.incomplete_context_files == 0
            && counts.stale_reviews == 0
            && counts.missing_oracles == 0
            && reviews.iter().all(|r| !r.evidence.trim().is_empty());
        self.counts = counts;
    }

    pub fn redact(&mut self) {
        for file in &mut self.files {
            file.path = None;
            for failure in &mut file.failures {
                failure.diagnostic = None;
            }
            for gap in &mut file.gaps {
                gap.name = None;
                // Locations can contain vendor paths; keep only role/position/count data.
                gap.definitions.clear();
            }
        }
        self.redacted = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> Report {
        Report {
            schema: 1,
            build_id: "test".into(),
            stdlib_id: "test".into(),
            target: "test".into(),
            runtime_support: false,
            redacted: false,
            scope: "independent".into(),
            accepted: false,
            counts: Counts::default(),
            files: vec![File {
                case_id: "case".into(),
                input_id: "f0".into(),
                path: Some("/private/vendor/example.jai".into()),
                fingerprint: Some(fingerprint("independent")),
                status: "audited".into(),
                identifiers: 1,
                syntax_tokens: 0,
                semantic: Some(jai_language_server::SemanticCoverage {
                    context_id: "test".into(),
                    analysis: Default::default(),
                    source_loaded: true,
                    recorded_references: 1,
                    recorded_fields: 0,
                    program_complete: true,
                    program_errors: 0,
                    target: "test".into(),
                }),
                failures: vec![],
                gaps: vec![],
            }],
            expected_inputs: 1,
            query_selection: None,
        }
    }

    #[test]
    fn errors_never_waive_identifier_gaps_and_redaction_has_no_source_payload() {
        let mut report = report();
        report.files[0].failures.push(Failure {
            kind: "check".into(),
            line: Some(2),
            character: Some(4),
            diagnostic: Some("private diagnostic text".into()),
        });
        report.files[0].gaps.push(Gap {
            line: 2,
            character: 4,
            role: "member".into(),
            name: Some("private_identifier".into()),
            missing: Missing {
                hover: true,
                definition: true,
                wrong_definition: false,
            },
            classification: None,
            definitions: vec![],
        });
        report.finish(&[], &[], 0);
        assert!(!report.accepted);
        assert_eq!(report.counts.unexpected_gaps, 1);
        assert_eq!(report.counts.typecheck_failed_files, 1);
        report.redact();
        let json = serde_json::to_string(&report).unwrap();
        assert!(!json.contains("private"));
        assert!(!json.contains("vendor"));
        assert!(!json.contains("diagnostic text"));
    }

    #[test]
    fn empty_unexecuted_and_stale_review_inputs_are_not_acceptance() {
        let mut report = report();
        report.files[0].status = "unexecuted".into();
        report.finish(&[], &[], 0);
        assert!(!report.accepted);
        assert_eq!(report.counts.unexecuted_files, 1);
        report.files.clear();
        report.finish(&[], &[], 0);
        assert!(!report.accepted);
        report.files = self::report().files;
        report.finish(&[], &[false], 0);
        assert!(!report.accepted);
        assert_eq!(report.counts.stale_reviews, 1);
    }

    #[test]
    fn selection_and_unreached_context_cannot_pass_with_empty_local_diagnostics() {
        let mut report = report();
        report.finish(&[], &[], 0);
        assert!(report.accepted);
        report.expected_inputs = 2;
        report.finish(&[], &[], 0);
        assert!(!report.accepted);
        assert_eq!(report.counts.unqueried_files, 1);
        assert_eq!(report.counts.unexecuted_files, 1);
        report.expected_inputs = 1;
        report.files[0].semantic = None;
        report.finish(&[], &[], 0);
        assert!(!report.accepted);
        assert_eq!(report.counts.semantic_unreached_files, 1);
    }

    #[test]
    fn exact_definition_oracles_compare_sets_and_preserve_wrong_locations() {
        let a = Target {
            uri: "input:f0".into(),
            line: 1,
            character: 2,
        };
        let b = Target {
            uri: "input:f1".into(),
            line: 3,
            character: 4,
        };
        assert!(definitions_match(
            &[a.clone(), b.clone()],
            &[b.clone(), a.clone()]
        ));
        assert!(!definitions_match(&[a], &[b]));
    }
}
