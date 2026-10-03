//! Source check directives override only the marked procedure or lexical block.
use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckPolicy {
    #[default]
    Inherited,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SafetyChecks {
    pub array_bounds: CheckPolicy,
    pub arithmetic_overflow: CheckPolicy,
}

impl SafetyChecks {
    pub fn has_overrides(self) -> bool {
        self.array_bounds != CheckPolicy::Inherited
            || self.arithmetic_overflow != CheckPolicy::Inherited
    }
}

pub(super) fn is_check_directive(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Directive(Directive::NoArrayBoundsCheck | Directive::NoArithmeticOverflowCheck)
    )
}

impl Parser<'_> {
    pub(super) fn safety_checks(
        &mut self,
        mut checks: SafetyChecks,
    ) -> Result<SafetyChecks, Diagnostic> {
        while is_check_directive(self.token().kind) {
            if !self.allow_qualified {
                return Err(self.error("check directives require scoped policy resolution"));
            }
            let policy = match self.token().kind {
                Kind::Directive(Directive::NoArrayBoundsCheck) => &mut checks.array_bounds,
                Kind::Directive(Directive::NoArithmeticOverflowCheck) => {
                    &mut checks.arithmetic_overflow
                }
                _ => unreachable!("check directive was validated"),
            };
            if *policy != CheckPolicy::Inherited {
                return Err(self.error(format!("duplicate {} check directive", self.text())));
            }
            *policy = CheckPolicy::Disabled;
            self.at += 1;
        }
        Ok(checks)
    }

    pub(super) fn safety_check_scope(&mut self) -> Result<StatementKind, Diagnostic> {
        let checks = self.safety_checks(SafetyChecks::default())?;
        let body = self.block()?;
        Ok(StatementKind::CheckScope {
            checks,
            body,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    fn file(text: &str) -> ParsedFile {
        let mut sources = SourceMap::default();
        let id = sources.insert("checks.jai".into(), text.into());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap()
    }

    #[test]
    fn runtime_support_procedures_and_nested_scopes_keep_independent_flags() {
        let parsed = file(
            "__element_duplicate :: (start: *u8, num_elements: s64, size: s64) #c_call #no_aoc { for 1..num_elements-1 { memcpy(start,start,size); } } write :: () #no_abc #no_context #no_aoc { #no_aoc { value := 1; } for 0..1 #no_abc { value := 2; } }",
        );
        let procedures: Vec<_> = parsed
            .items()
            .iter()
            .map(|item| {
                let FileItem::Declaration(FileDeclaration {
                    kind: FileDeclarationKind::Procedure(procedure),
                    ..
                }) = item
                else {
                    panic!("expected procedure")
                };
                procedure
            })
            .collect();
        assert_eq!(procedures[0].checks.array_bounds, CheckPolicy::Inherited);
        assert_eq!(
            procedures[0].checks.arithmetic_overflow,
            CheckPolicy::Disabled
        );
        assert_eq!(procedures[1].checks.array_bounds, CheckPolicy::Disabled);
        let StatementKind::CheckScope {
            checks, ..
        } = &procedures[1].body[0].kind
        else {
            panic!("expected scope")
        };
        assert_eq!(checks.array_bounds, CheckPolicy::Inherited);
        assert_eq!(checks.arithmetic_overflow, CheckPolicy::Disabled);
        let StatementKind::Range(range) = &procedures[1].body[1].kind else {
            panic!("expected range")
        };
        let StatementKind::CheckScope {
            checks, ..
        } = &range.body[0].kind
        else {
            panic!("expected scope")
        };
        assert_eq!(checks.array_bounds, CheckPolicy::Disabled);
        assert_eq!(checks.arithmetic_overflow, CheckPolicy::Inherited);
        assert_eq!(range.body[0].span.text("__element_duplicate :: (start: *u8, num_elements: s64, size: s64) #c_call #no_aoc { for 1..num_elements-1 { memcpy(start,start,size); } } write :: () #no_abc #no_context #no_aoc { #no_aoc { value := 1; } for 0..1 #no_abc { value := 2; } }"), "#no_abc { value := 2; }");
    }

    #[test]
    fn duplicate_check_directives_and_bodyless_policies_are_rejected() {
        for (text, message) in [
            (
                "test :: () #no_aoc #no_context #no_aoc {}",
                "duplicate #no_aoc check directive",
            ),
            (
                "test :: () #no_abc #foreign Lib;",
                "check directives require a source body",
            ),
            (
                "Callback :: () #no_aoc;",
                "check directives require a source body",
            ),
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("invalid-checks.jai".into(), text.into());
            assert_eq!(
                parse_file(sources.get(id).unwrap(), &mut Symbols::default())
                    .unwrap_err()
                    .message,
                message
            );
        }
    }
}
