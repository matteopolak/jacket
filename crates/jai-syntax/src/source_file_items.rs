//! File-only operations remain typed source items, never fabricated declarations.
use super::*;
#[derive(Clone, Copy, Debug)]
pub enum PokedName {
    Name(Symbol),
    Operator(OperatorKind),
}
#[derive(Clone, Debug)]
pub struct PokeNameDirective {
    pub namespace: NamePath,
    pub name: PokedName,
    pub span: Span,
}
impl Parser<'_> {
    pub(super) fn source_file_item(
        &mut self,
        source: jai_source::SourceId,
        nested: bool,
    ) -> Result<Option<FileItem>, Diagnostic> {
        let start = self.token().span.start;
        if matches!(
            self.token().kind,
            Kind::Directive(Directive::Library | Directive::SystemLibrary)
        ) {
            let declaration = self.anonymous_library_declaration()?;
            return Ok(Some(FileItem::Library {
                declaration,
                location: self.location_from(source, start),
            }));
        }
        if self.token().kind == Kind::UnknownDirective && self.text() == "#poke_name" {
            self.at += 1;
            let namespace = self.name_path()?;
            let name = if let Some((operator, _)) = self.operator_prefix()? {
                PokedName::Operator(operator)
            } else {
                PokedName::Name(self.name()?)
            };
            self.need(Punct::Semicolon)?;
            return Ok(Some(FileItem::PokeName {
                directive: PokeNameDirective {
                    namespace,
                    name,
                    span: Span::new(start, self.tokens[self.at - 1].span.end),
                },
                location: self.location_from(source, start),
            }));
        }
        // Authentic file conditionals may contain compile-time calls. Do not reinterpret
        // arbitrary top-level assignments or malformed declarations as executable code.
        if (nested || self.file_conditional_depth > 0) && self.token().kind == Kind::Ident {
            let mut at = self.at + 1;
            while self
                .tokens
                .get(at)
                .is_some_and(|t| t.kind == Kind::Punctuation(Punct::Dot))
            {
                if !self
                    .tokens
                    .get(at + 1)
                    .is_some_and(|t| t.kind == Kind::Ident)
                {
                    break;
                }
                at += 2;
            }
            if self
                .tokens
                .get(at)
                .is_some_and(|t| t.kind == Kind::Punctuation(Punct::OpenParen))
            {
                let expression = self.expression(0)?;
                if !matches!(
                    expression.kind,
                    ExpressionKind::Call(..) | ExpressionKind::QualifiedCall(..)
                ) {
                    return Err(Diagnostic::new(
                        expression.span,
                        "file executable item requires a direct call",
                    ));
                }
                self.need(Punct::Semicolon)?;
                return Ok(Some(FileItem::Execute {
                    expression,
                    location: self.location_from(source, start),
                }));
            }
        }
        Ok(None)
    }
}
