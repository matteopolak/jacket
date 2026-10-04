//! Source case tables retain every original branch until typed semantic selection.
use super::*;

#[derive(Clone, Debug)]
pub struct CompileTimeCaseArm<T> {
    pub label: Expression,
    pub body: Vec<T>,
    pub falls_through: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct CompileTimeCaseDefault<T> {
    pub body: Vec<T>,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct CompileTimeCases<T> {
    pub default_position: Option<usize>,
    pub default_through: bool,
    pub value: Expression,
    pub operator: CaseOperator,
    pub arms: Vec<CompileTimeCaseArm<T>>,
    pub default: Option<CompileTimeCaseDefault<T>>,
    pub complete: bool,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn source_conditional_header(
        &mut self,
    ) -> Result<(Expression, Option<CaseOperator>, bool), Diagnostic> {
        let complete = self.token().kind == Kind::Directive(Directive::Complete);
        if complete {
            self.at += 1;
        }
        let value = self.expression(0)?;
        let operator = if self.is(Punct::OpenBrace) {
            match self.tokens[self.at - 1].kind {
                Kind::Punctuation(Punct::Equal) => Some(CaseOperator::Equal),
                Kind::Punctuation(Punct::NotEqual) => Some(CaseOperator::NotEqual),
                _ => None,
            }
        } else {
            None
        };
        if complete && operator.is_none() {
            return Err(self.error("#complete requires if-case"));
        }
        Ok((value, operator, complete))
    }
    pub(super) fn source_case_body_ends(&self) -> bool {
        self.is(Punct::CloseBrace)
            || matches!(
                self.token().kind,
                Kind::Eof | Kind::Keyword(Keyword::Case) | Kind::Directive(Directive::Through)
            )
    }
    pub(super) fn source_cases<T>(
        &mut self,
        start: usize,
        value: Expression,
        operator: CaseOperator,
        complete: bool,
        mut body: impl FnMut(&mut Self) -> Result<Vec<T>, Diagnostic>,
    ) -> Result<CompileTimeCases<T>, Diagnostic> {
        self.need(Punct::OpenBrace)?;
        let mut arms = Vec::new();
        let mut default = None;
        let mut default_position = None;
        let mut default_through = false;
        while !self.take(Punct::CloseBrace) {
            let label_start = self.token().span.start;
            if !self.keyword(Keyword::Case) {
                return Err(self.error("expected case label"));
            }
            let label = if self.is(Punct::Semicolon) {
                None
            } else {
                Some(self.expression(0)?)
            };
            self.need(Punct::Semicolon)?;
            let body = body(self)?;
            if self.token().kind == Kind::Eof {
                return Err(self.error("unterminated compile-time case block"));
            }
            let falls_through = if self.token().kind == Kind::Directive(Directive::Through) {
                self.at += 1;
                self.need(Punct::Semicolon)?;
                if !self.is(Punct::CloseBrace) && self.token().kind != Kind::Keyword(Keyword::Case)
                {
                    return Err(self.error("#through must be the last case statement"));
                }
                true
            } else {
                false
            };
            let span = Span::new(label_start, self.tokens[self.at - 1].span.end);
            match label {
                Some(label) => arms.push(CompileTimeCaseArm {
                    label,
                    body,
                    falls_through,
                    span,
                }),
                None => {
                    if default.is_some() {
                        return Err(self.error("duplicate default case"));
                    }
                    default_position = Some(arms.len());
                    default_through = falls_through;
                    default = Some(CompileTimeCaseDefault {
                        body,
                        span,
                    });
                }
            }
        }
        let final_through = if default_position == Some(arms.len()) {
            default_through
        } else {
            arms.last().is_some_and(|arm| arm.falls_through)
        };
        if final_through {
            return Err(self.error("last case cannot #through without a following case"));
        }
        Ok(CompileTimeCases {
            default_position,
            default_through,
            value,
            operator,
            arms,
            default,
            complete,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn source_case_statements(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        let mut body = Vec::new();
        while !self.source_case_body_ends() {
            body.push(self.statement()?);
        }
        Ok(body)
    }
    pub(super) fn source_case_record_members(&mut self) -> Result<Vec<RecordMember>, Diagnostic> {
        let mut body = Vec::new();
        while !self.source_case_body_ends() {
            body.extend(self.record_member_group()?);
        }
        Ok(body)
    }
    pub(super) fn source_case_file_items(
        &mut self,
        source: jai_source::SourceId,
        mut visibility: Visibility,
    ) -> Result<Vec<FileItem>, Diagnostic> {
        let mut body = Vec::new();
        while !self.source_case_body_ends() {
            let items = self.file_item_list_with_limit(
                source,
                visibility,
                false,
                file_conditional_bodies::FileItemCount::One,
            )?;
            for item in &items {
                if let FileItem::Scope {
                    visibility: current,
                    ..
                } = item
                {
                    visibility = *current;
                }
            }
            body.extend(items);
        }
        Ok(body)
    }
}

/// Body-free, original source inputs for one semantic selection request.
#[derive(Clone, Debug)]
pub struct CompileTimeCaseHeader {
    pub value: Expression,
    pub operator: CaseOperator,
    pub labels: Vec<Expression>,
    pub complete: bool,
    pub has_default: bool,
    pub span: Span,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompileTimeCaseChoice {
    None,
    Arm(usize),
    Default,
}
impl<T> CompileTimeCases<T> {
    pub fn header(&self) -> CompileTimeCaseHeader {
        CompileTimeCaseHeader {
            value: self.value.clone(),
            operator: self.operator,
            labels: self.arms.iter().map(|arm| arm.label.clone()).collect(),
            complete: self.complete,
            has_default: self.default.is_some(),
            span: self.span,
        }
    }
    pub fn selected_body_refs(&self, choice: CompileTimeCaseChoice) -> Option<Vec<&T>> {
        use jai_types::{CaseOrder, CaseTarget};
        let order = CaseOrder::new(
            self.arms.len(),
            self.default
                .as_ref()
                .map(|_| self.default_position.unwrap_or(self.arms.len())),
        )
        .ok()?;
        let mut target = match choice {
            CompileTimeCaseChoice::None => return Some(Vec::new()),
            CompileTimeCaseChoice::Default => CaseTarget::Default,
            CompileTimeCaseChoice::Arm(index) => CaseTarget::Arm(index),
        };
        let mut result = Vec::new();
        loop {
            let through = match target {
                CaseTarget::End => return Some(result),
                CaseTarget::Default => {
                    result.extend(&self.default.as_ref()?.body);
                    self.default_through
                }
                CaseTarget::Arm(index) => {
                    let arm = self.arms.get(index)?;
                    result.extend(&arm.body);
                    arm.falls_through
                }
            };
            if !through {
                return Some(result);
            }
            target = order.following(target).ok()?;
            if target == CaseTarget::End {
                return None;
            }
        }
    }
}

impl<T: Clone> CompileTimeCases<T> {
    pub fn selected_body(&self, choice: CompileTimeCaseChoice) -> Option<Vec<T>> {
        self.selected_body_refs(choice)
            .map(|body| body.into_iter().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;
    fn parsed(text: &str) -> ParsedFile {
        let mut sources = SourceMap::default();
        let source = sources.insert("own-source-cases.jai".into(), text.to_owned());
        parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap()
    }
    #[test]
    fn source_cases_keep_original_selector_labels_bodies_and_through() {
        let text = "main::(){ #if OS == { case .WINDOWS; os_name:=\"win\"; case .MACOS; os_name:=\"mac\"; #through; case .LINUX; os_name:=\"linux\"; case; missing(); } }";
        let file = parsed(text);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &file.items()[0]
        else {
            panic!()
        };
        let StatementKind::CompileTimeCases(cases) = &procedure.body[0].kind else {
            panic!()
        };
        assert_eq!(cases.value.span.text(text), "OS");
        assert_eq!(cases.arms[1].label.span.text(text), ".MACOS");
        assert_eq!(cases.arms[1].body[0].span.text(text), "os_name:=\"mac\";");
        assert!(cases.arms[1].falls_through);
        assert_eq!(
            cases
                .selected_body(CompileTimeCaseChoice::Arm(1))
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            cases.default.as_ref().unwrap().body[0].span.text(text),
            "missing();"
        );
    }
    #[test]
    fn file_and_record_case_bodies_preserve_visibility_and_fields() {
        let file = parsed(
            "#scope_module #if OS == {case .LINUX; #scope_file VALUE::42; case; #load \"inactive.jai\";} Record::struct(T:Type){#if T == {case u8; value:u8; case; value:s32;}} FOLLOW::1;",
        );
        let FileItem::CompileTimeCases {
            cases, ..
        } = &file.items()[1]
        else {
            panic!()
        };
        let FileItem::Declaration(value) = &cases.arms[0].body[1] else {
            panic!()
        };
        assert_eq!(value.visibility, Visibility::File);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Record(record),
            ..
        }) = &file.items()[2]
        else {
            panic!()
        };
        let RecordMember::CompileTimeCases {
            cases, ..
        } = &record.members[0]
        else {
            panic!()
        };
        assert_eq!(cases.arms.len(), 1);
        assert!(matches!(cases.arms[0].body[0], RecordMember::Field(_)));
        let FileItem::Declaration(follow) = &file.items()[3] else {
            panic!()
        };
        assert_eq!(follow.visibility, Visibility::Module);
    }
    #[test]
    fn malformed_inactive_case_bodies_are_still_parsed() {
        for text in [
            "main::(){#if 1 == {case 1; return;case 2; broken:=;}}",
            "#if true == {case; VALUE::1;case; VALUE::2;}",
            "main::(){#if 1 == {case 1; #through; bad();}}",
        ] {
            let mut sources = SourceMap::default();
            let source = sources.insert("own-invalid-case.jai".into(), text.to_owned());
            assert!(parse_file(sources.get(source).unwrap(), &mut Symbols::default()).is_err());
        }
    }
}
