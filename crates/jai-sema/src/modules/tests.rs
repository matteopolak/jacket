use super::*;
use jai_modules::GraphOptions;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new(files: &[(&str, &str)]) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let directory = std::env::temp_dir().join(format!(
            "jai-scoped-sema-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        for (path, text) in files {
            let path = directory.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        Self(directory)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(
            &self.0.join("main.jai"),
            GraphOptions {
                import_dirs: vec![self.0.join("modules")],
            },
        )
        .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn returned_integer(procedure: &Procedure) -> &IntExpr {
    let Statement::Exit(exit) = procedure.body.statements.last().unwrap() else {
        panic!("expected return")
    };
    let Transfer::ReturnInt(value) = &exit.transfer else {
        panic!("expected integer return")
    };
    value
}
fn integer_call(value: &IntExpr) -> &Call {
    match value.kind() {
        IntExprKind::Call(call) => call,
        IntExprKind::Value(value) => match value.as_ref() {
            ValueExpr::Call { call, .. } => call,
            _ => panic!("expected integer call"),
        },
        _ => panic!("expected integer call"),
    }
}
fn integer_load(value: &IntExpr) -> Place {
    match value.kind() {
        IntExprKind::Load(place) => place.place(),
        IntExprKind::Value(value) => match value.as_ref() {
            ValueExpr::Load(place) => *place,
            _ => panic!("expected integer load"),
        },
        _ => panic!("expected integer load"),
    }
}
fn stored_load(statement: &Statement) -> (Place, Place) {
    let Statement::Store(target, value) = statement else {
        panic!("expected snapshot store")
    };
    let source = match value {
        ValueExpr::Load(place) => *place,
        ValueExpr::Int(value) => integer_load(value),
        _ => panic!("expected loaded snapshot"),
    };
    (*target, source)
}
#[test]
fn same_spelling_has_distinct_declaration_storage_and_procedure_identity() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "A :: #import \"A\"; B :: #import \"B\"; value := A.VALUE + B.VALUE; main :: () -> int { return A.answer() + B.answer(); }",
        ),
        (
            "modules/A.jai",
            "VALUE :: 10; value := VALUE; answer :: () -> int { return value; }",
        ),
        (
            "modules/B.jai",
            "VALUE :: 20; value := VALUE; answer :: () -> int { return value; }",
        ),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    let initializers: Vec<_> = program
        .globals()
        .iter()
        .map(|global| match global.initializer() {
            GlobalInitializer::Int(value) => value.value(),
            _ => panic!("expected integer global"),
        })
        .collect();
    assert_eq!(initializers, vec![30, 10, 20]);
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let IntExprKind::Binary(_, left, right) =
        returned_integer(&program.procedures()[main.index()]).kind()
    else {
        panic!()
    };
    let (left, right) = (integer_call(left), integer_call(right));
    assert_ne!(left.procedure, right.procedure);
    for call in [left, right] {
        let place = integer_load(returned_integer(
            &program.procedures()[call.procedure.index()],
        ));
        assert!(matches!(place.kind(), PlaceKind::Global(_)));
    }
}
#[test]
fn cross_module_constant_dependencies_and_defaults_use_defining_file() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Math :: #import \"Math\"; VALUE :: 99; result := Math.TOTAL; main :: () -> int { return Math.answer(); }",
        ),
        (
            "modules/Math.jai",
            "Other :: #import \"Other\"; VALUE :: 5; TOTAL :: Other.VALUE + VALUE; answer :: (amount := TOTAL) -> int { return amount; }",
        ),
        ("modules/Other.jai", "VALUE :: 37;"),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    assert!(
        matches!(program.globals()[0].initializer(), GlobalInitializer::Int(value) if value.value() == 42)
    );
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let call = integer_call(returned_integer(&program.procedures()[main.index()]));
    let ValueExpr::Int(argument) = &call.arguments[0].1 else {
        panic!()
    };
    assert!(matches!(argument.kind(), IntExprKind::Constant(value) if value.value() == 42));
}
#[test]
fn block_constants_resolve_module_paths_and_forward_lexical_dependencies() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Library :: #import \"Library\"; main :: () -> int { return Library.answer(); }",
        ),
        (
            "modules/Library.jai",
            "Other :: #import \"Other\"; VALUE :: 5; answer :: () -> int { LOCAL :: VALUE + Other.VALUE + LATER; LATER :: 1; return LOCAL; }",
        ),
        ("modules/Other.jai", "VALUE :: 36;"),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    let answer = program.procedures().iter().find(|procedure| matches!(returned_integer(procedure).kind(), IntExprKind::Constant(value) if value.value() == 42)).unwrap();
    assert!(answer.parameters.is_empty());
}
#[test]
fn imported_libraries_need_no_main_and_cannot_supply_application_main() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Library :: #import \"Library\"; main :: () -> int { return Library.answer(); }",
        ),
        ("modules/Library.jai", "answer :: () -> int { return 42; }"),
    ]);
    assert!(resolve_graph(&fixture.graph()).is_ok());
    let fixture = Fixture::new(&[
        ("main.jai", "#import \"Library\";"),
        ("modules/Library.jai", "main :: () -> int { return 42; }"),
    ]);
    let graph = fixture.graph();
    assert!(resolve_library(&graph).is_ok());
    assert!(
        resolve_graph(&graph)
            .unwrap_err()
            .message
            .contains("application main")
    );
}
#[test]
fn body_errors_and_constant_cycles_keep_original_source() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Library :: #import \"Library\"; main :: () -> int { return Library.answer(); }",
        ),
        (
            "modules/Library.jai",
            "\nanswer :: () -> int { return missing; }",
        ),
    ]);
    let graph = fixture.graph();
    let error = resolve_graph(&graph).unwrap_err();
    assert!(error.render(graph.sources()).contains("Library.jai:2:"));
    let fixture = Fixture::new(&[
        ("main.jai", "Library :: #import \"Library\"; main :: () {}"),
        ("modules/Library.jai", "LEFT :: RIGHT;\nRIGHT :: LEFT;"),
    ]);
    let graph = fixture.graph();
    let error = resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("cyclic constant"));
    assert!(error.render(graph.sources()).contains("Library.jai:2:"));
}
#[test]
fn qualified_privacy_and_local_namespace_shadowing_fail_at_use_site() {
    for source in [
        "return Library.hidden;",
        "Library := 1; return Library.answer();",
        "return Library.file_only;",
    ] {
        let fixture = Fixture::new(&[
            (
                "main.jai",
                &format!("Library :: #import \"Library\"; main :: () -> int {{ {source} }}"),
            ),
            (
                "modules/Library.jai",
                "#scope_module hidden :: 1; #scope_file file_only :: 2; #scope_export answer :: () -> int { return hidden + file_only; }",
            ),
        ]);
        let graph = fixture.graph();
        let error = resolve_graph(&graph).unwrap_err();
        assert!(
            error.render(graph.sources()).contains("main.jai:1:"),
            "{error}"
        );
    }
}

#[test]
fn file_private_constants_keep_each_loaded_procedures_defining_scope() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "#scope_file VALUE :: 1; #scope_export #load \"helper.jai\"; main :: () -> int { return answer(); }",
        ),
        (
            "helper.jai",
            "#scope_file VALUE :: 42; #scope_export answer :: () -> int { return VALUE; }",
        ),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let call = integer_call(returned_integer(&program.procedures()[main.index()]));
    assert!(
        matches!(returned_integer(&program.procedures()[call.procedure.index()]).kind(), IntExprKind::Constant(value) if value.value() == 42)
    );
}

#[test]
fn one_source_loaded_and_imported_has_independent_runtime_storage() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "#load \"shared.jai\"; Library :: #import,file \"shared.jai\"; main :: () -> int { return answer() + Library.answer(); }",
        ),
        (
            "shared.jai",
            "value := 0; answer :: () -> int { return value; }",
        ),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    assert_eq!(program.globals().len(), 2);
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let IntExprKind::Binary(_, left, right) =
        returned_integer(&program.procedures()[main.index()]).kind()
    else {
        panic!()
    };
    let (left, right) = (integer_call(left), integer_call(right));
    let places: Vec<_> = [left, right]
        .into_iter()
        .map(|call| {
            integer_load(returned_integer(
                &program.procedures()[call.procedure.index()],
            ))
        })
        .collect();
    assert_ne!(places[0], places[1]);
}

#[test]
fn nominal_record_arguments_results_copies_and_field_mutation_are_checked() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Point :: struct {x:int; y:int=7;} Alias :: Point; change :: (p:Alias)->Point {p.x += 2; return p;} main :: ()->int {original:Point=.{x=40}; copy:=change(original); return copy.x + original.y - 7;}",
    )]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    assert!(!program.places().is_empty());
    let signature = program
        .types()
        .procedure_definition(program.procedures()[0].signature)
        .unwrap();
    assert_eq!(signature.parameters[0], signature.results[0]);
    assert!(matches!(
        program.types().kind(signature.results[0]).unwrap(),
        TypeKind::Record(_)
    ));
}

#[test]
fn record_default_uses_declaration_file_and_initializer_effect_order_is_explicit() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "Math :: #import \"Math\"; OFFSET :: 100; ticks:=0; bump::()->int{ticks+=1;return ticks;} main::()->int{p:=Math.Pair.{b=bump(),a=bump()}; return p.a*10+p.b;}",
        ),
        (
            "modules/Math.jai",
            "#scope_file OFFSET :: 7; #scope_export Pair :: struct {a:int=OFFSET;b:int=OFFSET+1;}",
        ),
    ]);
    let graph = fixture.graph();
    let program = resolve_graph(&graph).unwrap();
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let Statement::Store(_, ValueExpr::RecordBuild { initializers, .. }) =
        &program.procedures()[main.index()].body.statements[0]
    else {
        panic!("expected typed record store")
    };
    assert_eq!(initializers[0].0.index(), 1);
    assert_eq!(initializers[1].0.index(), 0);
}

#[test]
fn enum_nominality_comparisons_flags_and_explicit_integer_conversion_are_checked() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "State::enum u8 {ZERO; READY::5;} Bits::enum_flags u8 {A::1; B::2;} main::()->int{s:State=State.READY; f:=Bits.A|Bits.B; if s==State.READY {return cast(int)s+cast(int)f;} return 0;}",
    )]);
    assert!(resolve_graph(&fixture.graph()).is_ok());
    let fixture = Fixture::new(&[(
        "main.jai",
        "A::enum {ZERO;} B::enum {ZERO;} main::()->int{a:A=A.ZERO;b:B=B.ZERO; if a==b{return 1;} return 0;}",
    )]);
    assert!(
        resolve_graph(&fixture.graph())
            .unwrap_err()
            .message
            .contains("same nominal")
    );
}

#[test]
fn ordered_results_use_one_call_and_snapshots_before_destination_writes() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "ticks:=0; pair::()->(first:int,second:int){ticks+=1;return second=2,first=1;} main::()->int{a,b:=pair();a,b=b,a;return a*10+b+ticks;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let Statement::Block(declaration) = &program.procedures()[main.index()].body.statements[0]
    else {
        panic!()
    };
    assert_eq!(
        declaration
            .statements
            .iter()
            .filter(|statement| matches!(statement, Statement::CallResults { .. }))
            .count(),
        1
    );
    let Statement::CallResults { destinations, .. } = &declaration.statements[0] else {
        panic!()
    };
    assert_eq!(destinations.len(), 2);
    assert!(destinations.iter().all(Option::is_some));
    let Statement::Block(assignment) = &program.procedures()[main.index()].body.statements[1]
    else {
        panic!()
    };
    assert_eq!(assignment.statements.len(), 6);
    assert!(matches!(
        &assignment.statements[0],
        Statement::Store(_, ValueExpr::AddressOf { .. })
    ));
    assert!(matches!(
        &assignment.statements[1],
        Statement::Store(_, ValueExpr::AddressOf { .. })
    ));
    let (first_snapshot, first_source) = stored_load(&assignment.statements[2]);
    let (second_snapshot, second_source) = stored_load(&assignment.statements[3]);
    assert_ne!(first_source, second_source);
    assert_ne!(first_snapshot, second_snapshot);
    assert_eq!(stored_load(&assignment.statements[4]).1, first_snapshot);
    assert_eq!(stored_load(&assignment.statements[5]).1, second_snapshot);
}

#[test]
fn flags_default_to_zero_without_a_named_zero_member() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Bits::enum_flags u32 {A::1;B::2;} global_flags:Bits; Holder::struct{flags:Bits;} main::()->int{local:Bits;h:Holder;return cast(int)global_flags+cast(int)local+cast(int)h.flags;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let GlobalInitializer::Value(value) = program.globals()[0].initializer() else {
        panic!()
    };
    assert!(matches!(value.kind,jai_ir::ConstantKind::Enum(value) if value.value()==0));
}

#[test]
fn contextual_record_conditionals_preserve_the_nominal_type() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Point::struct{x:int=7;} ticks:=0; bump::()->int{ticks+=1;return ticks;} main::()->int{p:Point=ifx false then .{x=bump()} else .{x=40};return p.x+ticks;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let EntryPoint::Int(main) = program.entry() else {
        panic!()
    };
    let Statement::Store(_, ValueExpr::Conditional { ty, expression }) =
        &program.procedures()[main.index()].body.statements[0]
    else {
        panic!()
    };
    assert!(matches!(
        program.types().kind(*ty).unwrap(),
        TypeKind::Record(_)
    ));
    assert!(matches!(
        expression.then_value,
        ValueExpr::RecordBuild { .. }
    ));
    assert!(matches!(
        expression.else_value,
        ValueExpr::RecordBuild { .. }
    ));
}

#[test]
fn multiple_local_nominal_declarations_keep_field_defaults_and_copy_storage() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "main::()->int{Point::struct{x:int=7;} a,b:Point; a.x=20; return a.x+b.x;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("execution failed: {execution:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result")
    };
    assert_eq!(value.value(), 27);
}

#[test]
fn forward_enum_constant_aliases_preserve_nominality_in_globals_and_defaults() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Bits::enum_flags u8{A::1;B::2;} COPY::MASK; MASK::Bits.A|Bits.B; global:=COPY; Box::struct{flags:Bits=MASK;} main::()->int{b:Box; f:Bits=global; if f{return cast(int)f+cast(int)b.flags;}return 0;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("execution failed: {execution:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result")
    };
    assert_eq!(value.value(), 6);
}

#[test]
fn file_constant_comparisons_reject_different_enum_identities() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "A::enum{ZERO;} B::enum{ZERO;} SAME::A.ZERO==B.ZERO; main::()->int{return 0;}",
    )]);
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("same nominal"), "{error}");
    assert!(
        error
            .render(fixture.graph().sources())
            .contains("main.jai:1:")
    );
}

#[test]
fn nominal_constant_dependency_cycles_have_source_diagnostics() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Bits::enum_flags{ONE::1;} A::B|Bits.ONE; B::A; main::()->int{return 0;}",
    )]);
    let graph = fixture.graph();
    let error = resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains("cyclic nominal"), "{error}");
    assert!(error.render(graph.sources()).contains("main.jai:1:"));
}

#[test]
fn inferred_global_enum_operations_and_contextual_members_are_nominal() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Bits::enum_flags u8{A::1;B::2;} global:=Bits.A|Bits.B; typed:Bits=.A; main::()->int{return cast(int)global+cast(int)typed;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("execution failed: {execution:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result")
    };
    assert_eq!(value.value(), 4);
    let fixture = Fixture::new(&[(
        "main.jai",
        "A::enum_flags u8{ONE::1;} B::enum_flags u8{ONE::1;} global:=A.ONE|B.ONE; main::()->int{return 0;}",
    )]);
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("same nominal"), "{error}");
}

#[test]
fn typed_record_constants_with_enum_fields_keep_the_composite_phase() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Bits::enum_flags u8{A::1;B::2;} Box::struct{flags:Bits;} OPTIONS::Box.{flags=Bits.B}; ALIAS::OPTIONS; main::()->int{return cast(int)ALIAS.flags;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("execution failed: {execution:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result")
    };
    assert_eq!(value.value(), 2);
}

#[test]
fn inferred_record_fields_retain_enum_alias_types_and_defaults() {
    let fixture = Fixture::new(&[(
        "main.jai",
        "Bits::enum_flags u8{A::1;B::2;} MASK::ALIAS; ALIAS::Bits.A|Bits.B; Box::struct{flags:=MASK;} main::()->int{box:Box; box.flags=.B; return cast(int)box.flags;}",
    )]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("execution failed: {execution:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result")
    };
    assert_eq!(value.value(), 2);
    let fixture = Fixture::new(&[(
        "main.jai",
        "A::enum_flags u8{ONE::1;} B::enum_flags u8{ONE::1;} MASK::A.ONE; Box::struct{flags:=MASK;} main::()->int{box:Box; box.flags=B.ONE; return 0;}",
    )]);
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("nominal"), "{error}");
}

#[test]
fn qualified_enum_constant_field_inference_uses_the_defining_module_identity() {
    let fixture = Fixture::new(&[
        (
            "main.jai",
            "L::#import \"Lib\"; Box::struct{flags:=L.MASK;} main::()->int{box:Box;box.flags=.B;return cast(int)box.flags;}",
        ),
        (
            "modules/Lib/module.jai",
            "#scope_export Bits::enum_flags u8{A::1;B::2;} MASK::Bits.A|Bits.B;",
        ),
    ]);
    let program = resolve_graph(&fixture.graph()).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("execution failed: {execution:?}")
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("expected integer result")
    };
    assert_eq!(value.value(), 2);
}
