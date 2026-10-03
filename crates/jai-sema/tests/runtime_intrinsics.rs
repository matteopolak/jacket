//! Self-written source fixtures; no original Jai code or native library is executed.
use jai_ir::{PrototypeOrigin, RuntimeIntrinsic};
use jai_modules::{GraphOptions, ModuleGraph};
use jai_sema::{ResolveOptions, resolve_graph_with_options};
use jai_types::{ContextMode, LayoutPolicy};
use jai_vm::{Limits, NoEffects, Outcome, Value};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "jai-runtime-source-{}-{}",
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
fn options() -> ResolveOptions {
    ResolveOptions {
        layout: Some(LayoutPolicy::lp64()),
        ..ResolveOptions::default()
    }
}
fn run_integer(source: &str) -> i128 {
    let fixture = Fixture::new(source);
    let program = resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap();
    match jai_vm::execute(&program, Limits::default()).outcome {
        Outcome::Complete(values) => match values.as_slice() {
            [Value::Int(value)] => value.value(),
            other => panic!("unexpected result: {other:?}"),
        },
        other => panic!("runtime source did not complete: {other:?}"),
    }
}

const MEMORY: &str = "memcpy :: (dest:*void, source:*void, count:s64) #intrinsic; memcmp :: (a:*void,b:*void,count:s64)->s16 #must #intrinsic; memset :: (dest:*void,value:u8,count:s64) #intrinsic;";
const MEMORY_BODY: &str = "fill :: () -> int { value:u64=0; memset(cast(*void) *value,42,8); copy:u64=0; memcpy(cast(*void) *copy,cast(*void) *value,8); if memcmp(cast(*void) *value,cast(*void) *copy,8) != 0 return 1; return cast(int) (copy & 255); }";

const FLAT_POOL: &str = "Flat_Pool :: struct { memblock_size:s64; bytes_left:s64; current_block:*void; current_pos:s64; alignment:s64=8; } get :: (pool:*Flat_Pool,size:s64)->*void #intrinsic; reset :: (pool:*Flat_Pool,overwrite:bool) #intrinsic; fini :: (pool:*Flat_Pool) #intrinsic;";

#[test]
fn pool_lifetime_operations_execute_in_runtime_and_real_source_run_requests() {
    let body = "proof :: () -> int { pool:Flat_Pool; pool.memblock_size=128; first:=cast(*u8) get(*pool,16); first[0]=42; reset(*pool,true); if first[0]!=204 return 1; reused:=cast(*u8) get(*pool,16); if reused!=first return 2; fini(*pool); if pool.bytes_left!=0 || pool.current_block!=null return 3; return 42; }";
    for call in ["proof()", "#run proof()"] {
        assert_eq!(
            run_integer(&format!(
                "{FLAT_POOL} {body} main :: () -> int {{ return {call}; }}"
            )),
            42
        );
    }
}

#[test]
fn pool_default_buffer_work_is_charged_before_source_run_allocation() {
    let source = format!(
        "{FLAT_POOL} proof :: () -> int {{ pool:Flat_Pool; get(*pool,1); fini(*pool); return 42; }} main :: () -> int {{ return #run proof(); }}"
    );
    let fixture = Fixture::new(&source);
    let graph = fixture.graph();
    let mut bounded = options();
    bounded.compile_time_limits.fuel = 1000;
    let error = resolve_graph_with_options(&graph, &bounded, &mut NoEffects).unwrap_err();
    assert!(error.message.to_lowercase().contains("fuel"), "{error:?}");
    assert_eq!(run_integer(&source), 42);
}

#[test]
fn source_memory_intrinsics_bind_typed_bodyless_contextless_prototypes_and_execute() {
    let source = format!("{MEMORY} {MEMORY_BODY} main :: () -> int {{ return fill(); }}");
    let fixture = Fixture::new(&source);
    let program = resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap();
    assert_eq!(program.library().prototypes().len(), 3);
    for prototype in program.library().prototypes() {
        assert!(matches!(prototype.origin, PrototypeOrigin::Intrinsic(_)));
        assert_eq!(
            program
                .types()
                .procedure_definition(prototype.signature)
                .unwrap()
                .context,
            ContextMode::None
        );
        assert!(program.procedure_by_id(prototype.id).is_none());
    }
    assert_eq!(run_integer(&source), 42);
}

#[test]
fn source_run_executes_runtime_intrinsics_and_embeds_the_completed_result() {
    assert_eq!(
        run_integer(&format!(
            "{MEMORY} {MEMORY_BODY} main :: () -> int {{ return #run fill(); }}"
        )),
        42
    );
}

#[test]
fn ordinary_same_named_procedures_keep_their_own_behavior() {
    assert_eq!(
        run_integer("memset :: () -> int { return 42; } main :: () -> int { return memset(); }"),
        42
    );
}

#[test]
fn source_unknown_tags_wrong_signatures_and_missing_target_fail_precisely() {
    for (source, expected) in [
        (
            "danger :: () #intrinsic \"invented\"; main :: () {}",
            "unsupported #intrinsic `invented`",
        ),
        (
            "danger :: (value:$T) #intrinsic \"invented\"; main :: () {}",
            "unsupported #intrinsic `invented`",
        ),
        (
            "memset :: (dest:*void,value:u8,count:s64)->s64 #intrinsic; main :: () {}",
            "memset requires",
        ),
        (
            "memset :: (#discard dest:*void,value:u8,count:s64) #intrinsic; main :: () {}",
            "fixed evaluated runtime parameters",
        ),
        (
            "memset :: (dest:*void,$value:u8,count:s64) #intrinsic; main :: () { value:u8=0; memset(cast(*void) *value,42,1); }",
            "fixed evaluated runtime parameters",
        ),
        (
            "memset :: (dest:*void,$$value:u8,count:s64) #intrinsic; main :: () { value:u8=0; memset(cast(*void) *value,42,1); }",
            "fixed evaluated runtime parameters",
        ),
    ] {
        let fixture = Fixture::new(source);
        let error =
            resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap_err();
        assert!(error.message.contains(expected), "{error:?}");
    }
    let fixture = Fixture::new("trap_here :: () #intrinsic \"llvm.debugtrap\"; main :: () {}");
    let error =
        resolve_graph_with_options(&fixture.graph(), &ResolveOptions::default(), &mut NoEffects)
            .unwrap_err();
    assert!(
        error.message.contains("selected target layout"),
        "{error:?}"
    );
}

#[test]
fn local_tagged_intrinsics_keep_their_identity_and_debugtrap_halts_the_vm() {
    let fixture =
        Fixture::new("main :: () { trap_here :: () #intrinsic \"llvm.debugtrap\"; trap_here(); }");
    let program = resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap();
    assert!(matches!(
        program.library().prototypes()[0].origin,
        PrototypeOrigin::Intrinsic(RuntimeIntrinsic::DebugTrap)
    ));
    assert_eq!(
        jai_vm::execute(&program, Limits::default()).outcome,
        Outcome::Failed(jai_vm::Error::RuntimeTrap)
    );
}

#[test]
fn concrete_compare_and_swap_preserves_boolean_multiresults() {
    assert_eq!(
        run_integer(
            "compare_and_swap :: (pointer:*bool,old:bool,new:bool)->(success:bool,old_value:bool) #intrinsic; main :: () -> int { flag:bool=false; success, old := compare_and_swap(*flag,false,true); if success && !old && flag return 42; return 1; }"
        ),
        42
    );
}

#[test]
fn local_intrinsics_are_available_inside_a_real_compile_time_request() {
    let fixture = Fixture::new(
        "main :: () { #run { trap_here :: () #intrinsic \"llvm.debugtrap\"; trap_here(); } }",
    );
    let error =
        resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap_err();
    assert!(error.message.contains("runtime debug trap"), "{error:?}");
}

const GENERIC_CAS: &str =
    "compare_and_swap :: (pointer:*$T,old:T,new:T)->(success:bool,old_value:T) #intrinsic;";
const GENERIC_BODY: &str = "change :: () -> int { flag:bool=false; flag_success,old_flag:=compare_and_swap(*flag,false,true); number:s64=7; success,old_number:=compare_and_swap(*number,7,42); if flag_success && !old_flag && flag && success && old_number == 7 return number; return 1; }";

#[test]
fn generic_runtime_prototypes_specialize_without_fake_bodies_and_execute() {
    let source = format!("{GENERIC_CAS} {GENERIC_BODY} main :: () -> int {{ return change(); }}");
    let fixture = Fixture::new(&source);
    let program = resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap();
    assert_eq!(program.library().prototypes().len(), 2);
    for prototype in program.library().prototypes() {
        assert!(matches!(
            prototype.origin,
            PrototypeOrigin::Intrinsic(RuntimeIntrinsic::CompareAndSwap { .. })
        ));
        assert!(program.procedure_by_id(prototype.id).is_none());
    }
    assert_eq!(run_integer(&source), 42);
}

#[test]
fn generic_runtime_specialization_is_available_during_source_run() {
    assert_eq!(
        run_integer(&format!(
            "{GENERIC_CAS} {GENERIC_BODY} main :: () -> int {{ return #run change(); }}"
        )),
        42
    );
}

#[test]
fn intrinsic_procedure_values_dispatch_by_checked_identity() {
    assert_eq!(
        run_integer(
            "memset :: (dest:*void,value:u8,count:s64) #intrinsic; main :: () -> int { value:u64=0; fill:=memset; fill(cast(*void) *value,42,8); return cast(int) (value & 255); }"
        ),
        42
    );
}

#[test]
fn required_runtime_results_cannot_be_discarded() {
    for call in [
        "memcmp(cast(*void) *value,cast(*void) *value,1);",
        "compare:=memcmp; compare(cast(*void) *value,cast(*void) *value,1);",
        "compare :: (a:*void,b:*void,count:s64)->s16 #must #intrinsic \"memcmp\"; compare(cast(*void) *value,cast(*void) *value,1);",
    ] {
        let fixture = Fixture::new(&format!("{MEMORY} main :: () {{ value:u8=0; {call} }}"));
        let error =
            resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap_err();
        assert!(error.message.contains("#must"), "{error:?}");
    }
}

#[test]
fn source_run_memory_operations_cannot_publish_address_dependent_scalars() {
    for body in [
        "origin:s64=7; value:u8=0; memset(cast(*void) *value,0,cast(s64) *origin); return 42;",
        "origin:s64=7; value:u8=0; memset(cast(*void) *value,cast,no_check(u8) cast(u64) *origin,1); return cast(int) value;",
        "origin:s64=7; value:u8=0; copy:u8=0; memset(cast(*void) *value,cast,no_check(u8) cast(u64) *origin,1); memcpy(cast(*void) *copy,cast(*void) *value,1); return cast(int) copy;",
        "origin:s64=7; number:u64=cast(u64) *origin; changed,old:=compare_and_swap(*number,number,0); return cast(int) old;",
        "first:s64=1; second:s64=2; a:=*first; b:=*second; return cast(int) memcmp(cast(*void) *a,cast(*void) *b,8);",
    ] {
        let fixture = Fixture::new(&format!(
            "{MEMORY} {GENERIC_CAS} probe :: () -> int {{ {body} }} main :: () -> int {{ return #run probe(); }}"
        ));
        let error =
            resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap_err();
        assert!(error.message.contains("address"), "{body}: {error:?}");
    }
    for body in [
        "origin:s64=7; value:u8=0; filled:=memset(cast(*void) *value,0,cast(s64) *origin); return 42;",
        "origin:s64=7; value:u8=0; copy:u8=0; filled:=memset(cast(*void) *value,cast(s64) *origin,1); copied:=memcpy(cast(*void) *copy,cast(*void) *value,1); return cast(int) copy;",
    ] {
        let fixture = Fixture::new(&format!(
            "{DESTINATION_MEMORY} probe :: () -> int {{ {body} }} main :: () -> int {{ return #run probe(); }}"
        ));
        let error =
            resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap_err();
        assert!(error.message.contains("address"), "{body}: {error:?}");
    }
}

#[test]
fn source_run_comparing_equal_complete_pointer_values_has_a_portable_zero_result() {
    assert_eq!(
        run_integer(&format!(
            "{MEMORY} probe :: () -> int {{ origin:s64=7; first:=*origin; second:=cast(*s64) cast(*u8) *origin; return cast(int) memcmp(cast(*void) *first,cast(*void) *second,8); }} main :: () -> int {{ return 42 + #run probe(); }}"
        )),
        42
    );
}

const DESTINATION_MEMORY: &str = "memcpy :: (dest:*void,source:*void,count:s64)->*void #intrinsic; memset :: (dest:*void,value:s64,count:s64)->*void #intrinsic;";
const DESTINATION_BODY: &str = "probe :: () -> int { value:u64=0; fill:=memset; filled:=fill(cast(*void) *value,-214,8); if filled!=cast(*void) *value return 1; copy:u64=0; copied:=memcpy(cast(*void) *copy,cast(*void) *value,8); if copied!=cast(*void) *copy return 2; if memcpy(null,null,0)!=null return 3; return cast(int) (copy & 255); }";
const SWAP: &str = "swap :: (a:*$T,b:*T) #intrinsic;";
const SWAP_BODY: &str = "Box :: struct { value:s64; target:*s64; } probe :: () -> int { first:s64=1; second:s64=42; swap(*first,*second); if first!=42 || second!=1 return 1; a:Box=.{value=7,target=*first}; b:Box=.{value=42,target=*second}; swap(*a,*b); swap(*a,*a); if a.value!=42 || a.target!=*second || b.value!=7 || b.target!=*first return 2; flag:bool=false; other:bool=true; swap(*flag,*other); if !flag || other return 3; swap_boxes :: (a:*Box,b:*Box) #intrinsic \"swap\"; exchange:=swap_boxes; exchange(*a,*b); if b.value!=42 return 4; return first; }";

#[test]
fn destination_returning_memory_contracts_preserve_pointer_identity_and_low_byte_fill() {
    let source =
        format!("{DESTINATION_MEMORY} {DESTINATION_BODY} main :: () -> int {{ return probe(); }}");
    let fixture = Fixture::new(&source);
    let program = resolve_graph_with_options(&fixture.graph(), &options(), &mut NoEffects).unwrap();
    assert!(
        program
            .library()
            .prototypes()
            .iter()
            .any(|prototype| matches!(
                prototype.origin,
                PrototypeOrigin::Intrinsic(RuntimeIntrinsic::MemoryCopyReturningDestination)
            ))
    );
    assert!(
        program
            .library()
            .prototypes()
            .iter()
            .any(|prototype| matches!(
                prototype.origin,
                PrototypeOrigin::Intrinsic(RuntimeIntrinsic::MemorySetReturningDestination)
            ))
    );
    assert_eq!(run_integer(&source), 42);
}

#[test]
fn generic_and_local_swap_intrinsics_preserve_aggregate_pointer_values() {
    assert_eq!(
        run_integer(&format!(
            "{SWAP} {SWAP_BODY} main :: () -> int {{ return probe(); }}"
        )),
        42
    );
}

#[test]
fn source_run_executes_specialized_swap_and_destination_returning_memory() {
    for declarations in [
        format!("{SWAP} {SWAP_BODY}"),
        format!("{DESTINATION_MEMORY} {DESTINATION_BODY}"),
    ] {
        assert_eq!(
            run_integer(&format!(
                "{declarations} main :: () -> int {{ return #run probe(); }}"
            )),
            42
        );
    }
}
