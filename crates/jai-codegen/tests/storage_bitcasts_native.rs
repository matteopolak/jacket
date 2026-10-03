//! Authored storage views execute through selected-target O0/O2 LLVM objects.
#[path = "support/native_tools.rs"]
mod native_tools;
use jai_codegen::{
    optimization::Optimization,
    target::{NativeTarget, TargetOptions},
};
use jai_types::BitcodeOptimization;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str, c: Option<&str>) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let scratch = Scratch(std::env::temp_dir().join(format!(
        "jai-storage-bitcasts-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    fs::create_dir(&scratch.0).unwrap();
    let input = scratch.0.join("main.jai");
    fs::write(&input, source).unwrap();
    let c_path = scratch.0.join("fixture.c");
    if let Some(c) = c {
        fs::write(&c_path, c).unwrap();
    }
    let graph =
        jai_modules::ModuleGraph::load(&input, jai_modules::GraphOptions::default()).unwrap();
    for bitcode in [BitcodeOptimization::O0, BitcodeOptimization::O2] {
        let target = NativeTarget::select(&TargetOptions {
            optimization: Optimization {
                bitcode,
                ..Default::default()
            },
            ..Default::default()
        })
        .unwrap();
        let program = jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                target: Some(target.build_target().unwrap()),
                layout: Some(target.layout_policy().unwrap()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .unwrap();
        if c.is_none() {
            let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
            assert!(
                matches!(outcome,jai_vm::Outcome::Complete(ref values) if matches!(values.as_slice(),[jai_vm::Value::Int(value)] if value.value()==42)),
                "{outcome:?}"
            );
        }
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        module.verify().unwrap();
        let object = scratch.0.join("program.o");
        let executable = scratch.0.join("program");
        target.write_object(&module, &object).unwrap();
        let mut link = native_tools::clang_command();
        link.arg(&object);
        if c.is_some() {
            link.arg(&c_path);
        }
        link.arg(if bitcode == BitcodeOptimization::O0 {
            "-O0"
        } else {
            "-O2"
        })
        .arg("-o")
        .arg(&executable);
        let output = link.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut child = Command::new(&executable).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert_eq!(status.code(), Some(42), "{bitcode:?}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("storage bitcast fixture timed out");
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
#[test]
fn two_word_signed_unsigned_views_are_owned_and_rvalues_execute_once() {
    check(
        include_str!("../../jai-sema/tests/fixtures/int128-storage-bitcasts.jai.pending"),
        None,
    );
}
#[test]
fn fixed_array_element_views_and_smaller_prefix_keep_target_byte_order() {
    check(
        include_str!("../../jai-sema/tests/fixtures/force-array-prefix.jai.pending"),
        None,
    );
}
#[test]
fn initialized_prefix_does_not_read_uninitialized_source_tail() {
    check(
        r#"
        Wide::struct { low:u64; high:u64; }
        Prefix::struct { low:u64; }
        main::()->int {
            wide:Wide=---;
            wide.low=42;
            prefix:=cast,FORCE(Prefix)wide;
            return cast(int)prefix.low;
        }
    "#,
        None,
    );
}
#[test]
fn zero_byte_views_still_execute_value_producer() {
    check(
        r#"
        EmptyA::struct {} EmptyB::struct {}
        calls:int=0;
        produce::()->EmptyA { calls+=1; return EmptyA.{}; }
        main::()->int { empty:=cast,force(EmptyB)produce(); return calls+41; }
    "#,
        None,
    );
}
#[test]
fn source_storage_is_realigned_for_stronger_destination() {
    check(
        r#"
        Packed::struct { low:u64; high:u64; } #no_padding
        Aligned::struct #align 16 { low:u64; high:u64; }
        main::()->int {
            source:Packed=.{20,22};
            view:=cast,force(Aligned)source;
            if view.low!=20 || view.high!=22 return 1;
            return cast(int)(view.low+view.high);
        }
    "#,
        None,
    );
}
#[test]
fn reinterpreted_pair_crosses_actual_clang_c_abi_without_mutating_original() {
    check(
        r#"
        Signed::struct { low:u64; high:s64; }
        Unsigned::struct { low:u64; high:u64; }
        mutate::(value:Unsigned)->Unsigned #foreign "mutate_pair";
        main::()->int {
            original:Signed=.{41,-1};
            converted:=cast,force(Unsigned)original;
            returned:=mutate(converted);
            if original.low!=41 || original.high!=-1 || converted.low!=41 return 1;
            if returned.high!=0xffffffffffffffff return 2;
            return cast(int)returned.low;
        }
    "#,
        Some(
            r#"
        struct Pair { unsigned long long low; unsigned long long high; };
        _Static_assert(sizeof(struct Pair)==16,"two words");
        _Static_assert(__builtin_offsetof(struct Pair,high)==8,"second word");
        struct Pair mutate_pair(struct Pair value) { value.low+=1; return value; }
    "#,
        ),
    );
}
