//! Lexical name promotion preserves selector names or computed name lists.
use super::*;

#[derive(Clone, Debug)]
pub struct UsingDirective {
    pub target: Expression,
    pub selection: UsingSelection,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum UsingSelection {
    All,
    Only(UsingNames),
    Except(UsingNames),
    Map(Box<Expression>),
}

#[derive(Clone, Debug)]
pub enum UsingNames {
    Names(Vec<UsingName>),
    Expression(Box<Expression>),
}

#[derive(Clone, Copy, Debug)]
pub struct UsingName {
    pub name: Symbol,
    pub span: Span,
}
