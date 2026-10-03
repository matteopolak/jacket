use jai_ir::{BlockPath, DebugBranch, DebugPathStep, LocalDeclaration, Program, StatementPath};
use jai_modules::{GraphOptions, ModuleGraph, SourceOverlay};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use std::path::Path;

fn graph(files: &[(&str, &str)]) -> ModuleGraph {
    let mut overlay = SourceOverlay::new();
    for (name, text) in files {
        overlay
            .insert(
                &Path::new("/jai-debug-sources").join(name),
                text.as_bytes().to_vec(),
            )
            .unwrap();
    }
    ModuleGraph::load_with_provider(
        Path::new("/jai-debug-sources/main.jai"),
        GraphOptions::default(),
        &overlay,
    )
    .unwrap()
}

fn resolve(files: &[(&str, &str)]) -> (ModuleGraph, Program) {
    let graph = graph(files);
    let program = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    (graph, program)
}

fn text<'a>(graph: &'a ModuleGraph, location: &jai_ir::DebugSourceLocation) -> &'a str {
    location
        .span()
        .span
        .text(graph.sources().get(location.span().source).unwrap().text())
}

fn stored_place(statement: &jai_ir::Statement) -> Option<jai_ir::Place> {
    match statement {
        jai_ir::Statement::Store(place, _) => Some(*place),
        jai_ir::Statement::StoreInt(place, _) => Some(place.place()),
        jai_ir::Statement::StoreBool(place, _) => Some(place.place()),
        _ => None,
    }
}

fn targets_local(statement: &jai_ir::Statement, local: jai_ir::LocalId) -> bool {
    if stored_place(statement).is_some_and(|place| place.kind() == jai_ir::PlaceKind::Local(local))
    {
        return true;
    }
    match statement {
        jai_ir::Statement::Store(_, jai_ir::ValueExpr::AddressOf { place, .. }) => {
            place.kind() == jai_ir::PlaceKind::Local(local)
        }
        jai_ir::Statement::Block(block) => block
            .statements
            .iter()
            .any(|statement| targets_local(statement, local)),
        _ => false,
    }
}

#[test]
fn actual_statement_paths_preserve_declaration_shadowing_and_nested_scopes() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "// é\r\nmain :: () -> int {\r\n value := 1;\r\n { value := 2; value += 3; }\r\n return value;\r\n}",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let main = program
        .procedures()
        .iter()
        .find(|p| sources.procedure(p.id).unwrap().name == "main")
        .unwrap();
    let root = BlockPath::procedure(main.id);
    let nested = root.child(1, DebugBranch::Block);
    assert_eq!(
        text(
            &graph,
            sources
                .statement(&StatementPath::in_block(&root, 0))
                .unwrap()
        ),
        "value := 1;"
    );
    assert_eq!(
        text(
            &graph,
            sources
                .statement(&StatementPath::in_block(&nested, 0))
                .unwrap()
        ),
        "value := 2;"
    );
    assert_eq!(
        sources
            .statement(&StatementPath::in_block(&root, 0))
            .unwrap()
            .line()
            .get(),
        3
    );
    assert_eq!(
        sources
            .statement(&StatementPath::in_block(&root, 0))
            .unwrap()
            .column()
            .get(),
        2
    );
    let outer = sources.local(main.locals[0].id()).unwrap();
    let inner = sources.local(main.locals[1].id()).unwrap();
    assert_eq!(outer.name, "value");
    assert_eq!(inner.name, "value");
    assert_eq!(outer.scope, root);
    assert_eq!(inner.scope, nested);
    assert_eq!(
        inner.declaration,
        LocalDeclaration::Statement(StatementPath::in_block(&nested, 0))
    );
}

#[test]
fn cleanup_and_generated_sequence_prefixes_use_their_actual_emitted_paths() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "main :: () -> int {\n total := 0;\n data: [2]int = .[1,2];\n for value,index: data {\n  defer total += 10;\n  total += value + index;\n }\n return total;\n}",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let main = &program.procedures()[0];
    let sequence = match &main.body.statements[2] {
        jai_ir::Statement::Block(block) => block,
        _ => panic!("generated sequence block"),
    };
    let while_index = sequence.statements.len() - 1;
    let body = match &sequence.statements[while_index] {
        jai_ir::Statement::While { body, .. } => body,
        _ => panic!("generated sequence while"),
    };
    let total = sources
        .locals()
        .find(|(_, source)| source.name == "total")
        .unwrap()
        .0;
    let real_index = body
        .statements
        .iter()
        .position(|statement| targets_local(statement, total))
        .unwrap();
    let body_path = BlockPath::procedure(main.id)
        .child(2, DebugBranch::Block)
        .child(while_index, DebugBranch::While);
    let index = sources
        .locals()
        .find_map(|(_, source)| (source.name == "index").then_some(source))
        .unwrap();
    assert_eq!(index.scope, body_path);
    assert_eq!(
        index.declaration,
        LocalDeclaration::Statement(StatementPath::in_block(
            &BlockPath::procedure(main.id).child(2, DebugBranch::Block),
            while_index
        ))
    );
    assert_eq!(
        text(
            &graph,
            sources
                .statement(&StatementPath::in_block(&body_path, real_index))
                .unwrap()
        ),
        "total += value + index;"
    );
    assert!(
        text(
            &graph,
            sources
                .statement(&StatementPath::in_block(&body_path, 0))
                .unwrap()
        )
        .starts_with("for value,index: data")
    );
    // The first cleanup is the compiler's iteration latch. The defer is the next root.
    let cleanup = BlockPath::cleanup(main.id, jai_ir::CleanupId::new(1));
    assert_eq!(
        sources.cleanup_parent(main.id, jai_ir::CleanupId::new(1)),
        Some(&body_path)
    );
    assert_eq!(
        text(
            &graph,
            sources
                .statement(&StatementPath::in_block(&cleanup, 0))
                .unwrap()
        ),
        "total += 10;"
    );
    assert!(
        matches!(jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 24))
    );
}

#[test]
fn parameters_and_range_bindings_keep_actual_storage_identity_and_source_ordinal() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "sum :: (first: int, second: int) -> int {\n result := first;\n for index: 1..second { result += index; }\n return result;\n}\nmain :: () -> int { return sum(1,2); }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let sum = program
        .procedures()
        .iter()
        .find(|p| sources.procedure(p.id).unwrap().name == "sum")
        .unwrap();
    for (ordinal, parameter) in sum.parameters.iter().enumerate() {
        let source = sources.local(parameter.id()).unwrap();
        assert_eq!(source.declaration, LocalDeclaration::Parameter(ordinal));
        assert_eq!(source.scope, BlockPath::procedure(sum.id));
        assert!(text(&graph, &source.location).starts_with(&source.name));
    }
    let index = sources.local(sum.locals[3].id()).unwrap();
    assert_eq!(index.name, "index");
    assert_eq!(
        index.scope,
        BlockPath::procedure(sum.id).child(1, DebugBranch::Range)
    );
}

#[test]
fn current_scope_insertion_keeps_quoted_source_bytes_from_another_file() {
    let (graph, program) = resolve(&[
        (
            "main.jai",
            "#load \"quote.jai\";\nmain :: () -> int {\n value := 1;\n #insert,scope() BODY;\n return value;\n}",
        ),
        (
            "quote.jai",
            "// original quote\nBODY :: #code {\n value += 41;\n};",
        ),
    ]);
    let sources = program.library().debug_sources().unwrap();
    let main = &program.procedures()[0];
    let path = BlockPath::procedure(main.id).child(1, DebugBranch::Block);
    let location = sources
        .statement(&StatementPath::in_block(&path, 0))
        .unwrap();
    assert_eq!(location.path(), Path::new("/jai-debug-sources/quote.jai"));
    assert_eq!(location.line().get(), 3);
    assert_eq!(text(&graph, location), "value += 41;");
    assert_eq!(
        path.steps.as_ref(),
        &[
            DebugPathStep::Statement(1),
            DebugPathStep::Child(DebugBranch::Block)
        ]
    );
}

#[test]
fn selected_static_branches_record_the_flattened_emission_and_original_range() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "main :: () -> int {\n result := 0;\n #if true {\n  result += 42;\n } else { missing = 7; }\n return result;\n}",
    )]);
    let main = &program.procedures()[0];
    let sources = program.library().debug_sources().unwrap();
    let path = StatementPath::in_block(&BlockPath::procedure(main.id), 1);
    let location = sources.statement(&path).unwrap();
    assert_eq!(location.line().get(), 4);
    assert_eq!(text(&graph, location), "result += 42;");
    let result = sources
        .locals()
        .find(|(_, source)| source.name == "result")
        .unwrap()
        .0;
    assert!(
        targets_local(&main.body.statements[1], result),
        "selected source assignment writes the named result local"
    );
    assert!(matches!(
        jai_vm::execute(&program, jai_vm::Limits::default()).outcome,
        jai_vm::Outcome::Complete(values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42)
    ));
}

#[test]
fn macro_body_and_runtime_argument_prefix_keep_their_separate_source_origins() {
    let (graph, program) = resolve(&[
        (
            "main.jai",
            "#load \"macro.jai\";\nmain :: () -> int {\n value := 1;\n bump(value);\n return value;\n}",
        ),
        (
            "macro.jai",
            "bump :: (argument: int) #expand {\n temporary := argument + 1;\n}",
        ),
    ]);
    let sources = program.library().debug_sources().unwrap();
    let main = program
        .procedures()
        .iter()
        .find(|p| sources.procedure(p.id).unwrap().name == "main")
        .unwrap();
    let body = BlockPath::procedure(main.id).child(1, DebugBranch::Block);
    let initializer = sources
        .statement(&StatementPath::in_block(&body, 0))
        .unwrap();
    assert_eq!(initializer.path(), Path::new("/jai-debug-sources/main.jai"));
    assert_eq!(initializer.line().get(), 4);
    assert_eq!(text(&graph, initializer), "value");
    let temporary = sources
        .statement(&StatementPath::in_block(&body, 1))
        .unwrap();
    assert_eq!(temporary.path(), Path::new("/jai-debug-sources/macro.jai"));
    assert_eq!(temporary.line().get(), 2);
    assert_eq!(text(&graph, temporary), "temporary := argument + 1;");
    let argument = sources
        .locals()
        .find_map(|(_, source)| (source.name == "argument").then_some(source))
        .unwrap();
    assert_eq!(argument.scope, body);
    assert_eq!(
        argument.location.path(),
        Path::new("/jai-debug-sources/macro.jai")
    );
    assert_eq!(argument.location.line().get(), 1);
    assert_eq!(
        argument.declaration,
        LocalDeclaration::Statement(StatementPath::in_block(&body, 0))
    );
}

#[test]
fn generated_return_operations_inherit_the_actual_triggering_statement() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "pair :: () -> (int,int) {\n return 20,22;\n}\nmain :: () -> int { first,second := pair(); return first + second; }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let pair = program
        .procedures()
        .iter()
        .find(|p| sources.procedure(p.id).unwrap().name == "pair")
        .unwrap();
    let generated = BlockPath::procedure(pair.id).child(0, DebugBranch::Block);
    let body = match &pair.body.statements[0] {
        jai_ir::Statement::Block(body) => body,
        _ => panic!("multi-return emission block"),
    };
    for index in 0..body.statements.len() {
        let location = sources
            .statement(&StatementPath::in_block(&generated, index))
            .unwrap();
        assert_eq!(location.line().get(), 2);
        assert_eq!(text(&graph, location), "return 20,22;");
    }
}

#[test]
fn nested_cleanups_retain_their_declaration_scopes_across_cleanup_roots() {
    let (_, program) = resolve(&[(
        "main.jai",
        "main :: () -> int { value := 0; { defer { temporary := 1; defer value += temporary; value += 2; } value += 3; } return value; }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let main = &program.procedures()[0];
    let inner = jai_ir::CleanupId::new(0);
    let outer = jai_ir::CleanupId::new(1);
    assert_eq!(
        sources.cleanup_parent(main.id, outer),
        Some(&BlockPath::procedure(main.id).child(1, DebugBranch::Block))
    );
    assert_eq!(
        sources.cleanup_parent(main.id, inner),
        Some(&BlockPath::cleanup(main.id, outer))
    );
}

#[test]
fn specialized_and_nested_procedures_publish_their_real_body_source() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "same :: (value: $T) -> T {\n copy: T = value;\n return copy;\n}\nmain :: () -> int {\n nested :: (input: int) -> int {\n  answer := input + 1;\n  return answer;\n }\n small: u8 = 1;\n return same(20) + cast(int) same(small) + nested(20);\n}",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let specializations: Vec<_> = program
        .procedures()
        .iter()
        .filter(|p| sources.procedure(p.id).unwrap().name == "same")
        .collect();
    assert_eq!(specializations.len(), 2);
    assert_ne!(specializations[0].id, specializations[1].id);
    for procedure in specializations {
        let location = sources
            .statement(&StatementPath::in_block(
                &BlockPath::procedure(procedure.id),
                0,
            ))
            .unwrap();
        assert_eq!(location.line().get(), 2);
        assert_eq!(text(&graph, location), "copy: T = value;");
        assert_eq!(
            sources
                .local(procedure.parameters[0].id())
                .unwrap()
                .declaration,
            LocalDeclaration::Parameter(0)
        );
    }
    let nested = program
        .procedures()
        .iter()
        .find(|p| sources.procedure(p.id).unwrap().name == "nested")
        .unwrap();
    let location = sources
        .statement(&StatementPath::in_block(
            &BlockPath::procedure(nested.id),
            0,
        ))
        .unwrap();
    assert_eq!(location.line().get(), 7);
    assert_eq!(text(&graph, location), "answer := input + 1;");
    assert_eq!(
        sources.local(nested.parameters[0].id()).unwrap().name,
        "input"
    );
}

#[test]
fn string_iterator_declaration_addresses_its_actual_generated_initializer() {
    let (_, program) = resolve(&[(
        "main.jai",
        "main :: () -> int { total := 0; for character,index: \"AB\" { total += cast(int) character + index; } return total; }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let (_, character) = sources
        .locals()
        .find(|(_, source)| source.name == "character")
        .unwrap();
    let LocalDeclaration::Statement(declaration) = &character.declaration else {
        panic!("local iterator declaration");
    };
    let mut containing = declaration.steps.to_vec();
    containing.pop();
    assert_eq!(
        character.scope,
        BlockPath {
            root: declaration.root,
            steps: containing.into()
        }
    );
    assert_eq!(declaration.steps.last(), Some(&DebugPathStep::Statement(1)));
    assert_eq!(character.location.line().get(), 1);
}

#[test]
fn branch_locals_keep_their_exact_emitted_child_scopes() {
    let (_, program) = resolve(&[(
        "main.jai",
        "pick :: (flag: bool) -> int { result := 0; if flag { branch := 1; result = branch; } else { branch := 2; result = branch; } return result; }\nmain :: () -> int { return pick(true); }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let pick = program
        .procedures()
        .iter()
        .find(|procedure| sources.procedure(procedure.id).unwrap().name == "pick")
        .unwrap();
    let expected = [
        BlockPath::procedure(pick.id).child(1, DebugBranch::IfThen),
        BlockPath::procedure(pick.id).child(1, DebugBranch::IfElse),
    ];
    let branches: Vec<_> = pick
        .locals
        .iter()
        .filter_map(|local| sources.local(local.id()))
        .filter(|source| source.name == "branch")
        .collect();
    assert_eq!(branches.len(), 2);
    for (branch, scope) in branches.into_iter().zip(expected) {
        assert_eq!(branch.scope, scope);
        assert_eq!(
            branch.declaration,
            LocalDeclaration::Statement(StatementPath::in_block(&scope, 0))
        );
    }
}

#[test]
fn quoted_run_failure_retains_its_original_file_after_context_restoration() {
    let graph = graph(&[
        (
            "main.jai",
            "#load \"quote.jai\";\nmain :: () {\n #insert,scope() BODY;\n}",
        ),
        (
            "quote.jai",
            "BODY :: #code {\n #run fail();\n};\nfail :: () { value := 1 / 0; }",
        ),
    ]);
    let error = resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap_err();
    let origin = graph.sources().get(error.location.source).unwrap();
    assert_eq!(origin.path(), Path::new("/jai-debug-sources/quote.jai"));
    assert_eq!(error.location.span.text(origin.text()), "#run fail()");
    assert_eq!(
        error.message,
        "invalid compile-time arithmetic: ZeroDivisor"
    );
}

#[test]
fn quoted_statement_error_retains_the_complete_original_source_range() {
    let graph = graph(&[
        (
            "main.jai",
            "#load \"quote.jai\";\nmain :: () { #insert,scope() BODY; }",
        ),
        (
            "quote.jai",
            "// é\r\nBODY :: #code {\r\n missing = 1;\r\n};",
        ),
    ]);
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    let origin = graph.sources().get(error.location.source).unwrap();
    assert_eq!(origin.path(), Path::new("/jai-debug-sources/quote.jai"));
    assert_eq!(error.location.span.text(origin.text()), "missing = 1;");
    assert!(error.render(graph.sources()).contains("quote.jai:3:2:"));
}

#[test]
fn a_local_macro_defined_by_current_scope_insertion_retains_its_quote_origin() {
    let (graph, program) = resolve(&[
        (
            "main.jai",
            "#load \"quote.jai\";\nmain :: () -> int { #insert,scope() BODY; return 42; }",
        ),
        (
            "quote.jai",
            "BODY :: #code {\n bump :: (argument: int) #expand {\n  temporary := argument + 1;\n }\n bump(41);\n};",
        ),
    ]);
    let sources = program.library().debug_sources().unwrap();
    let (_, temporary) = sources
        .locals()
        .find(|(_, local)| local.name == "temporary")
        .unwrap();
    assert_eq!(
        temporary.location.path(),
        Path::new("/jai-debug-sources/quote.jai")
    );
    assert_eq!(temporary.location.line().get(), 3);
    assert_eq!(
        text(&graph, &temporary.location),
        "temporary := argument + 1;"
    );
    let (_, argument) = sources
        .locals()
        .find(|(_, local)| local.name == "argument")
        .unwrap();
    assert_eq!(
        argument.location.path(),
        Path::new("/jai-debug-sources/quote.jai")
    );
    assert_eq!(argument.location.line().get(), 2);
    let LocalDeclaration::Statement(prefix) = &argument.declaration else {
        panic!("local macro argument initializer");
    };
    let initializer = sources.statement(prefix).unwrap();
    assert_eq!(
        initializer.path(),
        Path::new("/jai-debug-sources/quote.jai")
    );
    assert_eq!(initializer.line().get(), 5);
    assert_eq!(text(&graph, initializer), "41");
}

#[test]
fn named_record_storage_keeps_its_real_registry_type_and_layout() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "Packet :: struct { tag: u8; value: int; }\nAlias :: Packet;\nmain :: () -> int {\n packet: Alias = .{tag=1,value=42};\n return packet.value;\n}",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let main = &program.procedures()[0];
    let packet = main
        .locals
        .iter()
        .find(|local| {
            sources
                .local(local.id())
                .is_some_and(|source| source.name == "packet")
        })
        .unwrap();
    let source = sources.local(packet.id()).unwrap();
    assert_eq!(source.scope, BlockPath::procedure(main.id));
    assert_eq!(source.location.line().get(), 4);
    assert_eq!(
        text(&graph, &source.location),
        "packet: Alias = .{tag=1,value=42};"
    );
    let record = program.types().record_definition(packet.ty()).unwrap();
    let type_source = sources.type_source(packet.ty()).unwrap();
    assert_eq!(type_source.name.as_deref(), Some("Packet"));
    assert_eq!(type_source.location.line().get(), 1);
    assert_eq!(
        text(&graph, &type_source.location),
        "Packet :: struct { tag: u8; value: int; }"
    );
    assert_eq!(record.fields.len(), 2);
    for (index, name, declaration) in [(0, "tag", "tag: u8;"), (1, "value", "value: int;")] {
        let field = program.types().field(packet.ty(), index).unwrap();
        let source = sources.field_source(field.id).unwrap();
        assert_eq!(source.name, name);
        assert_eq!(
            source.location.span().source,
            type_source.location.span().source
        );
        assert_eq!(text(&graph, &source.location), declaration);
    }
    assert!(matches!(
        program.types().kind(record.fields[0]).unwrap(),
        jai_types::TypeKind::Integer(jai_types::IntegerType::U8)
    ));
    assert!(matches!(
        program.types().kind(record.fields[1]).unwrap(),
        jai_types::TypeKind::Integer(jai_types::IntegerType::S64)
    ));
    let mut layouts = jai_types::LayoutEngine::new(program.types(), LayoutPolicy::lp64());
    let layout = layouts.layout(packet.ty()).unwrap();
    assert_eq!(layout.size, 16);
    assert_eq!(layout.alignment, 8);
    assert_eq!(layout.field_offsets.as_ref(), &[0, 8]);
}

#[test]
fn loaded_generic_and_inline_records_keep_definition_names_and_original_field_sources() {
    let (graph, program) = resolve(&[
        (
            "main.jai",
            "#load \"types.jai\";\nmain :: () -> int { value: Box(int); return value.inner.amount; }",
        ),
        (
            "types.jai",
            "// original definitions\nBox :: struct(T: Type) {\n inner: struct { amount: T = 42; };\n}",
        ),
    ]);
    let sources = program.library().debug_sources().unwrap();
    let value = sources
        .locals()
        .find(|(_, local)| local.name == "value")
        .unwrap()
        .0;
    let ty = program
        .procedures()
        .iter()
        .find(|p| p.id == value.procedure())
        .unwrap()
        .locals[value.index()]
    .ty();
    let outer = sources.type_source(ty).unwrap();
    assert_eq!(outer.name.as_deref(), Some("Box"));
    assert_eq!(
        outer.location.path(),
        Path::new("/jai-debug-sources/types.jai")
    );
    assert_eq!(outer.location.line().get(), 2);
    let field = program.types().field(ty, 0).unwrap();
    assert_eq!(sources.field_source(field.id).unwrap().name, "inner");
    let inner = sources.type_source(field.ty).unwrap();
    assert!(inner.name.is_none());
    assert_eq!(inner.location.path(), outer.location.path());
    assert_eq!(text(&graph, &inner.location), "struct { amount: T = 42; }");
    let amount = program.types().field(field.ty, 0).unwrap();
    let source = sources.field_source(amount.id).unwrap();
    assert_eq!(source.name, "amount");
    assert_eq!(text(&graph, &source.location), "amount: T = 42;");
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn callable_aliases_keep_the_original_prototype_source_and_name() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "original :: (value: int) -> int #foreign;\nshortcut :: original;\nsecond :: shortcut;\nmain :: () -> int { return 42; }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let original = program
        .library()
        .prototypes()
        .iter()
        .find_map(|p| sources.procedure(p.id))
        .unwrap();
    assert_eq!(original.name, "original");
    assert_eq!(original.location.line().get(), 1);
    assert_eq!(
        text(&graph, &original.location),
        "original :: (value: int) -> int #foreign;"
    );
    assert!(
        !sources
            .procedures()
            .any(|(_, source)| source.name == "shortcut" || source.name == "second")
    );
}

#[test]
fn no_debug_suppresses_locations_and_locals_while_retaining_real_procedure_origin() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "nested :: () #expand { internal := 0; }\nhidden :: (input: int) -> int #no_debug { value := input + 1; defer value += 0; nested(); return value; }\nmain :: () -> int { answer := hidden(41); return answer; }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    let hidden = program
        .procedures()
        .iter()
        .find(|p| sources.procedure(p.id).unwrap().name == "hidden")
        .unwrap();
    assert_eq!(
        sources.procedure_policy(hidden.id),
        jai_ir::DebugPolicy::Suppress
    );
    assert_eq!(
        text(&graph, &sources.procedure(hidden.id).unwrap().location),
        "hidden :: (input: int) -> int #no_debug { value := input + 1; defer value += 0; nested(); return value; }"
    );
    assert!(
        !sources
            .statements()
            .any(|(path, _)| path.root.procedure() == hidden.id)
    );
    assert!(
        !sources
            .blocks()
            .any(|(path, _)| path.root.procedure() == hidden.id)
    );
    assert!(!sources.locals().any(|(id, _)| id.procedure() == hidden.id));
    assert!(sources.locals().any(|(_, local)| local.name == "answer"));
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    assert!(
        matches!(execution.outcome, jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(), [jai_vm::Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn suppressed_macro_keeps_caller_initializer_source_and_restores_following_capture() {
    let (graph, program) = resolve(&[(
        "main.jai",
        "bump :: (argument: int) #expand #no_debug { temporary := argument + 1; }\nmain :: () -> int { bump(41); answer := 42; return answer; }",
    )]);
    let sources = program.library().debug_sources().unwrap();
    assert!(
        !sources
            .locals()
            .any(|(_, local)| matches!(local.name.as_str(), "argument" | "temporary"))
    );
    assert!(
        sources
            .statements()
            .any(|(_, location)| text(&graph, location) == "41")
    );
    assert!(
        !sources
            .statements()
            .any(|(_, location)| text(&graph, location) == "temporary := argument + 1;")
    );
    assert!(sources.locals().any(|(_, local)| local.name == "answer"));
}

#[test]
fn suppression_does_not_erase_quoted_error_source_identity() {
    let graph = graph(&[
        (
            "main.jai",
            "#load \"quote.jai\";\nmain :: () #no_debug { #insert,scope() BODY; }",
        ),
        ("quote.jai", "BODY :: #code {\n missing = 1;\n};"),
    ]);
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    let original = graph.sources().get(error.location.source).unwrap();
    assert_eq!(original.path(), Path::new("/jai-debug-sources/quote.jai"));
    assert_eq!(error.location.span.text(original.text()), "missing = 1;");
}

#[test]
fn a_suppressed_macro_failure_keeps_its_defining_file_and_statement_range() {
    let graph = graph(&[
        ("main.jai", "#load \"quiet.jai\";\nmain :: () { broken(); }"),
        (
            "quiet.jai",
            "broken :: () #expand #no_debug {\n missing = 1;\n}",
        ),
    ]);
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    let original = graph.sources().get(error.location.source).unwrap();
    assert_eq!(original.path(), Path::new("/jai-debug-sources/quiet.jai"));
    assert_eq!(error.location.span.text(original.text()), "missing = 1;");
}

#[test]
fn local_record_inserted_into_caller_scope_keeps_its_original_type_and_field_source() {
    let (graph, program) = resolve(&[
        (
            "main.jai",
            "#load \"quote.jai\";\nmain :: () -> int { #insert,scope() BODY; return 42; }",
        ),
        (
            "quote.jai",
            "BODY :: #code {\n Local :: struct { value: int; }\n data: Local = .{value=42};\n};",
        ),
    ]);
    let sources = program.library().debug_sources().unwrap();
    let data = sources
        .locals()
        .find(|(_, source)| source.name == "data")
        .unwrap()
        .0;
    let procedure = program
        .procedures()
        .iter()
        .find(|p| p.id == data.procedure())
        .unwrap();
    let ty = procedure.locals[data.index()].ty();
    let type_source = sources.type_source(ty).unwrap();
    assert_eq!(type_source.name.as_deref(), Some("Local"));
    assert_eq!(
        type_source.location.path(),
        Path::new("/jai-debug-sources/quote.jai")
    );
    assert_eq!(
        text(&graph, &type_source.location),
        "Local :: struct { value: int; }"
    );
    let field = program.types().field(ty, 0).unwrap();
    let field_source = sources.field_source(field.id).unwrap();
    assert_eq!(field_source.name, "value");
    assert_eq!(
        field_source.location.span().source,
        type_source.location.span().source
    );
    assert_eq!(text(&graph, &field_source.location), "value: int;");
}
