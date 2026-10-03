//! Assignment targets preserve the source place operation without assigning IDs.
use super::*;

#[derive(Clone, Debug)]
pub struct PlaceSyntax {
    pub kind: PlaceKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum PlaceKind {
    Insert(Box<InsertDirective>),
    Name(Symbol),
    Qualified(NamePath),
    Member {
        base: Box<Expression>,
        member: Symbol,
    },
    Index {
        base: Box<Expression>,
        index: Box<Expression>,
    },
    Dereference(Box<Expression>),
}
impl TryFrom<Expression> for PlaceSyntax {
    type Error = Diagnostic;
    fn try_from(expression: Expression) -> Result<Self, Diagnostic> {
        let kind = match expression.kind {
            ExpressionKind::Insert(directive) => PlaceKind::Insert(directive),
            ExpressionKind::Name(name) => PlaceKind::Name(name),
            ExpressionKind::QualifiedName(path) => PlaceKind::Qualified(path),
            ExpressionKind::Member {
                base,
                member,
            } => PlaceKind::Member {
                base,
                member,
            },
            ExpressionKind::Index {
                base,
                index,
            } => PlaceKind::Index {
                base,
                index,
            },
            ExpressionKind::Dereference(pointer) => PlaceKind::Dereference(pointer),
            _ => {
                return Err(Diagnostic::new(
                    expression.span,
                    "assignment requires a place",
                ));
            }
        };
        Ok(Self {
            kind,
            span: expression.span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;
    fn body(source: &str) -> Vec<Statement> {
        let mut sources = SourceMap::default();
        let id = sources.insert("places.jai".into(), source.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        procedure.body.clone()
    }
    #[test]
    fn index_member_and_dereference_writes_preserve_place_operations() {
        let body = body(
            "main :: () { array[1].x = 7; array[0] += 2; pointer.* = 3; a, b: int; array[0], pointer.* = pair(); }",
        );
        assert!(
            matches!(&body[0].kind, StatementKind::AssignPlace {target:PlaceSyntax {kind:PlaceKind::Member {base,..},..},..} if matches!(base.kind, ExpressionKind::Index{..}))
        );
        assert!(matches!(
            &body[1].kind,
            StatementKind::UpdatePlace {
                target: PlaceSyntax {
                    kind: PlaceKind::Index { .. },
                    ..
                },
                operation: BinaryOp::Add,
                ..
            }
        ));
        assert!(matches!(
            &body[2].kind,
            StatementKind::AssignPlace {
                target: PlaceSyntax {
                    kind: PlaceKind::Dereference(_),
                    ..
                },
                ..
            }
        ));
        assert!(
            matches!(&body[3].kind, StatementKind::DeclareResults {names,ty:Some(_),values} if names.len()==2 && values.is_empty())
        );
        assert!(
            matches!(&body[4].kind, StatementKind::AssignResults {targets,values,..} if targets.len()==2 && values.len()==1)
        );
    }
    #[test]
    fn inserted_code_can_denote_an_assignment_place() {
        let statements = body("main :: () { (#insert a) = (#insert b); }");
        assert!(matches!(
            &statements[0].kind,
            StatementKind::AssignPlace {
                target: PlaceSyntax {
                    kind: PlaceKind::Insert(_),
                    ..
                },
                value: Expression {
                    kind: ExpressionKind::Insert(_),
                    ..
                },
            }
        ));
    }

    #[test]
    fn nonplace_assignments_and_invalid_binding_targets_are_rejected() {
        for source in [
            "main :: () { 1 = 2; }",
            "main :: () { call() = 2; }",
            "main :: () { *value = 2; }",
            "main :: () { object.x, b := pair(); }",
            "main :: () { array[] = 3; }",
            "main :: () { a, = pair(); }",
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("bad.jai".into(), source.into());
            assert!(
                parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err(),
                "{source}"
            );
        }
    }
}
