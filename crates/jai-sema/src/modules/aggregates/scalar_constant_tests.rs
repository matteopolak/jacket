use super::*;

fn decimal() -> ConstantValue {
    let mut sources = jai_source::SourceMap::default();
    let source = sources.insert("constant.jai".into(), "VALUE :: 1.0000000596046448;".into());
    let mut symbols = jai_source::Symbols::default();
    let file = syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
    let syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!("expected declaration")
    };
    let syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!("expected constant")
    };
    jai_eval::evaluate_paths(&constant.initializer, |_, span| {
        Err(Diagnostic::new(span, "unexpected name"))
    })
    .unwrap()
}

#[test]
fn cached_decimal_materializes_directly_at_each_requested_width() {
    let types = TypeRegistry::new();
    let value = decimal();
    let narrow = scalar_constant(
        types.float(jai_types::FloatType::F32),
        value.clone(),
        &types,
        Span::default(),
    )
    .unwrap();
    let wide = scalar_constant(
        types.float(jai_types::FloatType::F64),
        value,
        &types,
        Span::default(),
    )
    .unwrap();
    assert_eq!(
        narrow.kind,
        jai_ir::ConstantKind::Float(jai_types::FloatValue::from_f32(f32::from_bits(
            1.0_f32.to_bits() + 1
        )))
    );
    let jai_ir::ConstantKind::Float(jai_types::FloatValue::F64(wide)) = wide.kind else {
        panic!("expected f64")
    };
    assert_ne!(
        narrow.kind,
        jai_ir::ConstantKind::Float(jai_types::FloatValue::from_f32(f64::from_bits(wide) as f32))
    );
}

#[test]
fn decimal_integer_target_produces_source_aware_diagnostic() {
    let types = TypeRegistry::new();
    let span = Span {
        start: 11,
        end: 29,
    };
    let error =
        scalar_constant(types.scalar(ScalarType::Bool), decimal(), &types, span).unwrap_err();
    assert_eq!(error.span, span);
    assert!(error.message.contains("requires a float type"));
}
