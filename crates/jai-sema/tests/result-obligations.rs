//! Required result contracts are source metadata, not ABI signature distinctions.
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{Library, resolve_library};
use jai_source::LocatedDiagnostic;
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-result-obligations-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("main.jai"), source).unwrap();
        Self(path)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str) -> Result<Library, LocatedDiagnostic> {
    resolve_library(&Fixture::new(source).graph())
}

#[test]
fn unused_required_declarations_and_consumed_results_are_accepted() {
    for source in [
        "unused::()->int #must{return 42;} main::(){}",
        "foreign::()->int #must #foreign; main::(){}",
        "required::()->int #must{return 42;} main::()->int{return required();}",
        "required::()->int #must{return 42;} main::()->int{x:=required();return x;}",
        "pair::()->(int,int #must){return 0,42;} main::()->int{_,x:=pair();return x;}",
        "pair::()->(int,int #must){return 0,42;} main::()->int{x:int;_,x=pair();return x;}",
        "pair::()->(int,int #must){return 0,42;} main::()->int{x:int;x=,answer:=pair();return answer;}",
        "identity::(x:$T)->T #must{return x;} main::()->int{return identity(42);}",
        "main::()->int{required::()->int #must{return 42;}return required();}",
        "Callback::#type()->int #must; required::()->int #must{return 42;} apply::(f:Callback)->int{return f();} main::()->int{return apply(required);}",
    ] {
        check(source).unwrap_or_else(|error| panic!("{source}\n{error:?}"));
    }
}

#[test]
fn ignored_results_report_the_call_span_across_binding_forms() {
    for (prefix, body, call) in [
        (
            "required::()->int #must{return 42;}",
            "required();",
            "required()",
        ),
        (
            "required::()->int #must{return 42;}",
            "f:=required;f();",
            "f()",
        ),
        (
            "required::()->int #must{return 42;}",
            "f:=required;g:=f;g();",
            "g()",
        ),
        (
            "required::()->int #must{return 42;} optional::()->int{return 42;}",
            "f:=ifx true then required else optional;f();",
            "f()",
        ),
        (
            "Callback::#type()->int #must; required::()->int{return 42;}",
            "f:Callback=required;f();",
            "f()",
        ),
        (
            "Callback::#type()->int #must; global:Callback;",
            "global();",
            "global()",
        ),
        (
            "Holder::struct{callback:()->int #must;}",
            "holder:Holder;holder.callback();",
            "holder.callback()",
        ),
        (
            "",
            "required::()->int #must{return 42;}required();",
            "required()",
        ),
        (
            "identity::(x:$T)->T #must{return x;}",
            "identity(42);",
            "identity(42)",
        ),
        (
            "pair::()->(int #must,int){return 42,0;}",
            "_,x:=pair();",
            "pair()",
        ),
        (
            "pair::()->(int,int #must){return 0,42;}",
            "x:int;x,_=pair();",
            "pair()",
        ),
        (
            "pair::()->(int,int #must){return 0,42;}",
            "pair();",
            "pair()",
        ),
        (
            "pair::()->(int,int #must){return 0,42;}",
            "x:int;x=,_:=pair();",
            "pair()",
        ),
        (
            "pair::()->(int #must,int){return 42,0;}",
            "_=,answer:=pair();",
            "pair()",
        ),
        (
            "required::()->int #must{return 42;}",
            "_:=required();",
            "required()",
        ),
        (
            "required::()->int #must{return 42;}",
            "_=required();",
            "required()",
        ),
    ] {
        let source = format!("{prefix}\nmain::(){{{body}}}");
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {source}"));
        assert!(error.message.contains("#must"), "{source}\n{error:?}");
        let start = source.rfind(call).unwrap();
        assert_eq!(error.location.span.start, start, "{source}\n{error:?}");
        assert_eq!(
            error.location.span.end,
            start + call.len(),
            "{source}\n{error:?}"
        );
    }
}

#[test]
fn callback_unused_and_optional_binding_contracts_keep_canonical_identity() {
    let library = check("required::()->int #must{return 42;} optional::()->int{return 42;} main::(){a:=required;b:=optional;f:()->int=required;f();}").unwrap();
    assert_eq!(
        library.procedures()[0].signature,
        library.procedures()[1].signature
    );
    check("pair::()->(int,int){return 42,0;} main::(){pair();f:=pair;f();_,x:=pair();}").unwrap();
}

#[test]
fn callback_parameter_contract_is_enforced_in_its_defining_body() {
    let source = "Callback::#type()->int #must;
apply::(callback:Callback){callback();}";
    let error = check(source)
        .err()
        .expect("required callback parameter discarded");
    assert!(error.message.contains("#must"), "{error:?}");
    assert_eq!(
        error.location.span.start,
        source.find("callback()").unwrap()
    );
}

#[test]
fn source_callback_result_annotations_reject_duplicates_and_parameter_usage() {
    for source in [
        "Callback::#type(int #must)->int;",
        "Callback::#type()->int #must #must;",
    ] {
        let fixture = Fixture::new(source);
        assert!(ModuleGraph::load(&fixture.0.join("main.jai"), GraphOptions::default()).is_err());
    }
}

#[test]
fn procedure_constants_global_inference_and_default_callbacks_keep_required_results() {
    for source in [
        "required::()->int #must{return 42;} alias::required; main::(){alias();}",
        "required::()->int #must{return 42;} alias::required; global:=alias; main::(){global();}",
        "required::()->int #must{return 42;} Holder::struct{callback:=required;} main::(){value:Holder;value.callback();}",
        "required::()->int #must{return 42;} apply::(callback:()->int #must=required){callback();}",
        "required::()->int #must{return 42;} apply::(callback:=required){callback();}",
        "Callback::#type()->int #must;optional::()->int{return 42;}callback:=cast(Callback)optional;main::(){callback();}",
        "Callback::#type()->int #must;optional::()->int{return 42;}Holder::struct{callback:=cast(Callback)optional;}main::(){holder:Holder;holder.callback();}",
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("accepted {source}"));
        assert!(error.message.contains("#must"), "{source}\n{error:?}");
    }
}

#[test]
fn imported_callback_aliases_globals_and_record_defaults_preserve_usage() {
    for body in [
        "Dependency.required();",
        "f:=Dependency.alias;f();",
        "Dependency.callback();",
        "holder:Dependency.Holder;holder.callback();",
        "Dependency.casted();",
        "holder:Dependency.CastedHolder;holder.callback();",
    ] {
        let fixture = Fixture::new(&format!(
            "Dependency::#import \"Dependency\"; main::(){{{body}}}"
        ));
        fs::write(fixture.0.join("Dependency.jai"), "#scope_file Callback::#type()->int #must;#scope_export required::()->int #must{return 42;} optional::()->int{return 42;} alias::required; callback:=alias; Holder::struct{callback:=alias;}casted:=cast(Callback)optional;CastedHolder::struct{callback:=cast(Callback)optional;}").unwrap();
        let graph = ModuleGraph::load(
            &fixture.0.join("main.jai"),
            GraphOptions {
                import_dirs: vec![fixture.0.clone()],
            },
        )
        .unwrap();
        let error = resolve_library(&graph)
            .err()
            .unwrap_or_else(|| panic!("accepted {body}"));
        assert!(error.message.contains("#must"), "{body}\n{error:?}");
    }
}

#[test]
fn returned_callbacks_keep_their_source_contract_through_calls_and_containers() {
    for body in [
        "f:=get();f();",
        "factory:=get;f:=factory();f();",
        "get()();",
        "f,g:=pair();f();",
        "factory:=pair;f,g:=factory();g();",
        "f:=inline get();f();",
        "f:=no_inline get();f();",
        "f:=generic(0);f();",
        "values:[2]Callback=.[required,required];values[0]();",
        "values:[2]Callback=.[required,required];view:[]Callback=values;view[0]();",
        "values:[2]Callback=.[required,required];view:[]Callback=values;view.data[0]();",
        "f:Callback=required;pointer:=*f;pointer.*();",
        "f:=cast(Callback)required;f();",
        "(cast(Callback)required)();",
        "f:=get_array()[0];f();",
        "factory:Factory=get;f:=factory();f();",
        "f:=default_factory();f();",
        "f:=ifx true then cast(Callback)required else required;f();",
        "values:=.[cast(Callback)required];values[0]();",
        "values:=Callback.[required];values[0]();",
    ] {
        let source = format!(
            "Callback::#type()->int #must;Factory::#type()->Callback;required::()->int{{return 42;}}must_source::()->int #must{{return 42;}}get::()->Callback{{return required;}}pair::()->(Callback,Callback){{return required,required;}}generic::(unused:$T)->Callback{{return required;}}get_array::()->[2]Callback{{return .[required,required];}}default_factory::()->(callback:=must_source){{return;}}main::(){{{body}}}"
        );
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {body}"));
        assert!(error.message.contains("#must"), "{body}\n{error:?}");
    }
}

#[test]
fn higher_order_parameter_contracts_are_checked_in_the_defining_body() {
    for source in [
        "Callback::#type()->int #must;Factory::#type()->Callback;apply::(factory:Factory){f:=factory();f();}",
        "Callback::#type()->int #must;apply::(values:[]Callback){values[0]();}",
        "Callback::#type()->int #must;apply::(pointer:*Callback){pointer.*();}",
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("accepted {source}"));
        assert!(error.message.contains("#must"), "{source}\n{error:?}");
    }
}

#[test]
fn equal_canonical_types_do_not_share_returned_or_field_obligations() {
    check("Required::#type()->int #must;Optional::#type()->int;answer::()->int{return 42;}required_factory::()->Required{return answer;}optional_factory::()->Optional{return answer;}RequiredHolder::struct{callback:Required=answer;}OptionalHolder::struct{callback:Optional=answer;}main::()->int{required:=required_factory();optional:=optional_factory();a:RequiredHolder;b:OptionalHolder;optional();b.callback();weakened:=cast(Optional)required;weakened();return required();}").unwrap();
    check("Required::#type()->int #must;Optional::#type()->int;answer::()->int #must{return 42;}pair::()->(Required,int){return answer,42;}main::()->int{callback:Optional=answer;callback=,value:=pair();callback();return value;}").unwrap();
}

#[test]
fn local_return_contracts_preserve_definition_aliases() {
    for body in [
        "Callback::#type()->int #must;get::()->Callback{return required;}f:=get();f();",
        "Callback::#type()->int #must;get::()->Callback{return required;}factory:=get;f:=factory();f();",
    ] {
        let source =
            format!("Callback::#type()->int;required::()->int{{return 42;}}main::(){{{body}}}");
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {body}"));
        assert!(error.message.contains("#must"), "{body}\n{error:?}");
    }
}

#[test]
fn imported_return_contracts_keep_private_defining_aliases() {
    let fixture = Fixture::new(
        "Dependency::#import \"Dependency\";Callback::#type()->int;main::(){f:=Dependency.get();f();}",
    );
    fs::write(fixture.0.join("Dependency.jai"), "#scope_file Callback::#type()->int #must;#scope_export required::()->int{return 42;}get::()->Callback{return required;}").unwrap();
    let graph = ModuleGraph::load(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: vec![fixture.0.clone()],
        },
    )
    .unwrap();
    let error = resolve_library(&graph)
        .err()
        .expect("discarded imported callback required result");
    assert!(error.message.contains("#must"), "{error:?}");
}

#[test]
fn context_factories_preserve_required_callback_results() {
    for body in [
        "f:=get(,,number=3);f();",
        "factory:=get;f:=factory(,,number=3);f();",
    ] {
        let source = format!(
            "#add_context number:int=0;Callback::#type()->int #must;answer::()->int{{return 42;}}get::()->Callback{{return answer;}}main::(){{{body}}}"
        );
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {body}"));
        assert!(error.message.contains("#must"), "{body}\n{error:?}");
    }
}

#[test]
fn generic_result_variables_keep_per_call_callback_contracts() {
    for body in [
        "f:=identity(required);f();",
        "f:=inline identity(required);f();",
        "f,g:=pair(required);g();",
        "array:[2]Callback=.[optional,optional];copy:=identity(array);copy[0]();",
        "f:=make(Callback);f();",
        "f:=inline make(Callback);f();",
        "make(Callback)();",
    ] {
        let source = format!(
            "Callback::#type()->int #must;required::()->int #must{{return 42;}}optional::()->int{{return 42;}}identity::(value:$T)->T{{return value;}}pair::(value:$T)->(T,T){{return value,value;}}make::($T:Type)->T{{return cast(T)optional;}}main::(){{{body}}}"
        );
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {body}"));
        assert!(error.message.contains("#must"), "{body}\n{error:?}");
    }
    check("required::()->int #must{return 42;}optional::()->int{return 42;}identity::(value:$T)->T{return value;}main::()->int{a:=identity(required);b:=identity(optional);b();return a();}").unwrap();
    check("Required::#type()->int #must;Optional::#type()->int;answer::()->int{return 42;}make::($T:Type)->T{return cast(T)answer;}main::()->int{a:=make(Required);b:=make(Optional);b();return a();}").unwrap();
}

#[test]
fn inferred_generic_callback_parameter_requires_consumption() {
    for (source, call) in [
        (
            "required::()->int #must{return 42;}apply::(callback:$T){callback();}main::(){apply(required);}",
            "callback()",
        ),
        (
            "required::()->int #must{return 42;}apply::(callback:$T){alias:=callback;alias();}main::(){apply(required);}",
            "alias()",
        ),
        (
            "Callback::#type()->int #must;optional::()->int{return 42;}apply::(values:$T){values[0]();}main::(){values:[2]Callback=.[optional,optional];apply(values);}",
            "values[0]()",
        ),
        (
            "required::()->int #must{return 42;}optional::()->int{return 42;}apply::(callback:$T,n:int){if n>0{apply(required,n-1);}callback();}main::(){apply(optional,0);}",
            "callback()",
        ),
    ] {
        let error = check(source)
            .err()
            .expect("generic inferred callback must retain its actual source contract");
        assert!(error.message.contains("#must"), "{source}\n{error:?}");
        let start = source.rfind(call).unwrap();
        assert_eq!(error.location.span.start, start, "{source}\n{error:?}");
        assert_eq!(
            error.location.span.end,
            start + call.len(),
            "{source}\n{error:?}"
        );
    }
    check(
        "optional::()->int{return 42;}apply::(callback:$T){callback();}main::(){apply(optional);}",
    )
    .unwrap();
}

#[test]
fn generic_record_callback_contracts_belong_to_each_source_application() {
    let prefix = "Required::#type(value:int)->int #must;Optional::#type(value:int)->int;answer::(value:int)->int{return value;}Holder::struct(T:Type){callback:T;}Node::struct(T:Type){callback:T;next:*Node(T);}Nested::struct(T:Type){holder:Holder(T);}DefaultHolder::struct(T:Type=Required){callback:T;}RequiredHolder::#type Holder(Required);OptionalHolder::#type Holder(Optional);get::()->Holder(Required){result:Holder(Required);result.callback=answer;return result;}make::($T:Type)->Holder(T){result:Holder(T);result.callback=answer;return result;}use::($T:Type){value:Holder(T);value.callback=answer;value.callback(value=42);}identity::(value:$T)->T{return value;}";
    for (body, call) in [
        (
            "value:Holder(Required);value.callback=answer;value.callback(value=42);",
            "value.callback(value=42)",
        ),
        (
            "value:RequiredHolder;value.callback=answer;value.callback(value=42);",
            "value.callback(value=42)",
        ),
        (
            "value:Holder(Required);value.callback=answer;copy:=identity(value);copy.callback(value=42);",
            "copy.callback(value=42)",
        ),
        (
            "value:Holder(Required);value.callback=answer;pointer:=*value;pointer.callback(value=42);",
            "pointer.callback(value=42)",
        ),
        (
            "values:[2]Holder(Required);values[0].callback=answer;values[0].callback(value=42);",
            "values[0].callback(value=42)",
        ),
        (
            "value:=get();value.callback(value=42);",
            "value.callback(value=42)",
        ),
        (
            "value:=make(Required);value.callback(value=42);",
            "value.callback(value=42)",
        ),
        (
            "value:DefaultHolder();value.callback=answer;value.callback(value=42);",
            "value.callback(value=42)",
        ),
        (
            "value:Nested(Required);value.holder.callback=answer;value.holder.callback(value=42);",
            "value.holder.callback(value=42)",
        ),
        (
            "value:Node(Required);value.callback=answer;value.next=*value;value.next.next.callback(value=42);",
            "value.next.next.callback(value=42)",
        ),
    ] {
        let source = format!("{prefix}main::(){{{body}}}");
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {source}"));
        assert!(error.message.contains("#must"), "{source}\n{error:?}");
        let start = source.rfind(call).unwrap();
        assert_eq!(error.location.span.start, start, "{source}\n{error:?}");
        assert_eq!(
            error.location.span.end,
            start + call.len(),
            "{source}\n{error:?}"
        );
    }
    check(&format!("{prefix}main::()->int{{required:Holder(Required);required.callback=answer;optional:Holder(Optional)=required;optional.callback(value=0);alias:OptionalHolder=required;alias.callback(value=0);return required.callback(value=42);}}")).unwrap();
    let source = format!("{prefix}main::(){{use(Required);}}");
    let error = check(&source)
        .err()
        .expect("generic baked callback contract must validate record field invocation");
    assert!(error.message.contains("#must"), "{error:?}");
    let start = source.find("value.callback(value=42)").unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    let source = "Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}Holder::struct(T:Type){Saved::#type T;callback:Saved;}main::(){value:Holder(Required);value.callback=answer;value.callback(value=42);}";
    let error = check(source)
        .err()
        .expect("record namespace alias must forward source type argument contract");
    assert!(error.message.contains("#must"), "{error:?}");
}

#[test]
fn imported_generic_record_arguments_keep_private_callback_aliases() {
    let fixture = Fixture::new(
        "Dependency::#import \"Dependency\";Callback::#type(other:int)->int;main::(){value:=Dependency.get();value.callback(value=42);}",
    );
    fs::write(fixture.0.join("Dependency.jai"), "#scope_file Callback::#type(value:int)->int #must;#scope_export Holder::struct(T:Type){callback:T;}answer::(value:int)->int{return value;}get::()->Holder(Callback){result:Holder(Callback);result.callback=answer;return result;}").unwrap();
    let graph = ModuleGraph::load(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: vec![fixture.0.clone()],
        },
    )
    .unwrap();
    let error = resolve_library(&graph)
        .err()
        .expect("private callback source argument must remain required");
    assert!(error.message.contains("#must"), "{error:?}");
}

#[test]
fn discarded_source_slots_preserve_compact_callback_argument_contracts() {
    let prefix = "required::()->int #must{return 42;}forward::(#discard ignored:int,callback:$T)->T{return callback;}discarded::(#discard callback:$T)->T{value:T=required;return value;}";
    for (body, call) in [
        ("callback:=forward(0,required);callback();", "callback()"),
        ("callback:=discarded(required);callback();", "callback()"),
    ] {
        let source = format!("{prefix}main::(){{{body}}}");
        let error = check(&source)
            .err()
            .expect("source contract must survive erased slots");
        assert!(error.message.contains("#must"), "{error:?}");
        let start = source.rfind(call).unwrap();
        assert_eq!(error.location.span.start, start, "{error:?}");
        assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
    }
    check(&format!(
        "{prefix}main::()->int{{callback:=forward(0,required);return callback();}}"
    ))
    .unwrap();
}

#[test]
fn pure_generic_callback_previews_preserve_field_and_pointer_sources() {
    for callback in ["holder.callback", "pointer.callback", "items[0].callback"] {
        let prefix = "Required::#type()->int #must;required::()->int #must{return 42;}Holder::struct{callback:Required=required;}";
        let locals = "holder:Holder;pointer:=*holder;items:[1]Holder;";
        check(&format!("{prefix}apply::(callback:$T)->int{{return callback();}}main::()->int{{{locals}return apply({callback});}}"))
            .unwrap_or_else(|error| panic!("{callback}: {error:?}"));
        let source = format!(
            "{prefix}apply::(callback:$T)->int{{callback();return 42;}}main::()->int{{{locals}return apply({callback});}}"
        );
        let error = check(&source)
            .err()
            .expect("field callback contract must remain required");
        assert!(error.message.contains("#must"), "{callback}: {error:?}");
        let start = source.find("callback()").unwrap();
        assert_eq!(error.location.span.start, start, "{error:?}");
        assert_eq!(
            error.location.span.end,
            start + "callback()".len(),
            "{error:?}"
        );
    }
}

#[test]
fn generic_slice_conversions_preserve_required_callback_elements() {
    let prefix = "Required::#type()->int #must;required::()->int #must{return 42;}";
    check(&format!("{prefix}consume::(items:[]$T)->int{{return items[0]();}}main::()->int{{items:[1]Required;items[0]=required;return consume(items);}}"))
        .unwrap();
    let source = format!(
        "{prefix}consume::(items:[]$T)->int{{items[0]();return 42;}}main::()->int{{items:[1]Required;items[0]=required;return consume(items);}}"
    );
    let error = check(&source)
        .err()
        .expect("slice conversion must preserve required element contract");
    assert!(error.message.contains("#must"), "{error:?}");
    let start = source.find("items[0]()").unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(
        error.location.span.end,
        start + "items[0]()".len(),
        "{error:?}"
    );
}

#[test]
fn equal_source_parameter_policies_keep_names_defaults_and_required_results() {
    let prefix =
        "first::(value:int=21)->int #must{return value;}second::(value:int=21)->int{return value;}";
    for body in [
        "callback:=ifx true then first else second;return callback(value=42);",
        "callback:=ifx true then first else second;return callback()+21;",
    ] {
        check(&format!("{prefix}main::()->int{{{body}}}")).unwrap();
    }
    let source = format!(
        "{prefix}main::(){{callback:=ifx true then first else second;callback(value=42);}}"
    );
    let error = check(&source)
        .err()
        .expect("equal argument policy must retain required result union");
    assert!(error.message.contains("#must"), "{error:?}");
}

#[test]
fn discarded_baked_types_cannot_escape_into_body_annotations() {
    for body in [
        "value:T;return 42;",
        "Alias::#type T;return 42;",
        "Record::struct{value:T;}return 42;",
        "nested::(value:T)->int{return 42;}return 42;",
    ] {
        let source = format!(
            "ignore::(#discard $T:Type)->int{{{body}}}main::()->int{{return ignore(int);}}"
        );
        let error = check(&source)
            .err()
            .unwrap_or_else(|| panic!("accepted {source}"));
        assert!(error.message.contains("#discard"), "{source}\n{error:?}");
    }
}

#[test]
fn ignored_baked_values_keep_distinct_specializations_without_body_bindings() {
    let library = check(
        "ignore::(#discard $value:int)->int{return 21;}main::()->int{return ignore(1)+ignore(2);}",
    )
    .unwrap();
    assert_eq!(library.procedures().len(), 3);
}

#[test]
fn promoted_captured_callback_fields_retain_required_source_contracts() {
    let prefix = "Required::#type(value:int)->int #must;Holder::struct{struct{callback:=answer;}}answer::(value:int=21)->int #must{return value;}make::()->Required{return answer;}";
    check(&format!("{prefix}main::()->int{{holder:=Holder.{{callback=make()}};copy:=holder;return copy.callback(value=42);}}"))
        .unwrap();
    let source = format!(
        "{prefix}main::(){{holder:=Holder.{{callback=make()}};copy:=holder;copy.callback();}}"
    );
    let error = check(&source)
        .err()
        .expect("captured required callback discarded");
    assert!(error.message.contains("#must"), "{error:?}");
    let start = source.rfind("copy.callback()").unwrap();
    assert_eq!(error.location.span.start, start);
    assert_eq!(error.location.span.end, start + "copy.callback()".len());
    check(&format!(
        "{prefix}main::()->int{{return Holder.{{callback=make()}}.callback(value=42);}}"
    ))
    .unwrap();
    let source = format!("{prefix}main::(){{Holder.{{callback=make()}}.callback(value=42);}}");
    let error = check(&source)
        .err()
        .expect("temporary captured callback discarded");
    assert!(error.message.contains("#must"), "{error:?}");
}

#[test]
fn positional_literals_preserve_required_callback_contracts() {
    let prefix = "Required::#type(value:int)->int #must;Holder::struct{callback:Required;tag:int=2;}answer::(value:int)->int{return value;}make::()->Required{return answer;}";
    check(&format!("{prefix}main::()->int{{holder:=Holder.{{make()}};copy:=holder;return copy.callback(value=40)+copy.tag;}}"))
        .unwrap();
    let source = format!(
        "{prefix}main::(){{holder:=Holder.{{make()}};copy:=holder;copy.callback(value=40);}}"
    );
    let error = check(&source)
        .err()
        .expect("captured positional callback result is required");
    assert!(error.message.contains("#must"), "{error:?}");
    let call = "copy.callback(value=40)";
    let start = source.find(call).unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
}

#[test]
fn positional_literals_reject_extra_values_and_unnamed_union_selection() {
    for (source, diagnostic) in [
        (
            "Pair::struct{value:int;}main::(){pair:=Pair.{1,2};}",
            "too many values",
        ),
        (
            "Choice::union{left:int;right:int;}main::(){choice:=Choice.{42};}",
            "explicit alternative",
        ),
    ] {
        let error = check(source)
            .err()
            .expect("invalid positional field selection");
        assert!(error.message.contains(diagnostic), "{error:?}");
    }
}

#[test]
fn positional_sequence_descriptors_preserve_required_element_contracts() {
    let prefix = "Required::#type(value:int)->int #must;Slice::#type []Required;answer::(value:int)->int{return value;}";
    check(&format!("{prefix}main::()->int{{items:[1]Required;items[0]=answer;view:=Slice.{{1,*items[0]}};return view[0](value=42);}}"))
        .unwrap();
    let source = format!(
        "{prefix}main::(){{items:[1]Required;items[0]=answer;view:=Slice.{{1,*items[0]}};view[0](value=42);}}"
    );
    let error = check(&source)
        .err()
        .expect("descriptor element result is required");
    assert!(error.message.contains("#must"), "{error:?}");
    let call = "view[0](value=42)";
    let start = source.find(call).unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
}

#[test]
fn local_operator_return_contracts_are_specific_to_each_call() {
    let declarations = "Box::struct{value:int;}Optional::#type(value:int)->int;Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}";
    let body = "operator []::(a:Box,callback:$F)->F{return callback;}required:=Box.{value=1}[cast(Required)answer];optional:=Box.{value=2}[cast(Optional)answer];";
    let source = format!(
        "{declarations}main::()->int{{{body}optional(value=0);return required(value=42);}}"
    );
    let library = check(&source).unwrap();
    // Both callback policies have one ABI and one executable operator body.
    assert_eq!(library.procedures().len(), 3);
    let rejected = format!(
        "{declarations}main::()->int{{{body}optional(value=0);required(value=42);return 0;}}"
    );
    let error = check(&rejected)
        .err()
        .expect("required returned callback result");
    assert!(error.message.contains("#must"), "{error:?}");
    let call = "required(value=42)";
    let start = rejected.rfind(call).unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
}

#[test]
fn nested_operator_callback_policy_is_not_inferred_from_its_abi() {
    let source = "Box::struct{value:int;}Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}apply::(callback:$F)->int{return callback(value=42);}main::()->int{operator []::(a:Box,callback:$F)->F{return callback;}return apply(Box.{value=1}[cast(Required)answer]);}";
    let error = check(source)
        .err()
        .expect("nested source policy requires an actual preview");
    assert!(
        error.message.contains("checked source result preview"),
        "{error:?}"
    );
    let expression = "Box.{value=1}[cast(Required)answer]";
    let start = source.find(expression).unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(
        error.location.span.end,
        start + expression.len(),
        "{error:?}"
    );
}

#[test]
fn baked_callbacks_use_source_contracts_without_runtime_slots() {
    for marker in ["$", "$$"] {
        for parameter in ["callback:$F", "callback:=answer"] {
            for discard in ["callback();", "_:=callback();", "_=callback();"] {
                let prefix = "Optional::#type()->int;Required::#type()->int #must;answer::()->int{return 42;}required_target::()->int #must{return 42;}";
                let optional = format!(
                    "{prefix}ignore::({marker}{parameter})->int{{{discard}return 42;}}main::()->int{{return ignore(cast(Optional)required_target);}}"
                );
                check(&optional).unwrap_or_else(|error| panic!("{optional}\n{error:?}"));
                let required = format!(
                    "{prefix}ignore::({marker}{parameter})->int{{{discard}return 42;}}main::()->int{{return ignore(cast(Required)answer);}}"
                );
                let error = check(&required)
                    .err()
                    .expect("required baked source callback result");
                assert!(error.message.contains("#must"), "{required}\n{error:?}");
                let start = required.find("callback()").unwrap();
                assert_eq!(error.location.span.start, start, "{error:?}");
                assert_eq!(
                    error.location.span.end,
                    start + "callback()".len(),
                    "{error:?}"
                );
            }
        }
    }
}

#[test]
fn graph_operator_results_retain_each_original_callback_cast_contract() {
    let declarations = "Box::struct{value:int;}Optional::#type(value:int)->int;Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}";
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
        let prefix = format!(
            "{declarations}{operator}main::()->int{{required:={required};optional:={optional};optional(value=0);"
        );
        let accepted = format!("{prefix}return required(value=42);}}");
        let library = check(&accepted).unwrap_or_else(|error| panic!("{accepted}\n{error:?}"));
        assert_eq!(library.procedures().len(), 3, "{accepted}");
        let rejected = format!("{prefix}required(value=42);return 0;}}");
        let error = check(&rejected)
            .err()
            .expect("required callback cast on graph operator result");
        assert!(error.message.contains("#must"), "{rejected}\n{error:?}");
        let call = "required(value=42)";
        let start = rejected.rfind(call).unwrap();
        assert_eq!(error.location.span.start, start, "{error:?}");
        assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
    }
    let body = "Holder::struct(T:Type){callback:T;}operator +::(a:$T,b:int)->T #symmetric{return a;}main::()->int{required_receiver:Holder(Required);required_receiver.callback=answer;optional_receiver:Holder(Optional);optional_receiver.callback=answer;required:=1+required_receiver;optional:=optional_receiver+2;optional.callback(value=0);";
    let accepted = format!("{declarations}{body}return required.callback(value=42);}}");
    let library = check(&accepted).unwrap_or_else(|error| panic!("{accepted}\n{error:?}"));
    assert_eq!(library.procedures().len(), 3, "{accepted}");
    let rejected = format!("{declarations}{body}required.callback(value=42);return 0;}}");
    let error = check(&rejected)
        .err()
        .expect("symmetric nominal callback result retains its per-use contract");
    assert!(error.message.contains("#must"), "{error:?}");
    let call = "required.callback(value=42)";
    let start = rejected.rfind(call).unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
}

#[test]
fn required_graph_operator_results_cannot_be_discarded_at_root_destinations() {
    for (ty, result) in [
        ("int", "a.value+index"),
        ("float64", "1.5"),
        ("bool", "true"),
    ] {
        let declarations = format!(
            "Box::struct{{value:int;}}operator []::(a:Box,index:int)->{ty} #must{{return {result};}}"
        );
        check(&format!(
            "{declarations}main::()->{ty}{{return Box.{{value=40}}[2];}}"
        ))
        .unwrap();
        for discard in [
            "Box.{value=40}[2];",
            "_:=Box.{value=40}[2];",
            "_=Box.{value=40}[2];",
        ] {
            let source = format!("{declarations}main::(){{{discard}}}");
            let error = check(&source)
                .err()
                .unwrap_or_else(|| panic!("required operator result root discard: {source}"));
            assert!(error.message.contains("#must"), "{error:?}");
            let call = "Box.{value=40}[2]";
            let start = source.rfind(call).unwrap();
            assert_eq!(error.location.span.start, start, "{error:?}");
            assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
        }
    }
    check("Box::struct{value:int;}operator []::(a:Box,index:int)->int #must{return a.value+index;}main::(){_:=Box.{value=40}[2]+1;}").unwrap();
}

#[test]
fn generic_omitted_callback_defaults_keep_their_defining_cast_contract() {
    let declarations = "Required::#type(value:int)->int #must;Optional::#type(value:int)->int;answer::(value:int)->int{return value;}forward::(callback:$F=cast(Required)answer)->F{return callback;}";
    let body = "required:=forward();optional:=forward(cast(Optional)answer);optional(value=0);";
    let accepted = format!("{declarations}main::()->int{{{body}return required(value=42);}}");
    check(&accepted).unwrap();
    let rejected = format!("{declarations}main::()->int{{{body}required(value=42);return 0;}}");
    let error = check(&rejected)
        .err()
        .expect("required defining callback default cast");
    assert!(error.message.contains("#must"), "{error:?}");
    let call = "required(value=42)";
    let start = rejected.rfind(call).unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(error.location.span.end, start + call.len(), "{error:?}");

    let caller = "Dependency::#import \"Dependency\";Required::#type(other:int)->int;main::(){callback:=Dependency.forward();callback(value=42);}";
    let fixture = Fixture::new(caller);
    fs::write(
        fixture.0.join("Dependency.jai"),
        "#scope_file Required::#type(value:int)->int #must;answer::(value:int)->int{return value;}#scope_export forward::(callback:$F=cast(Required)answer)->F{return callback;}",
    )
    .unwrap();
    let graph = ModuleGraph::load(
        &fixture.0.join("main.jai"),
        GraphOptions {
            import_dirs: vec![fixture.0.clone()],
        },
    )
    .unwrap();
    let error = resolve_library(&graph)
        .err()
        .expect("private defining cast must survive caller alias shadowing");
    assert!(error.message.contains("#must"), "{error:?}");
    let call = "callback(value=42)";
    let start = caller.find(call).unwrap();
    assert_eq!(error.location.span.start, start, "{error:?}");
    assert_eq!(error.location.span.end, start + call.len(), "{error:?}");
}
