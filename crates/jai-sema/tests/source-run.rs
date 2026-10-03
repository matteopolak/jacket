use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-source-run-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn forward_compile_time_call_is_embedded_as_a_constant() {
    let fixture = Fixture::new(
        "main :: () -> int { return #run factorial(5); } factorial :: (n:int) -> int { result := 1; for i: 1..n { result *= i; } return result; }",
    );
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap();
    let main = program.procedures().iter().find(|procedure| matches!(program.entry(), jai_ir::EntryPoint::Int(id) if id == procedure.id)).unwrap();
    assert!(
        !format!("{:?}", main.body).contains("Call"),
        "#run call must be removed from runtime IR"
    );
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 120))
    );
}

#[test]
fn file_and_procedure_run_calls_execute_during_compilation() {
    for source in [
        "#run fail(); fail :: () { x := 1 / 0; } main :: () {}",
        "main :: () { #run fail(); } fail :: () { x := 1 / 0; }",
    ] {
        let fixture = Fixture::new(source);
        let error = resolve_graph_with_options(
            &fixture.graph(),
            &ResolveOptions::default(),
            &mut NoEffects,
        )
        .unwrap_err();
        assert!(error.message.contains("ZeroDivisor"), "{error:?}");
    }
}

#[test]
fn compile_time_cycles_and_limits_are_explicit_failures() {
    let fixture = Fixture::new(
        "main :: () -> int { return #run first(); } first :: () -> int { return #run second(); } second :: () -> int { return #run first(); }",
    );
    let error =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap_err();
    assert!(
        error.message.contains("cyclic #run dependencies"),
        "{error:?}"
    );
    let fixture = Fixture::new(
        "main :: () -> int { return #run forever(); } forever :: () -> int { while true {} return 42; }",
    );
    let options = ResolveOptions {
        compile_time_limits: Limits {
            fuel: 100,
            ..Limits::default()
        },
        ..ResolveOptions::default()
    };
    let error = resolve_graph_with_options(&fixture.graph(), &options, &mut NoEffects).unwrap_err();
    assert!(error.message.contains("Fuel"), "{error:?}");
}

#[test]
fn runtime_local_cannot_supply_a_compile_time_argument() {
    let fixture = Fixture::new(
        "double :: (n:int) -> int { return n * 2; } main :: () -> int { n := 21; return #run double(n); }",
    );
    let error =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap_err();
    assert!(
        error.message.contains("local identity")
            || error
                .message
                .contains("local access without an execution frame"),
        "{error:?}"
    );
}

#[test]
fn named_and_transitive_compile_time_constants_use_defining_file_scope() {
    let fixture = Fixture::new(
        "answer :: #run triple(7); final :: answer + 21; main :: () -> int { return final; } triple :: (n:int) -> int { return n*3; }",
    );
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap();
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn file_runs_follow_the_declaration_readiness_phase() {
    let fixture = Fixture::new(
        "counter:int=1; #run { counter += 2; } answer :: #run -> int { return counter; }; main :: () -> int { return answer; }",
    );
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap();
    // Constants bind before file requests: this request sees the initial virtual global.
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 1))
    );
}

#[test]
fn nested_anonymous_results_and_lexical_constants_materialize() {
    let fixture = Fixture::new(
        "main :: () -> int { answer :: #run () -> int { n := 7; return n*6; }; return answer; }",
    );
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap();
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn file_runs_schedule_new_generic_bodies_and_specialized_runs_are_distinct() {
    let fixture = Fixture::new(
        "#run twice(2); twice :: (x:$T)->T { return x+x; } baked :: ($n:int)->int { return #run () -> int { return n*2; }; } main :: ()->int { return baked(3)+baked(18); }",
    );
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap();
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 42))
    );
}

#[test]
fn committed_anonymous_runs_share_virtual_global_state() {
    let fixture = Fixture::new(
        "counter:int=1; main :: ()->int { first :: #run ->int { counter+=1; return counter; }; second :: #run ->int { return counter; }; return first+second; }",
    );
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap();
    assert!(
        matches!(jai_vm::execute(&program, Limits::default()).outcome, Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value() == 4))
    );
}

#[test]
fn nested_anonymous_runs_keep_their_checked_binding_owner() {
    let fixture = Fixture::new(
        "Pair::struct{left:int;right:int;} Outer::struct{using pair:Pair;} main::()->int{return #run ->int{value::#run Outer.{left=20,right=22};return value.left+value.right;};}",
    );
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap();
    assert!(matches!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(&values[..], [Value::Int(value)] if value.value() == 42)
    ));
}

#[test]
fn replay_metadata_is_stable_when_a_source_rebuild_appends_declarations() {
    #[derive(Default)]
    struct Origins {
        values: Vec<jai_vm::SourceOrigin>,
        commits: usize,
    }
    impl jai_vm::CompilerEffects for Origins {
        fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
            self.values.push(origin);
        }
        fn begin(&mut self) {
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("pure run has no compiler request");
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            if commit {
                self.commits += 1;
            }
            Ok(())
        }
    }
    let source = "answer :: #run () -> int { return 42; }; main :: ()->int { return answer; }";
    let fixture = Fixture::new(source);
    let mut effects = Origins::default();
    resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut effects).unwrap();
    std::fs::write(
        fixture.0.join("main.jai"),
        format!("{source} appended :: () {{}}"),
    )
    .unwrap();
    resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut effects).unwrap();
    assert_eq!(
        effects.commits, 2,
        "a direct named #run executes once per semantic session"
    );
    assert_eq!(effects.values.len(), 2);
    assert_eq!(effects.values[0], effects.values[1]);
    assert!(effects.values[0].body.starts_with(b"#run"));
}

#[test]
fn invalid_string_materialization_rolls_back_before_effect_finalization() {
    #[derive(Default)]
    struct Effects {
        finished: Vec<bool>,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("fixture is pure");
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            self.finished.push(commit);
            Ok(())
        }
    }
    let fixture = Fixture::new(
        "broken :: ()->string { s:string; s.count=1; return s; } answer :: #run broken(); main :: () {}",
    );
    let mut effects = Effects::default();
    let error =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut effects)
            .unwrap_err();
    assert!(!error.message.is_empty());
    assert!(!effects.finished.is_empty());
    assert!(
        effects.finished.iter().all(|commit| !commit),
        "pending and invalid string runs must both roll back"
    );
}

#[test]
fn address_derived_results_never_publish_or_commit() {
    #[derive(Default)]
    struct Effects(Vec<bool>);
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("pure fixture");
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            self.0.push(commit);
            Ok(())
        }
    }
    for source in [
        "probe::()->u64{x:=1;return cast(u64)*x;}",
        "probe::()->u64{x:=1;p:=*x;return (cast(*u64)*p).*;}",
        "Bits::union{address:u64;bytes:[8]u8;} probe::()->u64{x:=1;bits:Bits;bits.address=cast(u64)*x;copy:=bits;return copy.address;}",
        "Box::struct{bits:u64;} probe::()->Box{x:=1;box:Box;box.bits=cast(u64)*x;return box;}",
        "probe::()->int{values:[3]int=.[1,2,3];x:=1;index:=cast(int)(cast(u64)*x % 3);return values[index];}",
    ] {
        let fixture = Fixture::new(&format!("{source} answer::#run probe(); main::(){{}}"));
        let mut effects = Effects::default();
        let error =
            resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut effects)
                .unwrap_err();
        assert!(
            error.message.contains("address-derived"),
            "{source}: {error:?}"
        );
        assert!(!effects.0.is_empty(), "execution must reach the VM");
        assert!(
            effects.0.iter().all(|commit| !commit),
            "failed publication must roll back"
        );
    }
}

#[test]
fn proven_address_independent_distances_and_alignment_publish() {
    for (source, expected) in [
        (
            "probe::()->int{values:[4]u16;base:=cast(u64)*values[0];last:=cast(u64)*values[3];return cast(int)(last-base);}",
            6,
        ),
        (
            "Aligned::struct{value:int;} #align 64 probe::()->int{item:Aligned;return cast(int)(cast(u64)*item % 64);}",
            0,
        ),
    ] {
        let fixture = Fixture::new(&format!(
            "{source} answer::#run probe(); main::()->int{{return answer;}}"
        ));
        let program = resolve_graph_with_options(
            &fixture.graph(),
            &ResolveOptions::default(),
            &mut NoEffects,
        )
        .unwrap();
        assert!(
            matches!(jai_vm::execute(&program, Limits::default()).outcome,
            Outcome::Complete(values) if matches!(values.as_slice(), [Value::Int(value)] if value.value()==expected))
        );
    }
}

#[test]
fn rejected_outer_call_does_not_execute_a_previewed_generic_body() {
    #[derive(Default)]
    struct Effects(usize);
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.0 += 1;
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("preview cannot execute effects");
        }
        fn finish(&mut self, _: bool) -> Result<(), jai_vm::Error> {
            Ok(())
        }
    }
    let fixture = Fixture::new(
        "nested::(value:$T)->T{return #run ->int{return 7;};} outer::(value:bool)->bool{return value;} main::(){outer(nested(1));}",
    );
    let mut effects = Effects::default();
    let error =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut effects)
            .unwrap_err();
    assert_eq!(
        effects.0, 0,
        "rejected preview must not schedule a generic body"
    );
    assert!(
        error.message.contains("matching overload")
            || error.message.contains("argument")
            || error.message.contains("expected bool value"),
        "{error:?}"
    );

    let source = "compiler_create_workspace::(name:string)->s64 #compiler; invoke::(value:$T,callback:(T)->int)->int{return callback(value);} main::()->int{return invoke(#run compiler_create_workspace(\"child\"),(x)=>true);}";
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..ResolveOptions::default()
    };
    let mut effects = Effects::default();
    let error = resolve_graph_with_options(&graph, &options, &mut effects).unwrap_err();
    assert_eq!(
        effects.0, 0,
        "invalid callback must precede argument effects"
    );
    assert_eq!(
        error.message,
        "argument type cannot be implicitly converted to the parameter type"
    );
    let rejected = source.find("true").unwrap();
    assert_eq!(
        error.location.span,
        jai_source::Span::new(rejected, rejected + "true".len())
    );
}

#[test]
fn local_callback_recheck_reuses_an_independent_committed_recipe() {
    #[derive(Default)]
    struct Effects {
        begins: usize,
        requests: usize,
        commits: usize,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.begins += 1;
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            self.requests += 1;
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(42).unwrap(),
            ))
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            self.commits += usize::from(commit);
            Ok(())
        }
    }
    let source = "compiler_create_workspace::(name:string)->s64 #compiler;Box::struct{value:int;}plain::()->int{return 7;}required::()->int #must{return 9;}seed::()->int{return compiler_create_workspace(\"child\");}main::()->int{operator +::(a:Box,callback:$F)->Box{receipt:=#run seed();callback();return a;}first:=Box.{value=5}+plain;second:=Box.{value=6}+required;return first.value+second.value;}";
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..ResolveOptions::default()
    };
    let mut effects = Effects::default();
    let error = resolve_graph_with_options(&graph, &options, &mut effects).unwrap_err();
    assert!(error.message.contains("#must"), "{error:?}");
    let rejected = source.find("callback();").unwrap();
    assert_eq!(error.location.span.start, rejected, "{error:?}");
    assert_eq!(
        (effects.begins, effects.requests, effects.commits),
        (1, 1, 1)
    );
}

#[test]
fn cached_recipe_reports_the_failure_of_an_actually_queried_local_body() {
    #[derive(Default)]
    struct Effects {
        begins: usize,
        requests: usize,
        commits: usize,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.begins += 1;
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            self.requests += 1;
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(42).unwrap(),
            ))
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            self.commits += usize::from(commit);
            Ok(())
        }
    }
    let source = "compiler_create_workspace::(name:string)->s64 #compiler;Box::struct{value:int;}plain::()->int{return 7;}required::()->int #must{return 9;}main::()->int{operator +::(a:Box,callback:$F)->Box{receipt:=compiler_create_workspace(\"child\");callback();return a;}first:=#run ->int{box:=Box.{value=5}+plain;return box.value;}second:=Box.{value=6}+required;return first+second.value;}";
    let fixture = Fixture::new(source);
    let graph = fixture.graph();
    let options = ResolveOptions {
        compiler: Some(jai_sema::CompilerBindingContext::from_graph(
            &graph,
            &[],
            jai_vm::WorkspaceId::from_raw(1).unwrap(),
        )),
        ..ResolveOptions::default()
    };
    let mut effects = Effects::default();
    let error = resolve_graph_with_options(&graph, &options, &mut effects).unwrap_err();
    assert!(error.message.contains("#must"), "{error:?}");
    let rejected = source.find("callback();").unwrap();
    assert_eq!(error.location.span.start, rejected, "{error:?}");
    assert_eq!(
        (effects.begins, effects.requests, effects.commits),
        (1, 1, 1)
    );
}

#[test]
fn one_quoted_recipe_in_distinct_owners_has_distinct_replay_identity() {
    #[derive(Default)]
    struct Effects(Vec<jai_vm::SourceOrigin>);
    impl jai_vm::CompilerEffects for Effects {
        fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
            self.0.push(origin);
        }
        fn begin(&mut self) {
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("pure recipe");
        }
        fn finish(&mut self, _: bool) -> Result<(), jai_vm::Error> {
            Ok(())
        }
    }
    let fixture = Fixture::new(
        "recipe::()#expand{#run ()->int{return 1;};} first::(){recipe();} second::(){recipe();} main::(){first();second();}",
    );
    let mut effects = Effects::default();
    resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut effects).unwrap();
    assert_eq!(effects.0.len(), 2);
    assert_eq!(effects.0[0].body, effects.0[1].body);
    assert_eq!(effects.0[0].start, effects.0[1].start);
    assert_ne!(effects.0[0].specialization, effects.0[1].specialization);
}

#[test]
fn quoted_run_captures_have_distinct_stable_replay_identity() {
    #[derive(Default)]
    struct Effects(Vec<jai_vm::SourceOrigin>);
    impl jai_vm::CompilerEffects for Effects {
        fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
            self.0.push(origin);
        }
        fn begin(&mut self) {
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("pure quoted recipe");
        }
        fn finish(&mut self, _: bool) -> Result<(), jai_vm::Error> {
            Ok(())
        }
    }
    let source = "emit::($N:int,target:Code)#expand{quoted::#code #run N; (#insert target)=#insert quoted;} main::()->int{left:=0;right:=0;emit(2,left);emit(40,right);return left+right;}";
    let fixture = Fixture::new(source);
    let mut original = Effects::default();
    let program =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut original)
            .unwrap();
    assert!(matches!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(&values[..], [Value::Int(value)] if value.value()==42)
    ));
    assert_eq!(original.0.len(), 2);
    assert_eq!(original.0[0].body, original.0[1].body);
    assert_ne!(original.0[0].specialization, original.0[1].specialization);
    for text in [
        source.to_owned(),
        format!("{source} unused::(){{}}"),
        format!("{source} Unused::#type int; Callback::#type()->int #must;"),
    ] {
        std::fs::write(fixture.0.join("main.jai"), text).unwrap();
        let mut fresh = Effects::default();
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut fresh)
            .unwrap();
        assert_eq!(original.0, fresh.0);
    }
}

#[test]
fn body_retries_preserve_committed_runs_with_promoted_and_anonymous_bindings() {
    #[derive(Default)]
    struct Effects {
        effect_origin: bool,
        begins: usize,
        requests: usize,
        commits: usize,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
            self.effect_origin = origin.body.starts_with(b"#run compiler_create_workspace");
        }
        fn begin(&mut self) {
            if self.effect_origin {
                self.begins += 1;
            }
        }
        fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            assert!(matches!(
                request,
                jai_vm::CompilerRequest::CreateWorkspace { .. }
            ));
            self.requests += 1;
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(1).unwrap(),
            ))
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            if self.effect_origin && commit {
                self.commits += 1;
            }
            Ok(())
        }
    }
    for source in [
        "compiler_create_workspace::(name:string)->s64 #compiler; Record::struct{number:int;} work::(using item:*Record)->int{receipt:=#run compiler_create_workspace(\"child\");return cast(int)receipt + #run later();} later::()->int{return 41;} main::()->int{item:Record;return work(*item);}",
        "compiler_create_workspace::(name:string)->s64 #compiler; main::()->int{return #run ->int{receipt:=#run compiler_create_workspace(\"child\");return cast(int)receipt + #run later();};} later::()->int{return 41;}",
    ] {
        let fixture = Fixture::new(source);
        let graph = fixture.graph();
        let options = ResolveOptions {
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &graph,
                &[],
                jai_vm::WorkspaceId::from_raw(1).unwrap(),
            )),
            ..ResolveOptions::default()
        };
        let mut effects = Effects::default();
        let program = resolve_graph_with_options(&graph, &options, &mut effects).unwrap();
        assert_eq!(
            (effects.begins, effects.requests, effects.commits),
            (1, 1, 1),
            "{source}"
        );
        assert!(matches!(
            jai_vm::execute(&program, Limits::default()).outcome,
            Outcome::Complete(values) if matches!(&values[..], [Value::Int(value)] if value.value() == 42)
        ));
    }
}

#[test]
fn module_parameter_instances_have_stable_distinct_run_origins() {
    #[derive(Default)]
    struct Effects(Vec<jai_vm::SourceOrigin>);
    impl jai_vm::CompilerEffects for Effects {
        fn set_source_origin(&mut self, origin: jai_vm::SourceOrigin) {
            self.0.push(origin);
        }
        fn begin(&mut self) {
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("pure module recipe")
        }
        fn finish(&mut self, _: bool) -> Result<(), jai_vm::Error> {
            Ok(())
        }
    }
    let fixture = Fixture::new(
        "A :: #import \"Lib\"(1); B :: #import \"Lib\"(2); main :: ()->int {return A.value()+B.value();}",
    );
    std::fs::create_dir(fixture.0.join("modules")).unwrap();
    let load = || {
        ModuleGraph::load(
            &fixture.0.join("main.jai"),
            GraphOptions {
                import_dirs: vec![fixture.0.join("modules")],
                ..GraphOptions::default()
            },
        )
        .unwrap()
    };
    for declaration in [
        "value :: ()->int {return #run Count;}",
        "value :: ()->int {local :: ()->int {return #run Count;} return local();}",
        "value :: () => #run Count;",
    ] {
        std::fs::write(
            fixture.0.join("modules/Lib.jai"),
            format!("#module_parameters(Count:int); {declaration}"),
        )
        .unwrap();
        let mut first = Effects::default();
        let program =
            resolve_graph_with_options(&load(), &ResolveOptions::default(), &mut first).unwrap();
        assert!(matches!(
            jai_vm::execute(&program, Limits::default()).outcome,
            Outcome::Complete(values) if matches!(&values[..], [Value::Int(value)] if value.value()==3)
        ));
        assert_eq!(first.0.len(), 2, "{declaration}");
        assert_eq!(first.0[0].body, first.0[1].body);
        assert_ne!(first.0[0].specialization, first.0[1].specialization);
        let mut second = Effects::default();
        resolve_graph_with_options(&load(), &ResolveOptions::default(), &mut second).unwrap();
        assert_eq!(
            first.0, second.0,
            "fresh arenas must preserve replay identity: {declaration}"
        );
    }
}

#[test]
fn expected_run_type_is_checked_before_effects_commit() {
    #[derive(Default)]
    struct Effects {
        pending: bool,
        workspace: u64,
        requests: usize,
        commits: usize,
        rollbacks: usize,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
        }
        fn request(&mut self, request: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            assert!(matches!(
                request,
                jai_vm::CompilerRequest::CreateWorkspace { .. }
            ));
            self.requests += 1;
            if self.pending {
                jai_vm::EffectOutcome::Pending(jai_vm::EffectKey(7))
            } else {
                jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                    jai_vm::WorkspaceId::from_raw(if self.workspace == 0 {
                        42
                    } else {
                        self.workspace
                    })
                    .unwrap(),
                ))
            }
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            if commit {
                self.commits += 1;
            } else {
                self.rollbacks += 1;
            }
            Ok(())
        }
    }
    fn options(graph: &ModuleGraph) -> ResolveOptions {
        ResolveOptions {
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                graph,
                &[],
                jai_vm::WorkspaceId::from_raw(1).unwrap(),
            )),
            ..ResolveOptions::default()
        }
    }
    let prototype = "compiler_create_workspace :: (name:string)->s64 #compiler;";
    for rejected in [
        "answer:bool:#run compiler_create_workspace(\"child\"); main::(){}",
        "answer:bool:cast(s64) #run compiler_create_workspace(\"child\"); main::(){}",
        "answer:bool:#run compiler_create_workspace(\"child\") + 1; main::(){}",
        "main::(){answer:bool = #run compiler_create_workspace(\"child\");}",
        "main::()->bool{return #run ->s64{return compiler_create_workspace(\"child\");};}",
    ] {
        let fixture = Fixture::new(&format!("{prototype}{rejected}"));
        let graph = fixture.graph();
        let mut effects = Effects::default();
        assert!(resolve_graph_with_options(&graph, &options(&graph), &mut effects).is_err());
        assert_eq!(
            effects.commits, 0,
            "a rejected destination cannot commit compiler effects"
        );
        assert_eq!(
            effects.requests, 0,
            "a statically invalid destination needs no execution"
        );
    }
    let fixture = Fixture::new(&format!(
        "{prototype}answer:s64:#run compiler_create_workspace(\"child\"); main::()->s64{{return answer;}}"
    ));
    let graph = fixture.graph();
    let mut effects = Effects::default();
    resolve_graph_with_options(&graph, &options(&graph), &mut effects).unwrap();
    assert_eq!((effects.requests, effects.commits), (1, 1));
    let cast_fixture = Fixture::new(&format!(
        "{prototype}answer::cast(s8) #run compiler_create_workspace(\"child\"); main::()->int{{return answer;}}"
    ));
    let cast_graph = cast_fixture.graph();
    let mut rejected_cast = Effects {
        workspace: 300,
        ..Effects::default()
    };
    let error = resolve_graph_with_options(&cast_graph, &options(&cast_graph), &mut rejected_cast)
        .unwrap_err();
    assert!(error.message.contains("cast"), "{error:?}");
    assert_eq!(
        (
            rejected_cast.requests,
            rejected_cast.commits,
            rejected_cast.rollbacks
        ),
        (1, 0, 1)
    );
    let mut pending = Effects {
        pending: true,
        ..Effects::default()
    };
    let error = resolve_graph_with_options(&graph, &options(&graph), &mut pending).unwrap_err();
    assert!(error.message.contains("dependencies"), "{error:?}");
    assert_eq!(pending.commits, 0);
    assert!(pending.requests > 0 && pending.rollbacks > 0);
}

#[test]
fn modifier_body_and_its_generic_dependencies_cannot_request_compiler_effects() {
    #[derive(Default)]
    struct Effects {
        begins: usize,
        requests: usize,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.begins += 1;
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            self.requests += 1;
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(42).unwrap(),
            ))
        }
        fn finish(&mut self, _: bool) -> Result<(), jai_vm::Error> {
            Ok(())
        }
    }
    for body in [
        "choose :: (a:$T)->T #modify { child := #run compiler_create_workspace(\"child\"); return true; } { return a; }",
        "dependency :: (a:$T)->s64 { return #run compiler_create_workspace(\"child\"); } choose :: (a:$T)->T #modify { child := dependency(1); return true; } { return a; }",
    ] {
        let fixture = Fixture::new(&format!(
            "compiler_create_workspace :: (name:string)->s64 #compiler; {body} main :: ()->int {{ return choose(1); }}"
        ));
        let graph = fixture.graph();
        let options = ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &graph,
                &[],
                jai_vm::WorkspaceId::from_raw(1).unwrap(),
            )),
            ..ResolveOptions::default()
        };
        let mut effects = Effects::default();
        let error = resolve_graph_with_options(&graph, &options, &mut effects).unwrap_err();
        assert_eq!((effects.begins, effects.requests), (0, 0), "{error:?}");
        assert!(
            error.message.contains("effect") || error.message.contains("unsupported"),
            "{error:?}"
        );
    }
}

#[test]
fn completed_run_does_not_hide_a_later_callback_contract_failure() {
    #[derive(Default)]
    struct Effects {
        commits: usize,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("this source fixture has no compiler requests")
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            self.commits += usize::from(commit);
            Ok(())
        }
    }
    let declarations = "plain :: ()->int {return 7;} required :: ()->int #must {return 9;} apply :: (callback:$T)->int {callback(); return 42;} answer :: #run apply(plain);";
    let positive = Fixture::new(&format!(
        "{declarations} main :: ()->int {{return answer;}}"
    ));
    let mut positive_effects = Effects::default();
    let program = resolve_graph_with_options(
        &positive.graph(),
        &ResolveOptions::default(),
        &mut positive_effects,
    )
    .unwrap();
    assert_eq!(positive_effects.commits, 1);
    assert!(matches!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Complete(values) if matches!(&values[..], [Value::Int(value)] if value.value()==42)
    ));
    // The left operand becomes ready only after the earlier run completes;
    // binding the right operand then strengthens the same generic body.
    let negative = Fixture::new(&format!(
        "{declarations} main :: ()->int {{return answer + apply(required);}}"
    ));
    let mut effects = Effects::default();
    let error =
        resolve_graph_with_options(&negative.graph(), &ResolveOptions::default(), &mut effects)
            .unwrap_err();
    assert_eq!(effects.commits, 1, "{error:?}");
    assert!(
        error.message.contains("required result") || error.message.contains("#must"),
        "{error:?}"
    );
}

#[test]
fn aggregate_storage_publication_is_validated_before_effect_commit() {
    #[derive(Default)]
    struct Effects {
        requests: usize,
        commits: usize,
        rollbacks: usize,
        staged_request: bool,
    }
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.staged_request = false;
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            self.requests += 1;
            self.staged_request = true;
            jai_vm::EffectOutcome::Ready(jai_vm::CompilerResponse::Workspace(
                jai_vm::WorkspaceId::from_raw(42).unwrap(),
            ))
        }
        fn finish(&mut self, commit: bool) -> Result<(), jai_vm::Error> {
            if commit {
                self.commits += 1;
            } else if self.staged_request {
                self.rollbacks += 1;
            }
            Ok(())
        }
    }
    for (wide, publishable) in [(9, true), (0x0102030405060708u64, false)] {
        let fixture = Fixture::new(&format!(
            "compiler_create_workspace::(name:string)->s64 #compiler; Bits::union{{wide:u64;small:u8;}} probe::()->Bits{{receipt:=compiler_create_workspace(\"child\");bits:Bits;bits.wide={wide};bits.small=9;return bits;}} answer::#run probe();main::(){{}}"
        ));
        let graph = fixture.graph();
        let options = ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            compiler: Some(jai_sema::CompilerBindingContext::from_graph(
                &graph,
                &[],
                jai_vm::WorkspaceId::from_raw(1).unwrap(),
            )),
            ..ResolveOptions::default()
        };
        let mut effects = Effects::default();
        let result = resolve_graph_with_options(&graph, &options, &mut effects);
        assert_eq!(effects.requests, 1, "{result:?}");
        if publishable {
            result.unwrap();
            assert_eq!((effects.commits, effects.rollbacks), (1, 0));
        } else {
            let error = result.unwrap_err();
            assert!(error.message.contains("inactive bytes"), "{error:?}");
            assert_eq!((effects.commits, effects.rollbacks), (0, 1));
        }
    }
}
