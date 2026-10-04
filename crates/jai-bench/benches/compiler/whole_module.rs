//! Isolate stages using the shared mixed-feature source module.
use super::mixed_module::{Fixture, resolve, target};
use divan::{Bencher, counter::BytesCount};
use jai_codegen::optimization::BitcodeOptimization;

#[divan::bench(args = [4, 64, 1024])]
fn mixed_lex(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    jai_lexer::lex(&fixture.source).unwrap();
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(|| jai_lexer::lex(divan::black_box(&fixture.source)).unwrap());
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_parse(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    jai_syntax::parse(&fixture.source).unwrap();
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(|| jai_syntax::parse(divan::black_box(&fixture.source)).unwrap());
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_graph_load(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    assert_eq!(fixture.graph().files().len(), 1);
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(|| fixture.graph());
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_resolve(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    let target = target(BitcodeOptimization::O0);
    fixture.program(&target);
    let graph = fixture.graph();
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(|| resolve(divan::black_box(&graph), &target));
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_ir_verify(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    let program = fixture.program(&target(BitcodeOptimization::O0));
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(|| {
            for procedure in program.procedures() {
                divan::black_box(
                    jai_ir::verify_procedure_with_context(
                        program.types(),
                        procedure,
                        program.signatures(),
                        program.globals(),
                        program.places(),
                        program.context(),
                    )
                    .unwrap(),
                );
            }
        });
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_lower_llvm(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    let target = target(BitcodeOptimization::O0);
    let program = fixture.program(&target);
    let context = jai_codegen::Context::create();
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(|| jai_codegen::lower_for_target(&context, &program, &target).unwrap());
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_source_to_llvm(bencher: Bencher, count: usize) {
    let fixture = Fixture::new(count);
    let target = target(BitcodeOptimization::O0);
    fixture.program(&target);
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(|| {
            let graph = fixture.graph();
            let program = resolve(&graph, &target);
            let context = jai_codegen::Context::create();
            jai_codegen::lower_for_target(&context, &program, &target)
                .unwrap()
                .print_to_string()
                .to_string()
        });
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_optimize_o0(bencher: Bencher, count: usize) {
    optimize(bencher, count, BitcodeOptimization::O0);
}
#[divan::bench(args = [4, 64, 1024])]
fn mixed_optimize_o2(bencher: Bencher, count: usize) {
    optimize(bencher, count, BitcodeOptimization::O2);
}
fn optimize(bencher: Bencher, count: usize, level: BitcodeOptimization) {
    let fixture = Fixture::new(count);
    let target = target(level);
    let program = fixture.program(&target);
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    target.prepare(&module.clone()).unwrap();
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .with_inputs(|| module.clone())
        .bench_local_values(|module| {
            target.prepare(&module).unwrap();
            module
        });
}

#[divan::bench(args = [4, 64, 1024])]
fn mixed_object_o0(bencher: Bencher, count: usize) {
    object(bencher, count, BitcodeOptimization::O0);
}
#[divan::bench(args = [4, 64, 1024])]
fn mixed_object_o2(bencher: Bencher, count: usize) {
    object(bencher, count, BitcodeOptimization::O2);
}
fn object(bencher: Bencher, count: usize, level: BitcodeOptimization) {
    let fixture = Fixture::new(count);
    let target = target(level);
    let program = fixture.program(&target);
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
    target.prepare(&module).unwrap();
    let emit = || {
        let bytes = target
            .machine
            .write_to_memory_buffer(&module, inkwell::targets::FileType::Object)
            .unwrap();
        assert!(!bytes.as_slice().is_empty());
        bytes
    };
    emit();
    bencher
        .counter(BytesCount::new(fixture.source.len()))
        .bench_local(emit);
}
