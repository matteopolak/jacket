//! Declaration metadata must not be selected by structural procedure TypeId.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};

fn compile(source: &str) -> Result<(), String> {
    let mut options = Options::host();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    options.import_paths = vec![root.clone()];
    options.preload = Some(root.join("prelude/module.jai"));
    let mut compiler = Compiler::new(options, Rc::new(NativeFs));
    compiler
        .compile_sources(&[ProgramSource::String(source.into())])
        .map_err(|error| error.render(&compiler.sources))
}

#[test]
fn globals_parameters_and_inferred_procedure_values_keep_their_own_headers() {
    compile(
        r#"
        Public :: (public_flag:=true)->s64;
        Other :: (hidden_flag:=false)->s64;
        choose :: (actual:bool)->s64 {if actual return 19; return 23;}
        direct :: (own_flag:=true)->s64 {if own_flag return 31; return 37;}
        first:Public=choose;
        second:Other=choose;
        invoke :: (callback:Public)->s64 {return callback(public_flag=false);}
        invoke_macro :: (callback:Public)->s64 #expand {return callback();}
        verify :: ()->bool {
            alias:=direct;
            typed:Public=direct;
            return first()==19 && second()==23 && invoke(choose)==23
                && invoke_macro(choose)==19 && alias(own_flag=false)==37
                && typed(public_flag=false)==37;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn promoted_filtered_and_overlaid_fields_keep_declaration_identity() {
    compile(
        r#"
        Public :: (public_flag:=true)->s64;
        Other :: (hidden_flag:=false)->s64;
        Inner :: struct {callback:Public; hidden:s64;}
        Outer :: struct {using,only(callback) inner:Inner;}
        Overlay :: struct {first:Public; #overlay(first) second:Other;}
        choose :: (actual:bool)->s64 {if actual return 19; return 23;}
        verify :: ()->bool {
            outer:Outer;
            outer.callback=choose;
            overlay:Overlay;
            overlay.first=choose;
            left,right:=overlay.first,overlay.second;
            using,only(callback) outer;
            return callback()==19 && outer.callback(public_flag=false)==23
                && left()==19 && right()==23
                && *overlay.first==*overlay.second;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn duplicate_named_arguments_reject_instead_of_overwriting() {
    let error = compile(
        r#"
        C :: (first:s64,second:s64)->s64;
        identity :: (a:s64,b:s64)->s64 {return a+b;}
        main :: () {value:C=identity; value(first=1,first=2,second=3);}
    "#,
    )
    .expect_err("duplicate named arguments must reject");
    assert!(error.contains("supplied more than once"), "{error}");
}

#[test]
fn unknown_names_without_defaults_reject() {
    let error = compile(
        r#"
        C :: (public_name:s64)->s64;
        D :: (private_name:s64)->s64;
        main :: () {a:C;b:D;a(private_name=3);}
    "#,
    )
    .expect_err("a structural peer must not supply a name");
    assert!(
        error.contains("no parameter named `private_name`"),
        "{error}"
    );
}

#[test]
fn type_queries_and_plain_type_aliases_retain_selected_field_header() {
    compile(
        r#"
        Public :: (public_flag:=true)->s64;
        Other :: (hidden_flag:=false)->s64;
        Holder :: struct {callback:Public;}
        Same :: #type Public;
        choose :: (actual:bool)->s64 {if actual return 19; return 23;}
        verify :: ()->bool {
            h:Holder;
            h.callback=choose;
            queried:type_of(h.callback)=choose;
            plain:Same=choose;
            renamed:=cast(Other) h.callback;
            return queried(public_flag=false)==23 && plain()==19 && renamed()==23;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn a_factory_result_uses_its_declared_return_header() {
    compile(
        r#"
        Public :: (public_flag:=true)->s64;
        Other :: (hidden_flag:=false)->s64;
        choose :: (actual:bool)->s64 {if actual return 19; return 23;}
        make :: ()->Public {return choose;}
        Maker :: ()->Public;
        verify :: ()->bool {
            factory:Maker=make;
            direct:=make();
            indirect:=factory();
            return direct(public_flag=false)==23 && indirect()==19;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn multiple_callback_results_retain_each_declared_header() {
    compile(
        r#"
        Public :: (public_flag:=true)->s64;
        Other :: (hidden_flag:=false)->s64;
        choose :: (actual:bool)->s64 {if actual return 19; return 23;}
        make :: ()->(Public,Other) {return choose,choose;}
        Maker :: ()->(Public,Other);
        verify :: ()->bool {
            factory:Maker=make;
            first,second:=make();
            third,fourth:=factory();
            return first()==19 && second()==23 && third(public_flag=false)==23
                && fourth(hidden_flag=true)==19;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn c_ellipsis_arguments_are_preserved_after_the_named_fixed_prefix() {
    compile(
        r#"
        C :: (first:s64,values:..Any)->s64 #c_call;
        target :: (actual:s64,values:..Any)->s64 #foreign "callable_provenance_unavailable";
        main :: () {callback:C=target; callback(first=1,2,3);}
    "#,
    )
    .unwrap();
}

#[test]
fn callback_context_fields_have_independent_defaults() {
    compile(
        r#"
        #add_context callback:(public_flag:=true)->s64;
        #add_context alternate:(hidden_flag:=false)->s64;
        choose :: (flag:bool)->s64 {if flag return 19; return 23;}
        verify :: ()->bool {
            copied:=context;
            copied.callback=choose;
            copied.alternate=choose;
            push_context,defer_pop copied;
            return context.callback()==19 && context.alternate()==23;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn filtered_module_alias_defaults_evaluate_in_the_declaring_private_scope() {
    use jaic::sema::VirtualFs;
    let mut fs = VirtualFs::default();
    fs.insert(
        "/lib/Backing/module.jai",
        br#"
        #scope_file DEFAULT::true;
        #scope_export Callback :: (public_flag:=DEFAULT)->s64;
        #scope_file Secret :: (secret_flag:=false)->s64;
    "#
        .as_slice(),
    );
    fs.insert(
        "/lib/Facade/module.jai",
        br#"using,only(Callback) #import "Backing";"#.as_slice(),
    );
    fs.insert(
        "/app/main.jai",
        br#"
        Facade :: #import "Facade";
        DEFAULT::false;
        Other :: (private_flag:=false)->s64;
        choose :: (actual:bool)->s64 {if actual return 19; return 23;}
        verify :: ()->bool {
            first:Facade.Callback=choose;
            second:Other=choose;
            return first()==19 && second()==23 && first(public_flag=false)==23;
        }
        #assert verify();
    "#
        .as_slice(),
    );
    let mut options = Options::host();
    options.runtime_support = false;
    options.import_paths = vec!["/lib".into()];
    let mut compiler = Compiler::new(options, Rc::new(fs));
    compiler
        .compile_program(std::path::Path::new("/app/main.jai"))
        .map_err(|e| e.render(&compiler.sources))
        .unwrap();
}

#[test]
fn ide_metadata_api_selects_fields_instead_of_equal_types_or_offsets() {
    use jaic::{
        intern::Sym,
        sema::{VirtualFs, scope::Resolved, value::Value},
        source::Span,
    };
    let mut options = Options::host();
    options.runtime_support = false;
    let mut compiler = Compiler::new(options, Rc::new(VirtualFs::default()));
    compiler
        .compile_sources(&[ProgramSource::String(
            r#"
        Public :: (public_flag:=true)->s64;
        Other :: (hidden_flag:=false)->s64;
        R :: struct {first:Public; #overlay(first) second:Other;}
        #assert size_of(R)>0;
    "#
            .into(),
        )])
        .map_err(|e| e.render(&compiler.sources))
        .unwrap();
    let scope = compiler.modules[compiler.main_module.unwrap().0 as usize].scope;
    let record = compiler.lookup(scope, Sym::intern("R")).unwrap()[0];
    let Resolved::Const {
        value: Value::Type(record),
        ..
    } = compiler.resolve_entity(record).unwrap()
    else {
        panic!("record declaration must denote a type");
    };
    let first = compiler
        .callable_member_parameters(record, Sym::intern("first"), Span::NONE)
        .unwrap()
        .unwrap();
    let pointer = compiler.types.pointer(record);
    let second = compiler
        .callable_member_parameters(pointer, Sym::intern("second"), Span::NONE)
        .unwrap()
        .unwrap();
    assert_eq!(first.names, vec![Some(Sym::intern("public_flag"))]);
    assert_eq!(second.names, vec![Some(Sym::intern("hidden_flag"))]);
    let public = compiler.lookup(scope, Sym::intern("Public")).unwrap()[0];
    let header = compiler
        .callable_entity_parameters(public)
        .unwrap()
        .unwrap();
    assert_eq!(header.names, first.names);
}

#[test]
fn selected_procedure_alias_headers_do_not_refresh_after_new_overloads() {
    compile(
        r#"
        evaluate :: (number:s64=7)->s64 {return number;}
        Alias :: evaluate;
        generate :: ()->string {
            Alias();
            return "evaluate :: (flag:bool)->s64 {if flag return 1;return 0;}";
        }
        #insert #run generate();
        verify :: ()->bool {
            callback:#type Alias=Alias;
            return callback(number=9)==9 && callback()==7;
        }
        #assert verify();
    "#,
    )
    .unwrap();
    let error = compile(
        r#"
        evaluate :: (number:s64)->s64 {return number;}
        Alias :: evaluate;
        generate :: ()->string {
            Alias(7);
            return "evaluate :: (flag:bool)->s64 {if flag return 1;return 0;}";
        }
        #insert #run generate();
        main :: () {Alias(true);}
    "#,
    )
    .expect_err("the previously selected alias must not acquire the inserted overload");
    assert!(
        error.contains("argument of type `bool` does not match parameter type `s64`"),
        "{error}"
    );
}

#[test]
fn metadata_discovery_does_not_evaluate_runtime_default_expressions() {
    compile(
        r#"
        calls:s64;
        next :: ()->bool {calls+=1;return true;}
        Public :: (public_flag:bool=next())->s64;
        Other :: (hidden_flag:bool=false)->s64;
        choose :: (flag:bool)->s64 {if flag return 19; return 23;}
        Holder :: struct {callback:Public;}
        verify :: ()->bool {
            calls=0;
            h:Holder;
            h.callback=choose;
            alias:=h.callback;
            explicit:type_of(h.callback)=choose;
            if calls!=0 return false;
            return alias()==19 && calls==1 && explicit(public_flag=false)==23 && calls==1;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn same_shaped_field_declarations_keep_names_defaults_and_alias_provenance() {
    compile(include_str!(
        "../../../tests/fixtures/callable-provenance/callable-declaration-provenance.jai"
    ))
    .unwrap();
}

#[test]
fn names_from_another_callback_declaration_are_not_accepted() {
    let error = compile(
        r#"
        Holder :: struct {callback:(public_option:=false)->s64;}
        Other :: struct {callback:(private_option:=false)->s64;}
        main :: () {first:Holder; second:Other; first.callback(private_option=true);}
    "#,
    )
    .expect_err("another same-shaped declaration cannot supply a named parameter");
    assert!(
        error.contains("no parameter named `private_option`"),
        "{error}"
    );
}

#[test]
fn named_arguments_without_defaults_follow_the_selected_declaration_order() {
    compile(
        r#"
        Holder :: struct {callback:(left:s64,right:s64)->s64;}
        Other :: struct {callback:(hidden_left:s64,hidden_right:s64)->s64;}
        subtract :: (a:s64,b:s64)->s64 {return a-b;}
        verify :: ()->bool {
            first:Holder; second:Other;
            first.callback=subtract;
            return first.callback(right=3,left=11)==8;
        }
        #assert verify();
    "#,
    )
    .unwrap();
}

#[test]
fn procedure_constant_from_run_binds_typed_struct_argument_without_losing_identity() {
    compile(
        r#"
        Callback :: (value:s64)->s64;
        identity :: (value:s64)->s64 { return value; }
        factory :: ()->Callback { return identity; }
        Holder :: struct(callback:Callback) { storage:s64; }
        Instance :: Holder(#run factory());
        #assert size_of(Instance)==8;
        #assert Instance.callback(9)==9;
        #assert Instance.callback==identity;
        main :: () {}
    "#,
    )
    .unwrap();
}

#[test]
fn annotated_procedure_sentinels_remain_constants_with_exact_pointer_bits() {
    compile(
        r#"
        Handler :: #type (signal:s32)->void #c_call;
        IGNORE :: cast(Handler) 1;
        ERROR :: cast(Handler) -1;
        Table :: struct { callback:Handler=IGNORE; }
        verify :: ()->bool {
            table:Table;
            return cast(s64)table.callback==1 && cast(s64)ERROR==-1;
        }
        #assert verify();
        main :: () {}
    "#,
    )
    .unwrap();
}
