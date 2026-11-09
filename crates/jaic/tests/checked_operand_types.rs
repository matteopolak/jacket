//! Exported operands retain contextual types from actual checking, including folded globals.
use jaic::{
    records::{FieldRef, Item, RecordTag, Records},
    sema::{Compiler, NativeFs, Options, ProgramSource, VirtualFs},
};
use std::{path::Path, rc::Rc};

fn reference(r: &Records, id: i64, field: &str) -> i64 {
    match r.field(id, field) {
        Some(FieldRef::Ref(id)) if id != 0 => id,
        _ => panic!("missing {field}: {:?}", r.get(id)),
    }
}

fn list(r: &Records, id: i64, field: &str) -> Vec<i64> {
    match r.field(id, field) {
        Some(FieldRef::List(xs)) => xs
            .iter()
            .filter_map(|x| {
                if let Item::Ref(id) = x {
                    Some(id)
                } else {
                    None
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn comparisons(source: &str) -> Vec<(i64, i64, i64)> {
    let mut fs = VirtualFs::default();
    fs.insert("/prelude.jai", "Operating_System_Tag :: enum u32 { WINDOWS; LINUX; MACOS; WASM; } CPU_Tag :: enum u32 { X64; ARM64; CUSTOM; }".as_bytes());
    fs.insert("/main.jai", source.as_bytes());
    let mut options = Options::host();
    options.runtime_support = false;
    options.preload = Some("/prelude.jai".into());
    let mut c = Compiler::new(options, Rc::new(fs));
    c.compile_program(Path::new("/main.jai")).unwrap();
    let mut records = Records::default();
    let mut binaries = Vec::new();
    while let Some(message) = c.export_typechecked(&mut records) {
        for checked in list(&records, message, "all") {
            let expression = reference(&records, checked, "expression");
            let header = match records.get(expression).unwrap().tag {
                RecordTag::CodeProcedureBody => Some(reference(&records, expression, "header")),
                RecordTag::CodeDeclaration => match records.field(expression, "expression") {
                    Some(FieldRef::Ref(id)) if id != 0 => Some(id),
                    _ => None,
                },
                RecordTag::CodeProcedureHeader => Some(expression),
                _ => None,
            };
            if let Some(header) = header
                && records.get(header).unwrap().tag == RecordTag::CodeProcedureHeader
                && matches!(records.field(header,"procedure_flags"), Some(FieldRef::Int(flags)) if flags & 4 != 0)
            {
                continue;
            }
            for node in list(&records, checked, "subexpressions") {
                if records.get(node).unwrap().tag == RecordTag::CodeBinaryOperator
                    && matches!(
                        records.field(node, "operator_type"),
                        Some(FieldRef::Int(131))
                    )
                {
                    binaries.push(node);
                }
            }
        }
    }
    binaries.sort_unstable();
    binaries.dedup();
    binaries
        .into_iter()
        .map(|binary| {
            let left = reference(&records, binary, "left");
            let right = reference(&records, binary, "right");
            let left_ty = reference(&records, left, "type");
            let right_ty = reference(&records, right, "type");
            assert_eq!(
                left_ty,
                right_ty,
                "operand type descriptors differ: {:?} vs {:?}",
                records.get(left_ty),
                records.get(right_ty)
            );
            let Some(FieldRef::Int(size)) = records.field(left_ty, "runtime_size") else {
                panic!("missing size");
            };
            let Some(FieldRef::Int(tag)) = records.field(left_ty, "type") else {
                panic!("missing type tag");
            };
            let result = reference(&records, binary, "type");
            assert!(
                matches!(records.field(result, "type"), Some(FieldRef::Int(2))),
                "comparison result must be bool"
            );
            (tag, size, left_ty)
        })
        .collect()
}

#[test]
fn globals_export_builtin_call_and_inferred_enum_operand_types() {
    let result = comparisons(
        "A :: OS == .WINDOWS; B :: CPU == .ARM64; C :: BUILD_OS == OS; D :: size_of(int) == 8; E :: enum { RED; GREEN; } F :: E.RED == .GREEN; main :: () {}",
    );
    assert_eq!(result.len(), 5);
}

#[test]
fn folded_and_runtime_operands_keep_converted_literal_widths() {
    let result = comparisons(
        "A :: cast(u8)3 == 3; B :: cast(s32)3 == 3; C :: cast(float32)3 == 3; main :: () { x:u8=3; a:=x==3; y:float32=3; b:=y==3; }",
    );
    let mut sizes: Vec<_> = result.into_iter().map(|(_, size, _)| size).collect();
    sizes.sort_unstable();
    assert_eq!(sizes, [1, 1, 4, 4, 4]);
}

#[test]
fn instantiated_struct_defaults_export_in_the_struct_scope() {
    let result = comparisons("R :: struct { flag:bool = OS == .WINDOWS; } main :: () {r:R;}");
    assert_eq!(result.len(), 1);
}

#[test]
fn generic_instances_and_local_constants_keep_their_own_facts() {
    let result = comparisons(
        "compare :: (x:$T)->bool { LOCAL :: cast(u8)3 == 3; return LOCAL && x==3; } main :: () {a:u8=3; b:u16=4; compare(a); compare(b);}",
    );
    let mut sizes: Vec<_> = result.into_iter().map(|(_, size, _)| size).collect();
    sizes.sort_unstable();
    assert_eq!(sizes, [1, 1, 1, 2]);
}

#[test]
fn mixed_numeric_comparisons_export_the_arithmetic_operand_type() {
    let result = comparisons("A :: cast(s32)3 == 3.0; main :: () { x:s32=3; same:=x==3.0; }");
    assert_eq!(result.len(), 2);
    assert!(
        result.iter().all(|(tag, size, _)| *tag == 1 && *size == 4),
        "{result:?}"
    );
}

#[test]
fn arguments_boxed_for_any_keep_their_own_types() {
    let stdlib = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    c.compile_sources(&[ProgramSource::String(
        "#import \"Basic\"; show :: (args: ..Any) {} main :: () { x: u8 = 1; show(0, x, 1.5, true, 7.0); }"
            .to_string(),
    )])
    .unwrap();
    let mut records = Records::default();
    let mut calls: Vec<Vec<i64>> = Vec::new();
    while let Some(message) = c.export_typechecked(&mut records) {
        for checked in list(&records, message, "all") {
            for node in list(&records, checked, "subexpressions") {
                if records.get(node).unwrap().tag != RecordTag::CodeProcedureCall {
                    continue;
                }
                let sorted = list(&records, node, "arguments_sorted");
                if sorted.len() != 5 {
                    continue;
                }
                let tags: Option<Vec<i64>> = sorted
                    .into_iter()
                    .map(|argument| {
                        let Some(FieldRef::Ref(ty)) = records.field(argument, "type") else {
                            return None;
                        };
                        let Some(FieldRef::Int(tag)) = records.field(ty, "type") else {
                            return None;
                        };
                        Some(tag)
                    })
                    .collect();
                calls.extend(tags);
            }
        }
    }
    // Integer, integer, float, bool, float: never the `Any` the arguments are converted to.
    assert!(calls.contains(&vec![0, 0, 1, 2, 1]), "{calls:?}");
}
