use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::LayoutPolicy;
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let directory = std::env::temp_dir().join(format!(
            "jai-any-values-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join("main.jai"), source).unwrap();
        Self(directory)
    }
    fn program(&self) -> Result<jai_ir::Program, String> {
        let graph = ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default())
            .map_err(|error| error.to_string())?;
        resolve_graph_with_options(
            &graph,
            &ResolveOptions {
                layout: Some(LayoutPolicy::lp64()),
                ..ResolveOptions::default()
            },
            &mut NoEffects,
        )
        .map_err(|error| error.to_string())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn outcome(source: &str, limits: Limits) -> Outcome {
    let fixture = Fixture::new(source);
    let program = fixture.program().unwrap();
    jai_vm::execute(&program, limits).outcome
}
fn run(source: &str) -> i128 {
    let result = outcome(source, Limits::default());
    let Outcome::Complete(values) = result else {
        panic!("Any source did not complete: {result:?}");
    };
    let [Value::Int(value)] = values.as_slice() else {
        panic!("Any source did not return one integer: {values:?}");
    };
    value.value()
}

#[test]
fn lvalue_conversion_borrows_and_any_copy_preserves_both_pointers() {
    assert_eq!(
        run(r#"
            main :: () -> int {
                value:s32=6;
                first:Any=value;
                second:Any=first;
                value=42;
                if first.value_pointer != second.value_pointer return 1;
                if first.type != second.type return 2;
                if cast(*void)first.type != cast(*void)type_info(s32) return 3;
                return (cast(*s32)second.value_pointer).*;
            }
        "#),
        42
    );
}

#[test]
fn boxed_field_and_index_remain_aliases_to_original_places() {
    assert_eq!(
        run(r#"
            Pair :: struct { value:int; }
            main :: () -> int {
                pair:Pair;
                array:[2]int=.[3,4];
                field:Any=pair.value;
                element:Any=array[1];
                pair.value=20;
                array[1]=22;
                return (cast(*int)field.value_pointer).* + (cast(*int)element.value_pointer).*;
            }
        "#),
        42
    );
}

#[test]
fn boxed_mutable_descriptor_member_retains_its_slot_alias() {
    assert_eq!(
        run(
            "main :: () -> int { text:string=\"abc\"; boxed:Any=text.count; text.count=2; return (cast(*int)boxed.value_pointer).*+40; }"
        ),
        42
    );
}

#[test]
fn boxing_pointer_index_evaluates_address_and_index_once() {
    assert_eq!(
        run(r#"
            calls:int=0;
            index :: () -> int { calls+=1; return 1; }
            main :: () -> int {
                array:[2]int=.[0,0];
                pointer:=*array[0];
                boxed:Any=pointer[index()];
                array[1]=41;
                return (cast(*int)boxed.value_pointer).* + calls;
            }
        "#),
        42
    );
}

#[test]
fn rvalue_is_materialized_once_and_its_address_survives_callee_return() {
    assert_eq!(
        run(r#"
            calls:int=0;
            next :: () -> int { calls+=1; return 41; }
            inspect :: (value:Any) -> int { return (cast(*int)value.value_pointer).*; }
            main :: () -> int { value:Any=next(); return inspect(value)+calls; }
        "#),
        42
    );
}

#[test]
fn conditional_boxing_executes_only_the_selected_operand() {
    assert_eq!(
        run(r#"
            calls:int=0;
            next :: () -> int { calls+=1; return 41; }
            main :: () -> int {
                value:Any=ifx true then next() else 1/0;
                return (cast(*int)value.value_pointer).*+calls;
            }
        "#),
        42
    );
}

#[test]
fn jai_varargs_box_each_value_and_forward_existing_descriptors() {
    assert_eq!(
        run(r#"
            sum :: (args:..Any) -> int {
                result:int=0;
                for i:0..args.count-1 {
                    result+=(cast(*int)args[i].value_pointer).*;
                }
                return result;
            }
            forward :: (args:..Any) -> int { return sum(..args); }
            main :: () -> int { original:=20; boxed:Any=original; return forward(boxed,22); }
        "#),
        42
    );
}

#[test]
fn mixed_any_packs_preserve_payload_aliases_and_caller_owned_temporaries() {
    assert_eq!(
        run(r#"
            original:int=9;
            calls:int=0;
            prefix :: () -> int { original=10; calls+=1; return 41; }
            read :: (args:..Any) -> int {
                if args.count!=3 return 1;
                if args[1].value_pointer!=args[2].value_pointer return 2;
                if args[1].type!=args[2].type return 3;
                return (cast(*int)args[0].value_pointer).*
                     + (cast(*int)args[1].value_pointer).* - 9;
            }
            forward :: (args:..Any) -> int { return read(prefix(),..args); }
            main :: () -> int {
                descriptor:Any=original;
                result:=forward(descriptor,descriptor);
                if calls!=1 return 4;
                return result;
            }
        "#),
        42
    );
}

#[test]
fn aggregate_payloads_retain_real_nominal_and_array_descriptors() {
    assert_eq!(
        run(r#"
            Pair :: struct { left:int; right:int; }
            main :: () -> int {
                pair:Pair=.{left=20,right=22};
                boxed_pair:Any=pair;
                array:[2]int=.[20,22];
                boxed_array:Any=array;
                if cast(*void)boxed_pair.type != cast(*void)type_info(Pair) return 1;
                if cast(*void)boxed_array.type != cast(*void)type_info([2]int) return 2;
                pair.left=21;
                return (cast(*Pair)boxed_pair.value_pointer).left
                     + (cast(*[2]int)boxed_array.value_pointer).*[1] - 1;
            }
        "#),
        42
    );
}

#[test]
fn string_rvalue_boxes_its_descriptor_and_keeps_literal_backing() {
    assert_eq!(
        run(
            "main :: () -> int { boxed:Any=\"hello\"; text:=cast(*string)boxed.value_pointer; if cast(int)boxed.type.type!=3 return 1; return text.count+37; }"
        ),
        42
    );
}

#[test]
fn floating_payload_uses_its_actual_width_and_original_slot() {
    assert_eq!(
        run(
            "main :: () -> int { value:float32=1.5; boxed:Any=value; value=2.0; if boxed.type.runtime_size!=4 return 1; return cast(int)(cast(*float32)boxed.value_pointer).*+40; }"
        ),
        42
    );
}

#[test]
fn enum_payload_keeps_its_nominal_tag_and_integer_representation() {
    assert_eq!(
        run(
            "Kind :: enum u8 { FIRST::1; ANSWER::42; } main :: () -> int { value:Kind=.FIRST; boxed:Any=value; value=.ANSWER; if cast(int)boxed.type.type!=11 return 1; return cast(int)(cast(*Kind)boxed.value_pointer).*; }"
        ),
        42
    );
}

#[test]
fn distinct_payload_keeps_its_variant_descriptor_and_storage_alias() {
    assert_eq!(
        run(
            "Unit :: #type,distinct int; main :: () -> int { value:Unit=cast(Unit)20; boxed:Any=value; value=cast(Unit)42; if cast(int)boxed.type.type!=18 return 1; return cast(int)(cast(*Unit)boxed.value_pointer).*; }"
        ),
        42
    );
}

#[test]
fn universal_zero_and_reflection_tag_have_source_compatible_shape() {
    assert_eq!(
        run(r#"
            main :: () -> int {
                value:Any;
                if value.type != null return 1;
                if value.value_pointer != null return 2;
                info:=type_info(Any);
                if cast(int)info.type != 10 return 3;
                if info.runtime_size != 16 return 4;
                return 42;
            }
        "#),
        42
    );
}

#[test]
fn mutable_descriptor_fields_use_normal_pointer_and_place_rules() {
    assert_eq!(
        run(r#"
            main :: () -> int {
                value:=42;
                descriptor:Any;
                descriptor.type=cast(*Type_Info)type_info(int);
                descriptor.value_pointer=cast(*void)*value;
                return (cast(*int)descriptor.value_pointer).*;
            }
        "#),
        42
    );
}

#[test]
fn changing_a_public_type_field_does_not_enlarge_or_initialize_payload_storage() {
    let mismatched = outcome(
        r#"
        main::()->int {
            original:s32=42;
            boxed:Any=original;
            boxed.type=cast(*Type_Info)type_info(s64);
            return (cast(*s64)boxed.value_pointer).*;
        }
    "#,
        Limits::default(),
    );
    assert!(matches!(
        mismatched,
        Outcome::Failed(jai_vm::Error::OutOfBounds { .. })
    ));
    let empty = outcome(
        r#"
        main::()->int {
            boxed:Any;
            boxed.type=cast(*Type_Info)type_info(int);
            return (cast(*int)boxed.value_pointer).*;
        }
    "#,
        Limits::default(),
    );
    assert!(matches!(empty, Outcome::Failed(jai_vm::Error::NullPointer)));
}

#[test]
fn uninitialized_descriptor_can_be_assembled_by_both_pointer_field_writes() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            value:=42;
            descriptor:Any=---;
            descriptor.type=cast(*Type_Info)type_info(int);
            descriptor.value_pointer=cast(*void)*value;
            copy:Any=descriptor;
            return (cast(*int)copy.value_pointer).*;
        }
    "#),
        42
    );
}

#[test]
fn initialized_descriptor_field_reads_do_not_read_its_unwritten_sibling() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            value:=42;
            descriptor:Any=---;
            descriptor.type=cast(*Type_Info)type_info(int);
            if descriptor.type==null return 1;
            descriptor.value_pointer=cast(*void)*value;
            return (cast(*int)descriptor.value_pointer).*;
        }
    "#),
        42
    );
}

#[test]
fn explicit_descriptor_literals_and_empty_literals_keep_the_canonical_type() {
    assert_eq!(
        run(r#"
        main :: () -> int {
            value:=42;
            descriptor:=Any.{type=cast(*Type_Info)type_info(int),value_pointer=cast(*void)*value};
            empty:=Any.{};
            if empty.type!=null return 1;
            return (cast(*int)descriptor.value_pointer).*;
        }
    "#),
        42
    );
}

#[test]
fn universal_call_arguments_accept_contextual_and_explicit_descriptor_literals() {
    assert_eq!(
        run(r#"
        read :: (value:Any) -> int {
            if value.type==null return 0;
            return (cast(*int)value.value_pointer).*;
        }
        main :: () -> int {
            if read(.{})!=0 return 1;
            value:=42;
            return read(Any.{type=cast(*Type_Info)type_info(int),value_pointer=cast(*void)*value});
        }
    "#),
        42
    );
}

#[test]
fn source_storage_mirror_pointer_views_keep_distinct_universal_conversion_rules() {
    assert_eq!(
        run(r#"
        Mirror :: struct { type:*Type_Info; value_pointer:*void; }
        main :: () -> int {
            original:=1;
            replacement:=42;
            descriptor:Any=original;
            mirror:=cast(*Mirror)*descriptor;
            mirror.value_pointer=cast(*void)*replacement;
            mirror_copy:=mirror.*;
            boxed_mirror:Any=mirror_copy;
            if cast(*void)boxed_mirror.type!=cast(*void)type_info(Mirror) return 1;
            return (cast(*int)descriptor.value_pointer).*;
        }
    "#),
        42
    );
}

#[test]
fn unused_universal_parameters_have_complete_runtime_storage() {
    assert_eq!(
        run("unused :: (value:Any) {} main :: () -> int { return 42; }"),
        42
    );
}

#[test]
fn default_universal_globals_have_two_null_pointers() {
    assert_eq!(
        run(
            "value:Any; main :: () -> int { if value.type!=null return 1; if value.value_pointer!=null return 2; return 42; }"
        ),
        42
    );
}

#[test]
fn boxed_pointer_payload_supports_scan2_double_pointer_access() {
    assert_eq!(
        run(r#"
            main :: () -> int {
                value:=0;
                pointer:=*value;
                argument:Any=pointer;
                actual:=(cast(**int)argument.value_pointer).*;
                actual.*=42;
                return value;
            }
        "#),
        42
    );
}

#[test]
fn formatter_style_records_embed_universal_values_with_normal_field_storage() {
    assert_eq!(
        run(
            "Format :: struct { value:Any; width:int; } main :: () -> int { original:=20; item:Format=.{value=original,width=22}; original=21; return (cast(*int)item.value.value_pointer).*+item.width-1; }"
        ),
        42
    );
}

#[test]
fn array_of_any_is_an_array_payload_with_embedded_descriptors() {
    assert_eq!(
        run(r#"
            main :: () -> int {
                array:[2]Any;
                original:=20;
                array[0]=original;
                array[1]=22;
                boxed:Any=array;
                values:=cast(*[2]Any)boxed.value_pointer;
                original=21;
                return (cast(*int)values.*[0].value_pointer).*
                     + (cast(*int)values.*[1].value_pointer).* - 1;
            }
        "#),
        42
    );
}

#[test]
fn escaped_local_payload_reports_released_storage() {
    assert_eq!(
        outcome(
            "escape :: () -> Any { local:=42; return local; } main :: () -> int { value:=escape(); return (cast(*int)value.value_pointer).*; }",
            Limits::default(),
        ),
        Outcome::Failed(jai_vm::Error::DanglingPointer)
    );
}

#[test]
fn escaped_materialized_literal_does_not_gain_hidden_heap_ownership() {
    assert_eq!(
        outcome(
            "escape :: () -> Any { return 42; } main :: () -> int { value:=escape(); return (cast(*int)value.value_pointer).*; }",
            Limits::default(),
        ),
        Outcome::Failed(jai_vm::Error::DanglingPointer)
    );
}

#[test]
fn frame_temporaries_use_the_existing_allocation_budget() {
    let source = "main :: () -> int { for i:0..100 { boxed:Any=i+1; } return 42; }";
    let result = outcome(
        source,
        Limits {
            allocations: 64,
            ..Limits::default()
        },
    );
    assert!(
        matches!(
            result,
            Outcome::Failed(jai_vm::Error::Limit(jai_vm::LimitKind::Allocations))
        ),
        "{result:?}"
    );
}

#[test]
fn compile_time_calls_box_arguments_in_their_request_scope_and_publish_scalar_results() {
    assert_eq!(
        run(r#"
        read :: (value:Any) -> int { return (cast(*int)value.value_pointer).*; }
        answer :: #run read(42);
        main :: () -> int { return answer; }
    "#),
        42
    );
}

#[test]
fn unsized_runtime_payloads_are_rejected_explicitly() {
    let fixture = Fixture::new("main :: () { value:Any=#code {}; }");
    let error = fixture.program().unwrap_err();
    assert!(
        error.contains("storage") || error.contains("layout") || error.contains("unsized"),
        "{error}"
    );
}

#[test]
fn runtime_type_boxing_contains_a_canonical_descriptor_pointer_cell() {
    assert_eq!(
        run(r#"
            main :: () -> int {
                boxed:Any=s32;
                if cast(int)boxed.type.type!=13 return 1;
                if boxed.type.runtime_size!=size_of(Type) return 2;
                descriptor:=(cast(**Type_Info)boxed.value_pointer).*;
                if cast(*void)descriptor!=cast(*void)type_info(s32) return 3;
                if descriptor.runtime_size!=4 return 4;
                return 42;
            }
        "#),
        42
    );
}

#[test]
fn boxed_runtime_type_lvalues_borrow_their_mutable_pointer_cell() {
    assert_eq!(
        run(r#"
            main :: () -> int {
                chosen:Type=s32;
                boxed:Any=chosen;
                chosen=s64;
                descriptor:=(cast(**Type_Info)boxed.value_pointer).*;
                if cast(*void)descriptor!=cast(*void)type_info(s64) return 1;
                if descriptor.runtime_size!=8 return 2;
                return 42;
            }
        "#),
        42
    );
}

#[test]
fn runtime_type_any_arguments_work_during_compile_time_and_native_ready_calls() {
    assert_eq!(
        run(r#"
            inspect :: (value:Any) -> int {
                descriptor:=(cast(**Type_Info)value.value_pointer).*;
                return descriptor.runtime_size+38;
            }
            answer :: #run inspect(s32);
            main :: () -> int { return answer+inspect(s64)-46; }
        "#),
        42
    );
}

#[test]
fn explicit_lower_case_universal_tag_preserves_ordinary_any_name_shadowing() {
    assert_eq!(
        run(r#"
            Universal::#type any;
            any::struct{value:int;}
            main::()->int{
                ordinary:any=.{value=41};
                {
                    any:=ordinary.value+1;
                    boxed:Universal=any;
                    if cast(*void)boxed.type!=cast(*void)type_info(int) return 1;
                    return (cast(*int)boxed.value_pointer).*;
                }
            }
        "#),
        42
    );
}

#[test]
fn procedure_payloads_preserve_the_canonical_signature_and_callable_pointer_cell() {
    assert_eq!(
        run(r#"
            Increment::#type (value:s32)->s32;
            increment::(value:s32)->s32{return value+1;}
            main::()->int {
                boxed:Any=increment;
                if cast(*void)boxed.type!=cast(*void)type_info(Increment) return 1;
                callable:=(cast(*Increment)boxed.value_pointer).*;
                return callable(41);
            }
        "#),
        42
    );
}
