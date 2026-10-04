//! Ordered enum bodies retain inactive members and generator requests as source syntax.
use super::*;

#[derive(Clone, Debug)]
pub enum EnumBodyItem {
    Member(EnumMember),
    Conditional {
        condition: Expression,
        then_items: Vec<EnumBodyItem>,
        else_items: Vec<EnumBodyItem>,
        span: Span,
    },
    Insert(InsertDirective),
}
impl EnumBodyItem {
    pub fn span(&self) -> Span {
        match self {
            Self::Member(member) => member.span,
            Self::Conditional {
                span, ..
            } => *span,
            Self::Insert(directive) => directive.span,
        }
    }
}

/// Selection is incremental: a condition can resolve previously admitted members.
/// All checked enum producers use this cursor; syntax tooling visits both branches.
pub struct EnumMemberCursor<'a> {
    pending: Vec<(std::slice::Iter<'a, EnumBodyItem>, usize)>,
    visited: usize,
}
impl<'a> EnumMemberCursor<'a> {
    pub fn new(items: &'a [EnumBodyItem]) -> Self {
        Self {
            pending: vec![(items.iter(), 0)],
            visited: 0,
        }
    }
    pub fn next_member<E>(
        &mut self,
        mut condition: impl FnMut(&Expression) -> Result<bool, E>,
        mut invalid: impl FnMut(Diagnostic) -> E,
    ) -> Result<Option<&'a EnumMember>, E> {
        while let Some((iter, depth)) = self.pending.last_mut() {
            let depth = *depth;
            let Some(item) = iter.next() else {
                self.pending.pop();
                continue;
            };
            self.visited += 1;
            if self.visited > 65_536 || depth > 128 {
                return Err(invalid(Diagnostic::new(
                    item.span(),
                    "enum body selection exceeds its syntax work limit",
                )));
            }
            match item {
                EnumBodyItem::Member(member) => return Ok(Some(member)),
                EnumBodyItem::Conditional {
                    condition: expression,
                    then_items,
                    else_items,
                    ..
                } => {
                    let selected = if condition(expression)? {
                        then_items
                    } else {
                        else_items
                    };
                    self.pending.push((selected.iter(), depth + 1));
                }
                EnumBodyItem::Insert(directive) => {
                    return Err(invalid(Diagnostic::new(
                        directive.span,
                        "enum #insert requires an enum-member expansion producer",
                    )));
                }
            }
        }
        Ok(None)
    }
}

impl Parser<'_> {
    pub(super) fn enum_body_items(
        &mut self,
        depth: usize,
    ) -> Result<Vec<EnumBodyItem>, Diagnostic> {
        if depth > 64 {
            return Err(self.error("enum conditional nesting exceeds the parser limit of 64"));
        }
        let mut items = Vec::new();
        while !self.take(Punct::CloseBrace) {
            if self.token().kind == Kind::Eof {
                return Err(self.error("unterminated enum declaration"));
            }
            if self.take(Punct::Semicolon) {
                continue;
            }
            let start = self.token().span.start;
            if self.token().kind == Kind::Directive(Directive::If) {
                self.at += 1;
                let condition = self.expression(0)?;
                self.need(Punct::OpenBrace)?;
                let then_items = self.enum_body_items(depth + 1)?;
                let else_items = if self.keyword(Keyword::Else) {
                    if self.token().kind == Kind::Directive(Directive::If) {
                        // Reuse the same item grammar for a single chained conditional.
                        vec![self.enum_conditional_item(depth + 1)?]
                    } else {
                        self.need(Punct::OpenBrace)?;
                        self.enum_body_items(depth + 1)?
                    }
                } else {
                    Vec::new()
                };
                items.push(EnumBodyItem::Conditional {
                    condition,
                    then_items,
                    else_items,
                    span: Span::new(start, self.tokens[self.at - 1].span.end),
                });
            } else if self.token().kind == Kind::Directive(Directive::Insert) {
                let directive = self.insert_directive(0)?;
                self.insert_terminator(&directive)?;
                items.push(EnumBodyItem::Insert(directive));
            } else {
                let name = self.name()?;
                let initializer = if self.take(Punct::Constant) {
                    Some(self.expression(0)?)
                } else {
                    None
                };
                self.need(Punct::Semicolon)?;
                let notes = self.notes()?;
                items.push(EnumBodyItem::Member(EnumMember {
                    name,
                    initializer,
                    notes,
                    span: Span::new(start, self.tokens[self.at - 1].span.end),
                }));
            }
        }
        Ok(items)
    }
    fn enum_conditional_item(&mut self, depth: usize) -> Result<EnumBodyItem, Diagnostic> {
        if depth > 64 {
            return Err(self.error("enum conditional nesting exceeds the parser limit of 64"));
        }
        let start = self.token().span.start;
        self.at += 1;
        let condition = self.expression(0)?;
        self.need(Punct::OpenBrace)?;
        let then_items = self.enum_body_items(depth + 1)?;
        let else_items = if self.keyword(Keyword::Else) {
            if self.token().kind == Kind::Directive(Directive::If) {
                vec![self.enum_conditional_item(depth + 1)?]
            } else {
                self.need(Punct::OpenBrace)?;
                self.enum_body_items(depth + 1)?
            }
        } else {
            Vec::new()
        };
        Ok(EnumBodyItem::Conditional {
            condition,
            then_items,
            else_items,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
}

/// Read-only source tooling sees every declared member, including inactive branches.
/// This iterator does not imply that a member exists in the checked enum.
pub struct EnumMemberSyntax<'a> {
    pending: Vec<std::slice::Iter<'a, EnumBodyItem>>,
}
impl<'a> EnumMemberSyntax<'a> {
    pub fn new(items: &'a [EnumBodyItem]) -> Self {
        Self {
            pending: vec![items.iter()],
        }
    }
}
impl<'a> Iterator for EnumMemberSyntax<'a> {
    type Item = &'a EnumMember;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(iter) = self.pending.last_mut() {
            match iter.next() {
                Some(EnumBodyItem::Member(member)) => return Some(member),
                Some(EnumBodyItem::Conditional {
                    then_items,
                    else_items,
                    ..
                }) => {
                    self.pending.push(else_items.iter());
                    self.pending.push(then_items.iter());
                }
                Some(EnumBodyItem::Insert(_)) => {}
                None => {
                    self.pending.pop();
                }
            }
        }
        None
    }
}
