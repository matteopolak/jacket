//! Measure pure exact aliases, with the compiler benchmark's real allocator profiler.
use divan::{Bencher, counter::ItemsCount};
use jai_eval::{Value, evaluate_paths};
use jai_source::{Diagnostic, SourceMap, Span, Symbols};
use jai_syntax::{Expression, FileDeclarationKind, FileItem, NamePath};
use jai_types::{FloatType, FloatValue};

fn expression(text: &str) -> Expression {
    let mut sources = SourceMap::default();
    let source = sources.insert("float-alias-bench.jai".into(), format!("VALUE :: {text};"));
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!("expected a benchmark constant");
    };
    let FileDeclarationKind::Constant(constant) = &declaration.kind else {
        panic!("expected a benchmark initializer");
    };
    constant.initializer.clone()
}

fn no_names(_: &NamePath, span: Span) -> Result<Value, Diagnostic> {
    Err(Diagnostic::new(span, "unexpected benchmark name"))
}

fn aliases(initial: &Expression, repeated: &Expression, count: usize) -> Value {
    let mut previous = evaluate_paths(initial, no_names).unwrap();
    for _ in 0..count {
        previous = evaluate_paths(repeated, |_, _| Ok(previous.clone())).unwrap();
    }
    previous
}

fn round(value: &Value) -> FloatValue {
    let Value::WeakFloat(value) = value else {
        panic!("alias unexpectedly lost its exact weak representation");
    };
    value.round(FloatType::F64, Span::default()).unwrap()
}

#[divan::bench(args = [4, 64, 128])]
fn weak_float_alias_build_and_round(bencher: Bencher, count: usize) {
    let initial = expression("0.125");
    let repeated = expression("PREVIOUS + PREVIOUS");
    let expected = FloatValue::from_f64(0.125 * 2.0_f64.powi(count as i32));
    assert_eq!(round(&aliases(&initial, &repeated, count)), expected);
    bencher.counter(ItemsCount::new(count)).bench_local(|| {
        let value = aliases(divan::black_box(&initial), &repeated, count);
        assert_eq!(round(&value), expected);
        value
    });
}

#[divan::bench(args = [4, 64, 128])]
fn weak_float_alias_successful_round_reuse(bencher: Bencher, count: usize) {
    let value = aliases(
        &expression("0.125"),
        &expression("PREVIOUS + PREVIOUS"),
        count,
    );
    let expected = FloatValue::from_f64(0.125 * 2.0_f64.powi(count as i32));
    assert_eq!(round(&value), expected);
    bencher.counter(ItemsCount::new(1usize)).bench_local(|| {
        let result = round(divan::black_box(&value));
        assert_eq!(result, expected);
        result
    });
}
