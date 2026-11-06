use super::*;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn compiler(source: &str, windows: bool) -> Compiler {
    compiler_target(source, windows.then_some(jaic::target::TargetOs::Windows))
}

fn compiler_target(source: &str, target: Option<jaic::target::TargetOs>) -> Compiler {
    compiler_target_cpu(source, target, None)
}

fn compiler_target_cpu(
    source: &str,
    target: Option<jaic::target::TargetOs>,
    cpu: Option<jaic::target::TargetCpu>,
) -> Compiler {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let stdlib = jacket_config::env::STDLIB
        .os()
        .map_or(stdlib, std::path::PathBuf::from);
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    if let Some(target) = target {
        options.os = target;
        options.cpu = jaic::target::TargetCpu::X64;
    }
    if let Some(cpu) = cpu {
        options.cpu = cpu;
    }
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    c
}

/// Prefer the explicit fixture module; imported carriers need one checked instance.
/// For example, `id(c, "acquire")` must not select an imported lock helper.
fn id(c: &Compiler, name: &str) -> ProcId {
    let named: Vec<_> =
        c.pr.procs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name.as_str() == name)
            .collect();
    let fixture: Vec<_> = named
        .iter()
        .copied()
        .filter(|(_, p)| {
            c.src
                .files
                .iter()
                .any(|f| f.id == p.span.file && Some(f.module) == c.ns.main_module)
        })
        .collect();
    let candidates: Vec<_> = if fixture.is_empty() {
        named
            .into_iter()
            .filter(|(_, p)| p.sig.is_some() && p.target.is_some())
            .collect()
    } else {
        fixture
    };
    let [(index, _)] = candidates.as_slice() else {
        panic!("expected one selected {name}, found {}", candidates.len());
    };
    ProcId(*index as u32)
}

#[test]
fn scalar_handles_native_validity_and_copied_inverted_guard() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (path:*u8, mode:*u8)->*void #foreign C "fopen";
        fd :: (path:*u8, flags:s32)->s32 #foreign C "open";
        wrapper :: (p:*u8)->*void,bool {
            h:=acquire(p,p); ok:=h!=null; copy:=ok; return h,!copy;
        }
        descriptor :: (p:*u8)->s32 {return fd(p,0);}
        pass :: (old:*void)->*void {return old;}
        fopen :: (p:*u8, q:*u8)->*void {return p;}
        fake :: (p:*u8)->*void {return fopen(p,p);}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    let o = a.opening(id(&c, "wrapper")).unwrap();
    assert_eq!(o.alternatives[0].kind, NativeKind::Stream);
    assert_eq!(
        o.alternatives[0].guard,
        Some(ResultGuard {
            result: 1,
            valid_when: false
        })
    );
    assert_eq!(
        a.opening(id(&c, "descriptor")).unwrap().alternatives[0].validity,
        Validity::NonNegative
    );
    assert!(Validity::NonNegative.accepts(0));
    assert!(!Validity::NonNegative.accepts(u64::MAX));
    assert!(!Validity::NonNegative.accepts(u32::MAX as u64));
    assert!(a.opening(id(&c, "pass")).is_none());
    assert!(a.opening(id(&c, "fake")).is_none());
}

#[test]
fn scalar_handles_real_file_carriers_both_checked_targets() {
    for windows in [false, true] {
        let mut c = compiler(
            r#"#import "File"; main :: () { f,ok:=file_open("x"); file_close(*f); }"#,
            windows,
        );
        let a = Analysis::build(&c);
        let o = a
            .opening(id(&c, "file_open"))
            .unwrap_or_else(|| panic!("no file_open, windows={windows}"));
        assert!(!o.alternatives.is_empty());
        assert!(
            o.incomplete,
            "allocator callbacks after native acquisition require a lifetime frame proof"
        );
        assert!(
            o.alternatives
                .iter()
                .all(|c| !c.requires_noninterference.is_empty())
        );
        assert!(
            o.alternatives
                .iter()
                .all(|r| r.result == 0 && r.slot.is_some())
        );
        assert!(o.alternatives.iter().all(|r| r.kind
            == if windows {
                NativeKind::WindowsHandle
            } else {
                NativeKind::Stream
            }));
        assert!(
            o.alternatives.iter().all(|r| r.guard
                == Some(ResultGuard {
                    result: 1,
                    valid_when: true
                })),
            "windows={windows}: {o:?}"
        );
        c.release_lowered_code();
        assert_eq!(a.openings, Analysis::build(&c).openings);
    }
}

#[test]
fn scalar_handles_fresh_sites_copies_and_release_outcomes() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        release :: (p:*void)->s32 #foreign C "fclose";
        maybe_release :: (h:s32)->s32 #foreign C "close";
        fd :: (p:*u8,f:s32)->s32 #foreign C "open";
        opaque :: (p:*void) #foreign C "unknown_resource_effect";
        Pair :: struct { pad:s64; resource:*void; }
        first :: (p:*u8)->*void,*void {return acquire(p,p),acquire(p,p);}
        stale :: (p:*u8)->*void {h:=acquire(p,p); old:=h; release(h); return old;}
        replaced :: (p:*u8)->*void {h:=acquire(p,p); release(h); h=acquire(p,p); return h;}
        retained :: (p:*u8)->s32 {h:=fd(p,0); maybe_release(h); return h;}
        transfer :: (h:*void)->*void {opaque(h);return h;}
        unknown :: (p:*u8)->*void {return transfer(acquire(p,p));}
        epoch :: (p:*u8)->Pair {a:=Pair.{3,acquire(p,p)}; b:=a; a.resource=null; return b;}
        old_return :: (p:*u8,old:*void)->*void {h:=acquire(p,p);return old;}
        looped :: (p:*u8,n:s64)->*void {h:*void; for 0..n h=acquire(p,p);return h;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    let both = &a.opening(id(&c, "first")).unwrap().alternatives;
    assert_eq!(both.len(), 2);
    assert_ne!(both[0].template, both[1].template);
    assert!(a.opening(id(&c, "stale")).is_none());
    assert!(a.opening(id(&c, "replaced")).is_some());
    assert!(
        a.opening(id(&c, "retained")).is_none(),
        "unsupported release failure is unknown, not retained-live proof"
    );
    assert!(a.opening(id(&c, "unknown")).is_none());
    assert!(a.opening(id(&c, "old_return")).is_none());
    assert!(a.opening(id(&c, "looped")).is_none());
    let epoch = &a.opening(id(&c, "epoch")).unwrap().alternatives[0];
    assert_eq!(epoch.slot.unwrap().offset, 8);
}

#[test]
fn scalar_handles_authentication_and_unrelated_success() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        private :: #library "untrusted";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        wrong_library :: (p:*u8,q:*u8)->*void #foreign private "fopen";
        wrong_abi :: (p:s64)->s64 #foreign C "fopen";
        unrelated :: (p:*u8,ok:bool)->*void,bool {return acquire(p,p),ok;}
        narrow :: (p:*u8)->*void,bool {h:=acquire(p,p);return h,h!=null && h!=cast(*void) 2;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    assert!(a.opening(id(&c, "wrong_library")).is_none());
    assert!(a.opening(id(&c, "wrong_abi")).is_none());
    assert!(
        a.opening(id(&c, "unrelated"))
            .unwrap()
            .alternatives
            .iter()
            .all(|r| r.guard.is_none())
    );
    assert!(
        a.opening(id(&c, "narrow"))
            .unwrap()
            .alternatives
            .iter()
            .all(|r| r.guard.is_none())
    );
}

#[test]
fn scalar_handles_checked_formals_nonzero_result_and_stale_body() {
    let mut c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        select :: (left:*void,right:*void)->*void {return left;}
        nonzero :: (p:*u8)->bool,*void {return true,acquire(p,p);}
        named :: (p:*u8,old:*void)->*void {return select(right=old,left=acquire(p,p));}
        inverse :: (p:*u8,old:*void)->*void {return select(right=acquire(p,p),left=old);}
        output :: (p:*u8,dst:**void)->bool {dst.*=acquire(p,p);return dst.*!=null;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    assert_eq!(
        a.opening(id(&c, "nonzero")).unwrap().alternatives[0].result,
        1
    );
    assert!(a.opening(id(&c, "named")).is_some());
    assert!(a.opening(id(&c, "inverse")).is_none());
    assert!(a.opening(id(&c, "output")).is_none());
    let nonzero = id(&c, "nonzero");
    let other = id(&c, "inverse");
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&nonzero)
        .unwrap()
        .procedure = other;
    assert!(
        Analysis::build(&c).opening(nonzero).is_none(),
        "mismatched body key cannot supply an acquisition proof"
    );
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&nonzero)
        .unwrap()
        .procedure = nonzero;
    let named = id(&c, "named");
    let p = &mut c.pr.procs[named.0 as usize];
    p.lit = std::sync::Arc::new((*p.lit).clone());
    assert!(Analysis::build(&c).opening(named).is_none());
}

#[test]
fn scalar_handles_close_status_does_not_certify_failure_liveness() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,f:s32)->s32 #foreign C "open";
        release :: (h:s32)->s32 #foreign C "close";
        closed :: (p:*u8)->s32 {h:=acquire(p,0); if release(h)==0 return h; return -1;}
        retained :: (p:*u8)->s32 {h:=acquire(p,0); if release(h)!=0 return h; return -1;}
        forwarded :: (h:s32)->s32 {return release(h);}
        indirect_status :: (p:*u8)->s32 {h:=acquire(p,0); if forwarded(h)==0 return h; return -1;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    assert!(a.opening(id(&c, "closed")).is_none());
    assert!(a.opening(id(&c, "indirect_status")).is_none());
    assert!(
        a.opening(id(&c, "retained")).is_none(),
        "unsupported release failure is unknown, not retained-live proof"
    );
}

#[test]
fn scalar_handles_windows_close_status_and_wrong_resource_kind() {
    let c = compiler(
        r#"
        W :: #system_library "kernel32";
        C :: #system_library "c";
        acquire :: (p:*u16,a:u32,s:u32,x:*void,d:u32,f:u32,t:*void)->*void #foreign W "CreateFileW";
        release :: (h:*void)->s32 #foreign W "CloseHandle";
        stream :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        stream_release :: (h:*void)->s32 #foreign C "fclose";
        closed :: (p:*u16)->*void {h:=acquire(p,0,0,null,0,0,null);if release(h)!=0 return h;return null;}
        retained :: (p:*u16)->*void {h:=acquire(p,0,0,null,0,0,null);if release(h)==0 return h;return null;}
        wrong_kind :: (p:*u8)->*void {h:=stream(p,p);if release(h)==0 return h;return null;}
        other_wrong_kind :: (p:*u16)->*void {h:=acquire(p,0,0,null,0,0,null);stream_release(h);return h;}
        main :: () {}
    "#,
        true,
    );
    let a = Analysis::build(&c);
    assert!(a.opening(id(&c, "closed")).is_none());
    assert!(
        a.opening(id(&c, "retained")).is_none(),
        "unsupported release failure is unknown, not retained-live proof"
    );
    assert!(a.opening(id(&c, "wrong_kind")).is_none());
    assert!(a.opening(id(&c, "other_wrong_kind")).is_none());
}

#[test]
fn scalar_handles_cancellation_discards_partial_analysis_and_polls_body() {
    const CHILD: jacket_config::env::Var = jacket_config::env::LIFETIME_CANCELLATION_CHILD;
    if !CHILD.is_set() {
        let name = std::thread::current().name().unwrap().to_owned();
        let status = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &name, "--test-threads=1"])
            .env(CHILD.name, "1")
            .status()
            .unwrap();
        assert!(status.success());
        return;
    }
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        wrapper :: (p:*u8)->*void {h:=acquire(p,p);copy:=h;return copy;}
        main :: () {}
    "#,
        false,
    );
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            jaic::interrupt::disarm();
            jaic::interrupt::request(false);
            jaic::interrupt::stop_after(0);
        }
    }
    let _reset = Reset;
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(3);
    assert!(Analysis::build(&c).openings.is_empty());
    assert!(jaic::interrupt::disarm());
    let facts = c.ed.facts().unwrap();
    let body = &facts.lifetime_bodies[&id(&c, "wrapper")];
    let mut solver = Solver {
        c: &c,
        bodies: facts
            .lifetime_bodies
            .iter()
            .map(|(id, b)| (*id, b))
            .collect(),
        cache: HashMap::default(),
        active: HashSet::default(),
    };
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(2);
    assert!(solver.relation(body.procedure).is_none());
    assert!(jaic::interrupt::fired());
}

#[test]
fn scalar_handles_straight_line_boolean_composition_widens_before_tree_growth() {
    let mut source = String::from(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        composed :: (p:*u8)->*void,bool {h:=acquire(p,p);ok:=h!=null;
    "#,
    );
    for _ in 0..256 {
        source.push_str("ok=!ok;\n");
    }
    source.push_str("return h,ok;} main :: () {} ");
    let c = compiler(&source, false);
    let a = Analysis::build(&c);
    let capability = &a
        .opening(id(&c, "composed"))
        .expect("resource origin survives predicate widening")
        .alternatives[0];
    assert!(
        capability.guard.is_none(),
        "exhausted expression budget must not export a guard"
    );
}

#[test]
fn scalar_handles_boolean_fanout_checks_combined_node_budget() {
    let mut expression = Expr::Input(
        0,
        Slot {
            offset: 0,
            width: 1,
            ty: TypeId::BOOL,
        },
    );
    let mut widened = false;
    for _ in 0..128 {
        expression = Expr::and(Some(&expression), Some(&expression));
        let shape = expression
            .shape()
            .expect("constructed expressions stay bounded");
        assert!(shape.nodes <= super::expression::MAX_NODES);
        assert!(shape.depth <= super::expression::MAX_DEPTH);
        widened |= expression == Expr::Unknown;
    }
    assert!(
        widened,
        "duplicated subexpressions must exhaust the node budget"
    );
}

#[test]
fn scalar_handles_adjusted_as_pointer_is_not_original_native_handle() {
    let mut c = compiler(
        r#"
        W :: #system_library "kernel32";
        acquire :: (p:*u16,a:u32,s:u32,x:*void,d:u32,f:u32,t:*void)->*void #foreign W "CreateFileW";
        Derived :: struct{pad:s64;#as inner:s64;}
        Same :: struct{#as inner:s64;}
        shifted :: (p:*u16)->*void {h:=acquire(p,0,0,null,0,0,null);return cast(*void) cast(*s64) cast(*Derived) h;}
        preserved :: (p:*u16)->*void {h:=acquire(p,0,0,null,0,0,null);return cast(*void) cast(*s64) cast(*Same) h;}
        main :: () {}
    "#,
        true,
    );
    let a = Analysis::build(&c);
    assert!(a.opening(id(&c, "preserved")).is_some());
    assert!(
        a.opening(id(&c, "shifted")).is_none(),
        "nonzero #as address adjustment is not a live native capability"
    );
    c.release_lowered_code();
    assert_eq!(a.openings, Analysis::build(&c).openings);
}

#[test]
fn scalar_handles_linux_close_failure_and_fclose_failure_do_not_return_live() {
    let source = r#"
        C :: #system_library "c";
        acquire :: (p:*u8,f:s32)->s32 #foreign C "open";
        release :: (h:s32)->s32 #foreign C "close";
        stream :: (p:*u8,m:*u8)->*void #foreign C "fopen";
        stream_release :: (p:*void)->s32 #foreign C "fclose";
        fd_failure :: (p:*u8)->s32{h:=acquire(p,0);if release(h)!=0 return h;return -1;}
        stream_failure :: (p:*u8)->*void{h:=stream(p,p);if stream_release(h)!=0 return h;return null;}
        reopened_after_failure :: (p:*u8)->s32{h:=acquire(p,0);if release(h)!=0 return acquire(p,0);return -1;}
        main :: () {}
    "#;
    let c = compiler_target(source, Some(jaic::target::TargetOs::Linux));
    let a = Analysis::build(&c);
    assert!(
        a.opening(id(&c, "reopened_after_failure")).is_some(),
        "a later acquisition is independently fresh"
    );
    assert!(
        a.opening(id(&c, "fd_failure")).is_none(),
        "Linux closes the fd before reporting late I/O failure"
    );
    assert!(
        a.opening(id(&c, "stream_failure")).is_none(),
        "fclose invalidates the stream after either outcome"
    );
}

#[test]
fn scalar_handles_closehandle_failure_has_no_generic_retained_guarantee() {
    let c = compiler(
        r#"
        W :: #system_library "kernel32";
        acquire :: (p:*u16,a:u32,s:u32,x:*void,d:u32,f:u32,t:*void)->*void #foreign W "CreateFileW";
        release :: (h:*void)->s32 #foreign W "CloseHandle";
        failure :: (p:*u16)->*void{h:=acquire(p,0,0,null,0,0,null);if release(h)==0 return h;return null;}
        main :: () {}
    "#,
        true,
    );
    assert!(
        Analysis::build(&c).opening(id(&c, "failure")).is_none(),
        "zero status alone does not certify a live protected handle"
    );
}

#[test]
fn scalar_handles_target_failure_disposition_is_not_generic_consumption() {
    let source = r#"C :: #system_library "c";fd_release_contract_target :: (h:s32)->s32 #foreign C "close";probe :: (h:s32)->s32{return fd_release_contract_target(h);}main :: () {result:=probe(0);}"#;
    let linux = compiler_target(source, Some(jaic::target::TargetOs::Linux));
    let linux = native::native(&linux, id(&linux, "fd_release_contract_target")).unwrap();
    assert!(
        linux
            .exits
            .iter()
            .all(|e| !e.consumed.is_empty() && e.uncertain.is_empty())
    );
    let mac = compiler_target(source, Some(jaic::target::TargetOs::MacOS));
    let mac = native::native(&mac, id(&mac, "fd_release_contract_target")).unwrap();
    assert!(
        mac.exits
            .iter()
            .any(|e| e.consumed.is_empty() && !e.uncertain.is_empty()),
        "unverified target failure is uncertain, not guaranteed consumed"
    );
}

#[test]
fn scalar_handles_authenticated_global_stream_close_invalidates_private_alias() {
    let c = compiler_target(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,m:*u8)->*void #foreign C "fopen";
        release_all :: ()->s32 #foreign C "fcloseall";
        closed :: (p:*u8)->*void{h:=acquire(p,p);copy:=h;release_all();return copy;}
        reopened :: (p:*u8)->*void{h:=acquire(p,p);release_all();return acquire(p,p);}
        main :: () {}
    "#,
        Some(jaic::target::TargetOs::Linux),
    );
    let a = Analysis::build(&c);
    assert!(a.opening(id(&c, "reopened")).is_some());
    assert!(
        a.opening(id(&c, "closed")).is_none(),
        "private pointer bits do not preserve a globally closed stream"
    );
}

#[test]
fn scalar_handles_numeric_release_alias_respects_checked_guards() {
    let c = compiler_target(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,f:s32)->s32 #foreign C "open";
        release :: (h:s32)->s32 #foreign C "close";
        equal :: (p:*u8)->s32{h:=acquire(p,0);if h==3 {if release(3)==0 return h;}return -1;}
        distinct :: (p:*u8)->s32{h:=acquire(p,0);if h!=3 && h>=0 {release(3);return h;}return -1;}
        main :: () {}
    "#,
        Some(jaic::target::TargetOs::Linux),
    );
    let a = Analysis::build(&c);
    assert!(a.opening(id(&c, "distinct")).is_some());
    assert!(
        a.opening(id(&c, "equal")).is_none(),
        "guard equality makes close(3) release the acquired fd"
    );
}

#[test]
fn scalar_handles_global_frames_are_conditional_and_respect_acquisition_order() {
    let c = compiler_target(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,m:*u8)->*void #foreign C "fopen";
        release :: (h:*void)->s32 #foreign C "fclose";
        all :: ()->s32 #foreign C "fcloseall";
        callback :: () #foreign C "unknown_global_effect";
        helper :: (){callback();}
        close_helper :: (){all();}
        after :: (p:*u8)->*void{h:=acquire(p,p);helper();return h;}
        before :: (p:*u8)->*void{helper();return acquire(p,p);}
        hidden_closed :: (p:*u8)->*void{h:=acquire(p,p);close_helper();return h;}
        independent :: (p:*u8)->*void{a:=acquire(p,p);b:=acquire(p,p);release(a);return b;}
        main :: () {}
    "#,
        Some(jaic::target::TargetOs::Linux),
    );
    let a = Analysis::build(&c);
    let after = a.opening(id(&c, "after")).unwrap();
    assert!(after.incomplete);
    assert!(
        after
            .alternatives
            .iter()
            .all(|c| !c.requires_noninterference.is_empty())
    );
    let before = a.opening(id(&c, "before")).unwrap();
    assert!(!before.incomplete);
    assert!(
        before
            .alternatives
            .iter()
            .all(|c| c.requires_noninterference.is_empty())
    );
    assert!(a.opening(id(&c, "hidden_closed")).is_none());
    let independent = a.opening(id(&c, "independent")).unwrap();
    assert!(!independent.incomplete);
}

#[test]
fn scalar_handles_closed_or_framed_old_value_can_alias_a_reused_fd() {
    let c = compiler_target(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,f:s32)->s32 #foreign C "open";
        release :: (h:s32)->s32 #foreign C "close";
        callback :: () #foreign C "unknown_global_effect";
        reused :: (p:*u8)->s32{old:=acquire(p,0);release(old);next:=acquire(p,0);release(old);return next;}
        framed :: (p:*u8)->s32{old:=acquire(p,0);callback();next:=acquire(p,0);release(old);return next;}
        main :: () {}
    "#,
        Some(jaic::target::TargetOs::Linux),
    );
    let a = Analysis::build(&c);
    for name in ["reused", "framed"] {
        assert!(
            a.opening(id(&c, name)).is_none(),
            "{name}: site identities cannot disprove API-value reuse"
        );
    }
}

#[test]
fn scalar_handles_windows_protocol_requires_selected_windows_target() {
    let source = r#"
        W :: #system_library "kernel32";
        acquire :: (p:*u16,a:u32,s:u32,x:*void,d:u32,f:u32,t:*void)->*void #foreign W "CreateFileW";
        windows_release_contract_target :: (h:*void)->s32 #foreign W "CloseHandle";
        wrapper :: (p:*u16)->*void {return acquire(p,0,0,null,0,0,null);}
        probe_windows_release_contract_target :: (h:*void)->s32 {return windows_release_contract_target(h);}
        main :: () {h:=wrapper(null);result:=probe_windows_release_contract_target(h);}
    "#;
    for os in [jaic::target::TargetOs::Linux, jaic::target::TargetOs::MacOS] {
        let c = compiler_target(source, Some(os));
        assert!(
            Analysis::build(&c).opening(id(&c, "wrapper")).is_none(),
            "wrong selected target {os:?}"
        );
        assert!(
            native::native(&c, id(&c, "windows_release_contract_target")).is_none(),
            "wrong-target release contract {os:?}"
        );
    }
    let c = compiler(source, true);
    assert!(Analysis::build(&c).opening(id(&c, "wrapper")).is_some());
    assert!(native::native(&c, id(&c, "windows_release_contract_target")).is_some());
    let mut unsupported = compiler(source, true);
    unsupported.options.cpu = jaic::target::TargetCpu::Wasm;
    assert!(
        Analysis::build(&unsupported)
            .opening(id(&unsupported, "wrapper"))
            .is_none()
    );
    assert!(
        native::native(
            &unsupported,
            id(&unsupported, "windows_release_contract_target")
        )
        .is_none()
    );
}

#[test]
fn scalar_handles_narrow_signed_constants_do_not_prune_real_release() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        release :: (h:*void)->s32 #foreign C "fclose";
        negative :: (p:*u8)->*void {h:=acquire(p,p);x:s8=-128;if x<0 release(h);return h;}
        narrowed :: (p:*u8)->*void {h:=acquire(p,p);x:s8=cast,trunc(s8)128;if x<0 release(h);return h;}
        narrowed32 :: (p:*u8)->*void {h:=acquire(p,p);x:s32=cast,trunc(s32)0xffffffff;if x<0 release(h);return h;}
        tracked_rhs :: (p:*u8)->*void {h:=acquire(p,p);x:s32=-1;y:s32=0;if y>x release(h);return h;}
        unsigned32 :: (p:*u8)->*void {h:=acquire(p,p);x:u32=0xffffffff;if x>0x80000000 release(h);return h;}
        safe_unsigned :: (p:*u8)->*void {h:=acquire(p,p);x:u32=0xffffffff;if x<0x80000000 release(h);return h;}
        safe :: (p:*u8)->*void {h:=acquire(p,p);x:s8=127;if x<0 release(h);return h;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    for name in [
        "negative",
        "narrowed",
        "narrowed32",
        "unsigned32",
        "tracked_rhs",
    ] {
        assert!(
            a.opening(id(&c, name)).is_none(),
            "typed comparison must retain the real close branch: {name}"
        );
    }
    assert!(a.opening(id(&c, "safe")).is_some());
    assert!(a.opening(id(&c, "safe_unsigned")).is_some());
}

#[test]
fn scalar_handles_system_library_contracts_require_supported_target_matrix() {
    let source = r#"
        C :: #system_library "LIBRARY";
        scalar_matrix_open :: (p:*u8,flags:s32)->s32 #foreign C "open";
        scalar_matrix_close :: (h:s32)->s32 #foreign C "close";
        scalar_matrix_fopen :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        scalar_matrix_fclose :: (h:*void)->s32 #foreign C "fclose";
        descriptor :: (p:*u8)->s32{return scalar_matrix_open(p,0);}
        stream :: (p:*u8)->*void{return scalar_matrix_fopen(p,p);}
        release_fd :: (h:s32)->s32{return scalar_matrix_close(h);}
        release_stream :: (h:*void)->s32{return scalar_matrix_fclose(h);}
        main :: () {h:=descriptor(null);r:=release_fd(h);s:=stream(null);r=release_stream(s);}
    "#;
    for (os, library) in [
        (jaic::target::TargetOs::Windows, "c"),
        (jaic::target::TargetOs::Windows, "System"),
        (jaic::target::TargetOs::Linux, "System"),
    ] {
        let c = compiler_target(&source.replace("LIBRARY", library), Some(os));
        let a = Analysis::build(&c);
        assert!(
            a.opening(id(&c, "descriptor")).is_none(),
            "{os:?}/{library}"
        );
        assert!(a.opening(id(&c, "stream")).is_none(), "{os:?}/{library}");
        assert!(native::native(&c, id(&c, "scalar_matrix_close")).is_none());
        assert!(native::native(&c, id(&c, "scalar_matrix_fclose")).is_none());
    }
    for (os, library) in [
        (jaic::target::TargetOs::Linux, "c"),
        (jaic::target::TargetOs::MacOS, "System"),
    ] {
        let c = compiler_target(&source.replace("LIBRARY", library), Some(os));
        let a = Analysis::build(&c);
        assert!(a.opening(id(&c, "descriptor")).is_some());
        assert!(a.opening(id(&c, "stream")).is_some());
    }
}

#[test]
fn scalar_handles_derived_stack_addresses_do_not_establish_private_ownership() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        Larger :: struct {pad:s64;field:*void;}
        dynamic_alias :: (p:*u8)->*void {h:=acquire(p,p);dummy:u8;delta:=cast(s64)*h-cast(s64)*dummy;(cast(**void)(*dummy+delta)).*=null;return h;}
        outside_actual_root :: (p:*u8)->*void {h:=acquire(p,p);dummy:u8;(cast(*Larger)*dummy).field=null;return h;}
        ordinary_private :: (p:*u8)->*void {h:=acquire(p,p);dummy:Larger;dummy.field=null;return h;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    let unsupported: Vec<_> = ["dynamic_alias", "outside_actual_root"]
        .into_iter()
        .filter(|name| a.opening(id(&c, name)).is_some())
        .collect();
    assert!(
        unsupported.is_empty(),
        "derived/OOB slot ownership: {unsupported:?}"
    );
    assert!(a.opening(id(&c, "ordinary_private")).is_some());
}

#[test]
fn scalar_handles_unsupported_aggregate_copy_overwrites_old_capability() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        Holder :: struct {handle:*void;}
        bad :: (src:*Holder,p:*u8)->Holder{a:Holder;a.handle=acquire(p,p);a=src.*;return a;}
        good :: (p:*u8)->Holder{a:Holder;a.handle=acquire(p,p);b:=a;return b;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    assert!(
        a.opening(id(&c, "bad")).is_none(),
        "unsupported source must overwrite prior destination facts"
    );
    assert!(a.opening(id(&c, "good")).is_some());
}

#[test]
fn scalar_handles_persistent_exposure_clobbers_slots_not_captured_bits() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        remember :: (p:**void) #foreign;
        mutate_saved :: () #foreign;
        bad :: (p:*u8)->*void{h:=acquire(p,p);remember(*h);h=acquire(p,p);mutate_saved();return h;}
        captured :: (p:*u8)->*void{h:=acquire(p,p);remember(*h);h=acquire(p,p);saved:=h;mutate_saved();return saved;}
        main :: () {}
    "#,
        false,
    );
    let a = Analysis::build(&c);
    assert!(
        a.opening(id(&c, "bad")).is_none(),
        "escaped slot may change after a no-argument callback"
    );
    let captured = a
        .opening(id(&c, "captured"))
        .expect("captured SSA bits remain under native liveness frame");
    assert!(captured.incomplete);
    assert!(
        captured
            .alternatives
            .iter()
            .all(|x| !x.requires_noninterference.is_empty())
    );
}

#[test]
fn scalar_handles_capability_proof_budget_bounds_threshold_exit_products() {
    let mut source = String::from(
        r#"
        C :: #system_library "c";
        acquire :: (path:*u8,flags:s32)->s32 #foreign C "open";
        simple :: (path:*u8)->s32 {return acquire(path,0);}
        threshold :: (path:*u8)->s32,bool {h:=acquire(path,0);
    "#,
    );
    for threshold in 0..40 {
        source.push_str(&format!("if h < {threshold} return h,false;\n"));
    }
    source.push_str("return h,true;} main :: () {} ");
    let c = compiler(&source, false);
    let mut solver = Solver {
        c: &c,
        bodies: c
            .ed
            .facts()
            .unwrap()
            .lifetime_bodies
            .iter()
            .map(|(id, body)| (*id, body))
            .collect(),
        cache: HashMap::default(),
        active: HashSet::default(),
    };
    let relation = solver.relation(id(&c, "threshold")).unwrap();
    let nodes: usize = relation
        .exits
        .iter()
        .flat_map(|state| {
            state
                .conditions
                .iter()
                .map(|(expr, _)| expr)
                .chain(state.returns.iter().map(|(_, _, expr)| expr))
        })
        .map(|expr| expr.shape().unwrap().nodes)
        .sum();
    assert!(nodes < 4096, "fixture must reach the product gate: {nodes}");
    let token = relation
        .exits
        .iter()
        .flat_map(|state| &state.returns)
        .find_map(|(_, _, expr)| {
            if let Expr::Fresh(token) = expr {
                Some(token)
            } else {
                None
            }
        })
        .unwrap();
    assert!(guards::samples(&relation, token).len() < 4096);
    let analysis = Analysis::build(&c);
    assert!(analysis.opening(id(&c, "simple")).is_some());
    assert!(
        analysis.opening(id(&c, "threshold")).is_none(),
        "bounded CFG and expression shapes still need a total capability proof-work limit"
    );
}

#[test]
fn scalar_handles_requested_source_callers_preserve_all_matching_checked_calls() {
    let c = compiler(
        r#"
        C :: #system_library "c";
        acquire :: (path:*u8,mode:*u8)->*void #foreign C "fopen";
        factory :: (path:*u8)->*void {return acquire(path,path);}
        leaked :: (path:*u8) {h:=factory(path);if h!=null {}}
        returned :: (path:*u8)->*void {h:=factory(path);return h;}
        main :: () {}
    "#,
        false,
    );
    let (_, all) = Analysis::build_internal(&c, true, &HashSet::default());
    assert!(all.probed_bodies > 0 && !all.calls.is_empty());
    let caller = id(&c, "leaked");
    let file = c.pr.procs[caller.0 as usize].span.file;
    let files = [file].into_iter().collect();
    let (_, selected) = Analysis::build_internal(&c, true, &files);
    for (key, positions) in &all.calls {
        let body = &c.ed.facts().unwrap().lifetime_bodies[&key.0];
        if matches!(body.blocks[key.1].events[key.2],Event::Call{span,..} if span.file==file) {
            assert_eq!(
                selected.calls.get(key),
                Some(positions),
                "every checked instance of a requested source call must remain"
            );
        }
    }
    let none = [FileId(u32::MAX)].into_iter().collect();
    let (_, unrequested) = Analysis::build_internal(&c, true, &none);
    assert_eq!(
        unrequested.probed_bodies, 0,
        "unrequested caller CFGs must not be probed"
    );
    assert!(unrequested.calls.is_empty());
}

#[test]
fn scalar_handles_native_boundaries_reject_variadic_prototypes() {
    for variant in 0..3 {
        let (acquire_params, release_params) = match variant {
            0 => ("p:*u8,q:*u8", "p:*void"),
            1 => ("p:*u8,q:*u8,extras:..u8", "p:*void,extras:..u8"),
            _ => ("p:*u8,q:..*u8", "p:..*void"),
        };
        let source = format!(
            r#"C :: #system_library "c";
            probe_fixed_abi_acquire :: ({acquire_params})->*void #foreign C "fopen";
            probe_fixed_abi_release :: ({release_params})->s32 #foreign C "fclose";
            probe :: (p:*u8){{h:=probe_fixed_abi_acquire(p,p);probe_fixed_abi_release(h);}}
            main :: (){{}}"#
        );
        let c = compiler_target(&source, Some(jaic::target::TargetOs::Linux));
        for name in ["probe_fixed_abi_acquire", "probe_fixed_abi_release"] {
            let selected = &c.pr.procs[id(&c, name).0 as usize];
            assert_eq!(
                native(&c, id(&c, name)).is_some(),
                variant == 0,
                "{name}: {:?} {:?}",
                selected.target,
                selected.sig
            );
        }
    }
}

#[test]
fn scalar_handles_native_boundaries_reject_variadic_fixed_role_open() {
    for (params, args, expected) in [
        ("p:*u8,flags:s32", "p,0", true),
        ("p:*u8,flags:s32,mode:..u32", "p,0,8", true),
        ("p:*u8,flags:..s32", "p,0", false),
    ] {
        let source = format!(
            r#"C :: #system_library "c";
            probe_abi_descriptor :: ({params})->s32 #foreign C "open";
            probe :: (p:*u8)->s32{{return probe_abi_descriptor({args});}}
            main :: (){{}}"#
        );
        let c = compiler_target(&source, Some(jaic::target::TargetOs::Linux));
        assert_eq!(
            native(&c, id(&c, "probe_abi_descriptor")).is_some(),
            expected,
            "{params}"
        );
    }
}

#[test]
fn scalar_handles_native_boundaries_reject_variadic_apple_fixed_mode() {
    use jaic::target::{TargetCpu, TargetOs};
    for (os, cpu) in [
        (TargetOs::Linux, TargetCpu::X64),
        (TargetOs::Linux, TargetCpu::Arm64),
        (TargetOs::MacOS, TargetCpu::X64),
        (TargetOs::MacOS, TargetCpu::Arm64),
    ] {
        for variadic in [true, false] {
            let mode = if variadic {
                "..u32"
            } else {
                "u32"
            };
            let source = format!(
                r#"C :: #system_library "c";
                probe_abi_mode :: (p:*u8,flags:s32,mode:{mode})->s32 #foreign C "open";
                probe :: (p:*u8)->s32{{return probe_abi_mode(p,0,8);}}
                main :: (){{}}"#
            );
            let c = compiler_target_cpu(&source, Some(os), Some(cpu));
            let expected = variadic || (os, cpu) != (TargetOs::MacOS, TargetCpu::Arm64);
            assert_eq!(
                native(&c, id(&c, "probe_abi_mode")).is_some(),
                expected,
                "{os:?} {cpu:?} variadic={variadic}"
            );
        }
    }
}

#[test]
fn scalar_handles_test_selector_uses_fixture_module_for_colliding_names() {
    let c = compiler_target(
        r#"C :: #system_library "c";
        acquire :: (p:*u8,q:*u8)->*void #foreign C "fopen";
        release :: (p:*void)->s32 #foreign C "fclose";
        probe :: (p:*u8){h:=acquire(p,p);release(h);}
        main :: (){}"#,
        Some(jaic::target::TargetOs::Linux),
    );
    let chosen = id(&c, "acquire");
    let file = c.pr.procs[chosen.0 as usize].span.file;
    let module = c.src.files.iter().find(|f| f.id == file).unwrap().module;
    assert_eq!(Some(module), c.ns.main_module);
    assert!(native(&c, chosen).is_some());
}

#[test]
fn scalar_handles_unknown_original_extent_cannot_become_a_borrowed_view() {
    let mut c = compiler("main :: () {x:u8;}", false);
    let main = id(&c, "main");
    let body =
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .get_mut(&main)
            .unwrap();
    let root = body.stack_roots.first_mut().unwrap();
    root.bytes = None;
    let value = root.value;
    let facts = Rc::new(StackRoots::capture(body, MAX_STEPS).unwrap());
    let mut state = State {
        roots: Some(facts.clone()),
        ..State::default()
    };
    state.region(value, 128);
    assert!(state.original(value));
    assert_eq!(state.extent(value), None);
    assert!(!state.checked(
        Address {
            root: value,
            offset: 0
        },
        1
    ));
    assert!(state.borrowed.is_empty());
    assert!(
        State::default().roots.is_none(),
        "native/default states allocate no root object"
    );
    let fork = state.clone();
    assert!(Rc::ptr_eq(fork.roots.as_ref().unwrap(), &facts));
    let mut missing =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .get(&main)
            .unwrap()
            .clone();
    missing.procedure = ProcId(u32::MAX);
    assert!(!current_body(&c, ProcId(u32::MAX), &missing));
}

#[test]
fn cached_scalar_exits_do_not_retain_private_path_distinctions() {
    let c = compiler(
        r#"
        C::#system_library "c";
        acquire::(p:*u8,q:*u8)->*void #foreign C "fopen";
        flag:bool;
        probe_private_exit::(p:*u8)->*void {
            h:=acquire(p,p);
            if flag {local:*void=h;address:=*local;}
            return h;
        }
        main::(){}
    "#,
        false,
    );
    let mut solver = Solver {
        c: &c,
        bodies: c
            .ed
            .facts()
            .unwrap()
            .lifetime_bodies
            .iter()
            .filter(|(id, b)| current_body(&c, **id, b))
            .map(|(id, b)| (*id, b))
            .collect(),
        cache: HashMap::default(),
        active: HashSet::default(),
    };
    let relation = solver.relation(id(&c, "probe_private_exit")).unwrap();
    assert_eq!(
        relation.exits.len(),
        1,
        "private exposure is not an exported relation alternative"
    );
    assert_eq!(guards::capabilities(&relation).alternatives.len(), 1);
    assert!(std::mem::size_of_val(&relation.exits[0]) < std::mem::size_of::<State>());
    let roots = Rc::new(
        StackRoots::capture(solver.bodies[&id(&c, "probe_private_exit")], MAX_STEPS).unwrap(),
    );
    let state = State {
        roots: Some(roots.clone()),
        ..State::default()
    };
    assert_eq!(Rc::strong_count(&roots), 2);
    let projected = state.into_exit().unwrap();
    assert_eq!(
        Rc::strong_count(&roots),
        1,
        "cached exits must not retain root certificates"
    );
    drop(projected);
    let exhausted = State {
        liveness_exhausted: true,
        ..State::default()
    };
    assert!(exhausted.into_exit().is_none());
    eprintln!(
        "working_state={} exported_exit={} exits={}",
        std::mem::size_of::<State>(),
        std::mem::size_of::<Exit>(),
        relation.exits.len()
    );
}

#[test]
fn compact_exit_projection_does_not_merge_paths_before_existing_cutoffs() {
    let mut source = String::from(
        "C::#system_library \"c\";acquire::(p:*u8,q:*u8)->*void #foreign C \"fopen\";flag:bool;probe_cutoff::(p:*u8)->*void {h:=acquire(p,p);\n",
    );
    for n in 0..10 {
        use std::fmt::Write;
        writeln!(
            source,
            "if flag {{local_{n}:*void=h;address_{n}:=*local_{n};}}"
        )
        .unwrap();
    }
    source.push_str("return h;}main::(){}");
    let c = compiler(&source, false);
    assert!(
        Analysis::build(&c)
            .opening(id(&c, "probe_cutoff"))
            .is_none(),
        "working paths must still exhaust the original bound"
    );
}
