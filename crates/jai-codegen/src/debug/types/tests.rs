use super::*;
use crate::{debug::DebugInformation, optimization::Optimization, target::NativeTarget};
use inkwell::context::Context;
use inkwell::values::BasicValue;
use jai_ir::{FieldSource, TypeSource};
use jai_types::{IntegerType, RecordLayout, ScalarType, TypeRegistry};
#[test]
fn registry_fields_and_custom_layouts_drive_debug_members_without_synthetic_names() {
    let context = Context::create();
    let target = NativeTarget::new().unwrap();
    let policy = crate::types::layout_policy(&context, &target.data).unwrap();
    let mut registry = TypeRegistry::new();
    let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
    let word = registry.scalar(ScalarType::Int(IntegerType::S64));
    let packed = registry.reserve_record(RecordKind::Struct);
    registry
        .define_record_with_layout(
            packed,
            vec![byte, word].into_boxed_slice(),
            RecordLayout {
                packed: true,
                ..RecordLayout::default()
            },
        )
        .unwrap();
    let union = registry.reserve_record(RecordKind::Union);
    registry
        .define_record(union, vec![byte, word].into_boxed_slice())
        .unwrap();
    let array = registry.fixed_array(packed, 2).unwrap();
    let text = "Packed :: struct #packed { tag:u8; payload:s64; }\nChoice :: union { tag:u8; payload:s64; }\nmain :: () -> int { return 42; }\n";
    let mut map = jai_source::SourceMap::default();
    let source_id = map.insert("/own-source/registry-layouts.jai".into(), text.into());
    let source = map.get(source_id).unwrap();
    let mut sources = jai_ir::DebugSources::default();
    sources.retain_source(source);
    let at = text.find("main").unwrap();
    let location = sources
        .source_location(
            source,
            jai_source::SourceSpan {
                source: source_id,
                span: jai_source::Span::new(at, at + 4),
            },
        )
        .unwrap();
    sources.insert(
        jai_ir::ProcedureId::new(27),
        jai_ir::ProcedureSource {
            name: "main".into(),
            location: location.clone(),
        },
    );
    for (ty, name) in [(packed, "Packed"), (union, "Choice")] {
        let at = text.find(name).unwrap();
        let type_location = sources
            .source_location(
                source,
                jai_source::SourceSpan {
                    source: source_id,
                    span: jai_source::Span::new(at, at + name.len()),
                },
            )
            .unwrap();
        sources.insert_type_source(
            ty,
            TypeSource {
                name: Some(name.into()),
                location: type_location,
            },
        );
        for (index, name) in ["tag", "payload"].into_iter().enumerate() {
            let field = registry.field(ty, index).unwrap();
            let record_name = if ty == packed { "Packed" } else { "Choice" };
            let start = text.find(record_name).unwrap();
            let at = start + text[start..].find(name).unwrap();
            let field_location = sources
                .source_location(
                    source,
                    jai_source::SourceSpan {
                        source: source_id,
                        span: jai_source::Span::new(at, at + name.len()),
                    },
                )
                .unwrap();
            sources.insert_field_source(
                field.id,
                FieldSource {
                    name: name.into(),
                    location: field_location,
                },
            );
        }
    }
    let module = context.create_module("own.registry.debug.types");
    let tables = LineTables::new_with_policy(
        &module,
        &context,
        &sources,
        Optimization::default(),
        DebugInformation::Variables,
        policy,
    )
    .unwrap()
    .unwrap();
    assert!(
        tables
            .runtime_type(packed, &registry, None)
            .unwrap()
            .is_none(),
        "missing field provenance must not produce f0/f1 names"
    );
    let packed_metadata = tables
        .runtime_type(packed, &registry, Some(&sources))
        .unwrap()
        .unwrap();
    let array_metadata = tables
        .runtime_type(array, &registry, Some(&sources))
        .unwrap()
        .unwrap();
    let union_metadata = tables
        .runtime_type(union, &registry, Some(&sources))
        .unwrap()
        .unwrap();
    let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let id = sources.procedures().next().unwrap().0;
    let scope = tables
        .attach_scope(&context, &builder, function, id, &sources)
        .unwrap()
        .unwrap();
    for (name, metadata, bytes, alignment) in [
        ("packed", packed_metadata, 9, 1),
        ("array", array_metadata, 18, 1),
        ("choice", union_metadata, 8, 8),
    ] {
        let variable = tables
            .session
            .typed_variable(
                scope.metadata,
                source_descriptor(&location).unwrap(),
                name,
                super::super::VariableKind::Automatic,
                metadata,
                alignment,
            )
            .unwrap();
        let storage = builder
            .build_alloca(context.i8_type().array_type(bytes), name)
            .unwrap();
        tables.session.declare(&builder, storage, variable).unwrap();
    }
    builder
        .build_return(Some(&context.i32_type().const_int(42, false)))
        .unwrap();
    tables.finish();
    module.verify().unwrap();
    let ir = module.print_to_string().to_string();
    assert!(ir.contains("DW_TAG_union_type"), "{ir}");
    assert!(ir.contains("DW_TAG_array_type"), "{ir}");
    assert!(
        ir.lines().any(|line| line.contains("DW_TAG_member")
            && line.contains("name: \"payload\"")
            && line.contains("align: 8")
            && line.contains("offset: 8")),
        "packed member must use target byte offset1: {ir}"
    );
    assert!(
        ir.contains("name: \"Packed\"") && ir.contains("size: 72"),
        "{ir}"
    );
    assert!(!ir.contains("name: \"f0\""), "{ir}");
}

#[test]
fn cross_target_pointer_width_and_record_offsets_reach_native_dwarf() {
    use crate::debug::variables::tests::{Scratch, debug_tool_command};
    use crate::target::{TargetOptions, TargetSelection, Triple};
    use jai_ir::{ProcedureId, ProcedureSource};
    use jai_source::{SourceMap, SourceSpan, Span};

    let scratch = Scratch::new();
    for (triple, pointer_bits) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let context = Context::create();
        let target = NativeTarget::select(&TargetOptions {
            selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
            ..Default::default()
        })
        .unwrap();
        let policy = crate::types::layout_policy(&context, &target.data).unwrap();
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let word = registry.scalar(ScalarType::Int(IntegerType::S64));
        let pointer = registry.pointer(byte).unwrap();
        let record = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(record, vec![pointer, word].into_boxed_slice())
            .unwrap();
        let callback = registry
            .procedure(jai_types::ProcedureType {
                parameters: vec![word].into_boxed_slice(),
                results: vec![word, word].into_boxed_slice(),
                convention: jai_types::CallingConvention::Jai,
                context: jai_types::ContextMode::Implicit,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let text = "Packet :: struct { address:*u8; payload:s64; }\nmain :: () -> int { packet:Packet; callback:(argument:s64)->s64,s64; return 42; }\n";
        let mut map = SourceMap::default();
        let source_id = map.insert("/own-source/target-packet.jai".into(), text.into());
        let source = map.get(source_id).unwrap();
        let mut sources = jai_ir::DebugSources::default();
        sources.retain_source(source);
        let location = |name: &str| {
            let start = text.find(name).unwrap();
            jai_ir::DebugSourceLocation::from_source(
                source,
                SourceSpan {
                    source: source_id,
                    span: Span::new(start, start + name.len()),
                },
            )
            .unwrap()
        };
        let procedure = ProcedureId::new(27);
        sources.insert(
            procedure,
            ProcedureSource {
                name: "main".into(),
                location: location("main"),
            },
        );
        sources.insert_type_source(
            record,
            TypeSource {
                name: Some("Packet".into()),
                location: location("Packet"),
            },
        );
        for (index, name) in ["address", "payload"].into_iter().enumerate() {
            sources.insert_field_source(
                registry.field(record, index).unwrap().id,
                FieldSource {
                    name: name.into(),
                    location: location(name),
                },
            );
        }
        let types = registry.freeze().unwrap();
        let mut layouts = LayoutEngine::new(&types, policy);
        let layout = layouts.layout(record).unwrap().clone();
        let mut lowerer = crate::types::TypeLowerer::with_target(&context, &types, &target.data);
        let native = lowerer.basic(record).unwrap();
        lowerer.verify_layout(record, &target.data).unwrap();
        let module = context.create_module("own.cross.target.debug");
        let tables = LineTables::new_with_policy(
            &module,
            &context,
            &sources,
            Optimization::default(),
            DebugInformation::Variables,
            policy,
        )
        .unwrap()
        .unwrap();
        let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
        let builder = context.create_builder();
        builder.position_at_end(context.append_basic_block(function, "entry"));
        let scope = tables
            .attach_scope(&context, &builder, function, procedure, &sources)
            .unwrap()
            .unwrap();
        let storage = builder.build_alloca(native, "packet").unwrap();
        storage
            .as_instruction_value()
            .unwrap()
            .set_alignment(layout.alignment)
            .unwrap();
        let variable = tables
            .create_variable_with_sources(
                &context,
                scope,
                &location("packet"),
                "packet",
                super::super::VariableKind::Automatic,
                record,
                &types,
                layout.alignment,
                Some(&sources),
            )
            .unwrap()
            .unwrap();
        tables.emit_variable(&builder, storage, variable).unwrap();
        let callback_layout = layouts.layout(callback).unwrap().clone();
        let storage = builder
            .build_alloca(lowerer.basic(callback).unwrap(), "callback")
            .unwrap();
        let variable = tables
            .create_variable_with_sources(
                &context,
                scope,
                &location("callback"),
                "callback",
                super::super::VariableKind::Automatic,
                callback,
                &types,
                callback_layout.alignment,
                Some(&sources),
            )
            .unwrap()
            .unwrap();
        tables.emit_variable(&builder, storage, variable).unwrap();
        builder
            .build_return(Some(&context.i32_type().const_int(42, false)))
            .unwrap();
        tables.finish();
        module.verify().unwrap();
        let ir = module.print_to_string().to_string();
        assert!(
            ir.lines().any(|line| line.contains("DW_TAG_pointer_type")
                && line.contains(&format!("size: {pointer_bits}"))),
            "{ir}"
        );
        assert!(
            ir.lines().any(|line| line.contains("DW_TAG_member")
                && line.contains("name: \"payload\"")
                && line.contains(&format!("offset: {}", layout.field_offsets[1] * 8))),
            "{ir}"
        );
        let object = scratch.0.join(format!("{triple}.o"));
        target.write_object(&module, &object).unwrap();
        let output = debug_tool_command("llvm-dwarfdump")
            .arg("--debug-info")
            .arg(object)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let dwarf = String::from_utf8(output.stdout).unwrap();
        assert!(dwarf.contains("DW_TAG_pointer_type"), "{dwarf}");
        assert!(
            dwarf.contains(&format!("addr_size = 0x{:02x}", pointer_bits / 8)),
            "{dwarf}"
        );
        let pointer_description = dwarf
            .split_once("DW_TAG_pointer_type")
            .unwrap()
            .1
            .split("\n\n")
            .next()
            .unwrap();
        if pointer_description.contains("DW_AT_byte_size") {
            assert!(
                pointer_description
                    .contains(&format!("DW_AT_byte_size\t(0x{:02x})", pointer_bits / 8)),
                "{dwarf}"
            );
        }
        assert!(
            dwarf.contains(&format!("DW_AT_byte_size\t(0x{:02x})", layout.size)),
            "{dwarf}"
        );
        assert!(
            dwarf.contains(&format!(
                "DW_AT_data_member_location\t(0x{:02x})",
                layout.field_offsets[1]
            )),
            "{dwarf}"
        );
        assert!(dwarf.contains("DW_AT_name\t(\"packet\")"), "{dwarf}");
        assert!(dwarf.contains("DW_AT_name\t(\"callback\")"), "{dwarf}");
        assert!(dwarf.contains("DW_TAG_subroutine_type"), "{dwarf}");
        assert!(
            ir.contains("DISubroutineType(flags: DIFlagPrototyped"),
            "{ir}"
        );
        let signature = ir
            .lines()
            .find(|line| line.contains("DISubroutineType(flags: DIFlagPrototyped"))
            .unwrap();
        let list = signature
            .split("types: ")
            .nth(1)
            .unwrap()
            .split([',', ')'])
            .next()
            .unwrap();
        let entries = ir
            .lines()
            .find(|line| line.starts_with(&format!("{list} =")))
            .unwrap();
        assert_eq!(
            entries.matches('!').count(),
            4,
            "one result tuple and one declared argument; implicit context is excluded: {entries}"
        );
    }
}
