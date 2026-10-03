//! Lookahead distinguishes bodyless procedure types from bodies and bound prototypes.
use crate::{BuiltinType, Directive, Kind, Parser, Punct};

impl Parser<'_> {
    pub(super) fn bodyless_procedure_type_prefix(&self) -> bool {
        let header = self.at + 2;
        let mut depth = 0usize;
        let mut typed_parameter = false;
        let mut close = None;
        for (index, token) in self.tokens[header..].iter().enumerate() {
            match token.kind {
                Kind::Punctuation(Punct::OpenParen) => depth += 1,
                Kind::Punctuation(Punct::CloseParen) => {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(header + index);
                        break;
                    }
                }
                Kind::Punctuation(Punct::Colon) if depth == 1 => typed_parameter = true,
                Kind::Eof => return false,
                _ => {}
            }
        }
        let Some(close) = close else {
            return false;
        };
        let next = self.tokens[close + 1].kind;
        let single_builtin = close == header + 2
            && self.tokens[header + 1].kind == Kind::Ident
            && BuiltinType::from_spelling(&self.tokens[header + 1].spelling(self.source)).is_some();
        if !typed_parameter
            && close != header + 1
            && !single_builtin
            && next != Kind::Punctuation(Punct::Arrow)
        {
            return false;
        }
        let mut brackets = 0usize;
        let mut parentheses = 0usize;
        for token in &self.tokens[close + 1..] {
            match token.kind {
                Kind::Directive(
                    Directive::Foreign
                    | Directive::Compiler
                    | Directive::Intrinsic
                    | Directive::EntryPoint
                    | Directive::Expand
                    | Directive::Deprecated
                    | Directive::Modify,
                )
                | Kind::Eof => return false,
                Kind::Punctuation(Punct::QuickLambda) => return false,
                Kind::Punctuation(Punct::OpenBrace) if parentheses == 0 && brackets == 0 => {
                    return false;
                }
                Kind::Punctuation(Punct::OpenParen) => parentheses += 1,
                Kind::Punctuation(Punct::CloseParen) => {
                    let Some(inner) = parentheses.checked_sub(1) else {
                        return false;
                    };
                    parentheses = inner;
                }
                Kind::Punctuation(Punct::OpenBracket) => brackets += 1,
                Kind::Punctuation(Punct::CloseBracket) => {
                    let Some(inner) = brackets.checked_sub(1) else {
                        return false;
                    };
                    brackets = inner;
                }
                Kind::Punctuation(Punct::Semicolon) if parentheses == 0 && brackets == 0 => {
                    return true;
                }
                _ => {}
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use jai_source::SourceMap;

    fn parse_source(text: &str) -> ParsedFile {
        let mut sources = SourceMap::default();
        let id = sources.insert("callback-alias.jai".into(), text.into());
        parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap()
    }

    #[test]
    fn basic_allocator_callback_is_a_procedure_type_alias() {
        let file = parse_source(
            "Allocator_Proc :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory_pointer: *void, proc_data: *void) -> *void;",
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::TypeAlias(alias),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected type alias")
        };
        let TypeSyntax::Procedure(signature) = &alias.ty else {
            panic!("expected procedure type")
        };
        assert_eq!(signature.parameters.len(), 5);
        assert_eq!(signature.results.len(), 1);
        assert!(matches!(signature.results[0].ty, TypeSyntax::Pointer(_)));
        assert!(
            signature
                .parameters
                .iter()
                .all(|parameter| parameter.name.is_some())
        );
    }

    #[test]
    fn callback_aliases_do_not_reclassify_bodies_prototypes_or_constants() {
        let file = parse_source(
            "Handler :: (value: int) #no_context; Empty :: (); defined :: () -> int { return 1; } bound :: () -> int #intrinsic; value :: (1 + 2);",
        );
        let kinds: Vec<_> = file
            .items()
            .iter()
            .map(|item| {
                let FileItem::Declaration(declaration) = item else {
                    panic!()
                };
                &declaration.kind
            })
            .collect();
        assert!(matches!(
            kinds.as_slice(),
            [
                FileDeclarationKind::TypeAlias(_),
                FileDeclarationKind::TypeAlias(_),
                FileDeclarationKind::Procedure(_),
                FileDeclarationKind::ProcedurePrototype(_),
                FileDeclarationKind::Constant(_)
            ]
        ));
    }

    #[test]
    fn local_callback_alias_reaches_type_dispatch_before_procedure_bodies() {
        let file = parse_source(
            "main :: () { Callback :: (value: int) -> int #no_context; callback: Callback; }",
        );
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected procedure")
        };
        assert!(matches!(
            procedure.body[0].kind,
            StatementKind::TypeAlias(TypeAliasDeclaration {
                ty: TypeSyntax::Procedure(_),
                ..
            })
        ));
    }
}
