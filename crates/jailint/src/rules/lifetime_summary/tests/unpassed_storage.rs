//! External promises authenticate declarations without granting purity or ownership.
use super::*;

const SOURCE: &str = r#"
    A::#library "private-a";
    B::#library "private-b";
    promised::()->s32 #foreign A "initialize"; @PreservesUnpassedStorage
    other::()->s32 #foreign B "initialize"; @PreservesUnpassedStorage
    unannotated::()->s32 #foreign A "initialize";
    wrong_param::(p:*void)->s32 #foreign A "initialize"; @PreservesUnpassedStorage
    variadic::(p:..u64)->s32 #foreign A "initialize"; @PreservesUnpassedStorage
    global_heap_clear::()->s32 #foreign A "clear_all_heap";
    wrong_return::()->s64 #foreign A "initialize"; @PreservesUnpassedStorage
    reset::()->s32 #foreign A "initialize_reset"; @PreservesUnpassedStorage @ResetsTemporaryStorage
    ordinary::()->s32 {return 0;} @PreservesUnpassedStorage
    main::(){promised();other();unannotated();wrong_param(null);wrong_return();variadic(1);global_heap_clear();reset();ordinary();}
"#;
fn id(c: &Compiler, name: &str) -> ProcId {
    ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == name)
            .unwrap() as u32,
    )
}
#[test]
fn unpassed_storage_requires_selected_promise_and_no_independent_resource_effect() {
    let mut c = stdlib_allocations::checked_source_with_contracts(SOURCE, Options::host());
    let before = Summaries::build(&c);
    for name in ["promised", "other"] {
        let s = &before.procs[&id(&c, name)];
        let proof = s.external_preservation.as_ref().expect(name);
        assert!(proof.current(&c, id(&c, name), s));
        assert!(proof.payload_bytes() > 0);
        assert!(s.frame.is_none() && s.native_allocation.is_none() && s.no_growth.is_none());
    }
    for name in [
        "unannotated",
        "wrong_param",
        "wrong_return",
        "variadic",
        "global_heap_clear",
        "reset",
        "ordinary",
    ] {
        assert!(
            before.procs[&id(&c, name)].external_preservation.is_none(),
            "{name}"
        );
    }
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
}
#[test]
fn unpassed_storage_rejects_stale_selected_library_and_lowered_abi() {
    let mut c = stdlib_allocations::checked_source_with_contracts(SOURCE, Options::host());
    let all = Summaries::build(&c);
    let p = id(&c, "promised");
    let s = &all.procs[&p];
    let proof = s.external_preservation.as_ref().unwrap();
    let jaic::sema::procs::ProcTarget::Foreign(fid) = c.proc(p).target.unwrap() else {
        panic!()
    };
    let other = id(&c, "other");
    let jaic::sema::procs::ProcTarget::Foreign(other) = c.proc(other).target.unwrap() else {
        panic!()
    };
    let original = c.out.program.foreigns[fid.0 as usize].library;
    c.out.program.foreigns[fid.0 as usize].library =
        c.out.program.foreigns[other.0 as usize].library;
    assert!(!proof.current(&c, p, s));
    c.out.program.foreigns[fid.0 as usize].library = original;
    assert!(proof.current(&c, p, s));
    c.out.program.foreigns[fid.0 as usize].sig.c_varargs = true;
    assert!(!proof.current(&c, p, s));
    c.out.program.foreigns[fid.0 as usize].sig.c_varargs = false;
    c.pr.proc_decl_notes.remove(&p);
    let mut lit = (*c.proc(p).lit).clone();
    let mut header = (*lit.header).clone();
    header.notes.clear();
    lit.header = std::sync::Arc::new(header);
    c.pr.procs[p.0 as usize].lit = std::sync::Arc::new(lit);
    assert!(
        !proof.current(&c, p, s),
        "removed declaration promise is stale"
    );
}

#[test]
fn unpassed_storage_own_native_initializer_is_a_conditional_boundary() {
    let c = stdlib_allocations::checked_source_with_contracts(
        r#"
        Crash::#import "Runtime_Support_Crash_Handler";
        main::(){Crash.init();}
    "#,
        Options::host(),
    );
    let all = Summaries::build(&c);
    let p = id(&c, "native_init");
    let s = &all.procs[&p];
    let proof = s
        .external_preservation
        .as_ref()
        .expect("own native declaration promise");
    assert!(proof.current(&c, p, s));
    assert!(s.frame.is_none() && s.native_allocation.is_none());
    let assertion = id(&c, "runtime_support_assertion_failed");
    assert_eq!(
        all.procs[&assertion].normal_return,
        Some(false),
        "actual selected default assertion reaches authenticated abort on every normal-return path"
    );
}

#[test]
fn unpassed_storage_native_range_requires_an_unpublished_instance() {
    let c = stdlib_allocations::checked_source_with_contracts(
        r#"
        C::#system_library "libc";
        A::#library "private-runtime";
        acquire::(n:u64)->*void #foreign C "malloc";
        initialize::()->s32 #foreign A "initialize"; @PreservesUnpassedStorage
        unknown::()->s32 #foreign A "clear_all_heap";
        published:*void;
        closed::(n:u64)->*void {p:=acquire(n);initialize();return p;}
        escaped::(n:u64)->*void {p:=acquire(n);published=p;initialize();return p;}
        opaque::(n:u64)->*void {p:=acquire(n);unknown();return p;}
        main::(){closed(64);escaped(64);opaque(64);}
    "#,
        Options::host(),
    );
    let all = Summaries::build(&c);
    let range = |name| {
        let p = id(&c, name);
        all.procs[&p]
            .native_allocation
            .as_ref()
            .and_then(|proof| proof.apply_nonnull(&c, &all.procs, (p, &[Some(64)]), &mut 0))
    };
    assert_eq!(
        range("closed")
            .expect("fresh object protected by explicit boundary")
            .bytes,
        64
    );
    assert!(range("escaped").is_none());
    assert!(range("opaque").is_none());
}
