use super::*;
use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    sync::atomic::{AtomicUsize, Ordering},
};

fn resolve_source(source: &str) -> Result<Library, jai_source::LocatedDiagnostic> {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = std::env::temp_dir().join(format!(
        "jai-procedure-values-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("main.jai");
    fs::write(&path, source).unwrap();
    let graph = ModuleGraph::load(&path, GraphOptions::default()).unwrap();
    let result = crate::resolve_library(&graph);
    fs::remove_dir_all(directory).unwrap();
    result
}

fn returned_integer(procedure: &Procedure) -> &IntExpr {
    let Statement::Exit(exit) = procedure.body.statements.last().unwrap() else {
        panic!()
    };
    match &exit.transfer {
        Transfer::ReturnInt(value) => value,
        Transfer::ReturnValues(values) => match values.as_slice() {
            [ValueExpr::Int(value)] => value,
            _ => panic!(),
        },
        _ => panic!(),
    }
}

#[test]
fn foreign_declarations_remain_bodyless_and_source_bodies_use_sparse_ids() {
    let library = resolve_source("external :: (value:int) -> int #foreign \"trusted_fixture\"; answer :: () -> int {return 42;}").unwrap();
    assert_eq!(library.prototypes().len(), 1);
    assert_eq!(library.procedures().len(), 1);
    assert!(
        library
            .procedure_by_id(library.prototypes()[0].id)
            .is_none()
    );
    assert!(
        library
            .procedure_by_id(library.procedures()[0].id)
            .is_some()
    );
    assert!(
        matches!(&library.prototypes()[0].origin, PrototypeOrigin::Foreign{symbol,..} if symbol=="trusted_fixture")
    );
}

#[test]
fn procedure_values_bind_typed_indirect_calls() {
    let library = resolve_source("inc :: (value:int) -> int {return value+1;} apply :: (callback:(int)->int, value:int)->int{return callback(value);} main :: () -> int {callback:=inc; return apply(callback,41);}").unwrap();
    assert_eq!(library.procedures().len(), 3);
    let apply = &library.procedures()[1];
    let value = returned_integer(apply);
    assert!(
        matches!(value.kind(), IntExprKind::Value(value) if matches!(value.as_ref(),ValueExpr::IndirectCall{..}))
    );
}

#[test]
fn procedure_constant_defaults_reject_different_signatures() {
    for source in [
        "answer::()->int{return 42;} callback:(int)->int=answer; main::()->int{return 0;}",
        "answer::()->int #no_context{return 42;} Holder::struct{callback:()->int=answer;}main::()->int{return 0;}",
        "answer::()->int #c_call{return 42;} callback:()->int=answer;main::()->int{return 0;}",
        "answer::()->bool{return true;} callback:()->int=answer;main::()->int{return 0;}",
        "answer::(values:..int)->int{return 42;} callback:(int)->int=answer;main::()->int{return 0;}",
        "answer::()->int{return 42;} Holder::struct{callback:(int)->int=answer;}main::()->int{return 0;}",
        "Callback::#type(int)->int; answer::()->int{return 42;} callback:Callback=cast(Callback)answer;main::()->int{return 0;}",
    ] {
        assert!(
            resolve_source(source).is_err(),
            "accepted an incompatible procedure constant: {source}"
        );
    }
}

#[test]
fn cyclic_inferred_callback_headers_report_the_declaration_cycle() {
    let error = resolve_source("first::(callback:=second)->int{return 0;}second::(callback:=first)->int{return 0;}main::()->int{return 0;}").unwrap_err();
    assert!(
        error
            .message
            .contains("cyclic inferred procedure signature"),
        "{}",
        error.message
    );
}

#[test]
fn positional_record_literals_reject_extra_values_and_ambiguous_types() {
    for (source, message) in [
        (
            "Pair::struct{left:int;right:int;}main::()->int{value:=Pair.{1,2,3};return 0;}",
            "too many values",
        ),
        (
            "main::()->int{value:=.{1,2};return 0;}",
            "explicit or contextual type",
        ),
        (
            "Choice::union{number:int;flag:bool;}main::()->int{value:=Choice.{1};return 0;}",
            "explicit alternative",
        ),
    ] {
        let error = resolve_source(source).unwrap_err();
        assert!(error.message.contains(message), "{}", error.message);
    }
}

#[test]
fn c_variadic_arguments_promote_narrow_integers_enums_bools_and_floats() {
    let library = resolve_source("Narrow::enum u8{VALUE::7;} Wide::enum u64{VALUE::0xffff_ffff_ffff_ffff;} collect :: (tag:int,args:..Any) -> int #foreign; main :: () -> int {n:s8=3; f:float32=1.5; return collect(0,n,true,f,ifx true then 1.0 else 2.0,Narrow.VALUE,Wide.VALUE);}").unwrap();
    let main = &library.procedures()[0];
    let value = returned_integer(main);
    let IntExprKind::Value(value) = value.kind() else {
        panic!()
    };
    let ValueExpr::Call {
        call, ..
    } = value.as_ref()
    else {
        panic!()
    };
    let types = library.types();
    assert_eq!(
        call.arguments[1].1.type_id(types),
        types.scalar(ScalarType::Int(IntegerType::S32))
    );
    assert_eq!(
        call.arguments[2].1.type_id(types),
        types.scalar(ScalarType::Int(IntegerType::S32))
    );
    assert_eq!(
        call.arguments[3].1.type_id(types),
        types.float(jai_types::FloatType::F64)
    );
    assert_eq!(
        call.arguments[4].1.type_id(types),
        types.float(jai_types::FloatType::F64)
    );
    assert_eq!(
        call.arguments[5].1.type_id(types),
        types.scalar(ScalarType::Int(IntegerType::S32))
    );
    assert_eq!(
        call.arguments[6].1.type_id(types),
        types.scalar(ScalarType::Int(IntegerType::U64))
    );
}

#[test]
fn jai_variadic_pack_is_a_slice_with_trailing_named_defaults() {
    let library = resolve_source("count :: (args:..int, extra:int=1) -> int {return args.count+extra;} main :: () -> int {return count(20,21,extra=2);}").unwrap();
    let signature = library
        .types()
        .procedure_definition(library.procedures()[0].signature)
        .unwrap();
    assert!(matches!(
        signature.variadic,
        jai_types::Variadic::Jai {
            parameter: 0,
            ..
        }
    ));
    let main = &library.procedures()[1];
    let value = returned_integer(main);
    let IntExprKind::Value(value) = value.kind() else {
        panic!()
    };
    let ValueExpr::Call {
        call, ..
    } = value.as_ref()
    else {
        panic!()
    };
    assert!(
        matches!(&call.arguments[0].1,ValueExpr::ArrayView{array,..} if matches!(array.as_ref(),ValueExpr::Array{elements,..} if elements.len()==2))
    );
}

#[test]
fn procedure_assignment_preserves_calling_convention_and_context_contract() {
    let error = resolve_source(
        "callback :: (value:int)->int #c_call {return value;} main :: () {f:(int)->int=callback;}",
    )
    .unwrap_err();
    assert!(
        error.message.contains("signature")
            || error.message.contains("type")
            || error.message.contains("conversion")
    );
}

fn run_integer(source: &str) -> i128 {
    let library = resolve_source(source).unwrap();
    let procedure = library.procedures().last().unwrap().id;
    let program = library.into_program(EntryPoint::Int(procedure)).unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = outcome.outcome else {
        panic!("{:?}", outcome.outcome)
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("{:?}", values)
    };
    value.value()
}

#[test]
fn callback_binding_names_do_not_change_canonical_type_identity() {
    assert_eq!(
        run_integer(
            "digits::(a:int,b:int)->int{return a*10+b;} main::()->int{f:(x:int,y:int)->int=digits; g:(y:int,x:int)->int=digits; return f(y=2,x=4)-g(y=2,x=4)+24;}"
        ),
        42
    );
}

#[test]
fn annotated_callback_parameters_and_record_fields_keep_names() {
    assert_eq!(
        run_integer(
            "digits::(a:int,b:int)->int{return a*10+b;} Holder::struct{callback:(left:int,right:int)->int;} apply::(callback:(left:int,right:int)->int)->int{return callback(right=2,left=4);} main::()->int{holder:Holder; holder.callback=digits; return apply(holder.callback)+holder.callback(right=2,left=4)-42;}"
        ),
        42
    );
}

#[test]
fn inferred_callbacks_keep_declaration_defaults_and_named_variadic_tail() {
    assert_eq!(
        run_integer(
            "digits::(a:int,b:int=2)->int{return a*10+b;} count::(args:..int,extra:int=2)->int{return args.count*20+extra;} main::()->int{f:=digits; pack:=count; return f(a=4)+pack(20,21,extra=2)-42;}"
        ),
        42
    );
}

#[test]
fn named_callback_arguments_evaluate_in_source_order_once() {
    assert_eq!(
        run_integer(
            "counter:int=0; next::()->int{counter+=1;return counter;} digits::(a:int,b:int)->int{return a*10+b;} main::()->int{f:=digits; return f(b=next(),a=next())*2;}"
        ),
        42
    );
}

#[test]
fn callback_type_aliases_keep_names_across_bindings() {
    assert_eq!(
        run_integer(
            "Callback::#type(left:int,right:int)->int; digits::(a:int,b:int)->int{return a*10+b;} apply::(callback:Callback)->int{return callback(right=2,left=4);} main::()->int{Local::#type(left:int,right:int)->int; f:Local=digits; return apply(f);}"
        ),
        42
    );
}

#[test]
fn null_procedure_defaults_assignment_and_truth_share_one_signature() {
    assert_eq!(
        run_integer(
            "answer::()->int{return 42;} main::()->int{callback:()->int; if callback {return 1;} if callback!=null{return 2;} callback=answer; if !callback{return 3;} if callback==answer{return callback();} return 4;}"
        ),
        42
    );
    assert_eq!(
        run_integer(
            "answer::()->int{return 42;} main::()->int{callback:()->int=null; callback=answer; if callback && true {return callback();} return 0;}"
        ),
        42
    );
}

#[test]
fn procedure_identity_rejects_different_signatures_and_address_arithmetic() {
    for source in [
        "a::()->int{return 42;} b::()->int #no_context{return 42;} main::(){if a==b{}}",
        "answer::()->int{return 42;} main::(){callback:=answer; moved:=callback+1;}",
    ] {
        assert!(resolve_source(source).is_err());
    }
}
