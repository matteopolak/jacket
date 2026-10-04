//! Ordered source construction follows genuine promoted physical field paths.
use jai_modules::{Filesystem, GraphDiscovery, GraphOptions, ModuleGraph};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "jai-promoted-literal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        std::fs::write(path.join("main.jai"), source).unwrap();
        Self(path)
    }
    fn graph(&self) -> ModuleGraph {
        let mut discovery = GraphDiscovery::new(
            &self.0.join("main.jai"),
            GraphOptions::default(),
            &Filesystem,
        )
        .unwrap();
        loop {
            let status = discovery.advance().unwrap();
            if status.is_complete() {
                return discovery.into_graph().unwrap();
            }
            let requests = discovery.pending_using_requests();
            assert!(!requests.is_empty(), "{status:?}");
            let outcome = jai_sema::resolve_discovery_using(
                discovery.graph(),
                &requests,
                &jai_sema::ResolveOptions::default(),
                &mut jai_vm::NoEffects,
            )
            .unwrap();
            let mut progressed = false;
            for key in outcome.specializations {
                progressed |= discovery.discover_specialization(key).unwrap();
            }
            for (request, decision) in outcome.decisions {
                discovery.resolve_using(request, decision).unwrap();
                progressed = true;
            }
            assert!(
                progressed,
                "typed using remains pending: {:?}",
                outcome.pending
            );
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let program = jai_sema::resolve_graph(&fixture.graph())
        .unwrap_or_else(|error| panic!("{source}\n{error:?}"));
    let execution = jai_vm::execute(&program, jai_vm::Limits::default());
    let jai_vm::Outcome::Complete(values) = execution.outcome else {
        panic!("{execution:?}");
    };
    let [jai_vm::Value::Int(value)] = values.as_slice() else {
        panic!("integer result required");
    };
    value.value()
}

#[test]
fn jaison_number() {
    assert_eq!(
        run(r###"
JSON_Type::enum u8{NULL::0;BOOLEAN::1;NUMBER::3;STRING::2;}
JSON_Value::struct{type:JSON_Type;union{boolean:bool;number:float64;str:string;}}
main::()->int{value:=JSON_Value.{type=.NUMBER,number=42.0}; copy:=value; if copy.type!=.NUMBER return 1; return cast(int)copy.number;}
"###),
        42
    );
}

#[test]
fn jaison_string() {
    assert_eq!(
        run(r###"
JSON_Type::enum u8{NULL::0;STRING::2;}
JSON_Value::struct{type:JSON_Type;union{number:float64;str:string;}}
main::()->int{value:JSON_Value=.{type=.STRING,str="forty-two"};copy:=value;if copy.type!=.STRING || copy.str!="forty-two" return 1;return 42;}
"###),
        42
    );
}

#[test]
fn interleaved_source_order() {
    assert_eq!(
        run(r###"
Owner::struct{struct{x,y:int;}middle:int;}
tick::(state:*int,n:int)->int{state.*=state.* * 10+n;return n;}
main::()->int{state:=0;value:Owner=.{x=tick(*state,1),middle=tick(*state,2),y=tick(*state,3)};if state!=123 || value.x!=1 || value.middle!=2 || value.y!=3 return 1;return 42;}
"###),
        42
    );
}

#[test]
fn placed_record_literals_materialize_overlapping_fields_in_record_order() {
    assert_eq!(
        run(r###"
Storage::struct{first:u32;#place first;second:u32;}
main::()->int{value:Storage=.{first=1,second=42};return cast(int)value.first+cast(int)value.second;}
"###),
        84
    );
}

#[test]
fn lazy_selected_arm() {
    assert_eq!(
        run(r###"
Owner::struct{struct{x,y:int;}}
tick::(state:*int)->int{state.*+=1;return 21;}
main::()->int{state:=0;value:Owner=ifx false then Owner.{x=tick(*state),y=tick(*state)} else Owner.{x=20,y=22};if state!=0 return 1;return value.x+value.y;}
"###),
        42
    );
}

#[test]
fn selected_branch_default() {
    assert_eq!(
        run(r###"
Owner::struct{union{struct{x:int=1;y:int=22;}other:int=99;}}
main::()->int{value:Owner=.{x=20};copy:=value;return copy.x+copy.y;}
"###),
        42
    );
}

#[test]
fn named_using_overlay() {
    assert_eq!(
        run(r###"
Child::struct{x:int=1;y:int=2;}
Owner::struct{using child:Child=.{x=7,y=22};}
main::()->int{value:Owner=.{x=20};return value.x+value.y;}
"###),
        42
    );
}

#[test]
fn ordered_default_override() {
    assert_eq!(
        run(r###"
Owner::struct{struct{x:int=1;y:int=2;}x=5;y=22;}
main::()->int{value:Owner=.{x=20};return value.x+value.y;}
"###),
        42
    );
}

#[test]
fn generic_canonical_fields() {
    assert_eq!(
        run(r###"
Box::struct(T:Type,Fill:T){struct{amount:T=Fill;}tail:T=Fill;}
main::()->int{value:Box(int,21)=.{amount=21,tail=21};copy:=value;return copy.amount+copy.tail;}
"###),
        42
    );
}

#[test]
fn pointer_provenance() {
    assert_eq!(
        run(r###"
Owner::struct{union{pointer:*int;number:int;}}
identity::(p:*int)->*int{return p;}
main::()->int{value:=42;box:Owner=.{pointer=identity(*value)};copy:=box;return copy.pointer.*;}
"###),
        42
    );
}

#[test]
fn callback_provenance() {
    assert_eq!(
        run(r###"
Callback::#type(x:int)->int #must;
Owner::struct{struct{callback:Callback;}}
add::(x:int)->int #must{return x+22;}
make::()->Callback{return add;}
main::()->int{value:Owner=.{callback=make()};copy:=value;return copy.callback(x=20);}
"###),
        42
    );
}

#[test]
fn raw_float_bits() {
    assert_eq!(
        run(r###"
Owner::struct{union{number:float64;bits:u64;}}
identity::(value:float64)->float64{return value;}
main::()->int{value:Owner=.{number=identity(0h7FF8_0000_0000_0042)};copy:=value;return ifx copy.bits==0x7ff8000000000042 then 42 else 1;}
"###),
        42
    );
}

#[test]
fn literal_record_parameter_default_keeps_the_ownerless_constant_boundary() {
    assert_eq!(
        run(r#"Owner::struct{struct{amount:int;}}
        read::(value:Owner=.{amount=42})->int{return value.amount;}
        main::()->int{return read();}"#),
        42
    );
}

#[test]
fn duplicate_promoted_leaf() {
    let fixture = Fixture::new(
        r###"
Owner::struct{struct{x:int;}}main::(){value:Owner=.{x=1,x=2};}
"###,
    );
    let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("duplicate"), "{:?}", error);
}

#[test]
fn whole_child_and_leaf() {
    let fixture = Fixture::new(
        r###"
Child::struct{x:int;}Owner::struct{using child:Child;}main::(){value:Owner=.{child=.{x=1},x=2};}
"###,
    );
    let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("overlap"), "{:?}", error);
}

#[test]
fn competing_union_branches() {
    let fixture = Fixture::new(
        r###"
Owner::struct{union{struct{x,y:int;}struct{r,g:int;}}}main::(){value:Owner=.{x=1,r=2};}
"###,
    );
    let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("alternative"), "{:?}", error);
}

#[test]
fn unknown_promoted_field() {
    let fixture = Fixture::new(
        r###"
Owner::struct{struct{x:int;}}main::(){value:Owner=.{missing=1};}
"###,
    );
    let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("unknown"), "{:?}", error);
}

#[test]
fn wrong_leaf_type() {
    let fixture = Fixture::new(
        r###"
Owner::struct{struct{x:int;}}main::(){value:Owner=.{x="wrong"};}
"###,
    );
    let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("type"), "{:?}", error);
}

#[test]
fn typed_generic_target_and_contextual_bare_value_use_the_real_specialization() {
    assert_eq!(
        run(r#"
Pair::struct(T:Type){left:T;right:T;}
make::(value:Pair(int))->int{return value.left+value.right;}
main::()->int{typed:=Pair(int).{left=20,right=22};return make({left=typed.left,right=typed.right});}
"#),
        42
    );
}

#[test]
fn indexed_defaults_preserve_untouched_values_and_written_source_order() {
    assert_eq!(
        run(r#"
Owner::struct{values:[3]int=.[10,20,30];tail:int;}
tick::(state:*int,n:int)->int{state.*=state.* * 10+n;return n;}
main::()->int{state:=0;value:=Owner.{values[2]=tick(*state,1),tail=tick(*state,2),values[0]=tick(*state,3)};if state!=123 || value.values[1]!=20 || value.tail!=2 return 1;return 42;}
"#),
        42
    );
}

#[test]
fn canonical_default_application_uses_preceding_actual_type_formals() {
    assert_eq!(
        run(r#"
Pair::struct(T:Type,U:Type=T){left:T;right:U;}
Holder::struct{pair:Pair(int)=Pair(int).{left=20,right=22};}
main::()->int{value:Holder;return value.pair.left+value.pair.right;}
"#),
        42
    );
}

#[test]
fn selected_field_promotion_hides_excluded_names_before_ambiguity_checks() {
    assert_eq!(
        run(r#"
Child::struct{value:int=1;length:int=99;}
Owner::struct{using,except(length) child:Child;length:int=2;}
main::()->int{item:=Owner.{value=40,length=2};using item;return value+length;}
"#),
        42
    );
}

#[test]
fn bare_record_promotion_and_nested_enum_namespace_keep_real_owners() {
    assert_eq!(
        run(r#"
Child::struct{x:int=20;y:int=99;}
Owner::struct{child:Child;using,only(x) child;using Codes::enum{ANSWER::22;OTHER::1;}}
main::()->int{value:Owner;return value.x+cast(int) Owner.ANSWER;}
"#),
        42
    );
}

#[test]
fn malformed_path_is_rejected_before_compile_time_initializer_execution() {
    let source = "Owner::struct{values:[2]int;} trap::()->int{return 1/0;} main::()->int{value:=Owner.{values[2]=#run trap()};return 42;}";
    let fixture = Fixture::new(source);
    let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("index"), "{error:?}");
    assert!(!error.message.contains("zero divisor"), "{error:?}");
}

#[test]
fn excluded_field_and_overlapping_literal_paths_do_not_gain_authority() {
    for (source, message) in [
        (
            "Child::struct{x:int;}Owner::struct{using,except(x) child:Child;}main::()->int{value:=Owner.{x=42};return 42;}",
            "unknown",
        ),
        (
            "Child::struct{x:int;}Owner::struct{child:Child;}main::()->int{value:=Owner.{child=.{x=1},child.x=2};return 42;}",
            "overlap",
        ),
    ] {
        let fixture = Fixture::new(source);
        let error = jai_sema::resolve_graph(&fixture.graph()).unwrap_err();
        assert!(error.message.contains(message), "{error:?}");
    }
}
