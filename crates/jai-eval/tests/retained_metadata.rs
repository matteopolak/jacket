use jai_eval::floats::{WeakFloatAdmissionError, WeakFloatEncodingError};
use jai_eval::{EvalRetainedMetadataError, Value, evaluate};
use jai_source::Symbols;

fn weak(text: &str) -> Value {
    let mut symbols = Symbols::default();
    let source = format!("X :: {text};");
    let mut sources = jai_source::SourceMap::default();
    let id = sources.insert("retained.jai".into(), source);
    let file = jai_syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
    let jai_syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!()
    };
    let jai_syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!()
    };
    evaluate(&constant.initializer, |_, span| {
        Err(jai_source::Diagnostic::new(span, "unknown"))
    })
    .unwrap()
}

#[test]
fn metered_exact_encoding_preserves_existing_bytes_and_limits() {
    let Value::WeakFloat(value) = weak("1.0000000596046448") else {
        panic!()
    };
    let key = value.request_key();
    let bytes = key.canonical_bytes(10, 100).unwrap();
    let mut work = 0usize;
    let mut backing = 0usize;
    let metered = key
        .canonical_bytes_with_work(10, 100, &mut |w, b| {
            work += w;
            backing += b;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(bytes, metered);
    assert!(work > bytes.len() && backing >= bytes.len());
    assert_eq!(
        key.canonical_bytes_with_work(1, 100, &mut |_, _| Ok::<_, ()>(())),
        Err(WeakFloatAdmissionError::Encoding(
            WeakFloatEncodingError::NodeLimit
        ))
    );
}

#[test]
fn denied_metadata_admission_precedes_any_owned_key_copy() {
    let Value::WeakFloat(value) = weak("1.0") else {
        panic!()
    };
    let mut calls = 0;
    let result = value
        .request_key()
        .canonical_bytes_with_work(100, 1000, &mut |_, _| {
            calls += 1;
            Err("denied")
        });
    assert_eq!(result, Err(WeakFloatAdmissionError::Admission("denied")));
    assert_eq!(calls, 1);
    assert!(value.request_key().canonical_bytes(100, 1000).is_ok());
}

#[test]
fn borrowed_value_and_independent_key_are_actual_distinct_roots() {
    let value = weak("1.0000000596046448 + 2.0");
    let Value::WeakFloat(float) = &value else {
        panic!()
    };
    let mut key_bytes = 0;
    float
        .request_key()
        .visit_retained_metadata(&mut |_, b| {
            key_bytes += b;
            Ok::<_, ()>(())
        })
        .unwrap();
    let mut value_bytes = 0;
    value
        .visit_retained_metadata(&mut |_, b| {
            value_bytes += b;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert!(value_bytes > key_bytes && key_bytes > 0);
    assert_eq!(
        value.visit_retained_metadata(&mut |_, _| Err("stop")),
        Err(EvalRetainedMetadataError::Admission("stop"))
    );
}

#[test]
fn primitive_scalar_has_no_external_backing() {
    let mut bytes = 0;
    Value::Bool(true)
        .visit_retained_metadata(&mut |_, b| {
            bytes += b;
            Ok::<_, ()>(())
        })
        .unwrap();
    assert_eq!(bytes, 0);
}
