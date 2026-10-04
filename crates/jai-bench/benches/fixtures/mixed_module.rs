//! Authored mixed-feature workloads, shared by stage and native runtime cases.
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
        let mut vm = jai_vm::Vm::new_with_execution_phase(
            &program,
            jai_vm::NoEffects,
            jai_vm::Limits {
                fuel: 64_000_000,
                ..Default::default()
            },
            jai_vm::ByteTarget::from(&target.build_target().unwrap()),
            jai_vm::ExecutionPhase::Runtime,
        )
        .unwrap();
        let jai_ir::EntryPoint::Int(entry) = program.entry() else {
            panic!("mixed module must have integer entry");
        };
        let result = vm.execute(entry, vec![]);
        assert!(
            matches!(result.outcome, jai_vm::Outcome::Complete(values)
            if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == self.expected(3))),
            "mixed module must complete with its independently calculated checksum"
        );
        drop(vm);
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

pub(super) fn resolve(graph: &ModuleGraph, target: &NativeTarget) -> jai_ir::Program {
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
