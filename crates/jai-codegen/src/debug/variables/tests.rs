use super::*;
pub(crate) use crate::test_native_tools::clang_command;
use crate::{optimization::Optimization, target::NativeTarget};
use jai_ir::{DebugSources, ProcedureId, ProcedureSource};
use jai_source::{SourceMap, SourceSpan, Span};
use jai_types::{ScalarType, TypeRegistry};
use std::num::NonZeroU32;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};

pub(crate) fn debug_tool_command(name: &str) -> Command {
    let prefix = std::env::var_os("LLVM_SYS_221_PREFIX").map(PathBuf::from);
    let tool = prefix
        .map(|prefix| prefix.join("bin").join(name))
        .filter(|p| p.is_file())
        .or_else(|| {
            std::env::var_os("PATH").and_then(|paths| {
                std::env::split_paths(&paths)
                    .map(|path| path.join(name))
                    .find(|path| path.is_file())
            })
        })
        .expect("independently installed LLVM tooling required")
        .canonicalize()
        .unwrap();
    // Absolute source paths also identify the real checkout in isolated manifests.
    let source_file = Path::new(file!());
    let repository = if source_file.is_absolute() {
        source_file
            .ancestors()
            .find(|path| path.file_name().is_some_and(|name| name == "crates"))
            .and_then(Path::parent)
            .expect("repository source location")
            .to_owned()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    };
    for protected in ["reference", "corpus", ".git"] {
        let root = repository.join(protected);
        let root = root.canonicalize().unwrap_or(root);
        assert!(!tool.starts_with(root));
    }
    let mut command = Command::new(tool);
    for variable in [
        "LD_PRELOAD",
        "LD_LIBRARY_PATH",
        "DYLD_INSERT_LIBRARIES",
        "DYLD_LIBRARY_PATH",
        "DYLD_FALLBACK_LIBRARY_PATH",
        "DYLD_FRAMEWORK_PATH",
        "DYLD_FALLBACK_FRAMEWORK_PATH",
    ] {
        command.env_remove(variable);
    }
    command
}
pub(crate) struct Scratch(pub(crate) PathBuf);
impl Scratch {
    pub(crate) fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-local-debug-{}-{}",
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

#[test]
fn scopes_and_variables_reject_other_debug_builder_ownership() {
    let sources = crate::debug::tests::sources();
    let context = Context::create();
    let first = context.create_module("own.debug.first");
    let second = context.create_module("own.debug.second");
    let make_tables = |module| {
        LineTables::new_with_information(
            module,
            &context,
            &sources,
            Optimization::default(),
            DebugInformation::Variables,
        )
        .unwrap()
        .unwrap()
    };
    let first_tables = make_tables(&first);
    let second_tables = make_tables(&second);
    let function = first.add_function("own_main", context.i32_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let (id, procedure) = sources.procedures().next().unwrap();
    let scope = first_tables
        .attach_scope(&context, &builder, function, id, &sources)
        .unwrap()
        .unwrap();
    let location = &procedure.location;
    assert!(matches!(
        second_tables.lexical_scope(scope, location),
        Err(Error::Ownership)
    ));
    assert!(matches!(
        second_tables.statement_location(&context, &builder, scope, location),
        Err(Error::Ownership)
    ));
    let types = TypeRegistry::new();
    let ty = types.scalar(ScalarType::Int(IntegerType::S32));
    let record = first_tables
        .create_variable(
            &context,
            scope,
            location,
            "answer",
            VariableKind::Automatic,
            ty,
            &types,
            4,
        )
        .unwrap()
        .unwrap();
    let storage = builder.build_alloca(context.i32_type(), "answer").unwrap();
    assert!(matches!(
        second_tables.emit_variable(&builder, storage, record),
        Err(Error::Ownership)
    ));
    first_tables
        .emit_variable(&builder, storage, record)
        .unwrap();
    builder
        .build_return(Some(&context.i32_type().const_int(42, false)))
        .unwrap();
    first_tables.finish();
    second_tables.finish();
    first.verify().unwrap();
    second.verify().unwrap();
}

#[test]
fn line_only_mode_omits_variables_and_quote_locations_keep_original_file() {
    let mut map = SourceMap::default();
    let primary = map.insert("/own-source/caller.jai".into(), "main :: () {\n}\n".into());
    let quote = map.insert(
        "/own-source/quoted.jai".into(),
        "// Original captured source.\nreturn 42;\n".into(),
    );
    let mut sources = DebugSources::default();
    let declaration = sources
        .source_location(
            map.get(primary).unwrap(),
            SourceSpan {
                source: primary,
                span: Span::new(0, 4),
            },
        )
        .unwrap();
    let quote_location = sources
        .source_location(
            map.get(quote).unwrap(),
            SourceSpan {
                source: quote,
                span: Span::new(29, 35),
            },
        )
        .unwrap();
    sources.insert(
        ProcedureId::new(1),
        ProcedureSource {
            name: "main".into(),
            location: declaration.clone(),
        },
    );
    let context = Context::create();
    let module = context.create_module("own.quote.debug");
    let tables = LineTables::new(&module, &context, &sources, Optimization::default())
        .unwrap()
        .unwrap();
    let function = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let scope = tables
        .attach_scope(&context, &builder, function, ProcedureId::new(1), &sources)
        .unwrap()
        .unwrap();
    let storage = builder.build_alloca(context.i32_type(), "local").unwrap();
    let mut registry = TypeRegistry::new();
    let ty = registry.scalar(ScalarType::Int(IntegerType::S32));
    assert!(
        !tables
            .declare_variable(
                &context,
                &builder,
                scope,
                &declaration,
                "temporary",
                VariableKind::Automatic,
                ty,
                &registry,
                storage,
                4
            )
            .unwrap()
    );
    let pointer = registry.pointer(ty).unwrap();
    assert!(tables.primitive_type(pointer, &registry).unwrap().is_none());
    tables
        .statement_location(&context, &builder, scope, &quote_location)
        .unwrap();
    builder
        .build_return(Some(&context.i32_type().const_int(42, false)))
        .unwrap();
    tables.finish();
    module.verify().unwrap();
    let ir = module.print_to_string().to_string();
    assert!(ir.contains("filename: \"quoted.jai\""), "{ir}");
    assert!(ir.contains("DILexicalBlock"), "{ir}");
    assert!(ir.contains("DILocation(line: 2"), "{ir}");
    assert!(!ir.contains("DILocalVariable"), "{ir}");
}

#[test]
fn native_lexical_shadowing_parameters_and_distinct_statement_lines() {
    let scratch = Scratch::new();
    let source_path = scratch.0.join("locals.jai");
    let text = "sum :: (input: s32) -> s32 {\n  answer: s32 = input;\n  {\n    answer: s32 = 2;\n    input += answer;\n  }\n  return input;\n}\n";
    fs::write(&source_path, text).unwrap();
    let mut map = SourceMap::default();
    let id = map.insert(source_path, text.into());
    let source = map.get(id).unwrap();
    let mut sources = DebugSources::default();
    let location = |sources: &mut DebugSources, needle: &str| {
        let at = text.find(needle).unwrap();
        sources
            .source_location(
                source,
                SourceSpan {
                    source: id,
                    span: Span::new(at, at + needle.len()),
                },
            )
            .unwrap()
    };
    let declaration = location(&mut sources, "sum");
    let argument = location(&mut sources, "input: s32");
    let outer = location(&mut sources, "answer: s32 = input");
    let inner = location(&mut sources, "answer: s32 = 2");
    let add = location(&mut sources, "input += answer");
    let ret = location(&mut sources, "return input");
    sources.insert(
        ProcedureId::new(17),
        ProcedureSource {
            name: "sum".into(),
            location: declaration,
        },
    );
    let context = Context::create();
    let module = context.create_module("own.local.debug");
    let tables = LineTables::new_with_information(
        &module,
        &context,
        &sources,
        Optimization::default(),
        DebugInformation::Variables,
    )
    .unwrap()
    .unwrap();
    let function = module.add_function(
        "own_sum",
        context
            .i32_type()
            .fn_type(&[context.i32_type().into()], false),
        None,
    );
    let builder = context.create_builder();
    builder.position_at_end(context.append_basic_block(function, "entry"));
    let scope = tables
        .attach_scope(&context, &builder, function, ProcedureId::new(17), &sources)
        .unwrap()
        .unwrap();
    let body_scope = tables.lexical_scope(scope, &outer).unwrap();
    let inner_scope = tables.lexical_scope(body_scope, &inner).unwrap();
    let registry = TypeRegistry::new();
    let ty = registry.scalar(ScalarType::Int(IntegerType::S32));
    let input_slot = builder.build_alloca(context.i32_type(), "input").unwrap();
    let outer_slot = builder
        .build_alloca(context.i32_type(), "outer.answer")
        .unwrap();
    let inner_slot = builder
        .build_alloca(context.i32_type(), "inner.answer")
        .unwrap();
    builder
        .build_store(input_slot, function.get_nth_param(0).unwrap())
        .unwrap();
    assert!(
        tables
            .declare_variable(
                &context,
                &builder,
                scope,
                &argument,
                "input",
                VariableKind::Parameter(NonZeroU32::new(1).unwrap()),
                ty,
                &registry,
                input_slot,
                4
            )
            .unwrap()
    );
    tables
        .statement_location(&context, &builder, body_scope, &outer)
        .unwrap();
    let input = builder
        .build_load(context.i32_type(), input_slot, "input.value")
        .unwrap();
    builder.build_store(outer_slot, input).unwrap();
    tables
        .declare_variable(
            &context,
            &builder,
            body_scope,
            &outer,
            "answer",
            VariableKind::Automatic,
            ty,
            &registry,
            outer_slot,
            4,
        )
        .unwrap();
    tables
        .statement_location(&context, &builder, inner_scope, &inner)
        .unwrap();
    builder
        .build_store(inner_slot, context.i32_type().const_int(2, false))
        .unwrap();
    tables
        .declare_variable(
            &context,
            &builder,
            inner_scope,
            &inner,
            "answer",
            VariableKind::Automatic,
            ty,
            &registry,
            inner_slot,
            4,
        )
        .unwrap();
    tables
        .statement_location(&context, &builder, inner_scope, &add)
        .unwrap();
    let input = builder
        .build_load(context.i32_type(), input_slot, "input.again")
        .unwrap()
        .into_int_value();
    let addend = builder
        .build_load(context.i32_type(), inner_slot, "answer.value")
        .unwrap()
        .into_int_value();
    let result = builder.build_int_add(input, addend, "sum").unwrap();
    builder.build_store(input_slot, result).unwrap();
    tables
        .statement_location(&context, &builder, body_scope, &ret)
        .unwrap();
    let result = builder
        .build_load(context.i32_type(), input_slot, "returned")
        .unwrap();
    builder.build_return(Some(&result)).unwrap();
    builder.unset_current_debug_location();
    let main = module.add_function("main", context.i32_type().fn_type(&[], false), None);
    builder.position_at_end(context.append_basic_block(main, "entry"));
    let result = builder
        .build_call(
            function,
            &[context.i32_type().const_int(40, false).into()],
            "result",
        )
        .unwrap()
        .try_as_basic_value()
        .basic()
        .unwrap();
    builder.build_return(Some(&result)).unwrap();
    tables.finish();
    module.verify().unwrap();
    let ir = module.print_to_string().to_string();
    assert!(ir.contains("DILexicalBlock"), "{ir}");
    assert!(ir.contains("name: \"input\", arg: 1"), "{ir}");
    let object = scratch.0.join("locals.o");
    NativeTarget::new()
        .unwrap()
        .write_object(&module, &object)
        .unwrap();
    let output = debug_tool_command("llvm-dwarfdump")
        .args(["--debug-info", "--debug-line"])
        .arg(&object)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let dwarf = String::from_utf8(output.stdout).unwrap();
    assert!(dwarf.contains("DW_TAG_formal_parameter"), "{dwarf}");
    assert!(dwarf.contains("DW_TAG_lexical_block"), "{dwarf}");
    assert_eq!(
        dwarf.matches("DW_AT_name\t(\"answer\")").count(),
        2,
        "{dwarf}"
    );
    for line in [2, 4, 5, 7] {
        assert!(
            dwarf.lines().any(|row| {
                let cols: Vec<_> = row.split_whitespace().collect();
                cols.first().is_some_and(|v| v.starts_with("0x"))
                    && cols.get(1).is_some_and(|v| *v == line.to_string())
            }),
            "missing source line {line}: {dwarf}"
        );
    }
    let executable = scratch.0.join("locals");
    let output = clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}
