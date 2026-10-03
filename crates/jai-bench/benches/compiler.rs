use divan::{Bencher, counter::BytesCount};
#[path = "compiler/sequence_indices.rs"]
mod sequence_indices;
use std::{fs, path::Path};

#[path = "compiler/float_aliases.rs"]
mod float_aliases;

#[path = "compiler/native_runtime.rs"]
mod native_runtime;
#[path = "compiler/whole_module.rs"]
mod whole_module;

#[global_allocator]
static ALLOC: divan::AllocProfiler = divan::AllocProfiler::system();

fn main() {
    divan::main();
}

#[divan::bench(args = [4, 64, 1024])]
fn source_run_resolve(bencher: Bencher, iterations: usize) {
    let source = format!(
        "main::()->int {{return #run sum({iterations});}} sum::(n:int)->int {{result:=0; for i:1..n result+=i; return result;}}"
    );
    let mut inputs = jai_modules::SourceOverlay::default();
    let path = Path::new("/bench/main.jai");
    inputs.insert(path, source.as_bytes().to_vec()).unwrap();
    let graph = jai_modules::ModuleGraph::load_with_provider(
        path,
        jai_modules::GraphOptions::default(),
        &inputs,
    )
    .unwrap();
    let options = jai_sema::ResolveOptions {
        layout: Some(jai_types::LayoutPolicy::lp64()),
        ..Default::default()
    };
    let program =
        jai_sema::resolve_graph_with_options(&graph, &options, &mut jai_vm::NoEffects).unwrap();
    let expected = iterations as i128 * (iterations as i128 + 1) / 2;
    assert!(
        matches!(jai_vm::execute(&program, jai_vm::Limits::default()).outcome, jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == expected))
    );
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| {
            jai_sema::resolve_graph_with_options(
                divan::black_box(&graph),
                &options,
                &mut jai_vm::NoEffects,
            )
            .unwrap()
        });
}

fn structural_types(registry: &mut jai_types::TypeRegistry, count: usize) -> jai_types::TypeId {
    let mut ty = registry.scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U64));
    for n in 0..count {
        let pointer = registry.pointer(ty).unwrap();
        let array = registry.fixed_array(pointer, n as u64 + 1).unwrap();
        let slice = registry.slice(array).unwrap();
        ty = registry.dynamic_array(slice).unwrap();
    }
    ty
}

#[divan::bench(args = [4, 64, 1024])]
fn type_intern_create(bencher: Bencher, count: usize) {
    let mut check = jai_types::TypeRegistry::new();
    let root = structural_types(&mut check, count);
    assert_eq!(root, structural_types(&mut check, count));
    assert_eq!(
        check.freeze().unwrap().iter().count(),
        jai_types::TypeRegistry::new()
            .freeze()
            .unwrap()
            .iter()
            .count()
            + 4 * count
    );
    bencher.bench_local(|| {
        let mut registry = jai_types::TypeRegistry::new();
        divan::black_box(structural_types(&mut registry, count));
        registry
    });
}

#[divan::bench(args = [4, 64, 1024])]
fn type_intern_reuse(bencher: Bencher, count: usize) {
    let mut registry = jai_types::TypeRegistry::new();
    let root = structural_types(&mut registry, count);
    assert_eq!(root, structural_types(&mut registry, count));
    bencher.bench_local(|| structural_types(&mut registry, divan::black_box(count)));
}

fn nominal_graph(count: usize) -> (jai_types::TypeRegistry, jai_types::TypeId) {
    use jai_types::{IntegerType, RecordKind, ScalarType, TypeRegistry};
    let mut registry = TypeRegistry::new();
    let mut root = registry.scalar(ScalarType::Int(IntegerType::U64));
    for _ in 0..count {
        let record = registry.reserve_record(RecordKind::Struct);
        let pointer = registry.pointer(record).unwrap();
        let slice = registry.slice(record).unwrap();
        let dynamic = registry.dynamic_array(record).unwrap();
        registry
            .define_record(record, [root, pointer, slice, dynamic])
            .unwrap();
        root = record;
    }
    (registry, root)
}

fn checked_nominal_graph(count: usize) -> (jai_types::Types, jai_types::TypeId) {
    let (registry, root) = nominal_graph(count);
    let types = registry.freeze().unwrap();
    assert_eq!(
        types.iter().count(),
        jai_types::TypeRegistry::new()
            .freeze()
            .unwrap()
            .iter()
            .count()
            + 4 * count
    );
    let mut engine = jai_types::LayoutEngine::new(&types, jai_types::LayoutPolicy::lp64());
    let layout = engine.layout(root).unwrap();
    assert_eq!(layout.size, 8 + 64 * count as u64);
    assert_eq!(layout.alignment, 8);
    assert_eq!(
        layout.field_offsets.as_ref(),
        &[
            0,
            8 + 64 * (count as u64 - 1),
            16 + 64 * (count as u64 - 1),
            32 + 64 * (count as u64 - 1)
        ]
    );
    (types, root)
}

#[divan::bench(args = [4, 64, 1024])]
fn nominal_graph_build_freeze(bencher: Bencher, count: usize) {
    checked_nominal_graph(count);
    bencher.bench_local(|| nominal_graph(divan::black_box(count)).0.freeze().unwrap());
}

#[divan::bench(args = [4, 64, 1024])]
fn type_layout_cold(bencher: Bencher, count: usize) {
    let (types, root) = checked_nominal_graph(count);
    bencher.bench_local(|| {
        let mut engine = jai_types::LayoutEngine::new(&types, jai_types::LayoutPolicy::lp64());
        divan::black_box(engine.layout(root).unwrap());
        engine
    });
}

#[divan::bench(args = [4, 64, 1024])]
fn type_layout_cached(bencher: Bencher, count: usize) {
    let (types, root) = checked_nominal_graph(count);
    let mut engine = jai_types::LayoutEngine::new(&types, jai_types::LayoutPolicy::lp64());
    engine.layout(root).unwrap();
    bencher.bench_local(|| {
        divan::black_box(engine.layout(divan::black_box(root)).unwrap());
    });
}
fn generated(procedures: usize) -> String {
    let mut source = String::new();
    for n in 0..procedures {
        use std::fmt::Write;
        writeln!(source, "f{n} :: (n:int, enabled:bool)->int {{ sum := 0; while n > 0 && enabled {{ sum = sum + n; n = n - 1; }} return sum; }}").unwrap();
    }
    source.push_str("main :: ()->int { return f0(9, true); }");
    source
}
#[divan::bench(args = [4, 64, 1024])]
fn lex(bencher: Bencher, procedures: usize) {
    let source = generated(procedures);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_lexer::lex(divan::black_box(&source)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn parse(bencher: Bencher, procedures: usize) {
    let source = generated(procedures);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_syntax::parse(divan::black_box(&source)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn resolve(bencher: Bencher, procedures: usize) {
    let source = generated(procedures);
    let module = jai_syntax::parse(&source).unwrap();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_sema::resolve(divan::black_box(&module)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn lower_llvm(bencher: Bencher, procedures: usize) {
    let source = generated(procedures);
    let module = jai_syntax::parse(&source).unwrap();
    let program = jai_sema::resolve(&module).unwrap();
    let context = jai_codegen::Context::create();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_codegen::lower(&context, divan::black_box(&program)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn emit_llvm(bencher: Bencher, procedures: usize) {
    let source = generated(procedures);
    let module = jai_syntax::parse(&source).unwrap();
    let program = jai_sema::resolve(&module).unwrap();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_codegen::emit(divan::black_box(&program)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn pipeline(bencher: Bencher, procedures: usize) {
    let source = generated(procedures);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| {
            let module = jai_syntax::parse(divan::black_box(&source)).unwrap();
            let program = jai_sema::resolve(&module).unwrap();
            jai_codegen::emit(&program).unwrap()
        });
}
fn range_source(procedures: usize) -> String {
    let mut source = String::new();
    for n in 0..procedures {
        use std::fmt::Write;
        writeln!(source, "f{n} :: (n:int)->int {{ sum := 0; for outer: 1..n {{ for < inner: 1..4 {{ if inner == 2 continue outer; sum += inner; if sum > 100 break outer; }} }} return sum; }}").unwrap();
    }
    source.push_str("main :: ()->int { return f0(9); }");
    source
}
#[divan::bench(args = [4, 64, 1024])]
fn range_lower_llvm(bencher: Bencher, procedures: usize) {
    let source = range_source(procedures);
    let module = jai_syntax::parse(&source).unwrap();
    let program = jai_sema::resolve(&module).unwrap();
    let context = jai_codegen::Context::create();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_codegen::lower(&context, divan::black_box(&program)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn range_pipeline(bencher: Bencher, procedures: usize) {
    let source = range_source(procedures);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| {
            let module = jai_syntax::parse(divan::black_box(&source)).unwrap();
            let program = jai_sema::resolve(&module).unwrap();
            jai_codegen::emit(&program).unwrap()
        });
}
fn cleanup_source(procedures: usize) -> String {
    let mut source = String::new();
    for n in 0..procedures {
        use std::fmt::Write;
        writeln!(source, "f{n} :: (n:int)->int {{ sum := 0; for outer: 1..n {{ defer sum += 1; for < inner: 1..4 {{ defer sum += 2; if inner == 2 continue outer; sum += inner; if sum > 100 break outer; }} }} return sum; }}").unwrap();
    }
    source.push_str("main :: ()->int { return f0(9); }");
    source
}
#[divan::bench(args = [4, 64, 1024])]
fn cleanup_lower_llvm(bencher: Bencher, procedures: usize) {
    let source = cleanup_source(procedures);
    let module = jai_syntax::parse(&source).unwrap();
    let program = jai_sema::resolve(&module).unwrap();
    let context = jai_codegen::Context::create();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_codegen::lower(&context, divan::black_box(&program)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn cleanup_pipeline(bencher: Bencher, procedures: usize) {
    let source = cleanup_source(procedures);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| {
            let module = jai_syntax::parse(divan::black_box(&source)).unwrap();
            let program = jai_sema::resolve(&module).unwrap();
            jai_codegen::emit(&program).unwrap()
        });
}
fn constants_source(count: usize) -> String {
    use std::fmt::Write;
    let mut source = String::new();
    for n in 0..count {
        writeln!(source, "C{n} :: C{} + 1;", n + 1).unwrap();
    }
    writeln!(
        source,
        "C{count} :: 0; global := C0; main :: ()->int {{ global += 1; return global; }}"
    )
    .unwrap();
    source
}
#[divan::bench(args = [4, 64, 1024])]
fn constants_resolve(bencher: Bencher, count: usize) {
    let source = constants_source(count);
    let module = jai_syntax::parse(&source).unwrap();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_sema::resolve(divan::black_box(&module)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn constants_pipeline(bencher: Bencher, count: usize) {
    let source = constants_source(count);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| {
            let module = jai_syntax::parse(divan::black_box(&source)).unwrap();
            let program = jai_sema::resolve(&module).unwrap();
            jai_codegen::emit(&program).unwrap()
        });
}
fn conditional_source(procedures: usize) -> String {
    use std::fmt::Write;
    let mut source = String::new();
    for n in 0..procedures {
        writeln!(source, "f{n} :: (n:int, enabled:bool)->int {{ active := ifx n > 0 then enabled && n < 100 else false; return ifx active then (ifx n < 5 n + 1 else n * 2) else 0; }}").unwrap();
    }
    source.push_str("main :: ()->int { return f0(9, true); }");
    source
}
#[divan::bench(args = [4, 64, 1024])]
fn conditional_lower_llvm(bencher: Bencher, procedures: usize) {
    let source = conditional_source(procedures);
    let module = jai_syntax::parse(&source).unwrap();
    let program = jai_sema::resolve(&module).unwrap();
    let context = jai_codegen::Context::create();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_codegen::lower(&context, divan::black_box(&program)).unwrap());
}
#[divan::bench(args = [4, 64, 1024])]
fn conditional_pipeline(bencher: Bencher, procedures: usize) {
    let source = conditional_source(procedures);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| {
            let module = jai_syntax::parse(divan::black_box(&source)).unwrap();
            let program = jai_sema::resolve(&module).unwrap();
            jai_codegen::emit(&program).unwrap()
        });
}
fn integer_source(procedures: usize) -> String {
    use std::fmt::Write;
    let mut source = String::new();
    for n in 0..procedures {
        writeln!(source,"f{n} :: (value:u64, enabled:bool=true)->int {{ small:=cast(u8)(value%200); if #complete enabled == {{ case true; return ifx small>100 then cast(int)small else cast(int)small+1; case false; return cast(int)cast,no_check(s8)small; }} }}").unwrap();
    }
    source.push_str("main :: ()->int { return f0(enabled=false,value=42); }");
    source
}
#[divan::bench(args=[4,64,1024])]
fn integer_lower_llvm(bencher: Bencher, procedures: usize) {
    let source = integer_source(procedures);
    let module = jai_syntax::parse(&source).unwrap();
    let program = jai_sema::resolve(&module).unwrap();
    let context = jai_codegen::Context::create();
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| jai_codegen::lower(&context, divan::black_box(&program)).unwrap());
}
#[divan::bench(args=[4,64,1024])]
fn integer_pipeline(bencher: Bencher, procedures: usize) {
    let source = integer_source(procedures);
    bencher
        .counter(BytesCount::new(source.len()))
        .bench_local(|| {
            let module = jai_syntax::parse(divan::black_box(&source)).unwrap();
            let program = jai_sema::resolve(&module).unwrap();
            jai_codegen::emit(&program).unwrap()
        });
}
fn corpus(path: &Path, out: &mut Vec<String>) {
    if !path.exists() {
        return;
    }
    let mut entries = fs::read_dir(path)
        .unwrap()
        .map(|e| e.unwrap())
        .collect::<Vec<_>>();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            corpus(&path, out);
        } else if kind.is_file() && path.extension().is_some_and(|e| e == "jai") {
            let bytes = fs::read(&path).unwrap();
            out.push(jai_lexer::decode_source(&bytes).unwrap().into_owned());
        }
    }
}
fn lex_corpus(bencher: Bencher, path: &Path) {
    let mut sources = Vec::new();
    corpus(path, &mut sources);
    assert!(
        !sources.is_empty(),
        "no corpus sources at {}",
        path.display()
    );
    let bytes = sources.iter().map(String::len).sum::<usize>();
    bencher.counter(BytesCount::new(bytes)).bench_local(|| {
        for source in &sources {
            divan::black_box(jai_lexer::lex(divan::black_box(source)).unwrap());
        }
    });
}
#[divan::bench(sample_count = 20, sample_size = 1)]
fn reference_lex(bencher: Bencher) {
    lex_corpus(bencher, &corpus_root().join("reference"));
}
#[divan::bench(sample_count = 20, sample_size = 1, ignore)]
fn upstream_lex(bencher: Bencher) {
    lex_corpus(bencher, &corpus_root().join("corpus/upstream"));
}

fn corpus_root() -> std::path::PathBuf {
    std::env::var_os("JAI_BENCH_CORPUS_ROOT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
}
