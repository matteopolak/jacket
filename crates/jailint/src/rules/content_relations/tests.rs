use super::*;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn compiler(source: &str) -> Compiler {
    let stdlib = jacket_config::env::STDLIB.os().map_or_else(
        || std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib"),
        std::path::PathBuf::from,
    );
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
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

fn id(c: &Compiler, name: &str) -> ProcId {
    ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == name && p.lit.body.is_some() && !p.is_poly)
            .unwrap() as u32,
    )
}

#[test]
fn content_relations_real_copy_carriers_retain_read_epoch() {
    let mut c = compiler(
        r#"#import "Basic";main :: () {a:=copy_string("a");b:=copy_temporary_string("b");}"#,
    );
    let a = Analysis::build(&c);
    for name in ["copy_string", "copy_temporary_string"] {
        let copy = a
            .returned_copy(id(&c, name))
            .unwrap_or_else(|| panic!("nonempty real carrier {name}"));
        assert!(!copy.incomplete);
        assert!(!copy.alternatives.is_empty());
        for relation in &copy.alternatives {
            assert_eq!(relation.source_formal, 0);
            assert_eq!(relation.result, 0);
            assert_eq!(relation.length, Length::InputStringCount(0));
            assert_eq!(relation.read_epoch, relation.write_epoch);
            assert!(
                relation
                    .preconditions
                    .separations
                    .iter()
                    .any(|s| s.store.is_some()
                        && matches!(s.left.base, RangeBase::DescriptorHeader { .. })),
                "return-header stores require an actual destination separation frame"
            );
        }
    }
    c.release_lowered_code();
    assert_eq!(a.copies, Analysis::build(&c).copies);
}

#[test]
fn content_relations_copy_snapshot_and_overlap() {
    let c = compiler(
        r"
        copied :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);return .{s.count,dst};}
        changed_source :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);s.data[0]=0;return .{s.count,dst};}
        moved :: (s:string)->string{dst:=s.data+1;memcpy(dst,s.data,s.count);return .{s.count,dst};}
        source_prefix_write :: (s:string)->string{dst:=s.data+1;memcpy(dst,s.data,s.count);s.data[0]=0;return .{s.count,dst};}
        overlapping_write :: (s:string)->string{dst:=s.data+1;memcpy(dst,s.data,s.count);s.data[1]=0;return .{s.count,dst};}
        empty :: (s:string)->string{memcpy(null,s.data,0);return .{0,cast(*u8)null};}
        main :: () {}
    ",
    );
    let a = Analysis::build(&c);
    for name in [
        "copied",
        "changed_source",
        "moved",
        "source_prefix_write",
        "empty",
    ] {
        assert!(a.returned_copy(id(&c, name)).is_some(), "{name}");
    }
    assert!(a.returned_copy(id(&c, "overlapping_write")).is_none());
    let changed = &a
        .returned_copy(id(&c, "changed_source"))
        .unwrap()
        .alternatives[0];
    assert!(
        changed
            .preconditions
            .separations
            .iter()
            .any(|s| s.store.is_some() && matches!(s.left.base, RangeBase::InputStringData(_))),
        "post-copy source mutation requires exact store/destination separation"
    );
    let moved = &a.returned_copy(id(&c, "moved")).unwrap().alternatives[0];
    assert!(
        !moved
            .preconditions
            .separations
            .iter()
            .any(|s| s.store.is_none() && matches!(s.left.base, RangeBase::InputStringData(_))),
        "selected intrinsic supports overlap"
    );
}

#[test]
fn content_relations_partial_overwrite_borrowed_and_unknown_are_not_exact() {
    let c = compiler(
        r"
        borrowed :: (s:string)->string{return s;}
        partial :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count-1);return .{s.count,dst};}
        overwritten :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);dst[0]=0;return .{s.count,dst};}
        external :: () #foreign;
        unknown :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);external();return .{s.count,dst};}
        conditional :: (s:string,dst:*u8,flag:bool)->string{if flag return s;memcpy(dst,s.data,s.count);return .{s.count,dst};}
        fake_copy :: (dst:*void,src:*void,n:s64){}
        fake :: (s:string,dst:*u8)->string{fake_copy(dst,s.data,s.count);return .{s.count,dst};}
        alien_write :: (s:string,dst:*u8,other:*u8)->string{memcpy(dst,s.data,s.count);other[0]=0;return .{s.count,dst};}
        negative_later_count :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);memcpy(dst+1,s.data,-1);return .{s.count,dst};}
        main :: () {}
    ",
    );
    let a = Analysis::build(&c);
    for name in [
        "borrowed",
        "partial",
        "overwritten",
        "unknown",
        "conditional",
        "fake",
        "alien_write",
        "negative_later_count",
    ] {
        assert!(a.returned_copy(id(&c, name)).is_none(), "{name}");
    }
}

#[test]
fn content_relations_formal_mapping_snapshot_alias_and_callback_epoch() {
    let c = compiler(
        r#"
        #import "Basic";
        external :: () #foreign;
        named :: (s:string,dst:*u8)->string{memcpy(count=s.count,source=s.data,dest=dst);return .{s.count,dst};}
        before :: (s:string,dst:*u8)->string{external();memcpy(dst,s.data,s.count);return .{s.count,dst};}
        forward :: (s:string)->string{return copy_string(s);}
        alias_write :: (s:string,dst:*u8)->string{alias:=dst;memcpy(dst,s.data,s.count);alias[0]=0;return .{s.count,dst};}
        returned_alias :: (p:*u8)->*u8 #foreign;
        exposed_private_write :: (s:string)->string{byte:u8;dst:=returned_alias(*byte);memcpy(dst,s.data,s.count);byte=0;return .{s.count,dst};}
        unexposed_private_write :: (s:string,dst:*u8)->string{byte:u8;memcpy(dst,s.data,s.count);byte=0;return .{s.count,dst};}
        changed_header :: (s:string,dst:*u8)->string{local:=s;n:=local.count;memcpy(dst,local.data,n);local.count=0;return .{n,dst};}
        header_overlap :: (s:string)->string{result:string;result.count=s.count;result.data=cast(*u8)*result;memcpy(result.data,s.data,s.count);return result;}
        main :: () {}
    "#,
    );
    let a = Analysis::build(&c);
    for name in [
        "named",
        "before",
        "forward",
        "changed_header",
        "unexposed_private_write",
    ] {
        assert!(a.returned_copy(id(&c, name)).is_some(), "{name}");
    }
    for name in ["alias_write", "header_overlap", "exposed_private_write"] {
        assert!(a.returned_copy(id(&c, name)).is_none(), "{name}");
    }
    let forward = &a.returned_copy(id(&c, "forward")).unwrap().alternatives[0];
    assert!(forward.read_epoch.chain.len() > 1);
    let before = &a.returned_copy(id(&c, "before")).unwrap().alternatives[0];
    let site = *before.read_epoch.chain.last().unwrap();
    let body = &c.ed.facts().unwrap().lifetime_bodies[&site.procedure];
    assert!(body.blocks[site.block].events[..site.event].iter().any(|e|matches!(e,jaic::ide::lifetimes::LifetimeEvent::Call{callee,..} if c.pr.procs[callee.0 as usize].name.as_str()=="external")),"copy read epoch follows callback");
}

#[test]
fn content_relations_predicate_budget_keeps_unknown_exit_incomplete() {
    let mut source = String::from(
        "external :: () #foreign;bounded :: (s:string,dst:*u8)->string {memcpy(dst,s.data,s.count);ok:=false;",
    );
    for _ in 0..256 {
        source.push_str("ok=!ok;\n");
    }
    source.push_str("if ok external();return .{s.count,dst};} main :: () {}");
    let c = compiler(&source);
    assert!(
        Analysis::build(&c)
            .returned_copy(id(&c, "bounded"))
            .is_none()
    );
}

#[test]
fn content_relations_cancellation_discards_partial_relations() {
    const CHILD: jacket_config::env::Var = jacket_config::env::LIFETIME_CANCELLATION_CHILD;
    if !CHILD.is_set() {
        let name = std::thread::current().name().unwrap().to_owned();
        assert!(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &name, "--test-threads=1"])
                .env(CHILD.name, "1")
                .status()
                .unwrap()
                .success()
        );
        return;
    }
    let c = compiler(
        "copied :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);return .{s.count,dst};} main :: () {}",
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
    assert!(Analysis::build(&c).copies.is_empty());
    assert!(jaic::interrupt::disarm());
    let layout = domain::Layout::of(&c).unwrap();
    let mut solver = checked::Solver::new(&c, layout);
    jaic::interrupt::arm();
    jaic::interrupt::stop_after(2);
    assert!(solver.summary(id(&c, "copied")).is_none());
    assert!(jaic::interrupt::fired());
}

#[test]
fn content_relations_exposed_storage_and_wrapped_offsets_are_unknown() {
    let c = compiler(
        r"
        returned_alias :: (p:*u8)->*u8 #foreign;
        Pair :: struct {first:s64;second:s64;}
        negative_slot_offset :: (s:string)->string{byte:u8;dst:=returned_alias(*byte);memcpy(dst,s.data,s.count);(cast(*u8)*byte-1)[0]=0;return .{s.count,dst};}
        descendant :: (s:string)->string{value:Pair;dst:=returned_alias(cast(*u8)*value);memcpy(dst,s.data,s.count);value.second=0;return .{s.count,dst};}
        whole_object :: (s:string)->string{value:Pair;dst:=returned_alias(cast(*u8)*value);memcpy(dst,s.data,s.count);value=.{0,0};return .{s.count,dst};}
        partial_object :: (s:string)->string{value:u64;dst:=returned_alias(cast(*u8)*value);memcpy(dst,s.data,s.count);(cast(*u8)*value)[0]=0;return .{s.count,dst};}
        wrapped_offset :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);p:=cast(*u8)(cast(*u64)dst-0x2000000000000000);p[0]=0;return .{s.count,dst};}
        far_ranges :: (s:string,dst:*u8)->string{q:=dst+0x7fffffffffffffff;memcpy(q,s.data,s.count);p:=dst-0x7fffffffffffffff;p=p-1;p[0]=0;return .{s.count,q};}
        composed_wrap :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);p:=dst-0x7fffffffffffffff;p=p-0x7fffffffffffffff;p=p-2;p[0]=0;return .{s.count,dst};}
        main :: () {}
    ",
    );
    let a = Analysis::build(&c);
    let unsupported: Vec<_> = [
        "negative_slot_offset",
        "descendant",
        "whole_object",
        "partial_object",
        "wrapped_offset",
        "composed_wrap",
        "far_ranges",
    ]
    .into_iter()
    .filter(|name| a.returned_copy(id(&c, name)).is_some())
    .collect();
    assert!(
        unsupported.is_empty(),
        "unsupported alias/wrapped-offset copies: {unsupported:?}"
    );
}

#[test]
fn content_relations_derived_stack_initialization_does_not_hide_alias_write() {
    let c = compiler(
        r"
        allocate :: ()->*u8 #foreign;
        Box :: struct {byte:u8;}
        dynamic_alias :: (s:string)->string{dst:=allocate();dummy:Box;delta:=cast(s64)dst-cast(s64)*dummy;memcpy(dst,s.data,s.count);(cast(*Box)(cast(*u8)*dummy+delta)).*=.{};return .{s.count,dst};}
        ordinary_private :: (s:string,dst:*u8)->string{dummy:Box;memcpy(dst,s.data,s.count);dummy=.{};return .{s.count,dst};}
        main :: () {}
    ",
    );
    let a = Analysis::build(&c);
    assert!(
        a.returned_copy(id(&c, "dynamic_alias")).is_none(),
        "derived Initialize is not an owned stack root"
    );
    assert!(a.returned_copy(id(&c, "ordinary_private")).is_some());
}

#[test]
fn content_relations_reject_mismatched_retained_body_key() {
    let mut c = compiler(
        r"good :: (s:string,dst:*u8)->string{memcpy(dst,s.data,s.count);return .{s.count,dst};} other :: (){} main :: (){}",
    );
    let good = id(&c, "good");
    let other = id(&c, "other");
    assert!(Analysis::build(&c).returned_copy(good).is_some());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&good)
        .unwrap()
        .procedure = other;
    assert!(Analysis::build(&c).returned_copy(good).is_none());
}

#[test]
fn content_relations_unclassified_masks_do_not_prove_noninterference() {
    let c = compiler(
        r"Fiber_Context :: struct {sp:*void;stack_low:*void;stack_high:*void;stack_dealloc:*void;}
        fiber_free :: (fiber:*Fiber_Context) #intrinsic;
        plain_copy_frame :: (s:string,dst:*u8)->string {memcpy(dst,s.data,s.count);return .{s.count,dst};}
        resource_frame :: (s:string,dst:*u8,ctx:*Fiber_Context)->string {memcpy(dst,s.data,s.count);fiber_free(ctx);return .{s.count,dst};}
        zero_mask_frame :: (s:string,dst:*u8)->string {memcpy(dst,s.data,s.count);memcmp(dst,dst,0);return .{s.count,dst};}
        zero_write_frame :: (s:string,dst:*u8)->string {memcpy(dst,s.data,s.count);memset(dest=null,value=0,count=0);return .{s.count,dst};}
        main :: (){}",
    );
    let analysis = Analysis::build(&c);
    assert!(analysis.returned_copy(id(&c, "plain_copy_frame")).is_some());
    assert!(analysis.returned_copy(id(&c, "zero_write_frame")).is_some());
    // This checks eligibility of an unclassified contract, not that native
    // fiber_free deallocates caller-provided stack bytes (it currently does not).
    assert!(analysis.returned_copy(id(&c, "resource_frame")).is_none());
    assert!(analysis.returned_copy(id(&c, "zero_mask_frame")).is_none());
}
