//! Independently authored source to checked SIMD, portable VM and new x86 objects.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{Features, NativeTarget, TargetOptions, TargetSelection, Triple},
};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::BitcodeOptimization;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

const FLOAT_X: &str = r#"
main :: () -> int {
    array: [4] float32;
    for i: 0..3 { array[i] = cast(float32) (i + 1); }
    ptr := *array[0];
    #asm { v: vec; movups.x v, [ptr]; addps.x v, v; movups.x [ptr], v; }
    for i: 0..3 { if array[i] != cast(float32) (2 * (i + 1)) return 1; }
    return 42;
}
"#;

const FLOAT_Y: &str = r#"
main :: () -> int {
    array: [8] float32;
    for i: 0..7 { array[i] = cast(float32) (i + 1); }
    ptr := *array[0];
    #asm AVX { v: vec; movups.y v, [ptr]; addps.y v, v, v; movups.y [ptr], v; }
    for i: 0..7 { if array[i] != cast(float32) (2 * (i + 1)) return 2; }
    return 42;
}
"#;

const BYTES_X: &str = r#"
main :: () -> int {
    a: [16] u8; b: [16] u8; c: [16] u8;
    for i: 0..15 { a[i] = cast(u8) (240 + i); b[i] = cast(u8) (i + 1); }
    pa := *a[0]; pb := *b[0]; pc := *c[0];
    #asm AVX, AVX2 {
        movdqu.x left:, [pa]; movdqu.x right:, [pb];
        paddb.x result:, left, right; movdqu.x [pc], result;
    }
    for i: 0..15 { if c[i] != cast(u8) ((241 + 2 * i) % 256) return 3; }
    return 42;
}
"#;

const OVERLAPPING_ADDRESS_CALLS: &str = r#"
hits: int;
address :: (p: *float32) -> *float32 { hits += 1; return p; }
main :: () -> int {
    array: [5] float32;
    for i: 0..4 { array[i] = cast(float32) (i + 1); }
    #asm {
        movups.x snapshot:, [address(*array[0])];
        movups.x [address(*array[1])], snapshot;
    }
    if hits != 2 return 4;
    if array[0] != 1.0 || array[1] != 1.0 return 5;
    for i: 2..4 { if array[i] != cast(float32) i return 6; }
    return 42;
}
"#;

const SOURCES: [&str; 4] = [FLOAT_X, FLOAT_Y, BYTES_X, OVERLAPPING_ADDRESS_CALLS];

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-simd-source-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn graph(&self, source: &str) -> jai_modules::ModuleGraph {
        let path = self.0.join("main.jai");
        fs::write(&path, source).unwrap();
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap()
    }
    fn program(&self, source: &str, target: &NativeTarget) -> jai_ir::Program {
        let graph = self.graph(source);
        resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                target: Some(target.build_target().unwrap()),
                layout: Some(target.layout_policy().unwrap()),
                ..ResolveOptions::default()
            },
            &mut NoEffects,
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn target(triple: &str, features: &str, optimization: BitcodeOptimization) -> NativeTarget {
    NativeTarget::select(&TargetOptions {
        selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
        features: Features::new(features).unwrap(),
        optimization: Optimization {
            bitcode: optimization,
            ..Optimization::default()
        },
        ..TargetOptions::default()
    })
    .unwrap()
}

fn assert_vm(program: &jai_ir::Program) {
    let outcome = jai_vm::execute(program, Limits::default()).outcome;
    assert!(
        matches!(&outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42)),
        "SIMD source VM outcome: {outcome:?}"
    );
}

#[test]
fn source_examples_check_every_lane_and_address_call_in_the_portable_vm() {
    let fixture = Fixture::new();
    // ARM resolution proves the portable VM does not inherit a native x86 restriction.
    let arm = target("aarch64-unknown-linux-gnu", "", BitcodeOptimization::O0);
    for source in SOURCES {
        assert_vm(&fixture.program(source, &arm));
    }
}

#[test]
fn source_blocks_emit_real_x86_64_objects_at_o0_and_o2() {
    let fixture = Fixture::new();
    let triple = if cfg!(target_os = "macos") {
        "x86_64-apple-darwin"
    } else {
        "x86_64-unknown-linux-gnu"
    };
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let native = target(triple, "+avx,+avx2", optimization);
        for (index, source) in SOURCES.into_iter().enumerate() {
            let program = fixture.program(source, &native);
            assert_vm(&program);
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &native).unwrap();
            module.verify().unwrap();
            if optimization == BitcodeOptimization::O0 {
                let llvm = module.print_to_string().to_string();
                let operation = [
                    "fadd <4 x float>",
                    "fadd <8 x float>",
                    "add <16 x i8>",
                    "load <4 x float>",
                ][index];
                assert!(
                    llvm.contains(operation),
                    "missing typed SIMD operation {operation}:\n{llvm}"
                );
                assert!(
                    llvm.contains("align 1"),
                    "unaligned SIMD memory operands must be explicit"
                );
            }
            let object = fixture.0.join(format!("simd-{optimization:?}-{index}.o"));
            native.write_object(&module, &object).unwrap();
            assert_x86_64_object(&object);
            execute_on_compatible_x86_host(&object, &fixture.0);
        }
    }
}

fn assert_x86_64_object(path: &Path) {
    let bytes = fs::read(path).unwrap();
    assert!(bytes.len() > 64, "native object must contain emitted code");
    if cfg!(target_os = "macos") {
        assert_eq!(&bytes[..4], &[0xcf, 0xfa, 0xed, 0xfe]);
        assert_eq!(
            u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
            0x0100_0007
        );
    } else {
        assert_eq!(&bytes[..4], b"\x7fELF");
        assert_eq!(bytes[4], 2); // ELFCLASS64
        assert_eq!(u16::from_le_bytes(bytes[18..20].try_into().unwrap()), 62);
    }
}

#[cfg(target_arch = "x86_64")]
fn execute_on_compatible_x86_host(object: &Path, directory: &Path) {
    if !std::arch::is_x86_feature_detected!("avx2") {
        return;
    }
    execute_host_object(object, directory);
}

fn execute_host_object(object: &Path, directory: &Path) {
    use std::{
        process::Command,
        time::{Duration, Instant},
    };
    let executable = directory.join("simd-program");
    let linked = native_tools::clang_command()
        .arg(object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let mut child = Command::new(executable).spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(status.code(), Some(42));
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generated SIMD executable timed out");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn execute_on_compatible_x86_host(_: &Path, _: &Path) {
}

#[test]
fn compile_time_simd_is_embedded_before_host_native_reachability() {
    let fixture = Fixture::new();
    let source = format!(
        "{} answer :: #run simd_helper(); main :: () -> int {{ return answer; }}",
        FLOAT_Y.replacen("main ::", "simd_helper ::", 1)
    );
    let host = NativeTarget::new().unwrap();
    let program = fixture.program(&source, &host);
    assert_vm(&program);
    let jai_ir::EntryPoint::Int(entry) = program.entry() else {
        panic!("expected integer entry")
    };
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &host).unwrap();
    for procedure in program.procedures() {
        assert_eq!(
            module
                .get_function(&format!("jai.p{}", procedure.id.index()))
                .is_some(),
            procedure.id == entry,
            "compile-time SIMD helper must be absent from host native code"
        );
    }
    let llvm = module.print_to_string().to_string();
    assert!(
        !llvm.contains("<8 x float>"),
        "staged SIMD must not leak to the host runtime module"
    );
    let object = fixture.0.join("host-staged-simd.o");
    host.write_object(&module, &object).unwrap();
    execute_host_object(&object, &fixture.0);
}

#[test]
fn arm_native_emission_and_missing_x86_features_are_rejected() {
    let fixture = Fixture::new();
    let arm = target("aarch64-unknown-linux-gnu", "", BitcodeOptimization::O0);
    let program = fixture.program(FLOAT_X, &arm);
    assert_vm(&program);
    let context = jai_codegen::Context::create();
    let error = jai_codegen::lower_for_target(&context, &program, &arm).unwrap_err();
    assert!(
        error.to_string().contains("unsupported for target"),
        "{error}"
    );
    for (source, features, missing) in [
        (FLOAT_Y, "", "avx"),
        (BYTES_X, "+avx,-avx2", "avx2"),
        (FLOAT_X, "-sse2", "sse2"),
    ] {
        let x86 = target(
            "x86_64-unknown-linux-gnu",
            features,
            BitcodeOptimization::O0,
        );
        let program = fixture.program(source, &x86);
        let error = jai_codegen::lower_for_target(&context, &program, &x86).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("enabled target feature {missing}")),
            "{error}"
        );
    }
}

#[test]
fn malformed_source_shapes_initialization_and_declared_features_are_located() {
    let fixture = Fixture::new();
    let native = target(
        "x86_64-unknown-linux-gnu",
        "+avx,+avx2",
        BitcodeOptimization::O0,
    );
    for (assembly, expected) in [
        ("#asm {movups.x a:, b;}", "one register and one memory"),
        ("#asm {v:vec; addps.x v,v;}", "uninitialized register"),
        ("#asm {movups.x v,[ptr];}", "declared before use"),
        ("#asm {movups.y v:,[ptr];}", "declared AVX support"),
        ("#asm AVX {movdqu.y v:,[ptr];}", "declared AVX2 support"),
        (
            "#asm {movups.x v:,[ptr]; addps.x r:,v,v;}",
            "three-operand SIMD adds",
        ),
        (
            "#asm AVX,AVX {movups.x v:,[ptr];}",
            "duplicate SIMD feature",
        ),
        ("#asm {movups.x v:,[number];}", "typed pointer"),
        ("#asm {v:vec; movups.x v:,[ptr];}", "already declared"),
        (
            "#asm {movups.x v:,[ptr];} #asm {movups.x [ptr],v;}",
            "declared before use",
        ),
    ] {
        let source = format!(
            "main :: () -> int {{ array:[8]float32; ptr:=*array[0]; number:=1; {assembly} return 42; }}"
        );
        let graph = fixture.graph(&source);
        let error = resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                target: Some(native.build_target().unwrap()),
                ..ResolveOptions::default()
            },
            &mut NoEffects,
        )
        .unwrap_err();
        assert!(error.to_string().contains(expected), "{source}\n{error}");
        let rendered = error.render(graph.sources());
        assert!(
            rendered.contains("main.jai:"),
            "source diagnostic must be located: {rendered}"
        );
        assert!(error.location.span.start > 0);
    }
}
