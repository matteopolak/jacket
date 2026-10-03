//! Array literal targets retain genuine source type syntax before checking elements.
use super::*;

impl Parser<'_> {
    pub(super) fn array_literal_type_target(
        &self,
        expression: Expression,
    ) -> Result<TypeSyntax, Diagnostic> {
        let span = expression.span;
        let application = |base, arguments| {
            TypeSyntax::Application(TypeApplicationSyntax {
                base: Box::new(base),
                arguments,
                span,
            })
        };
        Ok(match expression.kind {
            ExpressionKind::Type(ty) => ty,
            ExpressionKind::AddressOf(inner) => {
                TypeSyntax::Pointer(Box::new(self.array_literal_type_target(*inner)?))
            }
            ExpressionKind::TypeQuery {
                query: TypeQueryKind::TypeOf,
                value,
            } => TypeSyntax::TypeOf(value),
            ExpressionKind::Name(root) => self.literal_named_type(NamePath {
                root,
                members: Vec::new(),
            }),
            ExpressionKind::QualifiedName(path) => self.literal_named_type(path),
            ExpressionKind::Call(root, arguments) => application(
                self.literal_named_type(NamePath {
                    root,
                    members: Vec::new(),
                }),
                arguments,
            ),
            ExpressionKind::QualifiedCall(path, arguments) => {
                application(self.literal_named_type(path), arguments)
            }
            ExpressionKind::IndirectCall {
                callee,
                args,
            } => application(self.array_literal_type_target(*callee)?, args),
            _ => {
                return Err(Diagnostic::new(
                    span,
                    "aggregate literal target requires type syntax",
                ));
            }
        })
    }

    fn literal_named_type(&self, path: NamePath) -> TypeSyntax {
        if path.members.is_empty()
            && let Some(builtin) = BuiltinType::from_spelling(self.symbols.name(path.root))
        {
            return TypeSyntax::Builtin(builtin);
        }
        TypeSyntax::Named(path)
    }
}
