//! Generic struct arguments obey their declared type before binding and reflection.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};

fn compile(source: &str) -> Result<(), String> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![root.clone()];
    options.preload = Some(root.join("prelude/module.jai"));
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn typed_callback_arguments_instantiate_before_metadata_and_preserve_identity() {
    compile(include_str!(
        "../../../tests/fixtures/struct-arguments/typed-generic-procedure-parameter.jai"
    ))
    .unwrap();
}

#[test]
fn typed_callback_rejects_wrong_signature_at_binding_without_invoking_it() {
    let error = compile(
        r#"
        Holder :: struct(callback:(s64)->s64) {data:s64;}
        wrong :: (value:string)->s64 {return 17;}
        Bad :: Holder(wrong);
        #assert size_of(Bad)==8;
    "#,
    )
    .expect_err("an unused typed callback still requires its declared signature");
    assert!(
        error.contains("no overload matches procedure type"),
        "{error}"
    );
}

#[test]
fn typed_scalar_argument_keeps_narrowing_diagnostics_at_binding() {
    let error = compile(
        "Holder :: struct(value:u8) {data:s64;} Bad :: Holder(256); #assert size_of(Bad)==8;",
    )
    .expect_err("typed scalar generic arguments cannot bypass narrowing validation");
    assert!(error.contains("u8"), "{error}");
}

#[test]
fn generic_callback_with_an_incompatible_result_rejects_at_binding() {
    let error = compile(
        r#"
        Holder :: struct(callback:(s64)->s64) {data:s64;}
        wrong :: (value:$T)->string {return "different result type";}
        Bad :: Holder(wrong);
        #assert size_of(Bad)==8;
    "#,
    )
    .expect_err("generic specialization must match both arguments and results");
    assert!(
        error.contains("no overload matches procedure type"),
        "{error}"
    );
}

#[test]
fn typed_scalar_values_and_default_bindings_preserve_canonical_identity() {
    compile(
        r#"
        Holder :: struct(count:u8=7,ratio:float64=2) {data:[count]u8;}
        #assert Holder()==Holder(7,2.0);
        #assert size_of(Holder(255,3))==255;
        info:=type_info(Holder(255,3));
        #assert info.specified_parameters[0].type==type_info(u8);
        #assert info.specified_parameters[1].type==type_info(float64);
    "#,
    )
    .unwrap();
}

#[test]
fn context_callback_with_any_and_aggregate_results_survives_allocator_override() {
    compile(
        r#"
        #import "Basic";
        #add_context settings:Settings;
        Output :: struct {value:s64; padding:[64]u8;}
        Settings :: struct {callback:=ordinary;}
        ordinary :: (input:Any)->ok:=false,result:Output=.{} {return true,.{value=1};}
        custom :: (input:Any)->ok:=false,result:Output=.{} {return true,.{value=9};}
        nested :: (input:Any)->ok:=false,result:Output=.{} {
            return context.settings.callback(input,,temp);
        }
        outer :: (input:Any)->ok:=false,result:Output=.{} {return nested(input);}
        verify :: ()->bool {
            ok,value:=outer(7,,settings=.{callback=custom});
            if !ok || value.value!=9 return false;
            ok=, value=outer(7);
            return ok && value.value==1;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}
