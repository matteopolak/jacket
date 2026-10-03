//! Independently authored source runs in the VM and freshly generated native code.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

fn check(source: &str, expected: i32) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-short-lambda-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let path = fixture.0.join("main.jai");
    fs::write(&path, source).unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
    let target = jai_codegen::target::NativeTarget::new().unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            target: Some(target.build_target().unwrap()),
            layout: Some(target.layout_policy().unwrap()),
            ..jai_sema::ResolveOptions::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("VM did not complete: {execution:?}")
    };
    assert_eq!(values[0].integer().unwrap().value(), i128::from(expected));
    for level in [
        jai_codegen::optimization::BitcodeOptimization::O0,
        jai_codegen::optimization::BitcodeOptimization::O2,
    ] {
        let target =
            jai_codegen::target::NativeTarget::select(&jai_codegen::target::TargetOptions {
                optimization: jai_codegen::optimization::Optimization {
                    bitcode: level,
                    ..Default::default()
                },
                ..Default::default()
            })
            .unwrap();
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = fixture.0.join(format!("program-{level:?}.o"));
        let executable = fixture.0.join(format!("program-{level:?}"));
        target.write_object(&module, &object).unwrap();
        let linked = native_tools::clang_command()
            .arg(&object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            linked.status.success(),
            "{level:?}: {}",
            String::from_utf8_lossy(&linked.stderr)
        );
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(expected), "{level:?}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("freshly emitted {level:?} lambda program timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn contextual_callbacks_and_typed_inferred_results_share_the_native_abi() {
    check(
        include_str!("../../../tests/fixtures/short-lambdas.jai"),
        42,
    );
}

#[test]
fn block_callback_preview_checks_locals_updates_and_return_branches() {
    check(
        "apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{return apply(40,value=>{OFFSET::2;result:=value;result+=OFFSET;if result==42{return result;}else{return 0;}});}",
        42,
    );
}

#[test]
fn block_callback_overloads_select_the_checked_result_type() {
    check(
        "choose::(f:(int)->bool)->int{if f(1){return 0;}return 1;}choose::(f:(int)->int)->int{return f(40);}main::()->int{return choose(value=>{result:=value+2;return result;});}",
        42,
    );
}

#[test]
fn dependent_block_callback_infers_actual_return_types() {
    check(
        "apply::(value:$T,f:(T)->$R)->R{return f(value);}main::()->int{return cast(int)apply(cast(u8)40,value=>{result:u16=cast(u16)value;if value>0{return result+2;}else{return cast(u16)2;}});}",
        42,
    );
}

#[test]
fn dependent_block_callback_joins_strong_integer_return_ranges() {
    check(
        "apply::(value:$T,f:(T)->$R)->R{return f(value);}main::()->int{result:=apply(cast(u8)40,value=>{if value==40{return cast(u8)42;}else{return cast(u16)1000;}});if type_of(result)==u16{return cast(int)result;}return 0;}",
        42,
    );
}

#[test]
fn contextual_block_callback_keeps_all_weak_integer_return_ranges() {
    check(
        "apply::(value:$T,f:(T)->u64)->u64{return f(value);}main::()->int{result:=apply(40,value=>{if value==40{return 42;}else{return 18446744073709551615;}});if type_of(result)==u64{return cast(int)result;}return 0;}",
        42,
    );
}

#[test]
fn dependent_block_callback_joins_strong_float_return_widths() {
    check(
        "apply::(value:$T,f:(T)->$R)->R{return f(value);}main::()->int{result:=apply(40,value=>{if value==40{return cast(float32)42.0;}else{return cast(float64)100.0;}});if type_of(result)==float64{return cast(int)result;}return 0;}",
        42,
    );
}

#[test]
fn void_block_callback_checks_defer_without_executing_it_during_preview() {
    check(
        "count:int;sink::(value:int,f:(int)->void){f(value);}main::()->int{sink(40,value=>{defer{count+=2;}count=value;return;});return count;}",
        42,
    );
}

#[test]
fn block_callback_preview_tracks_loop_locals_and_fallthrough() {
    check(
        "apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{return apply(40,value=>{result:=0;while result<value{result+=1;}return result+2;});}",
        42,
    );
}

#[test]
fn block_callback_pointer_places_keep_the_real_mutable_pointee() {
    check(
        "apply::(value:*int,f:(*int)->void){f(value);}main::()->int{value:=40;apply(*value,pointer=>{pointer[0]+=2;});return value;}",
        42,
    );
}

#[test]
fn block_callback_type_queries_use_the_actual_parameter_type_fact() {
    check(
        "apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{return apply(40,value=>{T::type_of(value);result:T=value;return result+2;});}",
        42,
    );
}

#[test]
fn block_callback_decimal_constants_remain_contextual() {
    check(
        "apply::(value:float32,f:(float32)->float32)->float32{return f(value);}main::()->int{return cast(int)apply(40.0,value=>{OFFSET::2.0;return value+OFFSET;});}",
        42,
    );
}

#[test]
fn block_callback_integer_literals_acquire_the_float_constant_annotation() {
    check(
        "apply::(value:float32,f:(float32)->float32)->float32{return f(value);}main::()->int{return cast(int)apply(40.0,value=>{OFFSET:float32:2;return value+OFFSET;});}",
        42,
    );
}

#[test]
fn dependent_block_callback_keeps_annotated_constant_widths() {
    check(
        "apply::(value:$T,f:(T)->$R)->R{return f(value);}main::()->int{result:=apply(cast(u8)40,value=>{OFFSET:u16:cast(u8)2;return value+OFFSET;});if type_of(result)==u16{return cast(int)result;}return 0;}",
        42,
    );
}

#[test]
fn lexical_constants_and_global_storage_keep_their_defining_bindings() {
    check(
        "global:int=5; main::()->int{BASE::37; f:(int)->int=(value)=>value+BASE+global; {BASE::100;return f(0);} }",
        42,
    );
}

#[test]
fn contextual_record_results_keep_nominal_type_identity() {
    check(
        "Pair::struct{left:int;right:int;} main::()->int{f:(int)->Pair=(value)=>.{left=value,right=2};pair:=f(40);return pair.left+pair.right;}",
        42,
    );
}

#[test]
fn typed_short_lambda_float_result_is_inferred_from_the_checked_body() {
    check(
        "main::()->int{f:=(value:float32)=>value+2.0;return cast(int)f(40.0);}",
        42,
    );
}

#[test]
fn short_lambda_c_callback_adopts_the_expected_calling_convention() {
    check(
        "main::()->int{f:(int)->int #c_call=(value)=>value+2;return f(40);}",
        42,
    );
}

#[test]
fn directly_called_untyped_lambda_infers_arguments_without_reordering_effects() {
    check(
        "counter:int=0;next::()->int{counter+=1;return counter;}main::()->int{return ((left,right)=>left*10+right)(right=next(),left=next())+21;}",
        42,
    );
}

#[test]
fn named_local_lambda_specializes_multiple_actual_argument_types() {
    check(
        "main::()->int{identity::(value)=>value;byte:u8=20;wide:int=22;return cast(int)identity(byte)+identity(wide);}",
        42,
    );
}

#[test]
fn named_local_lambda_uses_its_definition_scope_after_shadowing() {
    check(
        "main::()->int{OFFSET::2;add::(value)=>value+OFFSET;{OFFSET::99;return add(40);}}",
        42,
    );
}

#[test]
fn named_lambda_expected_void_and_inferred_result_cache_keys_are_distinct() {
    check(
        "main::()->int{identity::(value)=>value;first:=identity(40);sink:(int)->void=identity;sink(1);return first+2;}",
        42,
    );
}

#[test]
fn local_record_namespace_short_lambdas_preserve_source_owner() {
    check(
        "hash::(key:int)->u64{return cast(u64)key;} main::()->int{Map::struct{HashKey::(key)=>hash(key);CompareKeys::(a,b)=>a==b;}if Map.CompareKeys(40,40){return cast(int)Map.HashKey(42);}return 0;}",
        42,
    );
}

#[test]
fn lambda_callback_overloads_validate_result_types_before_selection() {
    check(
        "apply::(value:int,callback:(int)->int)->int{return callback(value);}apply::(value:int,callback:(int)->bool)->int{if callback(value){return value;}return 0;}main::()->int{return apply(42,(value)=>value==42);}",
        42,
    );
}

#[test]
fn short_lambda_type_result_uses_the_canonical_runtime_descriptor() {
    check(
        "main::()->int{inferred:=(value:int)=>type_of(value);expected:(int)->Type=(value)=>type_of(value);if inferred(1)==expected(2){return 42;}return 0;}",
        42,
    );
}

#[test]
fn lambda_runtime_type_parameter_and_result_preserve_descriptor_identity() {
    check(
        "main::()->int{identity:=(value:Type)=>value;actual:=identity(int);if actual==int{return 42;}return 0;}",
        42,
    );
}

#[test]
fn module_named_lambda_aliases_remain_source_recipes_until_typed_use() {
    check(
        "inc::(value)=>value+1;alias::inc;alias2::alias;answer::#run alias2(41);main::()->int{return answer;}",
        42,
    );
}

#[test]
fn module_lambda_keeps_definition_scope_and_callback_context() {
    check(
        "OFFSET::2;add::(value)=>value+OFFSET;apply::(value:int,callback:(int)->int)->int{return callback(value);}other::()->int{return add(1);}main::()->int{OFFSET::99;return apply(40,add);}",
        42,
    );
}

#[test]
fn generic_hash_map_namespace_lambdas_use_actual_key_specializations() {
    check(
        "get_hash::(key:int)->u64{return cast(u64)key;}Map::struct(Key:Type){HashKey::(key)=>get_hash(key);CompareKeys::(a,b)=>a==b;}main::()->int{M::Map(int);equal:(int,int)->bool=M.CompareKeys;if equal(42,42){return cast(int)M.HashKey(42);}return 0;}",
        42,
    );
}

#[test]
fn contextual_void_lambda_body_keeps_the_call_effect_once() {
    check(
        "global:int=0;set::(value:int){global=value;}apply::(value:int,callback:(int)->void){callback(value);}main::()->int{apply(42,(value)=>set(value));return global;}",
        42,
    );
}

#[test]
fn bare_block_lambda_assignment_keeps_statement_source_and_returns() {
    check(
        "global:int=0;main::()->int{shutdown:(int)->void=value=>{global=value;};shutdown(40);add:(int)->int=value=>{return value+2;};return add(global);}",
        42,
    );
}

#[test]
fn typed_block_lambda_without_value_returns_infers_void() {
    check(
        "global:int=0;main::()->int{set:=(value:int)=>{global=value;};set(42);return global;}",
        42,
    );
}

#[test]
fn lambda_callback_parameter_call_is_checked_through_its_canonical_signature() {
    check(
        "add::(value:int)->int{return value+2;}apply::(callback:((int)->int,int)->int)->int{return callback(add,40);}main::()->int{return apply((callback,value)=>callback(value));}",
        42,
    );
}

#[test]
fn lambda_explicit_callback_parameter_retains_its_source_binding_names() {
    check(
        "add::(value:int)->int{return value+2;}main::()->int{invoke:=(callback:(left:int)->int)=>callback(left=40);return invoke(add);}",
        42,
    );
}

#[test]
fn typed_direct_lambda_arguments_receive_the_declared_context() {
    check(
        "Mode::enum u8{OFF;ON;}Pair::struct{left:int;right:int;}main::()->int{pick::(value:Mode)=>value;identity::(value:Pair)=>value;empty::(value:*int)=>value;pair:=identity(.{left=40,right=2});if pick(.ON)==.ON && empty(null)==null{return pair.left+pair.right;}return 0;}",
        42,
    );
}

#[test]
fn named_lambda_parameter_annotations_resolve_in_the_defining_scope() {
    check(
        "ValueType::int;identity::(value:ValueType)=>value;main::()->int{ValueType::bool;return identity(42);}",
        42,
    );
}

#[test]
fn explicitly_typed_lambda_callback_argument_keeps_its_source_body() {
    check(
        "main::()->int{invoke::(callback:(int)->int)=>callback(40);return invoke(value=>value+2);}",
        42,
    );
}

#[test]
fn named_inferred_lambda_statement_call_keeps_the_single_runtime_effect() {
    check(
        "count:int;total:int;tick::(value:int)->int{count+=1;total=value+1;return total;}main::()->int{sink::(value)=>tick(value);sink(41);return total+count-1;}",
        42,
    );
}

#[test]
fn generic_callback_lambda_infers_its_result_after_the_value_type() {
    check(
        "apply::(values:[]$T,callback:(T,T)->$R)->R{return callback(values[0],values[1]);}main::()->int{values:[2]u8=u8.[40,2];return cast(int)apply(values,(left,right)=>cast(u16)left+cast(u16)right);}",
        42,
    );
}

#[test]
fn generic_callback_lambda_context_does_not_depend_on_parameter_order() {
    check(
        "apply::(callback:(T,T)->$R,values:[]$T)->R{return callback(values[0],values[1]);}main::()->int{values:[2]u8=u8.[22,20];return cast(int)apply((left,right)=>cast(u16)left+cast(u16)right,values);}",
        42,
    );
}

#[test]
fn generic_callback_lambda_infers_void_from_the_actual_body_call() {
    check(
        "total:int;set::(value:int){total=value;}apply::(value:$T,callback:(T)->$R){callback(value);}main::()->int{apply(42,value=>set(value));return total;}",
        42,
    );
}

#[test]
fn typed_module_and_local_lambda_constants_supply_their_result_context() {
    check(
        "module_add:(u8)->(u16):(value)=>cast(u16)value+2;main::()->int{local_add:(u8)->(u16):(value)=>cast(u16)value+20;return cast(int)module_add(20)+cast(int)local_add(0);}",
        42,
    );
}

#[test]
fn named_module_lambda_alias_infers_the_dependent_callback_result() {
    check(
        "combine::(left,right)=>cast(u16)left+cast(u16)right;alias::combine;apply::(values:[]$T,callback:(T,T)->$R)->R{return callback(values[0],values[1]);}main::()->int{values:[2]u8=u8.[40,2];return cast(int)apply(values,alias);}",
        42,
    );
}

#[test]
fn named_local_dependent_callback_preview_uses_the_defining_constants() {
    check(
        "apply::(values:[]$T,callback:(T,T)->$R)->R{return callback(values[0],values[1]);}main::()->int{OFFSET::2;combine::(left,right)=>cast(u16)left+cast(u16)right+OFFSET;{OFFSET::100;values:[2]u8=u8.[20,20];return cast(int)apply(values,combine);}}",
        42,
    );
}

#[test]
fn annotated_lambda_constant_calls_use_the_annotation_parameter_names() {
    check(
        "add:(left:int)->(int):value=>value+2;main::()->int{return add(left=40);}",
        42,
    );
}

#[test]
fn annotated_lambda_constant_discard_policy_keeps_the_canonical_runtime_slots() {
    check(
        "count:int;tick::()->int{count+=1;return 0;}add:(#discard unused:int,left:int)->(int):value=>value+2;main::()->int{return add(tick(),40)+count;}",
        42,
    );
}
