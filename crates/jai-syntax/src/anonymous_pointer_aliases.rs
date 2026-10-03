//! Definite nominal leaves distinguish opaque pointer aliases from addresses.
#[cfg(test)]
mod tests {
    use crate::*;
    use jai_source::SourceMap;

    #[test]
    fn anonymous_nominal_pointer_aliases_keep_actual_type_bodies() {
        let source = "Opaque::*struct {}; UnionPtr::**union{a:int;b:float64;}; EnumPtr::*enum u8{ZERO::0;}; FlagsPtr::*enum_flags u8{A::1;}; NamedPtr::*Opaque; value:int; Address::*value; main::(){Local::*struct{value:int;};}";
        let mut sources = SourceMap::default();
        let id = sources.insert("opaque-aliases.jai".into(), source.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let declarations = file
            .items()
            .iter()
            .map(|item| {
                let FileItem::Declaration(declaration) = item else {
                    panic!("source declaration required");
                };
                &declaration.kind
            })
            .collect::<Vec<_>>();
        let FileDeclarationKind::TypeAlias(opaque) = declarations[0] else {
            panic!("opaque pointer alias required");
        };
        assert!(
            matches!(&opaque.ty,TypeSyntax::Pointer(inner) if matches!(inner.as_ref(),TypeSyntax::InlineRecord(record) if record.members.is_empty()))
        );
        let FileDeclarationKind::TypeAlias(union) = declarations[1] else {
            panic!("union pointer alias required");
        };
        assert!(
            matches!(&union.ty,TypeSyntax::Pointer(inner) if matches!(inner.as_ref(),TypeSyntax::Pointer(inner) if matches!(inner.as_ref(),TypeSyntax::InlineRecord(record) if record.kind==jai_types::RecordKind::Union)))
        );
        for (declaration, kind) in [
            (declarations[2], EnumKind::Values),
            (declarations[3], EnumKind::Flags),
        ] {
            let FileDeclarationKind::TypeAlias(alias) = declaration else {
                panic!("enum pointer alias required");
            };
            assert!(
                matches!(&alias.ty,TypeSyntax::Pointer(inner) if matches!(inner.as_ref(),TypeSyntax::InlineEnum(enumeration) if enumeration.kind==kind))
            );
        }
        assert!(matches!(declarations[4], FileDeclarationKind::Constant(_)));
        assert!(matches!(
            declarations[6],
            FileDeclarationKind::Constant(ConstantDeclaration {
                initializer: Expression {
                    kind: ExpressionKind::AddressOf(_),
                    ..
                },
                ..
            })
        ));
        let FileDeclarationKind::Procedure(procedure) = declarations[7] else {
            panic!("source procedure required");
        };
        assert!(matches!(
            procedure.body[0].kind,
            StatementKind::TypeAlias(_)
        ));
    }
}
