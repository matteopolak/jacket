use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::resolve_graph;
use jai_vm::{Limits, Outcome, Value};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

#[test]
fn nominal_header_restrictions_match_the_actual_generic_record_origin() {
    assert_eq!(
        run(
            "Table::struct(T:Type){value:T=42;}read::(value:$R/Table)->int{return value.value;}main::()->int{value:Table(int);return read(value);}"
        ),
        42,
    );
    let fixture = Fixture::new(
        "Table::struct(T:Type){value:T;}Other::struct{value:int=42;}read::(value:$R/Table)->int{return value.value;}main::()->int{value:Other;return read(value);}",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains("overload")
            || error.message.contains("argument")
            || error.message.contains("different type"),
        "{error:?}"
    );
}

#[test]
fn interface_header_restrictions_keep_canonical_field_types() {
    assert_eq!(
        run(
            "Readable::struct{value:int;}Other::struct{value:int=42;}read::(value:$R/interface Readable)->int{return value.value;}main::()->int{value:Other;return read(value);}"
        ),
        42,
    );
    let fixture = Fixture::new(
        "Readable::struct{value:int;}Other::struct{value:u8=42;}read::(value:$R/interface Readable)->int{return value.value;}main::()->int{value:Other;return read(value);}",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains("overload")
            || error.message.contains("argument")
            || error.message.contains("different type"),
        "{error:?}"
    );
}

#[test]
fn record_callable_aliases_refresh_completed_context_and_named_defaults() {
    assert_eq!(
        run(
            "#add_context marker:int=7; R::struct{value:int=14;} Methods::struct{read::(value:R=R.{},saved:#Context=.{})->int{return value.value+saved.marker;}} alias::Methods.read; forwarded::alias; main::()->int{return forwarded()+alias(saved=.{});}"
        ),
        42,
    );
}

#[test]
fn record_run_defaults_retry_while_other_record_provider_bodies_become_ready() {
    assert_eq!(
        run(
            "Methods::struct{read::(value:int=#run Seed.make())->int{return value;}} Seed::struct{make::()->int{return 21;}} alias::Methods.read; forwarded::alias; main::()->int{return forwarded()+alias();}"
        ),
        42,
    );
}

#[test]
fn specialized_field_run_defaults_wait_for_their_actual_record_method_body() {
    assert_eq!(
        run(
            "Owner::struct(T:Type){value:T=#run seed();seed::()->T{return 21;}}main::()->int{first:Owner(int);second:Owner(int);return first.value+second.value;}"
        ),
        42,
    );
}

#[test]
fn specialized_field_run_defaults_inherit_the_completed_context() {
    assert_eq!(
        run(
            "#add_context amount:int=21;Owner::struct(T:Type){value:T=#run seed();seed::()->T{return context.amount;}}main::()->int{first:Owner(int);second:Owner(int);return first.value+second.value;}"
        ),
        42,
    );
}

#[test]
fn record_field_run_jobs_finish_before_global_initializer_construction() {
    assert_eq!(
        run(
            "#add_context amount:int=21;Owner::struct{value:int=#run seed();seed::()->int{return context.amount;}}saved:Owner;read::(next:Owner=Owner.{}) ->int{return next.value;}main::()->int{return saved.value+read();}"
        ),
        42,
    );
}

#[test]
fn specialized_field_run_defaults_reject_incompatible_provider_results() {
    let fixture = Fixture::new(
        "Owner::struct(T:Type){value:T=#run seed();seed::()->bool{return true;}}main::()->int{value:Owner(int);return value.value;}",
    );
    let graph = fixture.graph();
    let error = resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("type") || error.message.contains("convert"),
        "{error:?}"
    );
    assert!(error.location.span.start < 40, "{error:?}");
}

#[test]
fn specialized_field_run_cycles_never_publish_zero_initializers() {
    let fixture = Fixture::new(
        "Owner::struct(T:Type){value:T=#run seed();seed::()->T{again:Owner(T);return again.value;}}main::()->int{value:Owner(int);return value.value;}",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains("pending")
            || error.message.contains("awaiting")
            || error.message.contains("waiting")
            || error.message.contains("cyclic"),
        "{error:?}"
    );
}

#[test]
fn specialized_callback_types_preserve_discarded_source_parameters() {
    assert_eq!(
        run(
            "counter:int;touch::()->int{counter+=1;return 9;}read::(#discard ignored:int,value:int)->int{return value;}Holder::struct(T:Type){callback:(#discard ignored:T,value:T)->T=read;}main::()->int{holder:Holder(int);value:=holder.callback(touch(),42);return value+counter;}"
        ),
        42,
    );
    let fixture = Fixture::new(
        "Holder::struct(T:Type){callback:(#discard ignored:Missing,value:T)->T;}main::(){holder:Holder(int);}",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains("Missing") || error.message.contains("unknown"),
        "{error:?}"
    );
}

#[test]
fn record_assertions_validate_messages_in_the_definition_scope() {
    assert_eq!(
        run(
            r#"Owner::struct(N:int=1,Reason:string="ready"){#assert(N>0,Reason);value:int=42;}main::()->int{value:Owner();return value.value;}"#
        ),
        42
    );
    let fixture = Fixture::new(
        r#"Owner::struct(N:int,Reason:string="positive required"){#assert(N>0,Reason);value:int;}main::(){value:Owner(0);}"#,
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("positive required"), "{error:?}");
    let fixture = Fixture::new("Owner::struct{#assert(true,7);value:int;}main::(){value:Owner;}");
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("string"), "{error:?}");
}

#[test]
fn promoted_default_overrides_keep_their_physical_root_field() {
    assert_eq!(
        run(
            "Pair::struct{left:int=1;right:int=2;}Derived::struct(T:Type){using base:Pair;left=19;right=23;}main::()->int{value:Derived(int);other:Pair;if other.left!=1 return 1;return value.base.left+value.right;}"
        ),
        42
    );
}

#[test]
fn anonymous_physical_fields_receive_owned_promoted_default_overrides() {
    assert_eq!(
        run(
            "Owner::struct(T:Type){struct{value:int=1;}value=42;}main::()->int{value:Owner(int);return value.value;}"
        ),
        42
    );
    let fixture = Fixture::new(
        "Owner::struct(T:Type){union{value:int;other:int;}value=42;}main::(){value:Owner(int);}",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("struct field path"), "{error:?}");
}

#[test]
fn baked_record_case_tables_select_original_members_and_through_paths() {
    assert_eq!(
        run(
            "Choice::struct(T:Type,N:int=1){#if T == {case int;#if N == {case 1;first:T=19;#through;case 2;second:T=23;case;unused:T;}case;other:T;}}main::()->int{value:Choice(int);return value.first+value.second;}"
        ),
        42
    );
    assert_eq!(
        run(
            "Mode::enum{FIRST;SECOND;}Choice::struct(M:Mode=.SECOND){#if #complete M == {case .FIRST;value:int=1;case .SECOND;value:int=42;}}main::()->int{value:Choice();return value.value;}"
        ),
        42
    );
    for (source, expected) in [
        (
            "Choice::struct(N:int){#if N == {case 1;value:int;case 1+0;other:int;}}main::(){value:Choice(1);}",
            "duplicate",
        ),
        (
            "Mode::enum{FIRST;SECOND;}Other::enum{FIRST;SECOND;}Choice::struct(M:Mode=.FIRST){#if M == {case Other.FIRST;value:int;}}main::(){value:Choice();}",
            "nominal",
        ),
        (
            "Choice::struct(N:int){#if N == {case 1;value:int;#through;}}main::(){value:Choice(1);}",
            "through",
        ),
        (
            "Mode::enum{FIRST;SECOND;}Choice::struct(M:Mode=.FIRST){#if #complete M == {case .FIRST;value:int;}}main::(){value:Choice();}",
            "cover",
        ),
    ] {
        let fixture = Fixture::new(source);
        let error = resolve_graph(&fixture.graph()).unwrap_err();
        assert!(error.message.contains(expected), "{expected}: {error:?}");
    }
}
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-parameterized-records-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn graph(&self) -> ModuleGraph {
        ModuleGraph::load(&self.0.join("main.jai"), GraphOptions::default()).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn run(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let program = resolve_graph(&fixture.graph())
        .unwrap_or_else(|error| panic!("parameterized source must resolve: {error:?}"));
    let execution = jai_vm::execute(&program, Limits::default());
    let Outcome::Complete(values) = execution.outcome else {
        panic!("parameterized source must execute: {:?}", execution.outcome);
    };
    let [Value::Int(result)] = values.as_slice() else {
        panic!("expected integer result");
    };
    result.value()
}
#[test]
fn record_type_arguments_materialize_fields_and_keep_nominal_identity() {
    assert_eq!(
        run(
            "Box :: struct(T:Type) { value:T; } main :: () -> int { a:Box(int)=.{value=19}; b:Box(int)=a; small:Box(u8)=.{value=7}; return b.value+cast(int) small.value; }"
        ),
        26
    );
}
#[test]
fn type_and_value_defaults_bind_against_prior_parameters() {
    assert_eq!(
        run(
            "Buffer :: struct(T:=int, N:int=3, Fill:T=7) { values:[N]T; tag:T=Fill; } main :: () -> int { a:Buffer(); b:Buffer(N=2, Fill=11); return a.tag+b.tag+a.values.count+b.values.count; }"
        ),
        23
    );
}
#[test]
fn aliases_and_equivalent_typed_arguments_reuse_the_same_instance() {
    assert_eq!(
        run(
            "Box :: struct(T:Type, N:int=7) { value:T=N; } Alias :: Box(int); main :: () -> int { a:Alias; b:Box(int,cast(int) 7)=a; return b.value; }"
        ),
        7
    );
}
#[test]
fn recursive_pointer_instances_and_mutual_templates_are_canonical() {
    assert_eq!(
        run(
            "Node :: struct(T:Type) { next:*Node(T); value:T=13; } Left :: struct(T:Type) { right:*Right(T); value:T=5; } Right :: struct(T:Type) { left:*Left(T); } main :: () -> int { n:Node(int); l:Left(int); if n.next != null return 99; return n.value+l.value; }"
        ),
        18
    );
}
#[test]
fn unused_templates_do_not_reserve_unbound_runtime_shapes() {
    assert_eq!(
        run("Unused :: struct(T:Type,N:int) { values:[N]T; } main :: () -> int { return 42; }"),
        42
    );
}
#[test]
fn parameter_diagnostics_precede_field_materialization() {
    for (source, expected) in [
        (
            "Box :: struct(T:Type,N:int) { value:T; } main :: () { x:Box(int); }",
            "missing required",
        ),
        (
            "Box :: struct(T:Type) { value:T; } main :: () { x:Box(int,T=int); }",
            "supplied more than once",
        ),
        (
            "Box :: struct(T:Type) { value:T; } main :: () { x:Box(T=int,int); }",
            "follows a named",
        ),
        (
            "Box :: struct(T:Type) { value:T; } main :: () { x:Box(Other=int); }",
            "unknown record template",
        ),
        (
            "Box :: struct(N:int) { values:[N]int; } main :: () { x:Box(-1); }",
            "nonnegative integer",
        ),
        (
            "Box :: struct(T:Type) { value:T; } main :: () { x:Box(7); }",
            "requires a type",
        ),
    ] {
        let fixture = Fixture::new(source);
        let error = resolve_graph(&fixture.graph()).unwrap_err();
        assert!(
            error.message.contains(expected),
            "expected {expected:?}, got {error:?}"
        );
    }
}
#[test]
fn nested_records_and_forward_member_constants_read_the_instance_overlay() {
    assert_eq!(
        run(
            "Table :: struct(T:Type, Fill:T=9) { values:[Capacity]Entry; Capacity :: Later; Later :: 2; Item :: T; Entry :: struct { next:*Entry; value:Item=Fill; } } main :: () -> int { table:Table(int); return table.values[0].value+table.values[1].value+table.values.count; }"
        ),
        20
    );
}
#[test]
fn unused_record_method_bodies_are_checked() {
    let fixture = Fixture::new(
        "Box :: struct(T:Type) { value:T; work :: () -> T { return Missing; } } main :: () { x:Box(int); }",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains("unknown") || error.message.contains("Unknown"),
        "{error:?}"
    );
}
#[test]
fn record_macro_members_keep_compile_only_identity_and_baked_scope() {
    assert_eq!(
        run(
            "Box :: struct(T:Type,Fill:T) { value:T=35; add :: (item:*Box(T,Fill)) #expand { item.value+=Fill; } } main :: ()->int { Owner :: Box(int,7); value:Owner; Owner.add(*value); return value.value; }"
        ),
        42
    );
}

#[test]
fn record_methods_resolve_the_instance_definition_scope() {
    assert_eq!(
        run(
            "Box :: struct(T:Type, Fill:T=11) { value:T=Fill; read :: (value:Box) -> T { return value.value; } seed :: () -> T { return Fill; } } IntBox :: Box(int); main :: ()->int { Fill :: 99; T :: u8; value:IntBox; return IntBox.read(value)+IntBox.seed(); }"
        ),
        22
    );
}

#[test]
fn record_method_signatures_are_reserved_before_recursive_bodies() {
    assert_eq!(
        run(
            "Counter :: struct(T:Type) { even :: (n:T) -> T { if n==0 return 1; return odd(n-1); } odd :: (n:T) -> T { if n==0 return 0; return even(n-1); } } Integers :: Counter(int); main :: ()->int { return Integers.even(8)+Integers.odd(7); }"
        ),
        2
    );
}

#[test]
fn direct_record_insertion_preserves_baked_fields_and_nested_member_identity() {
    assert_eq!(
        run(
            "Buffer :: struct(T:Type,N:int=2,Fill:T=7) { before:u8=1; #insert #code { Entry :: struct { value:T=Fill; } values:[N]Entry; Saved :: Fill; }; after:int=3; } main :: ()->int { a:Buffer(int); b:Buffer(int)=a; return b.values[0].value+b.values[1].value+b.values.count+cast(int)b.before+b.after; }"
        ),
        20
    );
}

#[test]
fn direct_static_record_insertion_keeps_original_field_default_ids() {
    assert_eq!(
        run(
            "Owner :: struct { first:int=2; #insert #code { value:int=13; }; last:int=7; } main :: ()->int { value:Owner; return value.first+value.value+value.last; }"
        ),
        22
    );
}

#[test]
fn record_insertion_rejects_runtime_statements_and_preserves_duplicate_checks() {
    for source in [
        "Owner :: struct { #insert #code { return 7; }; } main :: () {}",
        "Owner :: struct { value:int; #insert #code { value:int; }; } main :: () {}",
    ] {
        let fixture = Fixture::new(source);
        let error = resolve_graph(&fixture.graph()).unwrap_err();
        assert!(
            error.message.contains("record") || error.message.contains("duplicate"),
            "{error:?}"
        );
    }
}

#[test]
fn baked_code_arguments_retain_the_shared_caller_capture() {
    assert_eq!(
        run(
            "Held :: struct(Body:Code) {} main :: ()->int { value:=3; Quote :: #code { value+=7; }; Owner :: Held(Quote); #insert Owner.Body; return value; }"
        ),
        10
    );
}

#[test]
fn record_assertions_and_conditional_fields_read_baked_formals() {
    assert_eq!(
        run(
            "Choice :: struct(T:Type,N:int=2) { #assert N>0 && N<4; #if N==2 { value:T=17; } else { values:[N]T; } } main :: ()->int { first:Choice(int); second:Choice(int,3); return first.value+second.values.count; }"
        ),
        20
    );
    let fixture = Fixture::new(
        "Choice :: struct(N:int) { #assert N>0; values:[N]int; } main :: () { value:Choice(0); }",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("assertion failed"), "{error:?}");
}

#[test]
fn conditional_nested_members_preserve_source_nominal_identity() {
    assert_eq!(
        run(
            "Choice :: struct(T:Type,Enabled:bool=true) { #if Enabled { Inner :: struct { value:T=19; } } else { Inner :: struct { value:T=23; } } item:Inner; } main :: ()->int { first:Choice(int); copy:Choice(int)=first; second:Choice(int,false); return copy.item.value+second.item.value; }"
        ),
        42
    );
}

#[test]
fn record_conditions_read_nullable_typed_callback_defaults() {
    assert_eq!(
        run(
            "Choice :: struct(callback:()->int=null) { #if callback { value:int=19; } else { value:int=23; } } supply :: ()->int { return 7; } main :: ()->int { first:Choice(); second:Choice(supply); return first.value+second.value; }"
        ),
        42
    );
}

#[test]
fn record_namespace_aggregate_constants_keep_typed_values() {
    assert_eq!(
        run(
            "Pair :: struct { left:int; right:int; } Holder :: struct(T:Type) { Saved :: Pair.{13,29}; value:=Saved; } main :: ()->int { first:Holder(int); return first.value.left+first.value.right; }"
        ),
        42
    );
}
#[test]
fn inferred_cast_fields_use_the_canonical_callback_target() {
    assert_eq!(
        run(
            "Callback :: #type ()->int; answer :: ()->int { return 42; } Holder :: struct(T:Type) { Saved :: cast(Callback)answer; callback:=cast(Callback)answer; other:=Saved; } main :: ()->int { value:Holder(int); if value.other()!=42 return 99; return value.callback(); }"
        ),
        42
    );
}
#[test]
fn record_default_overrides_keep_each_owners_nested_defaults() {
    assert_eq!(
        run(
            "Base :: struct { left:int=1; right:int=2; } Derived :: struct { base:Base; base.left=19; base.right=23; } Other :: struct { base:Base; base.left=7; } GLOBAL:Derived; main :: ()->int { a:Derived; b:Other; original:Base; if b.base.left!=7 || original.left!=1 return 99; return a.base.left+GLOBAL.base.right; }"
        ),
        42
    );
}
#[test]
fn inserted_and_conditional_default_overrides_follow_source_order() {
    assert_eq!(
        run(
            "Base :: struct { left:int=1; right:int=2; } Owner :: struct(T:Type,Fill:T) { base:Base; base.left=3; #insert #code { base.left=Fill; }; #if Fill>0 { base.right=23; } else { base.right=99; } } main :: ()->int { value:Owner(int,19); return value.base.left+value.base.right; }"
        ),
        42
    );
    for source in [
        "Owner :: struct { value:int; absent=3; } main :: () { value:Owner; }",
        "Owner :: struct { value:int; value=\"bad\"; } main :: () { value:Owner; }",
        "Owner :: struct { values:[2]int; values[0]=3; } main :: () { value:Owner; }",
        "Owner :: union { value:int; value=3; } main :: () { value:Owner; }",
    ] {
        let fixture = Fixture::new(source);
        let error = resolve_graph(&fixture.graph()).unwrap_err();
        assert!(
            error.message.contains("default")
                || error.message.contains("type")
                || error.message.contains("scalar"),
            "{error:?}"
        );
    }
}

#[test]
fn forward_nested_aggregate_constants_wait_for_one_reserved_shape() {
    assert_eq!(
        run(
            "Holder :: struct(T:Type,Fill:T=19) { Saved :: Inner.{Fill,23}; Inner :: struct { left:T; right:T; } value:=Saved; } main :: ()->int { value:Holder(int); copy:Holder(int)=value; return copy.value.left+copy.value.right; }"
        ),
        42
    );
}
#[test]
fn deeply_nested_record_defaults_inherit_late_ancestor_constants_once() {
    let mut source = String::from("Owner :: struct(T:Type) {");
    for depth in 0..20 {
        source.push_str(&format!(" Layer{depth} :: struct {{"));
    }
    source.push_str(" value:T=Answer;");
    for _ in 0..20 {
        source.push('}');
    }
    source.push_str(" Answer :: 42; } main :: ()->int { Root :: Owner(int); value:Root");
    for depth in 0..20 {
        source.push_str(&format!(".Layer{depth}"));
    }
    source.push_str("; return value.value; }");
    assert_eq!(run(&source), 42);
}
#[test]
fn procedure_arguments_infer_record_type_and_count_variables_by_origin() {
    assert_eq!(
        run(
            "Box :: struct(T:Type) { value:T; } Counted :: struct(N:int) { values:[N]int; } read :: (box:Box($T)) -> T { copy:Box(T)=box; return copy.value; } count :: (x:Counted($N)) -> int { copy:Counted(N)=x; return copy.values.count; } main :: () -> int { box:Box(int)=.{value=19}; data:Counted(3); return read(box)+count(data); }"
        ),
        22
    );
}
#[test]
fn generic_record_result_uses_the_reserved_instance() {
    assert_eq!(
        run(
            "Box :: struct(T:Type) { value:T; } same :: (box:Box($T)) -> Box(T) { return box; } main :: () -> int { box:Box(int)=.{value=23}; result:=same(box); return result.value; }"
        ),
        23
    );
}
#[test]
fn dependent_defaults_remain_typed_constraints_during_inference() {
    assert_eq!(
        run(
            "Box :: struct(T:Type, Fill:T=7, callback:(T)->T=null) { value:T=Fill; } read :: (box:Box($T)) -> T { return box.value; } copy :: (box:Box($T)) -> Box(T) { return box; } main :: () -> int { box:Box(int); other:=copy(box); return read(other); }"
        ),
        7
    );
    let fixture = Fixture::new(
        "Box :: struct(T:Type, Fill:T=7) { value:T=Fill; } read :: (box:Box($T)) -> T { return box.value; } main :: () -> int { box:Box(int,9); return read(box); }",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains("default"),
        "custom baked value must not match omitted default: {error:?}"
    );
}
#[test]
fn using_fields_in_instances_validate_promotion_and_ambiguity() {
    assert_eq!(
        run(
            "Inner :: struct(T:Type) { value:T=9; } Outer :: struct(T:Type) { using inner:Inner(T); } main :: () -> int { x:Outer(int); x.value+=3; return x.inner.value; }"
        ),
        12
    );
    for source in [
        "Inner :: struct(T:Type) { value:T; } Outer :: struct(T:Type) { using inner:Inner(T); value:T; } main :: () { x:Outer(int); }",
        "Bad :: struct(T:Type) { using scalar:T; } main :: () { x:Bad(int); }",
        "Bad :: struct(T:Type) { using self:Bad(T); } main :: () { x:Bad(int); }",
    ] {
        let fixture = Fixture::new(source);
        let error = resolve_graph(&fixture.graph()).unwrap_err();
        assert!(
            error.message.contains("using") || error.message.contains("ambiguous"),
            "{error:?}"
        );
    }
}
#[test]
fn source_owned_nested_enum_types_keep_parent_nominal_identity() {
    assert_eq!(
        run(
            "Owner :: struct { flags:Flags; mode:Mode=.VIEW; Flags :: enum_flags u32 { FIRST :: 1; SECOND :: 2; } Mode :: enum u16 { FIXED; VIEW; } } Other :: struct { flags:Flags; Flags :: enum_flags u32 { FIRST :: 1; SECOND :: 2; } } main :: () -> int { value:Owner; f:Owner.Flags=.FIRST; value.flags=f|.SECOND; if cast(int)value.flags != 3 return 99; return cast(int)value.flags+cast(int)value.mode; }"
        ),
        4
    );
    let fixture = Fixture::new(
        "Owner :: struct { Flags :: enum_flags u32 { FIRST; } } Other :: struct { Flags :: enum_flags u32 { FIRST; } } main :: () { a:Owner.Flags=.FIRST; b:Other.Flags=a; }",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(
        error.message.contains("different") || error.message.contains("type"),
        "{error:?}"
    );
}
#[test]
fn dependent_baked_value_patterns_preserve_the_inferred_formal_type() {
    assert_eq!(
        run(
            "Box :: struct(T:Type, Fill:T) { value:T=Fill; } read :: (box:Box($T,$Fill))->T { copy:Box(T,Fill)=box; return copy.value; } main :: ()->int { box:Box(int,19); return read(box); }"
        ),
        19
    );
    let fixture = Fixture::new(
        "Box :: struct(T:Type, Fill:T) { value:T=Fill; } wrong :: (box:Box($T,$Fill))->Box(u8,Fill) { return .{value=7}; } main :: () { box:Box(int,19); wrong(box); }",
    );
    let error = resolve_graph(&fixture.graph()).unwrap_err();
    assert!(error.message.contains("formal parameter type"), "{error:?}");
}
#[test]
fn nested_enum_member_defaults_are_typed_and_nominal() {
    assert_eq!(
        run(
            "Owner :: struct(T:Type) { flags:Flags=Flags.SECOND; Flags :: enum_flags u32 { FIRST; SECOND; } } main :: ()->int { value:Owner(int); return cast(int)value.flags; }"
        ),
        2
    );
}
#[test]
fn aggregate_baked_arguments_reuse_normalized_typed_constants() {
    assert_eq!(
        run(
            "Holder :: struct(Data:[2]int) { values:[2]int=Data; } main :: ()->int { a:Holder(.[13,29]); b:Holder(.[13,29])=a; zero:Holder(.[0,0]); return b.values[0]+b.values[1]+zero.values[1]; }"
        ),
        42
    );
}
#[test]
fn caller_shadows_do_not_change_definition_scope_defaults() {
    assert_eq!(
        run(
            "Item :: int; Seed :: 7; Box :: struct(T:Type=Item, Fill:T=Seed) { value:T=Fill; } main :: ()->int { Item :: u8; Seed :: 19; caller:Box(Item,Seed); defined:Box(); return cast(int)caller.value+defined.value; }"
        ),
        26
    );
}
#[test]
fn nested_enum_values_can_supply_forward_member_constants_and_inferred_fields() {
    assert_eq!(
        run(
            "Owner :: struct(T:Type) { value:=Saved; Saved :: Mode.VIEW; Mode :: enum u16 { FIXED; VIEW; } } main :: ()->int { value:Owner(int); return cast(int)value.value; }"
        ),
        1
    );
}
#[test]
fn inferred_aggregate_defaults_keep_their_typed_shape() {
    assert_eq!(
        run(
            "Holder :: struct(Data:=.[13,29], Label:=\"hello\") { values:[2]int=Data; label:string=Label; } main :: ()->int { a:Holder(); b:Holder(Data=.[17,20],Label=\"later\"); return a.values[0]+b.values[1]+a.label.count+b.label.count; }"
        ),
        43
    );
}
