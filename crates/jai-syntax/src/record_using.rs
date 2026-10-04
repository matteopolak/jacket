//! Record promotions retain the original directive or declared child.
use super::*;
impl Parser<'_> {
    pub(super) fn record_using_group(&mut self) -> Result<Option<Vec<RecordMember>>, Diagnostic> {
        let checkpoint = self.at;
        let start = self.token().span.start;
        self.at += 1;
        let selection = self.using_selection()?;
        // A physical field qualifier is owned by the ordinary field parser.
        if self.named_prefix(Punct::Colon)
            || self.named_prefix(Punct::Infer)
            || self.token().kind == Kind::Directive(Directive::As)
            || (self.token().kind == Kind::UnknownDirective && self.text() == "#overlay")
        {
            self.at = checkpoint;
            return Ok(None);
        }
        if self.named_prefix(Punct::Constant) {
            let target_span = self.token().span;
            let name = self.symbols.intern(&self.token().spelling(self.source));
            let mut members = self.record_member_group()?;
            let span = Span::new(start, self.tokens[self.at - 1].span.end);
            members.push(RecordMember::Using(UsingDirective {
                target: Expression {
                    kind: ExpressionKind::Name(name),
                    span: target_span,
                },
                selection,
                span,
            }));
            return Ok(Some(members));
        }
        Ok(Some(vec![RecordMember::Using(
            self.using_target(start, selection)?,
        )]))
    }
}
