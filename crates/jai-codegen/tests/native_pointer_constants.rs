//! Native sentinels do not acquire bounded-VM data or procedure provenance.
#[path = "support/checked_execution.rs"]
#[allow(dead_code, reason = "Shared harness exposes additional runners.")]
mod checked_execution;

#[test]
fn native_sentinel_globals_records_and_parameter_defaults_preserve_the_bits() {
    checked_execution::check_vm_unsupported_optimized(
        r#"
        SENTINEL::cast,trunc(*void)-1;
        global:*void=cast,trunc(*void)-1;
        Holder::struct { address:*void=cast,trunc(*void)-1; }
        take::(address:*void=cast,trunc(*void)-1)->int {
            if address!=global || address!=SENTINEL return 1;
            value:Holder;
            if value.address!=address return 2;
            if size_of(*void)==8 && cast(u64)address!=0xffffffffffffffff return 3;
            if size_of(*void)==4 && cast(u64)address!=0xffffffff return 4;
            return 42;
        }
        main::()->int { return take(); }
        "#,
        42,
    );
}

#[test]
fn zero_pointer_cast_defaults_remain_canonical_null_in_both_executors() {
    checked_execution::check(
        r#"
        global:*void=cast,trunc(*void)0;
        take::(value:*void=xx,no_check 0)->int {
            if value!=null || global!=null return 1;
            return 42;
        }
        main::()->int { return take(); }
        "#,
        42,
    );
}

#[test]
fn actual_llvm_targets_normalize_constant_globals_at_32_and_64_bits() {
    use jai_codegen::target::{NativeTarget, TargetOptions, TargetSelection, Triple};
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    struct Fixture(std::path::PathBuf);
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    let fixture = Fixture(std::env::temp_dir().join(format!(
        "jai-native-address-targets-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
    )));
    fs::create_dir_all(&fixture.0).unwrap();
    let path = fixture.0.join("main.jai");
    fs::write(
        &path,
        r#"
        sentinel:*void=cast,trunc(*void)-1;
        checked:*void=cast(*void)cast(s32)-1;
        wrapping:*void=cast,trunc(*void)cast(u64)0x100000000;
        main::()->int {
            if sentinel!=checked return 1;
            if size_of(*void)==4 && wrapping!=null return 2;
            if size_of(*void)==8 && cast(u64)wrapping!=0x100000000 return 3;
            return 42;
        }
    "#,
    )
    .unwrap();
    let graph =
        jai_modules::ModuleGraph::load(&path, jai_modules::GraphOptions::default()).unwrap();
    for (triple, bits) in [
        ("i686-unknown-linux-gnu", 32),
        ("x86_64-unknown-linux-gnu", 64),
    ] {
        let target = NativeTarget::select(&TargetOptions {
            selection: TargetSelection::Triple(Triple::new(triple).unwrap()),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(target.data.get_pointer_byte_size(None) * 8, bits);
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
        let context = jai_codegen::Context::create();
        let module = jai_codegen::lower_for_target(&context, &program, &target).unwrap();
        module.verify().unwrap();
        let llvm = module.print_to_string().to_string();
        assert!(
            llvm.contains(&format!("inttoptr (i{bits} -1 to ptr)")),
            "{triple}: {llvm}"
        );
        let object = fixture.0.join("program.o");
        target.write_object(&module, &object).unwrap();
        assert!(fs::metadata(&object).unwrap().len() > 32);
    }
}
