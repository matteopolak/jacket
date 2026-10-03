//! Annotation queries use declaration types rather than evaluating operands.
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-type-query-annotation-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn program(&self) -> Result<jai_ir::Program, String> {
        let graph = jai_modules::ModuleGraph::load(&self.0.join("main.jai"), Default::default())
            .map_err(|error| error.to_string())?;
        jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(jai_types::LayoutPolicy::lp64()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .map_err(|error| error.to_string())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str) {
    let program = Fixture::new(source).program().unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn file_async_error_factory_preserves_its_inline_enum_field_type() {
    check(include_str!("fixtures/type-of-field-annotations.jai"));
}

#[test]
fn annotations_follow_using_fields_and_structural_containers() {
    check(
        "Base::struct{code:u16;} Wrap::struct{using base:Base;} read::(value:*type_of(Wrap.code))->int{return cast(int)value.*;} main::()->int{value:type_of(Wrap.code)=42; values:[2]type_of(Wrap.code); values[0]=value; return read(*values[0]);}",
    );
}

#[test]
fn local_type_and_value_shadows_determine_the_declared_field_type() {
    check(
        "Entry::struct{code:bool;} main::()->int{Entry::struct{code:s32;} value:Entry; value.code=42; copy:type_of(value.code)=value.code; return cast(int)copy;}",
    );
}

#[test]
fn local_pointer_type_aliases_query_the_original_nominal_field() {
    check(
        "main::()->int{Entry::struct{code:s32;} Pointer::**Entry; value:type_of(Pointer.code)=42; T::type_of(Pointer.code); copy:T=value; return cast(int)copy;}",
    );
}

#[test]
fn specialized_record_fields_supply_canonical_annotation_types() {
    check(
        "Box::struct(T:Type){value:T;} IntBox::Box(s32); copy::(value:type_of(IntBox.value))->int{return cast(int)value;} main::()->int{return copy(42);}",
    );
}

#[test]
fn file_type_query_alias_uses_the_reserved_record_identity_before_signatures() {
    check(
        "Error::struct{code:enum{Success::0;Failure;};} Code::type_of(Error.code); copy::(code:Code)->Code{return code;} main::()->int{return cast(int)copy(.Failure)+41;}",
    );
}

#[test]
fn unknown_declared_field_is_rejected_without_an_instance_load() {
    let error = Fixture::new(
        "Pair::struct{value:int;} f::(value:type_of(Pair.missing)){} main::()->int{return 0;}",
    )
    .program()
    .unwrap_err();
    assert!(error.contains("unknown record member"), "{error}");
}

#[test]
fn compile_time_operand_is_not_executed_to_obtain_an_annotation() {
    let error = Fixture::new("calls:int; tick::()->int{calls+=1;return 42;} main::()->int{value:type_of(#run tick());return 0;}")
        .program().unwrap_err();
    assert!(
        error.contains("annotation type_of requires a declared name"),
        "{error}"
    );
}

#[test]
fn expression_type_query_describes_run_result_without_executing_it() {
    check(
        "calls:int; tick::()->int{calls+=1;return 7;} main::()->int{T::type_of(#run tick());value:T=42;return value+calls*100;}",
    );
}
