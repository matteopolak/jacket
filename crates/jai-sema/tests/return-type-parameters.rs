//! Result introductions retain source identity and never occupy runtime formals.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::{FloatType, IntegerType, ScalarType, TypeKind, TypeView};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let path = std::env::temp_dir().join(format!(
            "jai-result-type-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("main.jai"), source).unwrap();
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn resolve(source: &str) -> Result<jai_ir::Program, jai_source::LocatedDiagnostic> {
    let fixture = Fixture::new(source);
    let graph = ModuleGraph::load(&fixture.0.join("main.jai"), GraphOptions::default()).unwrap();
    resolve_graph_with_options(
        &graph,
        &ResolveOptions {
            layout: Some(jai_types::LayoutPolicy::lp64()),
            ..ResolveOptions::default()
        },
        &mut NoEffects,
    )
}
fn result(program: &jai_ir::Program) -> i128 {
    match jai_vm::execute(program, Limits::default()).outcome {
        Outcome::Complete(values) => match &values[..] {
            [Value::Int(value)] => value.value(),
            other => panic!("unexpected result {other:?}"),
        },
        other => panic!("source program did not complete: {other:?}"),
    }
}
#[test]
fn float_types_preserve_the_original_string_parameter_and_multiple_results() {
    let program = resolve(
        r#"
        convert :: (_arg:string) -> result:$T, success:bool, remainder:string {
            result = 12.5; return result,true,_arg;
        }
        main :: () -> int {
            a,ok,remaining := convert("abc",T=float);
            b,ok64,remaining64 := convert("de",T=float64);
            if !ok || !ok64 || a != 12.5 || b != 12.5 return -1;
            return cast(int)a + cast(int)b + remaining.count + remaining64.count;
        }
    "#,
    )
    .unwrap();
    assert_eq!(result(&program), 29);
    let string = program.types().lookup(&TypeKind::String).unwrap();
    let boolean = program.types().scalar(ScalarType::Bool);
    for float in [FloatType::F32, FloatType::F64] {
        let output = program.types().float(float);
        let matching = program
            .procedures()
            .iter()
            .filter(|procedure| {
                let Ok(TypeKind::Procedure(id)) = program.types().kind(procedure.signature) else {
                    return false;
                };
                let signature = program.types().procedure_type(*id).unwrap();
                signature.parameters.as_ref() == [string]
                    && signature.results.as_ref() == [output, boolean, string]
            })
            .count();
        assert_eq!(
            matching, 1,
            "actual result types retain one-parameter runtime bodies"
        );
    }
}
#[test]
fn result_type_binding_precedes_dependent_parameter_checking() {
    assert_eq!(
        result(
            &resolve(
                "identity::(value:T)->$T{return value;}main::()->int{return identity(42,T=int);}"
            )
            .unwrap()
        ),
        42
    );
}
#[test]
fn nested_introduction_preserves_actual_record_origin() {
    assert_eq!(result(&resolve("Box::struct(T:Type){value:T;}make::()->Box($T){result:Box(T);result.value=42;return result;}main::()->int{return make(T=int).value;}").unwrap()), 42);
}
#[test]
fn named_order_and_canonical_aliases_share_the_actual_specialization() {
    let program = resolve("pair::()->(a:$A,b:$B){return 20,22;}main::()->int{a,b:=pair(A=int,B=s64);c,d:=pair(B=int,A=s64);return a+b+c+d;}").unwrap();
    assert_eq!(result(&program), 84);
    let int = program.types().scalar(ScalarType::Int(IntegerType::S64));
    let matching = program
        .procedures()
        .iter()
        .filter(|procedure| {
            let Ok(TypeKind::Procedure(id)) = program.types().kind(procedure.signature) else {
                return false;
            };
            let signature = program.types().procedure_type(*id).unwrap();
            signature.parameters.is_empty() && signature.results.as_ref() == [int, int]
        })
        .count();
    assert_eq!(
        matching, 1,
        "original result order and canonical Type determine identity"
    );
}
#[test]
fn callback_names_and_required_results_survive_actual_named_type_source() {
    let program = resolve("Required::#type(named:int)->int #must;Optional::#type(other:int)->int;answer::(value:int)->int{return value;}make::()->$T{return cast(T)answer;}main::()->int{a:=make(T=Required);b:=make(T=Optional);return a(named=40)+b(other=2);}").unwrap();
    assert_eq!(result(&program), 42);
    let error = resolve("Required::#type(value:int)->int #must;Optional::#type(value:int)->int;answer::(value:int)->int{return value;}make::()->$T{return cast(T)answer;}main::(){optional:=make(T=Optional);optional(1);required:=make(T=Required);required(2);}").unwrap_err();
    assert!(error.message.contains("#must"), "{error:?}");
}
#[test]
fn result_inputs_require_one_named_compile_time_type_and_one_introduction() {
    for (source, expected) in [
        (
            "make::()->$T{return 42;}main::()->int{return make();}",
            "missing required named result Type",
        ),
        (
            "make::()->$T{return 42;}main::()->int{return make(int);}",
            "too many arguments",
        ),
        (
            "make::()->$T{return 42;}main::()->int{return make(T=42);}",
            "compile-time Type value",
        ),
        (
            "make::()->$T{return 42;}main::()->int{return make(T=int,T=s64);}",
            "duplicate result Type",
        ),
        (
            "make::()->$T{return 42;}main::()->int{return make(U=int);}",
            "unknown named argument",
        ),
        (
            "make::(a:$T)->$T{return a;}main::()->int{return make(42,T=int);}",
            "introduced more than once",
        ),
        (
            "make::()->(a:$T,b:$T){return 20,22;}main::()->int{return 0;}",
            "introduced more than once",
        ),
    ] {
        let error = resolve(source).unwrap_err();
        assert!(error.message.contains(expected), "{expected}: {error:?}");
    }
}
#[test]
fn modifier_results_keep_their_existing_source_authority() {
    assert_eq!(result(&resolve("widen::(a:$T)->$R #modify{R=s64;return true;}{return a;}main::()->int{value:u8=42;return widen(value);}").unwrap()), 42);
}


#[test]
fn named_result_locals_use_original_defaults_and_result_positions() {
    assert_eq!(result(&resolve("pair::()->(first:$A=20,second:$B=21){second+=1;return;}main::()->int{a,b:=pair(B=int,A=int);return a+b;}").unwrap()), 42);
    assert_eq!(result(&resolve("pair::()->(first:int=40,second:=1){second=2;return first=40;}main::()->int{a,b:=pair();return a+b;}").unwrap()), 42);
}
#[test]
fn named_results_snapshot_actual_aggregate_storage_before_cleanup() {
    assert_eq!(result(&resolve("Box::struct{value:int=40;}make::()->answer:Box{answer.value+=2;defer answer.value=0;return;}main::()->int{return make().value;}").unwrap()), 42);
}
#[test]
fn omitted_named_results_use_owner_slots_despite_nested_and_macro_shadowing() {
    assert_eq!(result(&resolve("answer::()->result:int{result=42;{result:=100;return;}}main::()->int{return answer();}").unwrap()), 42);
    assert_eq!(result(&resolve("finish::()#expand{result:=100;`return;}answer::()->result:int{result=42;finish();}main::()->int{return answer();}").unwrap()), 42);
}
#[test]
fn nested_and_anonymous_bodies_bind_their_own_named_results() {
    assert_eq!(result(&resolve("main::()->int{local::()->result:int{result=20;return;}anonymous:=()->result:int{result=22;return;};return local()+anonymous();}").unwrap()), 42);
}
#[test]
fn named_callback_results_keep_declared_names_defaults_and_required_use() {
    let source = "Required::#type(named:int)->int #must;answer::(value:int)->int{return value+2;}make::()->callback:Required{callback=cast(Required)answer;return;}main::()->int{callback:=make();return callback(named=40);}";
    assert_eq!(result(&resolve(source).unwrap()), 42);
    assert_eq!(result(&resolve("target::(input:int=40)->int #must{return input+2;}make::()->(callback:=target){return;}main::()->int{callback:=make();return callback();}").unwrap()),42);
    let error=resolve("Required::#type(named:int)->int #must;answer::(value:int)->int{return value;}make::()->callback:Required{callback=cast(Required)answer;return;}main::(){callback:=make();callback(named=42);}").unwrap_err();
    assert!(error.message.contains("#must"), "{error:?}");
}
#[test]
fn unnamed_returns_and_duplicate_result_storage_still_reject() {
    for (source, message) in [
        (
            "answer::()->int{return;}main::()->int{return answer();}",
            "missing required return",
        ),
        (
            "answer::(result:int)->result:int{return;}main::()->int{return answer(42);}",
            "duplicate variable",
        ),
    ] {
        let error = resolve(source).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
    }
}
#[test]
fn named_result_debug_declaration_points_to_real_body_initializer() {
    let source =
        "answer::(input:int)->result:int=2{result+=input;return;}main::()->int{return answer(40);}";
    let program = resolve(source).unwrap();
    assert_eq!(result(&program), 42);
    let sources = program.library().debug_sources().unwrap();
    let procedure = program
        .procedures()
        .iter()
        .find(|procedure| {
            sources
                .procedure(procedure.id)
                .is_some_and(|source| source.name == "answer")
        })
        .unwrap();
    assert_eq!(procedure.parameters.len(), 1);
    let (id, declaration) = sources
        .locals()
        .find(|(id, source)| id.procedure() == procedure.id && source.name == "result")
        .unwrap();
    assert!(
        declaration
            .location
            .span()
            .span
            .text(source)
            .contains("result")
    );
    assert_eq!(
        declaration.scope,
        jai_ir::BlockPath::procedure(procedure.id)
    );
    assert_eq!(
        declaration.declaration,
        jai_ir::LocalDeclaration::Statement(jai_ir::StatementPath::in_block(&declaration.scope, 0))
    );
    assert!(
        matches!(&procedure.body.statements[0],jai_ir::Statement::Store(place,_) if place.kind()==jai_ir::PlaceKind::Local(id))
    );
}
