use jai_modules::{GraphOptions, ModuleGraph};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn graph(source: &str) -> (Self, ModuleGraph) {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let fixture = Self(std::env::temp_dir().join(format!(
            "jai-short-lambda-sema-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        fs::create_dir_all(&fixture.0).unwrap();
        let path = fixture.0.join("main.jai");
        fs::write(&path, source).unwrap();
        let graph = ModuleGraph::load(&path, GraphOptions::default()).unwrap();
        (fixture, graph)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn rejection(source: &str, message: &str) {
    let (_fixture, graph) = Fixture::graph(source);
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    assert!(error.message.contains(message), "{error:?}");
}

#[test]
fn unconstrained_parameter_does_not_get_a_fabricated_type() {
    rejection(
        "main::()->int{f:=(value)=>value;return 42;}",
        "requires a call or procedure type context",
    );
}

#[test]
fn lambda_runtime_capture_is_rejected_at_the_source_reference() {
    let source = "main::()->int{outer:=40;f:(int)->int=(value)=>value+outer;return f(2);}";
    let (_fixture, graph) = Fixture::graph(source);
    let error = jai_sema::resolve_graph(&graph).unwrap_err();
    assert!(
        error.message.contains("capture") && error.message.contains("runtime"),
        "{error:?}"
    );
    assert_eq!(
        &source[error.location.span.start..error.location.span.end],
        "outer"
    );
}

#[test]
fn lambda_parameter_count_and_type_annotations_remain_checked() {
    rejection(
        "main::()->int{f:(int)->int=(left,right)=>left+right;return 42;}",
        "parameter count",
    );
    rejection(
        "main::()->int{f:(u8)->u8=(value:int)=>value;return 42;}",
        "annotation does not match",
    );
    rejection(
        "main::()->int{f:(int,int)->int=(value,value)=>value;return 42;}",
        "duplicate short lambda",
    );
}

#[test]
fn invalid_callback_result_or_capture_is_rejected_before_source_effects() {
    #[derive(Default)]
    struct Effects(usize);
    impl jai_vm::CompilerEffects for Effects {
        fn begin(&mut self) {
            self.0 += 1;
        }
        fn request(&mut self, _: jai_vm::CompilerRequest) -> jai_vm::EffectOutcome {
            panic!("callback preview cannot execute effects");
        }
        fn finish(&mut self, _: bool) -> Result<(), jai_vm::Error> {
            Ok(())
        }
    }
    for (source, expected) in [
        (
            "apply::(value:int,callback:(int)->int)->int{return callback(value);}main::()->int{return apply(#run 40,(value)=>true);}",
            "implicitly converted",
        ),
        (
            "apply::(value:int,callback:(int)->int)->int{return callback(value);}main::()->int{outer:=2;return apply(#run 40,(value)=>value+outer);}",
            "capture",
        ),
        (
            "ordinary::(value:int)->int{return value;}apply::(value:int,callback:(int)->int #c_call)->int{return callback(value);}main::()->int{return apply(#run 40,(value)=>ordinary(value));}",
            "context",
        ),
        (
            "apply::(value:int,callback:((int)->int)->int)->int{return value;}main::()->int{return apply(#run 40,(callback)=>(callback)(true));}",
            "implicitly converted",
        ),
        (
            "required::(value:int)->int #must{return value;}apply::(value:int,callback:(int)->void){callback(value);}main::()->int{apply(#run 40,(value)=>required(value));return 42;}",
            "#must",
        ),
        (
            "main::()->int{return ((value:u8)=>#run 42)(256);}",
            "out of range",
        ),
        (
            "checked::(value:u8)=>#run 42;main::()->int{return checked(256);}",
            "out of range",
        ),
        (
            "apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{return apply(#run 40,value=>{if value>0{return value;}});}",
            "may reach its end",
        ),
        (
            "apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{return apply(#run 40,value=>{IMMUTABLE::2;IMMUTABLE=3;return value;});}",
            "not mutable storage",
        ),
        (
            "IMMUTABLE::\"text\";apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{return apply(#run 40,value=>{IMMUTABLE[0]=1;return value;});}",
            "not mutable storage",
        ),
        (
            "required::(value:int)->int #must{return value;}apply::(value:int,f:(int)->void){f(value);}main::()->int{apply(#run 40,value=>{required(value);});return 42;}",
            "#must",
        ),
        (
            "apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{return apply(#run 40,value=>{array:[#run 2]int;return value;});}",
            "typed compile-time",
        ),
        (
            "apply::(value:int,f:(int)->int)->int{return f(value);}main::()->int{result:=apply(#run 40,value=>{T::type_of(array);return value;});array:[#run 2]int;return result;}",
            "typed compile-time",
        ),
        (
            "apply::(value:$T,f:(T)->$R)->R{return f(value);}main::()->int{result:=apply(#run 40,value=>{if value==40{return 42;}else{return 18446744073709551615;}});if type_of(result)==u64{return cast(int)result;}return 0;}",
            "context-free default type",
        ),
    ] {
        let (_fixture, graph) = Fixture::graph(source);
        let mut effects = Effects::default();
        let error = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions::default(),
            &mut effects,
        )
        .unwrap_err();
        assert_eq!(effects.0, 0, "{error:?}");
        assert!(error.message.contains(expected), "{error:?}");
    }
}

#[test]
fn recursive_inferred_lambda_has_a_precise_diagnostic() {
    rejection(
        "main::()->int{recursive::(value)=>recursive(value);return recursive(42);}",
        "recursive",
    );
}

#[test]
fn annotated_lambda_constant_result_obligations_remain_binding_specific() {
    rejection(
        "required:(left:int)->(int #must):value=>value;main::()->int{required(left=0);return 42;}",
        "#must",
    );
}
