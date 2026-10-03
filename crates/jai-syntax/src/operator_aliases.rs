//! Namespace operator aliases retain a typed token edge, not a callable name.
use super::*;

#[derive(Clone, Debug)]
pub struct OperatorAlias {
    /// A diagnostic label; operator aliases never enter the ordinary namespace.
    pub name: Symbol,
    pub kind: OperatorKind,
    pub target_namespace: NamePath,
    pub target_kind: OperatorKind,
    pub span: Span,
}

impl Parser<'_> {
    pub(super) fn operator_alias(
        &mut self,
        kind: OperatorKind,
        name: Symbol,
        start: usize,
    ) -> Result<OperatorAlias, Diagnostic> {
        let mut target_namespace = NamePath {
            root: self.name()?,
            members: vec![],
        };
        loop {
            self.need(Punct::Dot)?;
            if self.token().kind == Kind::Keyword(Keyword::Operator) {
                break;
            }
            target_namespace.members.push(self.name()?);
        }
        let (target_kind, _) = self
            .operator_prefix()?
            .expect("operator alias target starts with the operator keyword");
        if !kind.shares_token(target_kind) {
            return Err(Diagnostic::new(
                Span::new(start, self.tokens[self.at - 1].span.end),
                "operator alias must preserve its target operator token",
            ));
        }
        self.need(Punct::Semicolon)?;
        Ok(OperatorAlias {
            name,
            kind,
            target_namespace,
            target_kind,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}
