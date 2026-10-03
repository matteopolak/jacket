use super::*;

#[test]
fn file_census_visits_nested_source_defaults_and_spare_capacity() {
    let mut sources = jai_source::SourceMap::default();
    let id = sources.insert(
        "metadata.jai".into(),
        "f :: (value: string = \"abc\") { local :: #code { return \"inside\"; }; }".into(),
    );
    let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let mut bytes = 0;
    file.visit_retained_metadata(&mut |_, value| {
        bytes += value;
        Ok::<_, ()>(())
    })
    .unwrap();
    assert!(bytes >= file.retained_items().capacity() * size_of::<FileItem>() + 9);
    let mut cloned = file.items()[0].clone();
    if let FileItem::Declaration(declaration) = &mut cloned {
        if let FileDeclarationKind::Procedure(procedure) = &mut declaration.kind {
            procedure.body.reserve(100);
        }
    }
    let mut clone_bytes = 0;
    cloned
        .visit_retained_metadata(&mut |_, value| {
            clone_bytes += value;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert!(
        clone_bytes > bytes,
        "a distinct cloned AST owns its actual spare statement capacity"
    );
}

#[test]
fn file_container_denial_stops_before_traversing_any_item() {
    let mut sources = jai_source::SourceMap::default();
    let id = sources.insert("empty.jai".into(), String::new());
    let file = ParsedFile::from_items(id, Vec::with_capacity(100));
    let mut calls = 0;
    assert_eq!(
        file.visit_retained_metadata(&mut |_, _| {
            calls += 1;
            Err("denied")
        }),
        Err(SourceMetadataError::Admission("denied"))
    );
    assert_eq!(calls, 1);
}

#[test]
fn source_metadata_nesting_has_a_typed_boundary() {
    let mut expression = Expression {
        kind: ExpressionKind::Bool(true),
        span: Span::default(),
    };
    for _ in 0..=MAX_SOURCE_METADATA_DEPTH {
        expression = Expression {
            kind: ExpressionKind::AddressOf(Box::new(expression)),
            span: Span::default(),
        };
    }
    assert_eq!(
        expression.visit_retained_metadata(&mut |_, _| Ok::<_, ()>(())),
        Err(SourceMetadataError::ExcessiveNesting)
    );
}

#[test]
fn baked_source_census_includes_callee_and_spare_argument_capacity() {
    let mut sources = jai_source::SourceMap::default();
    let id = sources.insert(
        "baked-metadata.jai".into(),
        "value :: #bake_arguments target(data = \"retained\");".into(),
    );
    let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
    let mut item = file.items()[0].clone();
    let measure = |item: &FileItem| {
        let mut bytes = 0;
        item.visit_retained_metadata(&mut |_, amount| {
            bytes += amount;
            Ok::<_, ()>(())
        })
        .unwrap();
        bytes
    };
    let before = measure(&item);
    let FileItem::Declaration(declaration) = &mut item else {
        panic!("actual constant declaration")
    };
    let FileDeclarationKind::Constant(constant) = &mut declaration.kind else {
        panic!("actual source constant")
    };
    let ExpressionKind::BakeArguments(baked) = &mut constant.initializer.kind else {
        panic!("actual bake expression")
    };
    baked.arguments.reserve(80);
    assert!(
        measure(&item) > before,
        "borrowed bake census must retain spare argument backing"
    );
    assert!(matches!(
        item.visit_retained_metadata(&mut |_, _| Err::<(), _>("denied")),
        Err(SourceMetadataError::Admission("denied"))
    ));
}
