//! Procedure fixtures run through the VM and freshly emitted native objects.
#[path = "support/native_tools.rs"]
mod native_tools;
use std::{
    fs,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

fn execute(source: &str, files: &[(&str, &str)]) {
    execute_fixture(source, files, false);
}

#[test]
fn local_operator_returns_keep_per_call_callback_contracts() {
    execute_fixture_with_optimizations(
        "Box::struct{value:int;}Optional::#type(value:int)->int;Required::#type(value:int)->int #must;counter:int;answer::(value:int)->int{counter+=1;return value;}main::()->int{operator []::(a:Box,callback:$F)->F{return callback;}required:=Box.{value=1}[cast(Required)answer];optional:=Box.{value=2}[cast(Optional)answer];optional(value=0);return required(value=40)+counter;}",
        &[],
        false,
        &[
            jai_types::BitcodeOptimization::O0,
            jai_types::BitcodeOptimization::O2,
        ],
    );
}

#[test]
fn graph_operator_returns_keep_source_casts_and_compact_baked_slots() {
    for (operator, required, optional) in [
        (
            "operator []::(a:Box,callback:$F)->F{return callback;}",
            "Box.{value=1}[cast(Required)answer]",
            "Box.{value=2}[cast(Optional)answer]",
        ),
        (
            "operator []::(a:Box,$callback:$F)->F{return callback;}",
            "Box.{value=1}[cast(Required)answer]",
            "Box.{value=2}[cast(Optional)answer]",
        ),
    ] {
        execute_fixture_with_optimizations(
            &format!(
                "Box::struct{{value:int;}}Optional::#type(value:int)->int;Required::#type(value:int)->int #must;counter:int;answer::(value:int)->int{{counter+=1;return value;}}{operator}main::()->int{{required:={required};optional:={optional};optional(value=0);return required(value=40)+counter;}}"
            ),
            &[],
            false,
            &[
                jai_types::BitcodeOptimization::O0,
                jai_types::BitcodeOptimization::O2,
            ],
        );
    }
    execute_fixture_with_optimizations(
        "Optional::#type(value:int)->int;Required::#type(value:int)->int #must;counter:int;answer::(value:int)->int{counter+=1;return value;}Holder::struct(T:Type){callback:T;}operator +::(a:$T,b:int)->T #symmetric{return a;}main::()->int{required_receiver:Holder(Required);required_receiver.callback=answer;optional_receiver:Holder(Optional);optional_receiver.callback=answer;required:=1+required_receiver;optional:=optional_receiver+2;optional.callback(value=0);return required.callback(value=40)+counter;}",
        &[],
        false,
        &[
            jai_types::BitcodeOptimization::O0,
            jai_types::BitcodeOptimization::O2,
        ],
    );
}

#[test]
fn omitted_generic_callback_defaults_keep_the_defining_cast_in_vm_and_native_calls() {
    execute_fixture_with_optimizations(
        "Required::#type(value:int)->int #must;Optional::#type(value:int)->int;counter:int;answer::(value:int)->int{counter+=1;return value;}forward::(callback:$F=cast(Required)answer)->F{return callback;}main::()->int{required:=forward();optional:=forward(cast(Optional)answer);optional(value=0);return required(value=40)+counter;}",
        &[],
        false,
        &[
            jai_types::BitcodeOptimization::O0,
            jai_types::BitcodeOptimization::O2,
        ],
    );
    execute_fixture_with_optimizations(
        "Dependency::#import \"Dependency\";Required::#type(other:int)->int;main::()->int{callback:=Dependency.forward();return callback(value=42);}",
        &[(
            "Dependency.jai",
            "#scope_file Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}#scope_export forward::(callback:$F=cast(Required)answer)->F{return callback;}",
        )],
        false,
        &[
            jai_types::BitcodeOptimization::O0,
            jai_types::BitcodeOptimization::O2,
        ],
    );
}

#[test]
fn baked_callbacks_keep_optional_source_policy_and_execute_once() {
    execute_fixture_with_optimizations(
        "Optional::#type()->int;counter:int;answer::()->int #must{counter+=1;return 0;}probe::($$callback:Optional)->int{#if is_constant(callback){callback();return 41;}else{return 0;}}main::()->int{return probe(cast(Optional)answer)+counter;}",
        &[],
        false,
        &[
            jai_types::BitcodeOptimization::O0,
            jai_types::BitcodeOptimization::O2,
        ],
    );
}

#[test]
fn promoted_callback_capture_runs_once_and_keeps_named_required_results() {
    execute(
        "counter:int;Required::#type(value:int)->int #must;Holder::struct{struct{callback:=answer;}tag:int;}answer::(value:int=21)->int #must{return value;}make::()->Required{counter+=1;return answer;}main::()->int{holder:=Holder.{tag=1,callback=make()};copy:=holder;return copy.callback()+copy.callback(value=19)+counter+copy.tag;}",
        &[],
    );
    execute(
        "counter:int;Required::#type(value:int)->int #must;Holder::struct{struct{callback:Required;}}answer::(value:int)->int{return value;}make::()->Required{counter+=1;return answer;}main::()->int{return Holder.{callback=make()}.callback(value=41)+counter;}",
        &[],
    );
}

#[test]
fn discarded_baked_values_keep_runtime_results_and_distinct_keys() {
    execute(
        "ignore::(#discard $value:int)->int{return 21;}main::()->int{return ignore(1)+ignore(2);}",
        &[],
    );
}
fn execute_fixture(source: &str, files: &[(&str, &str)], bootstrap: bool) {
    execute_fixture_with_optimizations(
        source,
        files,
        bootstrap,
        &[jai_types::BitcodeOptimization::Unset],
    );
}

fn execute_fixture_with_optimizations(
    source: &str,
    files: &[(&str, &str)],
    bootstrap: bool,
    optimizations: &[jai_types::BitcodeOptimization],
) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-procedure-native-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let input = fixture.0.join("main.jai");
    fs::write(&input, source).unwrap();
    for (path, source) in files {
        let path = fixture.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    let options = jai_modules::GraphOptions {
        import_dirs: vec![fixture.0.join("modules")],
    };
    let graph = if bootstrap {
        jai_modules::ModuleGraph::load_with_bootstrap_options(
            &input,
            options,
            jai_modules::BootstrapOptions {
                prelude: jai_modules::PreludeSource::Search,
                runtime_support: Some(jai_modules::RuntimeSupportOptions {
                    source: jai_modules::RuntimeSupportSource::Search,
                    parameters: jai_modules::RuntimeSupportParameters {
                        define_system_entry_point: false,
                        define_initialization: false,
                        enable_backtrace_on_crash: false,
                    },
                }),
            },
            &jai_modules::Filesystem,
            None,
        )
    } else {
        jai_modules::ModuleGraph::load(&input, options)
    }
    .unwrap();
    let program = jai_sema::resolve_graph(&graph).unwrap();
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("{:?}", execution.outcome)
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
    for &optimization in optimizations {
        let context = jai_codegen::Context::create();
        let target =
            jai_codegen::target::NativeTarget::select(&jai_codegen::target::TargetOptions {
                optimization: jai_codegen::optimization::Optimization {
                    bitcode: optimization,
                    ..jai_codegen::optimization::Optimization::default()
                },
                ..jai_codegen::target::TargetOptions::default()
            })
            .unwrap();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        let object = fixture.0.join("program.o");
        target.write_object(&module, &object).unwrap();
        let executable = fixture.0.join("program");
        let linked = native_tools::clang_command()
            .arg(&object)
            .arg("-o")
            .arg(&executable)
            .output()
            .unwrap();
        assert!(
            linked.status.success(),
            "{}",
            String::from_utf8_lossy(&linked.stderr)
        );
        let mut child = Command::new(executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(42), "{optimization:?}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("generated procedure fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

#[test]
fn binding_names_preserve_argument_order_and_signature_identity() {
    execute(
        "counter:int=0; next::()->int{counter+=1;return counter;} digits::(a:int,b:int)->int{return a*10+b;} main::()->int{f:(x:int,y:int)->int=digits; g:(y:int,x:int)->int=digits; first:=f(y=next(),x=next()); return first+g(x=next(),y=next())-22;}",
        &[],
    );
}
#[test]
fn callback_alias_parameter_and_record_metadata() {
    execute(
        "Callback::#type(left:int,right:int)->int; Holder::struct{callback:Callback;} digits::(a:int,b:int)->int{return a*10+b;} apply::(callback:Callback)->int{return callback(right=2,left=4);} main::()->int{holder:Holder;holder.callback=digits;return apply(holder.callback);}",
        &[],
    );
}
#[test]
fn inferred_callback_defaults_and_named_variadic_tail() {
    execute(
        "digits::(a:int,b:int=2)->int{return a*10+b;} count::(args:..int,extra:int=2)->int{return args.count*20+extra;} main::()->int{f:=digits;pack:=count;return f(a=4)+pack(20,21,extra=2)-42;}",
        &[],
    );
}
#[test]
fn nullable_procedures_keep_typed_identity() {
    execute(
        "answer::()->int{return 42;} main::()->int{callback:()->int=null;if callback{return 0;}if callback!=null{return 1;}callback=answer;if !callback{return 2;}if callback==answer{return callback();}return 3;}",
        &[],
    );
}
#[test]
fn explicit_context_type_default_and_callable_members() {
    execute(
        "#add_context number:int=40; read::(value:#Context)->int{return value.number;} #add_context callback::read; #add_context extra::2; main::()->int{value:#Context;return context.callback(value)+context.extra;}",
        &[],
    );
}

#[test]
fn imported_callback_metadata_resolves_private_aliases_in_defining_file() {
    execute(
        "Lib::#import \"Callbacks\"; digits::(a:int,b:int)->int{return a*10+b;} main::()->int{value:Lib.Holder; value.callback=digits; Lib.callback=digits; return value.callback(right=2,left=4)+Lib.callback(right=2,left=4)-42;}",
        &[(
            "modules/Callbacks.jai",
            "#scope_file Callback::#type(left:int,right:int)->int; #scope_export Holder::struct{callback:Callback;} callback:Callback;",
        )],
    );
}

#[test]
fn forwarded_pack_preserves_descriptor_and_trailing_fixed_arguments() {
    execute(
        "count::(args:..int,extra:int=2)->int{return args.count*20+extra;} forward::(values:[]int)->int{callback:=count;return callback(..values,2);} main::()->int{values:[2]int=.[20,21];return forward(values);}",
        &[],
    );
}

#[test]
fn context_overrides_copy_current_record_and_restore_after_call() {
    execute(
        "#add_context number:int=7; #add_context extra:int=0; read::()->int{return context.number+context.extra;} main::()->int{return read(,,number=40,extra=2)+context.number-7;}",
        &[],
    );
}

#[test]
fn context_allocator_shorthand_and_source_order_evaluate_once() {
    execute(
        "counter:int=0;#add_context allocator:int=7;next::()->int{counter+=1;return counter;}read::(arg:int)->int{return arg*10+context.allocator;}main::()->int{return read(next(),,next())+counter*15+context.allocator-7;}",
        &[],
    );
}

#[test]
fn context_call_multiple_results_capture_once() {
    execute(
        "#add_context number:int=7;#add_context extra:int=0;read::()->int,int{return context.number,context.extra;}main::()->int{a,b:=read(,,number=40,extra=2);return a+b;}",
        &[],
    );
}

#[test]
fn callback_string_defaults_use_typed_sequence_inference() {
    execute(
        "size::(name:=\"AB\")->int{return name.count*21;}main::()->int{callback:=size;return callback();}",
        &[],
    );
}

#[test]
fn nested_context_calls_restore_outer_override_and_propagate_to_indirect_callees() {
    execute(
        "#add_context number:int=7;read::()->int{return context.number;}identity::(value:int)->int{return value+context.number;}main::()->int{callback:=identity; return callback(read(,,number=20),,number=22)+context.number-7;}",
        &[],
    );
}

#[test]
fn embedded_context_fields_preserve_storage_and_call_override_paths() {
    execute(
        "Base::struct{number:int=40;} #add_context using base:Base; #add_context extra:int=2; read::()->int{return context.number+context.extra;} main::()->int{context.number=1; answer:=read(,,number=40); return answer+context.base.number-1;}",
        &[],
    );
}

#[test]
fn required_results_execute_through_source_and_callback_bindings() {
    execute(
        "Callback::#type()->int #must; required::()->int #must{return 42;} identity::(x:$T)->T #must{return x;} apply::(f:Callback)->int{return f();} main::()->int{local::()->int #must{return 42;} f:=local; first:=f(); second:=identity(first);return apply(required)+second-42;}",
        &[],
    );
}

#[test]
fn optional_result_discards_preserve_direct_and_indirect_call_effects() {
    execute(
        "counter:int=0; pair::()->(int,int){counter+=1;return 20,22;} required_pair::()->(int,int #must){return 0,42;} main::()->int{pair();f:=pair;f();_,answer:=required_pair();return answer+counter-2;}",
        &[],
    );
}

#[test]
fn preload_context_quote_precedes_extensions_and_promotes_original_storage() {
    let preload = format!(
        "{}\nFIRST_ADD_CONTEXT::#code #add_context #as using base:Context_Base;",
        include_str!("../../../stdlib/prelude/module.jai")
    );
    execute_fixture(
        "Context_Base::struct{wrong:int;} FIRST_ADD_CONTEXT::3; #add_context extra:int=2; read::()->int{return context.number+context.extra;} main::()->int{context.number=1; answer:=read(,,number=40); return answer+context.base.number-1;}",
        &[
            ("modules/module.jai", &preload),
            (
                "modules/Runtime_Support.jai",
                "#module_parameters(DEFINE_SYSTEM_ENTRY_POINT:bool,DEFINE_INITIALIZATION:bool,ENABLE_BACKTRACE_ON_CRASH:bool);Context_Base::struct{number:int=40;}",
            ),
        ],
        true,
    );
}

#[test]
fn context_callback_fields_resolve_private_alias_metadata_in_defining_module() {
    execute(
        "Extension::#import \"ContextCallbacks\";digits::(a:int,b:int)->int{return a*10+b;}main::()->int{context.callback=digits;return context.callback(right=2,left=4);}",
        &[(
            "modules/ContextCallbacks.jai",
            "#scope_file Callback::#type(left:int,right:int)->int;#add_context callback:Callback;",
        )],
    );
}

#[test]
fn explicit_context_record_constants_are_available_without_implicit_context() {
    execute(
        "#add_context answer::42;read::(value:#Context)->int #no_context{return value.answer;}main::()->int #no_context{value:#Context;return read(value);}",
        &[],
    );
}

#[test]
fn procedure_constants_initialize_globals_records_and_callback_defaults() {
    execute(
        "Callback::#type()->int;answer::()->int{return 42;} global:Callback=answer; inferred:=answer; Holder::struct{callback:Callback=answer;} holder:Holder; apply::(callback:Callback=answer)->int{return callback();} main::()->int{value:Holder;return global()+inferred()+holder.callback()+value.callback()+apply()-168;}",
        &[],
    );
}

#[test]
fn named_result_defaults_can_return_actual_procedure_constants() {
    execute(
        "Callback::#type()->int; answer::()->int{return 42;} choose::()->(callback:Callback=answer){return;}main::()->int{callback:=choose();return callback();}",
        &[],
    );
}

#[test]
fn inferred_procedure_fields_and_arrays_keep_nonnull_callback_roots() {
    execute(
        "answer::()->int{return 42;} unused::()->int{return 99;} Holder::struct{callback:=answer;} callbacks:[2]()->int=.[answer,answer]; main::()->int{value:Holder;return value.callback()+callbacks[0]()+callbacks[1]()-84;}",
        &[],
    );
}

#[test]
fn inferred_global_and_record_callbacks_retain_declaration_names_and_defaults() {
    execute(
        "digits::(left:int,right:int=2)->int{return left*10+right;} callback:=digits; Holder::struct{callback:=digits;} main::()->int{value:Holder;return callback(left=4)+value.callback(left=4)-42;}",
        &[],
    );
}

#[test]
fn imported_procedure_defaults_keep_the_defining_declaration_identity() {
    execute(
        "Callbacks::#import \"Constants\";answer::()->int{return 99;}main::()->int{value:Callbacks.Holder;return Callbacks.callback()+value.callback()-42;}",
        &[(
            "modules/Constants.jai",
            "answer::()->int{return 42;}callback:()->int=answer;Holder::struct{callback:()->int=answer;}",
        )],
    );
}

#[test]
fn inferred_callback_headers_follow_forward_procedures_and_aliases() {
    execute(
        "alias::answer; apply::(callback:=alias)->int{return callback();}answer::()->int{return 42;}main::()->int{return apply();}",
        &[],
    );
}

#[test]
fn inferred_callback_headers_follow_imports_on_either_side_of_the_header() {
    for source in [
        "Callbacks::#import \"ForwardCallbacks\";apply::(callback:=Callbacks.answer)->int{return callback();}main::()->int{return apply();}",
        "apply::(callback:=Callbacks.answer)->int{return callback();}Callbacks::#import \"ForwardCallbacks\";main::()->int{return apply();}",
    ] {
        execute(
            source,
            &[(
                "modules/ForwardCallbacks.jai",
                "answer::()->int{return 42;}",
            )],
        );
    }
}

#[test]
fn procedure_alias_constants_keep_the_selected_identity_in_inferred_fields() {
    execute(
        "answer::()->int{return 42;} alias::answer; callback:()->int=alias; Holder::struct{callback:=alias;}main::()->int{value:Holder;return callback()+value.callback()-42;}",
        &[],
    );
}

#[test]
fn record_method_procedure_constants_keep_the_selected_target() {
    execute(
        "Provider::struct{answer::()->int{return 42;}} alias::Provider.answer; callback:()->int=alias; Holder::struct{callback:()->int=Provider.answer;}main::()->int{value:Holder;return callback()+value.callback()-42;}",
        &[],
    );
}

#[test]
fn record_method_defaults_and_globals_keep_nonnull_procedure_identity() {
    execute(
        "Provider::struct{answer::()->int{return 42;}} callback:()->int=Provider.answer; Holder::struct{callback:()->int=Provider.answer;}main::()->int{value:Holder;return callback()+value.callback()-42;}",
        &[],
    );
}

#[test]
fn procedure_constant_casts_preserve_exact_signature_and_null() {
    execute(
        "Callback::#type()->int; answer::()->int{return 42;} callback:Callback=cast(Callback)answer; empty:Callback=cast(Callback)null; Holder::struct{callback:Callback=cast(Callback)answer;} main::()->int{value:Holder;local:=cast(Callback)answer;if empty{return 0;}return callback()+value.callback()+local()-84;}",
        &[],
    );
}

#[test]
fn inferred_procedure_casts_keep_the_annotated_type_and_actual_target() {
    execute(
        "Callback::#type()->int; constant::cast(Callback)answer; callback:=cast(Callback)answer; Holder::struct{callback:=cast(Callback)answer;} apply::(callback:=cast(Callback)answer)->int{return callback();}answer::()->int{return 42;}main::()->int{value:Holder;return constant()+callback()+value.callback()+apply()-126;}",
        &[],
    );
}

#[test]
fn inferred_procedure_casts_keep_annotation_parameter_names() {
    execute(
        "Callback::#type(left:int,right:int)->int;digits::(a:int,b:int)->int{return a*10+b;}callback:=cast(Callback)digits;Holder::struct{callback:=cast(Callback)digits;}apply::(callback:=cast(Callback)digits)->int{return callback(right=2,left=4);}main::()->int{value:Holder;local:=cast(Callback)digits;return callback(right=2,left=4)+value.callback(right=2,left=4)+apply()+local(right=2,left=4)-126;}",
        &[],
    );
}

#[test]
fn positional_aggregate_constants_keep_actual_procedure_initializers() {
    execute(
        "answer::()->int{return 42;} Holder::struct{callback:()->int;data:*void;} defaults::Holder.{answer,null}; value:Holder=defaults; main::()->int{return value.callback();}",
        &[],
    );
}

#[test]
fn scoped_aggregate_constants_initialize_inferred_context_callback_fields() {
    execute(
        "answer::()->int{return 42;} Allocator::struct{proc:()->int;data:*void;} Base::struct{allocator:=default_allocator;default_allocator::Allocator.{answer,null};} #add_context using base:Base; main::()->int{return context.allocator.proc();}",
        &[],
    );
}

#[test]
fn c_abi_procedure_constants_keep_the_declared_calling_convention() {
    execute(
        "Callback::#type()->int #c_call; answer::()->int #c_call{return 42;} callback:Callback=answer; Holder::struct{callback:Callback=answer;} main::()->int{value:Holder;return callback()+value.callback()-42;}",
        &[],
    );
}

#[test]
fn source_foreign_procedure_constants_retain_external_symbol_roots() {
    execute(
        "Callback::#type(value:s32)->s32 #c_call; external::(value:s32)->s32 #foreign \"abs\"; callback:Callback=external; main::()->int{if callback!=null{return 42;}return 0;}",
        &[],
    );
}

#[test]
fn positional_record_literals_apply_defaults_and_evaluate_values_once() {
    execute(
        "counter:int=0; next::()->int{counter+=1;return 40;} answer::()->int{return 42;} Holder::struct{number:int;extra:int=2;callback:()->int=answer;} main::()->int{first:=Holder.{next()};second:Holder=.{40};return first.number+first.extra+second.callback()+counter-43;}",
        &[],
    );
}

#[test]
fn local_positional_record_literals_use_the_local_declaration_order() {
    execute(
        "main::()->int{Pair::struct{left:int;right:int=2;} value:Pair=.{40};return value.left+value.right;}",
        &[],
    );
}

#[test]
fn context_field_defaults_keep_nonnull_procedure_identity_and_binding_names() {
    execute(
        "digits::(a:int,b:int)->int{return a*10+b;}#add_context callback:(left:int,right:int)->int=digits;main::()->int{return context.callback(right=2,left=4);}",
        &[],
    );
}

#[test]
fn required_results_execute_through_imported_constants_and_inferred_defaults() {
    execute(
        "Callbacks::#import \"MustCallbacks\";apply::(callback:=Callbacks.required)->int{return callback();} main::()->int{holder:Callbacks.Holder;return Callbacks.callback()+holder.callback()+apply()-84;}",
        &[(
            "modules/MustCallbacks.jai",
            "required::(value:int=42)->int #must{return value;} alias::required; callback:=alias; Holder::struct{callback:=alias;}",
        )],
    );
}

#[test]
fn returned_required_callbacks_execute_through_direct_and_indirect_factories() {
    execute(
        "Callback::#type(value:int)->int #must;required::(value:int)->int{return value;}get::()->Callback{return required;}pair::()->(Callback,Callback){return required,required;}generic::(unused:$T)->Callback{return required;}main::()->int{a:=get();factory:=get;b:=factory();c,d:=pair();pair_factory:=pair;e,f:=pair_factory();g:=generic(0);return a(value=6)+b(value=6)+c(value=6)+d(value=6)+e(value=6)+f(value=6)+g(value=6);}",
        &[],
    );
}

#[test]
fn callback_container_and_cast_contracts_execute_with_source_parameter_names() {
    execute(
        "Callback::#type(value:int)->int #must;required::(value:int)->int{return value;}get_array::()->[2]Callback{return .[required,required];}main::()->int{values:[2]Callback=.[required,required];view:[]Callback=values;callback:Callback=required;pointer:=*callback;casted:=cast(Callback)required;returned:=get_array()[0];return values[0](value=7)+view[1](value=7)+view.data[0](value=7)+pointer.*(value=7)+casted(value=7)+returned(value=7);}",
        &[],
    );
}

#[test]
fn imported_returned_callback_contracts_execute_from_private_definition_aliases() {
    execute(
        "Dependency::#import \"ReturnedCallbacks\";Callback::#type()->int;main::()->int{callback:=Dependency.get();return callback(value=42);}",
        &[(
            "modules/ReturnedCallbacks.jai",
            "#scope_file Callback::#type(value:int)->int #must;#scope_export required::(value:int)->int{return value;}get::()->Callback{return required;}",
        )],
    );
}

#[test]
fn context_and_hinted_factories_preserve_returned_callback_contracts() {
    execute(
        "#add_context number:int=0;Callback::#type(value:int)->int #must;answer::(value:int)->int{return value+context.number;}get::()->Callback{return answer;}main::()->int{a:=get(,,number=3);factory:=get;b:=factory(,,number=4);c:=inline get();d:=no_inline factory();return a(value=10)+b(value=10)+c(value=11)+d(value=11);}",
        &[],
    );
}

#[test]
fn generic_callback_and_container_results_keep_the_actual_call_contract() {
    execute(
        "Callback::#type(value:int)->int #must;required::(value:int)->int #must{return value;}optional::(value:int)->int{return value;}identity::(value:$T)->T{return value;}pair::(value:$T)->(T,T){return value,value;}main::()->int{a:=identity(required);b:=identity(optional);b(0);c,d:=pair(required);values:[2]Callback=.[optional,optional];copy:=identity(values);return a(value=6)+c(value=6)+d(value=6)+copy[0](value=12)+copy[1](value=12);}",
        &[],
    );
}

#[test]
fn baked_type_callback_results_keep_each_source_alias_contract() {
    execute(
        "Required::#type(value:int)->int #must;Optional::#type(value:int)->int;answer::(value:int)->int{return value;}make::($T:Type)->T{return cast(T)answer;}main::()->int{required:=make(Required);optional:=make(Optional);optional(value=0);return required(value=42);}",
        &[],
    );
}

#[test]
fn inferred_generic_callback_parameters_consume_required_results() {
    execute(
        "required::(value:int=42)->int #must{return value;}apply::(callback:$T)->int{return callback();}main::()->int{return apply(required);}",
        &[],
    );
}

#[test]
fn generic_record_callback_contracts_share_nominal_type_without_sharing_usage() {
    execute(
        "Required::#type(value:int)->int #must;Optional::#type(value:int)->int;answer::(value:int)->int{return value;}Holder::struct(T:Type){callback:T;}RequiredHolder::#type Holder(Required);OptionalHolder::#type Holder(Optional);identity::(value:$T)->T{return value;}get::()->RequiredHolder{result:RequiredHolder;result.callback=answer;return result;}main::()->int{required:Holder(Required);required.callback=answer;optional:Holder(Optional)=required;optional.callback(value=0);alias:OptionalHolder=required;alias.callback(value=0);copy:=identity(required);returned:=get();return copy.callback(value=21)+returned.callback(value=21);}",
        &[],
    );
}

#[test]
fn recursive_generic_record_callback_contracts_follow_live_pointers() {
    execute(
        "Required::#type(value:int)->int #must;Optional::#type(value:int)->int;answer::(value:int)->int{return value;}Node::struct(T:Type){callback:T;next:*Node(T);}main::()->int{required:Node(Required);required.callback=answer;required.next=*required;optional:Node(Optional)=required;optional.callback(value=0);optional.next.next.callback(value=0);return required.callback(value=21)+required.next.next.callback(value=21);}",
        &[],
    );
}

#[test]
fn imported_generic_record_callback_fields_keep_defining_parameter_names() {
    execute(
        "Dependency::#import \"ReturnedRecord\";Callback::#type(other:int)->int;main::()->int{value:=Dependency.get();return value.callback(value=42);}",
        &[(
            "modules/ReturnedRecord.jai",
            "#scope_file Callback::#type(value:int)->int #must;#scope_export Holder::struct(T:Type){callback:T;}answer::(value:int)->int{return value;}get::()->Holder(Callback){result:Holder(Callback);result.callback=answer;return result;}",
        )],
    );
}

#[test]
fn discarded_callback_source_slots_are_checked_but_have_no_vm_or_native_effects() {
    execute(
        "counter:int;tick::()->int{counter+=1;return 9;}Callback::#type(#discard ignored:int,value:int)->int;Alias::#type Callback;read::(#discard ignored:int=tick(),value:int=14)->int{return value;}main::()->int{callback:Alias=read;a:=read();b:=callback(value=14,ignored=tick());c:=callback(#run tick(),14);return a+b+c+counter;}",
        &[],
    );
}

#[test]
fn discarded_imported_callback_proofs_keep_private_nominal_types() {
    execute(
        "Lib::#import \"DiscardCallbacks\";counter:int;make::()->Lib.Value{counter+=1;return .{21};}main::()->int{callback:=Lib.get();value:Lib.Value=.{21};return callback(hidden=make(),value=value)+callback(hidden=#run make(),value=value)+counter;}",
        &[(
            "modules/DiscardCallbacks.jai",
            "#scope_file Private::struct{value:int;}Callback::#type(#discard hidden:Private,value:Private)->int;#scope_export Value::#type Private;read::(#discard hidden:Private,value:Private)->int{return value.value;}get::()->Callback{return read;}",
        )],
    );
}

#[test]
fn discarded_variadic_source_pack_has_no_vm_or_native_pack_storage() {
    execute(
        "counter:int;tick::()->int{counter+=1;return 9;}pack::(#discard values:..int,value:int=42)->int{return value;}main::()->int{return pack(tick(),#run tick(),value=42)+counter;}",
        &[],
    );
}

#[test]
fn discarded_nested_callback_calls_keep_source_slots_without_evaluation() {
    execute(
        "counter:int;tick::()->int{counter+=1;return 9;}Callback::#type(#discard ignored:int,value:int)->int;read::(#discard ignored:int,value:int)->int{counter+=100;return value;}ignore::(#discard value:int)->int{return 42;}main::()->int{callback:Callback=read;return ignore(callback(value=7,ignored=#run tick()))+counter;}",
        &[],
    );
}

#[test]
fn discarded_context_calls_check_real_fields_without_context_or_argument_effects() {
    execute(
        "#add_context number:int=7;counter:int;tick::()->int{counter+=1;return 9;}Callback::#type(#discard ignored:int,value:int)->int;read::(#discard ignored:int,value:int)->int{counter+=100;return value+context.number;}ignore::(#discard value:int)->int{return 42;}main::()->int{callback:Callback=read;return ignore(callback(value=7,ignored=#run tick(),,number=#run tick()))+counter+context.number-7;}",
        &[],
    );
}

#[test]
fn compact_generic_callback_slots_and_discarded_inference_execute_required_results() {
    execute(
        "calls:int;tick::()->int{calls+=1;return 9;}required::()->int #must{return 21;}forward::(#discard ignored:int,callback:$T)->T{return callback;}discarded::(#discard callback:$T)->T{value:T=required;return value;}main::()->int{first:=forward(tick(),required);second:=discarded(required);return first()+second()+calls*100;}",
        &[],
    );
}

#[test]
fn generic_callbacks_from_record_pointer_and_index_paths_execute_checked_results() {
    execute(
        "Required::#type(value:int)->int #must;required::(value:int)->int #must{return value;}Holder::struct{callback:Required=required;}apply::(callback:$T)->int{return callback(value=14);}main::()->int{holder:Holder;pointer:=*holder;items:[1]Holder;return apply(holder.callback)+apply(pointer.callback)+apply(items[0].callback);}",
        &[],
    );
}

#[test]
fn equal_conditional_callback_policies_preserve_named_and_default_arguments() {
    execute(
        "first::(value:int=21)->int #must{return value;}second::(value:int=21)->int{return value;}main::()->int{callback:=ifx false then first else second;return callback(value=21)+callback();}",
        &[],
    );
}

#[test]
fn overload_aliases_keep_forward_target_defaults_in_vm_and_native_calls() {
    execute(
        "pick::later;later::(value:int=19)->int{return value;}pick::(value:bool)->int{return ifx value then 23 else 0;}main::()->int{return pick()+pick(value=true);}",
        &[],
    );
}

#[test]
fn overload_aliases_keep_imported_private_target_defaults_in_vm_and_native_calls() {
    execute(
        "main::()->int{Library::#import,file \"library.jai\";return Library.pick()+Library.pick(value=true);}",
        &[(
            "library.jai",
            "#scope_file DEFAULT::19;hidden::(value:int=DEFAULT)->int{return value;}forward::hidden;#scope_export pick::forward;pick::(value:bool)->int{return ifx value then 23 else 0;}",
        )],
    );
}

#[test]
fn overload_aliases_keep_actual_generic_specializations_in_vm_and_native_calls() {
    execute(
        "pick::identity;identity::(value:$T)->T{return value;}pick::(value:bool)->int{return ifx value then 23 else 0;}main::()->int{left:int=20;right:int=22;return pick(value=left)+pick(value=right);}",
        &[],
    );
}

#[test]
fn discarded_structural_annotations_keep_source_proof_and_ready_counts_in_vm_and_native_calls() {
    execute(
        "counter:int;read::(input:int)->int{counter+=1;return input;}ignore::(#discard value:int)->int{return 42;}main::()->int{return ignore((cast((#discard hidden:bool,value:int)->int)read)(hidden=true,value=7))+counter;}",
        &[],
    );
    execute(
        "Count::1;ignore::(#discard value:[1]int)->int{return 42;}main::()->int{return ignore(#run ->[Count]int{source:[Count]int=.[7];return source;});}",
        &[],
    );
    execute(
        "Callback::#type(value:int,#discard hidden:bool)->int;read::(input:int)->int{return input;}main::()->int{callback:Callback=read;return callback(value=#run ->int{value:=0;for index:0..2{value+=14;}return value;},hidden=true);}",
        &[],
    );
}

#[test]
fn positional_capture_keeps_source_order_defaults_and_required_callbacks() {
    execute(
        "trace:int;take::(value:int)->int{trace=trace*10+value;return value;}Pair::struct{left:int;right:int;tail:int=8;}main::()->int{pair:=Pair.{take(1),take(2)};if trace==12 && pair.left==1 && pair.right==2 && pair.tail==8{return 42;}return 0;}",
        &[],
    );
    execute(
        "counter:int;Required::#type(value:int)->int #must;Holder::struct{callback:Required;tag:int=2;}answer::(value:int)->int{return value;}make::()->Required{counter+=1;return answer;}main::()->int{holder:=Holder.{make()};copy:=holder;return copy.callback(value=39)+copy.tag+counter;}",
        &[],
    );
}

#[test]
fn positional_and_named_sequence_descriptors_use_actual_slots() {
    for literal in ["Slice.{2,*items[0]}", "Slice.{count=2,data=*items[0]}"] {
        execute(
            &format!(
                "Slice::#type []int;main::()->int{{items:[2]int;items[0]=20;items[1]=22;view:={literal};return view[0]+view[1];}}"
            ),
            &[],
        );
    }
    execute(
        "Required::#type(value:int)->int #must;Slice::#type []Required;answer::(value:int)->int{return value;}main::()->int{items:[1]Required;items[0]=answer;view:=Slice.{1,*items[0]};return view[0](value=42);}",
        &[],
    );
}
