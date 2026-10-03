//! Record-body directives retain ordered members for specialization-time selection.
use super::*;

// Bound recursive parsing before the semantic selector's independent work budget.
const MAX_RECORD_CONDITIONAL_DEPTH: usize = 64;

impl Parser<'_> {
    pub(super) fn record_assertion(&mut self) -> Result<RecordMember, Diagnostic> {
        let start = self.token().span.start;
        let (condition, message) = self.assertion_arguments()?;
        Ok(RecordMember::Assert {
            condition,
            message,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }

    pub(super) fn record_conditional(&mut self) -> Result<RecordMember, Diagnostic> {
        if self.record_conditional_depth >= MAX_RECORD_CONDITIONAL_DEPTH {
            return Err(Diagnostic::new(
                self.token().span,
                "record conditional nesting exceeds the parser limit of 64",
            ));
        }
        self.record_conditional_depth += 1;
        let result = self.record_conditional_inner();
        self.record_conditional_depth -= 1;
        result
    }

    fn record_conditional_inner(&mut self) -> Result<RecordMember, Diagnostic> {
        let start = self.token().span.start;
        self.at += 1;
        let (condition, operator, complete) = self.source_conditional_header()?;
        if let Some(operator) = operator {
            let cases = self.source_cases(
                start,
                condition,
                operator,
                complete,
                Self::source_case_record_members,
            )?;
            let span = cases.span;
            return Ok(RecordMember::CompileTimeCases {
                cases,
                span,
            });
        }
        let then_members = self.record_conditional_body()?;
        let else_members = if self.keyword(Keyword::Else) {
            if self.token().kind == Kind::Directive(Directive::If) {
                vec![self.record_conditional()?]
            } else {
                self.record_conditional_body()?
            }
        } else {
            Vec::new()
        };
        Ok(RecordMember::Conditional {
            condition,
            then_members,
            else_members,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }

    fn record_conditional_body(&mut self) -> Result<Vec<RecordMember>, Diagnostic> {
        if self.is(Punct::OpenBrace) {
            self.record_members()
        } else {
            self.keyword(Keyword::Then);
            self.record_member_group()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn record_conditional_depth_is_bounded_on_a_two_mib_thread_stack() {
        std::thread::Builder::new()
            .stack_size(2 * 1024 * 1024)
            .spawn(|| {
                for (depth, allowed) in [(64, true), (65, false), (130, false)] {
                    let text = format!(
                        "main :: () -> int {{ Record :: struct {{ {}value: int = 42;{} }} return 0; }}",
                        "#if true {".repeat(depth),
                        "}".repeat(depth)
                    );
                    let mut sources = SourceMap::default();
                    let id = sources.insert("record-depth.jai".into(), text.clone());
                    let result = parse_file(sources.get(id).unwrap(), &mut Symbols::default());
                    if allowed {
                        assert!(result.is_ok(), "{result:?}");
                    } else {
                        let diagnostic = result.unwrap_err();
                        assert_eq!(diagnostic.location.source, id);
                        assert_eq!(diagnostic.location.span.text(&text), "#if");
                        assert_eq!(
                            diagnostic.message,
                            "record conditional nesting exceeds the parser limit of 64"
                        );
                    }
                }
                let branch = format!(
                    "{}value: int;{}",
                    "#if true {".repeat(63),
                    "}".repeat(63)
                );
                let text = format!("Record :: struct {{ #if true {{ {branch} }} else {{ {branch} }} }}");
                let mut sources = SourceMap::default();
                let id = sources.insert("sibling-record-depth.jai".into(), text);
                parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
            })
            .unwrap()
            .join()
            .unwrap();
    }

    #[test]
    fn hash_map_assertions_and_unbraced_alias_selections_preserve_member_order() {
        let text = "HashMap :: struct (Load_Factor := 70, hash_func: (int)->int = null) { #assert Load_Factor > 0 && Load_Factor < 100; before: int; #if hash_func HashKey :: hash_func; else HashKey :: fallback; after: int; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("hash-map-shape.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &parsed.items()[0]
        else {
            panic!("expected record")
        };
        assert!(matches!(
            record.members.as_slice(),
            [
                RecordMember::Assert { .. },
                RecordMember::Field(_),
                RecordMember::Conditional { .. },
                RecordMember::Field(_)
            ]
        ));
        let RecordMember::Assert {
            condition,
            span,
            ..
        } = &record.members[0]
        else {
            panic!()
        };
        assert_eq!(
            condition.span.text(text),
            "Load_Factor > 0 && Load_Factor < 100"
        );
        assert!(span.text(text).ends_with(';'));
        let RecordMember::Conditional {
            then_members,
            else_members,
            span,
            ..
        } = &record.members[2]
        else {
            panic!()
        };
        assert!(matches!(
            then_members.as_slice(),
            [RecordMember::Constant(_)]
        ));
        assert!(matches!(
            else_members.as_slice(),
            [RecordMember::Constant(_)]
        ));
        assert_eq!(
            span.text(text),
            "#if hash_func HashKey :: hash_func; else HashKey :: fallback;"
        );
    }

    #[test]
    fn braced_members_grouped_fields_and_nested_else_selections_are_not_flattened() {
        let text = "Choice :: struct { #if ENABLED { x,y: int; #assert LIMIT > 0; } else #if OTHER then Inner :: struct { z: int; } else fallback: u8; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("record-selection.jai".into(), text.into());
        let parsed = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &parsed.items()[0]
        else {
            panic!()
        };
        let RecordMember::Conditional {
            then_members,
            else_members,
            ..
        } = &record.members[0]
        else {
            panic!()
        };
        assert!(matches!(
            then_members.as_slice(),
            [
                RecordMember::Field(_),
                RecordMember::Field(_),
                RecordMember::Assert { .. }
            ]
        ));
        let [
            RecordMember::Conditional {
                then_members,
                else_members,
                ..
            },
        ] = else_members.as_slice()
        else {
            panic!("expected nested condition")
        };
        assert!(matches!(then_members.as_slice(), [RecordMember::Record(_)]));
        assert!(matches!(else_members.as_slice(), [RecordMember::Field(_)]));
    }
}
