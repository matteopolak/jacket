use jai_source::{SourceMap, Symbols};
use jai_syntax::{BinaryOp, FileDeclarationKind, FileItem, OperatorKind, StatementKind, UnaryOp};

fn parse(source: &str) -> Result<jai_syntax::ParsedFile, jai_source::LocatedDiagnostic> {
    let mut sources = SourceMap::default();
    let id = sources.insert("operators.jai".into(), source.into());
    jai_syntax::parse_file(sources.get(id).unwrap(), &mut Symbols::default())
}

#[test]
fn operators_retain_typed_kinds_and_source_procedure_bodies() {
    let file = parse("operator * :: (a:Box,b:float)->Box #symmetric #no_context { return a; } operator - :: (a:Box)->Box {return a;} operator []::(a:Box,index:int)->int{return index;} main::(){operator +::(a:Box,b:Box)->Box{return a;}} ").unwrap();
    let procedure = |index| {
        let FileItem::Declaration(declaration) = &file.items()[index] else {
            panic!();
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!();
        };
        procedure
    };
    let multiply = procedure(0).operator.unwrap();
    assert_eq!(multiply.kind, OperatorKind::Binary(BinaryOp::Multiply));
    assert!(multiply.symmetric);
    assert_eq!(
        procedure(1).operator.unwrap().kind,
        OperatorKind::Unary(UnaryOp::Negate)
    );
    assert_eq!(procedure(2).operator.unwrap().kind, OperatorKind::Index);
    let StatementKind::Procedure(local) = &procedure(3).body[0].kind else {
        panic!();
    };
    assert_eq!(
        local.operator.unwrap().kind,
        OperatorKind::Binary(BinaryOp::Add)
    );
}

#[test]
fn malformed_operator_headers_and_modifiers_are_rejected() {
    for source in [
        "operator ?::(a:Box)->Box{return a;}",
        "operator []::(a:Box)->int{return 0;}",
        "operator *::(a:Box,b:Box)->Box #symmetric #symmetric{return a;}",
        "operator -::(a:Box)->Box #symmetric{return a;}",
        "ordinary::()->int #symmetric{return 0;}",
        "operator +::(a:Box,b:..Box)->Box{return a;}",
        "operator +::(a:Box,b:Box)->Box #foreign;",
        "operator +::Ops.operator-;",
        "main::(){operator -::Ops.operator-;}",
        "Record::struct{operator -::Ops.operator-;}",
    ] {
        assert!(parse(source).is_err(), "{source}");
    }
}

#[test]
fn namespace_operator_aliases_keep_typed_edges_and_original_spans() {
    let source = "operator-::Ops.Numerics.operator-;";
    let file = parse(source).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!();
    };
    let FileDeclarationKind::OperatorAlias(alias) = &declaration.kind else {
        panic!();
    };
    assert_eq!(alias.kind, OperatorKind::Binary(BinaryOp::Subtract));
    assert_eq!(alias.target_kind, alias.kind);
    assert!(
        alias
            .kind
            .shares_token(OperatorKind::Unary(UnaryOp::Negate))
    );
    assert_eq!(alias.target_namespace.members.len(), 1);
    assert_eq!(&source[alias.span.start..alias.span.end], source);
}

#[test]
fn unchanged_optional_thread_source_parses_its_operator_alias_at_line_69() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "SKIP optional original-source operator gate: {} is absent",
                path.display()
            );
            return;
        }
        Err(error) => panic!(
            "cannot read optional original source {}: {error}",
            path.display()
        ),
    };
    let mut sources = SourceMap::default();
    let id = sources.insert(path, source);
    let source = sources.get(id).unwrap();
    let file = jai_syntax::parse_file(source, &mut Symbols::default())
        .unwrap_or_else(|error| panic!("{}", error.render(&sources)));
    let alias = file
        .items()
        .iter()
        .find_map(|item| match item {
            FileItem::Declaration(declaration) => match &declaration.kind {
                FileDeclarationKind::OperatorAlias(alias) => Some(alias),
                _ => None,
            },
            _ => None,
        })
        .expect("original Thread source must retain its alias declaration");
    assert_eq!(alias.kind, OperatorKind::Binary(BinaryOp::Subtract));
    assert_eq!(
        source.text()[..alias.span.start]
            .bytes()
            .filter(|&byte| byte == b'\n')
            .count()
            + 1,
        69
    );
}

#[test]
fn baked_operator_source_parameters_retain_typed_baking_and_arity() {
    let file = parse(
        "operator <<::(a:Box,$$x:u8)->Box{return a;} operator []::(a:Box,$i:int)->int{return i;}",
    )
    .unwrap();
    for (index, baking) in [
        jai_syntax::ParameterBaking::Optional,
        jai_syntax::ParameterBaking::Required,
    ]
    .into_iter()
    .enumerate()
    {
        let FileItem::Declaration(declaration) = &file.items()[index] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        assert_eq!(procedure.parameters.len(), 2);
        assert_eq!(procedure.parameters[1].baking, baking);
    }
}

#[test]
fn unchanged_optional_int128_source_parses_real_baked_shift_declarations() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
    let source = match std::fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            eprintln!(
                "SKIP optional original-source operator gate: {} is absent",
                path.display()
            );
            return;
        }
        Err(error) => panic!("cannot read original source {}: {error}", path.display()),
    };
    let mut sources = SourceMap::default();
    let id = sources.insert(path, source);
    let source = sources.get(id).unwrap();
    let file = jai_syntax::parse_file(source, &mut Symbols::default())
        .unwrap_or_else(|error| panic!("{}", error.render(&sources)));
    let shifts = file
        .items()
        .iter()
        .filter_map(|item| {
            let FileItem::Declaration(declaration) = item else {
                return None;
            };
            let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
                return None;
            };
            matches!(
                procedure.operator.map(|operator| operator.kind),
                Some(OperatorKind::Binary(
                    BinaryOp::ShiftLeft | BinaryOp::ShiftRight
                ))
            )
            .then_some(procedure)
        })
        .collect::<Vec<_>>();
    assert_eq!(shifts.len(), 4);
    for shift in shifts {
        assert_eq!(shift.parameters.len(), 2);
        assert_eq!(
            shift.parameters[1].baking,
            jai_syntax::ParameterBaking::Optional
        );
    }
}

#[test]
fn source_object_mutation_spellings_have_distinct_typed_identities() {
    let file = parse("operator []=::(a:*Obj,i:int,item:int){} operator *[]::(a:*Obj,i:int)->*int{return *a.value;} operator *=::(a:*Obj,scalar:int){}").unwrap();
    let kinds = file
        .items()
        .iter()
        .map(|item| {
            let FileItem::Declaration(declaration) = item else {
                panic!()
            };
            let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
                panic!()
            };
            procedure.operator.unwrap().kind
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        [
            OperatorKind::IndexAssign,
            OperatorKind::IndexAddress,
            OperatorKind::Compound(BinaryOp::Multiply)
        ]
    );
}

#[test]
fn original_object_pointer_use_retains_prefix_span_and_postfix_precedence() {
    let source = "main::()->int{return <<objects[index()].pointer+1;}";
    let parsed = parse(source).unwrap();
    let FileItem::Declaration(declaration) = &parsed.items()[0] else {
        panic!()
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!()
    };
    let StatementKind::Return(Some(expression)) = &procedure.body[0].kind else {
        panic!()
    };
    let jai_syntax::ExpressionKind::Binary(BinaryOp::Add, left, _) = &expression.kind else {
        panic!()
    };
    let jai_syntax::ExpressionKind::Dereference(pointer) = &left.kind else {
        panic!()
    };
    assert!(matches!(
        pointer.kind,
        jai_syntax::ExpressionKind::Member { .. }
    ));
    assert_eq!(left.span.text(source), "<<objects[index()].pointer");
}

#[test]
fn fixed_operator_operands_can_have_trailing_default_parameters() {
    assert!(parse("operator +=::(obj:*Obj,item:Obj,loc:=#caller_location){} operator *::(obj:Obj,item:int,tag:int=3)->Obj #symmetric{return obj;}").is_ok());
    assert!(parse("operator +=::(obj:*Obj,item:Obj,tag:int){}").is_err());
}

#[test]
fn discarded_operator_parameters_report_the_actual_parameter_span() {
    let source = "operator +::(a:Box,#discard b:Box)->Box{return a;}";
    let error = parse(source).unwrap_err();
    assert_eq!(error.location.span.text(source), "#discard b:Box");
    assert!(error.message.contains("captured argument binding"));
}
