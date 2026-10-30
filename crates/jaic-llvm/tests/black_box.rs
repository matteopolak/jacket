//! `black_box` survives optimization: no folding through it, and no deleting what feeds it.
use jaic::sema::{Compiler, Options, ProgramSource, TargetCpu, TargetOs, VirtualFs};
use jaic_llvm::{Codegen, OptLevel, emit_object};
use std::rc::Rc;
use test_support::TempDir;
use test_support::ir::body as function;

const SOURCE: &str = r#"
black_box :: (value: $T) -> T #intrinsic;

Triple :: struct { a: s64; b: s64; c: s64; }

#program_export "known" known :: () -> s64 #c_call { return 5 * 2; }
#program_export "hidden" hidden :: () -> s64 #c_call { return black_box(5) * 2; }

#program_export "sum_known" sum_known :: () -> s64 #c_call {
    total := 0;
    for 0..99 total += it;
    return total;
}

#program_export "sum_hidden" sum_hidden :: () -> s64 #c_call {
    total := 0;
    for 0..99 total += black_box(it);
    return total;
}

#program_export "sum_input" sum_input :: (n: s64) -> s64 #c_call {
    n = black_box(n);
    total := 0;
    for 0..n - 1 total += it;
    return total;
}

#program_export "real" real :: (x: float64) -> float64 #c_call { return black_box(x) + 1.5; }

#program_export "aggregate" aggregate :: (t: *Triple) -> s64 #c_call {
    copy := black_box(t.*);
    return copy.a + copy.c;
}
"#;

fn optimized_ir(
    name: &str,
    os: TargetOs,
    cpu: TargetCpu,
    triple: Option<jaic::target::Target>,
) -> String {
    let dir = TempDir::new(name);

    let mut options = Options::host();
    options.os = os;
    options.cpu = cpu;
    options.runtime_support = false;
    options.stack_trace = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    if let Err(e) = compiler.compile_sources(&[ProgramSource::String(SOURCE.into())]) {
        panic!("{}", e.render(&compiler.src.sources));
    }

    let options = jaic_llvm::Options {
        target: triple,
        opt_level: OptLevel::O2,
        codegen: Codegen {
            single_unit: true,
            ir_after: Some(dir.join("optimized.ll")),
            ..Codegen::default()
        },
        ..Default::default()
    };
    emit_object(&compiler.out.program, &options, &dir.join("black_box.o")).unwrap();

    std::fs::read_to_string(dir.join("optimized.ll")).unwrap()
}

/// The text of the function `name` in an LLVM IR module.
fn check(ir: &str) {
    // Without `black_box` the optimizer knows both answers.
    assert!(function(ir, "known").contains("ret i64 10"), "{ir}");
    assert!(function(ir, "sum_known").contains("ret i64 4950"), "{ir}");

    // With it: the 5 is unknown, so the product is computed, and the loop is kept.
    let hidden = function(ir, "hidden");
    assert!(!hidden.contains("ret i64 10"), "{hidden}");
    assert!(hidden.contains("asm sideeffect \"\""), "{hidden}");

    let sum = function(ir, "sum_hidden");
    assert!(!sum.contains("ret i64 4950"), "{sum}");
    assert!(sum.contains("phi"), "the loop was deleted:\n{sum}");

    let input = function(ir, "sum_input");
    assert!(input.contains("asm sideeffect"), "{input}");

    // The assembly is empty and takes no call: nothing but the block itself is added.
    for name in ["hidden", "sum_hidden", "sum_input", "real", "aggregate"] {
        let body = function(ir, name);
        for line in body.lines().filter(|l| l.contains("call ")) {
            let allowed = line.contains("asm sideeffect")
                || line.contains("@llvm.")
                || line.contains("@jaic.null_fail");
            assert!(allowed, "{name} calls something: {line}\n{body}");
        }
    }

    assert!(function(ir, "real").contains("asm sideeffect"), "{ir}");
    assert!(function(ir, "aggregate").contains("asm sideeffect"), "{ir}");
}

#[test]
fn host_keeps_the_value_unknown_and_the_loop_alive() {
    let host = Options::host();
    check(&optimized_ir("host", host.os, host.cpu, None));
}

#[test]
fn x64_linux_keeps_the_value_unknown_and_the_loop_alive() {
    let target = jaic::target::Target::parse("x86_64-unknown-linux-gnu").unwrap();
    check(&optimized_ir(
        "x64",
        TargetOs::Linux,
        TargetCpu::X64,
        Some(target),
    ));
}

#[test]
fn arm64_linux_keeps_the_value_unknown_and_the_loop_alive() {
    let target = jaic::target::Target::parse("aarch64-unknown-linux-gnu").unwrap();
    check(&optimized_ir(
        "arm64",
        TargetOs::Linux,
        TargetCpu::Arm64,
        Some(target),
    ));
}

#[test]
fn wasm_keeps_the_value_unknown_and_the_loop_alive() {
    check(&optimized_ir(
        "wasm",
        TargetOs::Wasm,
        TargetCpu::Wasm,
        Some(jaic::target::Target::wasm_module()),
    ));
}
