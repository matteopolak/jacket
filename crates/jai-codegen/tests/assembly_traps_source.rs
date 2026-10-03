//! Closed source trap profiles, portable VM failures, and newly generated code.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions, TargetSelection, Triple},
};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::BitcodeOptimization;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

const INT3: &str = "main :: () -> int { #asm { int3; } return 42; }";
const BRK1: &str = "main :: () -> int { #bytes .[0x20,0x00,0b001_0_0000,0b1101_0100]; return 42; }";

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-assembly-traps-source-{}-{}",
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
    fn options(target: &NativeTarget) -> ResolveOptions {
        ResolveOptions {
            target: Some(target.build_target().unwrap()),
            layout: Some(target.layout_policy().unwrap()),
            ..ResolveOptions::default()
        }
    }
    fn program(&self, source: &str, target: &NativeTarget) -> jai_ir::Program {
        resolve_graph_with_options(&self.graph(source), &Self::options(target), &mut NoEffects)
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn target(triple: &str, optimization: BitcodeOptimization) -> NativeTarget {
    NativeTarget::select(&TargetOptions {
        selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
        optimization: Optimization {
            bitcode: optimization,
            ..Optimization::default()
        },
        ..TargetOptions::default()
    })
    .unwrap()
}

#[test]
fn both_typed_traps_report_vm_runtime_trap_independently_of_native_architecture() {
    let fixture = Fixture::new();
    for triple in ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"] {
        let native = target(triple, BitcodeOptimization::O0);
        for source in [INT3, BRK1] {
            let program = fixture.program(source, &native);
            let outcome = jai_vm::execute(&program, Limits::default()).outcome;
            assert!(
                matches!(outcome, Outcome::Failed(jai_vm::Error::RuntimeTrap)),
                "{triple}: {outcome:?}"
            );
        }
    }
}

#[test]
fn source_traps_emit_exact_target_instructions_at_o0_and_o2() {
    let fixture = Fixture::new();
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        for (source, triple, machine, encoding) in [
            (INT3, "x86_64-unknown-linux-gnu", 62, &[0xcc][..]),
            (
                BRK1,
                "aarch64-unknown-linux-gnu",
                183,
                &[0x20, 0x00, 0x20, 0xd4][..],
            ),
        ] {
            let native = target(triple, optimization);
            let program = fixture.program(source, &native);
            let context = jai_codegen::Context::create();
            let module = jai_codegen::lower_for_target(&context, &program, &native).unwrap();
            module.verify().unwrap();
            let object = fixture.0.join(format!("trap-{machine}-{optimization:?}.o"));
            native.write_object(&module, &object).unwrap();
            let bytes = fs::read(object).unwrap();
            let text = elf_text(&bytes, machine);
            let contains_instruction = if machine == 183 {
                text.chunks_exact(4)
                    .any(|instruction| instruction == encoding)
            } else {
                text.windows(encoding.len())
                    .any(|instruction| instruction == encoding)
            };
            assert!(
                contains_instruction,
                "expected exact {encoding:02x?} instruction in .text: {text:02x?}"
            );
        }
    }
}

fn elf_text(bytes: &[u8], machine: u16) -> &[u8] {
    assert_eq!(&bytes[..4], b"\x7fELF");
    assert_eq!(bytes[4], 2); // ELF64
    assert_eq!(bytes[5], 1); // Little endian
    let u16_at = |offset| u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap());
    let u32_at =
        |offset| u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap()) as usize;
    let u64_at = |offset| {
        usize::try_from(u64::from_le_bytes(
            bytes[offset..offset + 8].try_into().unwrap(),
        ))
        .unwrap()
    };
    assert_eq!(u16_at(18), machine);
    let sections = u64_at(40);
    let stride = usize::from(u16_at(58));
    let count = usize::from(u16_at(60));
    let strings = sections + usize::from(u16_at(62)) * stride;
    let string_offset = u64_at(strings + 24);
    let names = &bytes[string_offset..string_offset + u64_at(strings + 32)];
    for index in 0..count {
        let section = sections + index * stride;
        let name = &names[u32_at(section)..];
        let end = name.iter().position(|byte| *byte == 0).unwrap();
        if &name[..end] == b".text" {
            let offset = u64_at(section + 24);
            return &bytes[offset..offset + u64_at(section + 32)];
        }
    }
    panic!("newly generated object must contain .text")
}

#[test]
fn native_source_traps_reject_the_wrong_instruction_set() {
    let fixture = Fixture::new();
    for (source, triple, message) in [
        (
            INT3,
            "aarch64-unknown-linux-gnu",
            "x86 SIMD assembly is unsupported",
        ),
        (
            BRK1,
            "x86_64-unknown-linux-gnu",
            "ARM64 BRK #1 is unsupported",
        ),
    ] {
        let native = target(triple, BitcodeOptimization::O0);
        let program = fixture.program(source, &native);
        let context = jai_codegen::Context::create();
        let error = jai_codegen::lower_for_target(&context, &program, &native).unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
    }
}

#[test]
fn selected_unsupported_assembly_has_located_diagnostics() {
    let fixture = Fixture::new();
    let native = NativeTarget::new().unwrap();
    for (statement, message) in [
        (
            "#asm { arbitrary [unresolved_address]; }",
            "assembly instruction is unsupported",
        ),
        (
            "#asm { temporary: general; }",
            "assembly instruction is unsupported",
        ),
        (
            "#asm { int 0x41; }",
            "unsupported platform interrupt capability",
        ),
        ("#bytes .[0x3f,0x20,0x03,0xd5];", "accepts only BRK #1"),
        ("#bytes .[];", "accepts only BRK #1"),
        (
            "#asm SYSCALL_SYSRET { syscall temporary:, unresolved; }",
            "assembly feature 'SYSCALL_SYSRET' is unsupported",
        ),
    ] {
        let source = format!("main :: () -> int {{ {statement} return 42; }}");
        let graph = fixture.graph(&source);
        let error = resolve_graph_with_options(&graph, &Fixture::options(&native), &mut NoEffects)
            .unwrap_err();
        assert!(error.message.contains(message), "{error}");
        assert!(error.location.span.start > 0);
        assert!(error.render(graph.sources()).contains("main.jai:"));
    }
}

#[test]
fn inactive_unsupported_payloads_are_selected_away_before_semantic_or_native_checking() {
    let fixture = Fixture::new();
    let native = NativeTarget::new().unwrap();
    let source = "main :: () -> int { #if false { #asm { temporary: general; arbitrary [unresolved_address]; int 0x41; int3; } #asm SYSCALL_SYSRET { syscall temporary:, unresolved; } #bytes .[0x3f,0x20,0x03,0xd5]; #bytes .[0x20,0x00,0x20,0xd4]; } return 42; }";
    let program = fixture.program(source, &native);
    let outcome = jai_vm::execute(&program, Limits::default()).outcome;
    assert!(
        matches!(&outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==42)),
        "{outcome:?}"
    );
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &native).unwrap();
    let object = fixture.0.join("inactive-payload.o");
    native.write_object(&module, &object).unwrap();
}

#[cfg(all(unix, any(target_arch = "x86_64", target_arch = "aarch64")))]
#[test]
fn newly_generated_host_trap_process_terminates_with_sigtrap() {
    use std::{
        os::unix::process::ExitStatusExt,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    let fixture = Fixture::new();
    let source = if cfg!(target_arch = "aarch64") {
        BRK1
    } else {
        INT3
    };
    let native = NativeTarget::new().unwrap();
    let program = fixture.program(source, &native);
    let context = jai_codegen::Context::create();
    let module = jai_codegen::lower_for_target(&context, &program, &native).unwrap();
    let object = fixture.0.join("host-trap.o");
    let executable = fixture.0.join("host-trap");
    native.write_object(&module, &object).unwrap();
    let linked = native_tools::clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    let mut child = Command::new(executable)
        .current_dir(&fixture.0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert_eq!(
                status.signal(),
                Some(5),
                "native debug trap must raise SIGTRAP"
            );
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("generated host trap exceeded five seconds");
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
