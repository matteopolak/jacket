//! Authored mixed-feature workloads, shared by stage and native runtime cases.
use divan::{Bencher, counter::BytesCount};
use jai_codegen::{
    optimization::{BitcodeOptimization, Optimization},
    target::{NativeTarget, TargetOptions},
};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use std::{fmt::Write, path::Path};

pub(super) struct Fixture {
    pub source: String,
    pub inputs: SourceOverlay,
    pub count: usize,
}

impl Fixture {
    pub fn new(count: usize) -> Self {
        let mut source =
            String::from("Pair::struct {left:int;right:int;} Mode::enum {ACTIVE;PASSIVE;}\n");
        for n in 0..count {
            let mode = if n % 2 == 0 {
                "ACTIVE"
            } else {
                "PASSIVE"
            };
            writeln!(source, "handler{n}::(seed:int)->int #c_call {{pair:Pair;pair.left=seed+{n};pair.right=3;values:[4]int;values[0]=pair.left;values[1]=pair.right;values[2]=2;values[3]=4;sum:=0;for i:0..3 sum+=values[i];mode:=Mode.{mode};if mode == .ACTIVE sum+=7;else sum-=2;return sum;}}").unwrap();
        }
        source.push_str(
            "#program_export \"bench_compute\" compute::(seed:int)->int #c_call {sum:=0;\n",
        );
        for n in 0..count {
            writeln!(source, "sum+=handler{n}(seed);").unwrap();
        }
        source.push_str("return sum;} main::()->int {return compute(3);}\n");
        let mut inputs = SourceOverlay::default();
        inputs
            .insert(
                Path::new("/bench/mixed/main.jai"),
                source.as_bytes().to_vec(),
            )
            .unwrap();
        Self {
            source,
            inputs,
            count,
        }
    }

    pub fn graph(&self) -> ModuleGraph {
        ModuleGraph::load_with_provider(
            Path::new("/bench/mixed/main.jai"),
            GraphOptions::default(),
            &self.inputs,
        )
        .unwrap()
    }

    pub fn expected(&self, seed: i128) -> i128 {
        let count = self.count as i128;
        count * seed + count * (count - 1) / 2 + ((count + 1) / 2) * 16 + (count / 2) * 7
    }

    pub fn program(&self, target: &NativeTarget) -> jai_ir::Program {
        let program = resolve(&self.graph(), target);
        let result = jai_vm::execute(
            &program,
            jai_vm::Limits {
                fuel: 64_000_000,
                ..Default::default()
            },
        );
        assert!(
            matches!(result.outcome, jai_vm::Outcome::Complete(values)
            if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == self.expected(3))),
            "mixed module must complete with its independently calculated checksum"
        );
        program
    }
}

pub(super) fn target(level: BitcodeOptimization) -> NativeTarget {
    NativeTarget::select(&TargetOptions {
        optimization: Optimization {
            bitcode: level,
            ..Default::default()
        },
        ..Default::default()
    })
    .unwrap()
}

fn resolve(graph: &ModuleGraph, target: &NativeTarget) -> jai_ir::Program {
    jai_sema::resolve_graph_with_options(
        graph,
        &jai_sema::ResolveOptions {
            layout: Some(target.layout_policy().unwrap()),
            target: Some(target.build_target().unwrap()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap()
}

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
