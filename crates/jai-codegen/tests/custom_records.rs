//! Source layout checks and execution of newly generated custom record objects.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
    types::TypeLowerer,
};
use jai_types::{BitcodeOptimization, TypeKind};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-custom-records-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn check(source: &str, expected_layouts: &[(u64, u32, &[u64])]) -> String {
    let scratch = Scratch::new();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
    let mut unoptimized_ir = String::new();
    for optimization in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode: optimization,
                ..Optimization::default()
            },
            ..TargetOptions::default()
        })
        .unwrap();
        let program = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(target.layout_policy().unwrap()),
                ..jai_sema::ResolveOptions::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        let context = jai_codegen::Context::create();
        let mut lowerer = TypeLowerer::with_target(&context, program.types(), &target.data);
        let mut custom_layouts = Vec::new();
        for (id, kind) in program.types().iter() {
            if matches!(kind, TypeKind::Record(_) | TypeKind::FixedArray { .. }) {
                let layout = lowerer.verify_layout(id, &target.data).unwrap();
                if matches!(kind, TypeKind::Record(_)) {
                    let options = &program.types().record_definition(id).unwrap().layout;
                    if options.packed
                        || options.minimum_alignment.is_some()
                        || options.field_alignments.iter().any(Option::is_some)
                    {
                        custom_layouts.push((
                            layout.size,
                            layout.alignment,
                            layout.field_offsets.into_vec(),
                        ));
                    }
                }
            }
        }
        let mut expected = expected_layouts
            .iter()
            .map(|(size, alignment, offsets)| (*size, *alignment, offsets.to_vec()))
            .collect::<Vec<_>>();
        custom_layouts.sort();
        expected.sort();
        assert_eq!(custom_layouts, expected);
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        if optimization == BitcodeOptimization::O0 {
            unoptimized_ir = module.print_to_string().to_string();
        }
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut command = native_tools::clang_command();
        command.arg(&object);
        // Mach-O chained fixups cannot encode unaligned packed pointers.
        if cfg!(target_os = "macos") {
            command.arg("-Wl,-no_fixup_chains");
        }
        let output = command.arg("-o").arg(&executable).output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stderr),
            module.print_to_string()
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(42), "{optimization:?}\n{source}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated custom record fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
    unoptimized_ir
}

fn assert_projected_access_alignment(ir: &str, alignment: u32) {
    let accesses = ir
        .lines()
        .filter(|line| {
            (line.contains("load ") || line.trim_start().starts_with("store "))
                && line.contains(", ptr %record.field.address")
        })
        .collect::<Vec<_>>();
    assert!(
        !accesses.is_empty(),
        "expected projected field accesses\n{ir}"
    );
    for access in accesses {
        assert!(
            access.trim_end().ends_with(&format!("align {alignment}")),
            "incorrect projected access alignment: {access}"
        );
    }
}

#[test]
fn packed_scalar_fields_use_byte_offsets_and_alias_storage() {
    let ir = check(
        r#"
        Packed :: struct { tag:u8; value:u64; tail:u16; } #no_padding
        write :: (p:*u64) { p.* = 42; }
        main :: ()->int {
            item:Packed = .{tag=7, value=5, tail=9};
            bytes := cast(*u8) *item;
            if cast(*u8) *item.value - bytes != 1 return 1;
            if cast(*u8) *item.tail - bytes != 9 return 2;
            if size_of(Packed) != 11 return 3;
            write(*item.value);
            if bytes[1] != 42 || item.tag != 7 || item.tail != 9 return 4;
            return cast(int) item.value;
        }
        "#,
        &[(11, 1, &[0, 1, 9])],
    );
    assert_projected_access_alignment(&ir, 1);
}

#[test]
fn reduced_pointer_and_u64_alignment_use_four_byte_offsets() {
    let ir = check(
        r#"
        Reduced :: struct {
            tag:u32 #align 4;
            pointer:*int #align 4;
            value:u64 #align 4;
            tail:u8;
        } #no_padding
        main :: ()->int {
            answer := 40;
            item:Reduced = .{tag=7, pointer=*answer, value=1, tail=9};
            bytes := cast(*u8) *item;
            if cast(*u8) *item.pointer - bytes != 4 return 1;
            if cast(*u8) *item.value - bytes != 12 return 2;
            if cast(*u8) *item.tail - bytes != 20 return 3;
            if size_of(Reduced) != 24 return 4;
            p := *item.value;
            p.* += 1;
            return item.pointer.* + cast(int) item.value;
        }
        "#,
        &[(24, 4, &[0, 4, 12, 20])],
    );
    assert_projected_access_alignment(&ir, 4);
}

#[test]
fn increased_record_alignment_controls_array_stride() {
    for alignment in [16, 32] {
        for (packing, value_offset) in [("", 8), ("#no_padding", 1)] {
            let source = format!(
                r#"
            Aligned :: struct #align {alignment} {{ tag:u8; value:int; }} {packing}
            pass :: (value:Aligned)->Aligned {{ return value; }}
            main :: ()->int {{
                values:[2]Aligned = .[.{{tag=1,value=20}}, .{{tag=2,value=22}}];
                first := cast(*u8) *values[0];
                second := cast(*u8) *values[1];
                if second - first != {alignment} return 1;
                if size_of(Aligned) != {alignment} return 2;
                if cast(*u8) *values[0].value - first != {value_offset} return 3;
                copy := pass(values[0]);
                values[0].value = 99;
                return copy.value + values[1].value;
            }}
            "#
            );
            check(
                &source,
                &[(alignment, alignment as u32, &[0, value_offset])],
            );
        }
    }
}

#[test]
fn packed_parent_preserves_natural_child_layout_and_misaligned_reads() {
    check(
        r#"
        Child :: struct { tag:u8; value:int; }
        Parent :: struct { prefix:u8; child:Child; tail:u8; } #no_padding
        main :: ()->int {
            item:Parent = .{prefix=3, child=.{tag=4,value=20}, tail=5};
            copy := item;
            bytes := cast(*u8) *item;
            if cast(*u8) *item.child - bytes != 1 return 1;
            if cast(*u8) *item.child.value - bytes != 9 return 2;
            if cast(*u8) *item.tail - bytes != 17 return 3;
            pointer := *item.child;
            pointer.*.value += 2;
            return pointer.*.value + copy.child.value;
        }
        "#,
        &[(18, 1, &[0, 1, 17])],
    );
}

#[test]
fn packed_record_arrays_preserve_stride_and_pointer_indexing() {
    check(
        r#"
        Packed :: struct { tag:u8; value:int; } #no_padding
        main :: ()->int {
            values:[3]Packed = .[.{tag=1,value=10}, .{tag=2,value=20}, .{tag=3,value=30}];
            copy := values;
            p := *values[0];
            if cast(*u8) (p + 1) - cast(*u8) p != 9 return 1;
            if cast(*u8) *values[2].value - cast(*u8) p != 19 return 2;
            p[1].value = 12;
            return p[1].value + copy[2].value;
        }
        "#,
        &[(9, 1, &[0, 1])],
    );
}

#[test]
fn packed_global_nested_union_constants_preserve_string_relocations() {
    check(
        r#"
        Inner :: union { text:string; number:u64; } #no_padding
        Choice :: union { inner:Inner; number:u64; } #no_padding
        Envelope :: struct { tag:u8; values:[2]Choice; } #no_padding
        global:Envelope = .{tag=7, values=.[.{inner=.{text="hello"}}, .{number=35}]};
        main :: ()->int {
            bytes := cast(*u8) *global;
            if cast(*u8) *global.values[0] - bytes != 1 return 1;
            if cast(*u8) *global.values[1] - bytes != 17 return 2;
            if global.values[0].inner.text[0] != 104 return 3;
            p := *global.values[1].number;
            p.* += 2;
            return global.values[0].inner.text.count + cast(int) global.values[1].number;
        }
        "#,
        &[(16, 1, &[0, 0]), (16, 1, &[0, 0]), (33, 1, &[0, 1])],
    );
}

#[test]
fn packed_initializers_parameters_returns_and_cleanup_preserve_snapshots() {
    check(
        r#"
        Packed :: struct { tag:u8; value:int; } #no_padding
        calls := 0;
        next :: ()->int { calls += 1; return calls; }
        make :: ()->Packed {
            item:Packed = .{value=next()+29, tag=cast(u8) next()};
            defer item.value = 99;
            return item;
        }
        change :: (item:Packed)->Packed { item.value += 10; return item; }
        main :: ()->int {
            original := make();
            copy := change(original);
            if calls != 2 || original.value != 30 || original.tag != 2 return 1;
            original.value = 99;
            return copy.value + cast(int) copy.tag;
        }
        "#,
        &[(9, 1, &[0, 1])],
    );
}
