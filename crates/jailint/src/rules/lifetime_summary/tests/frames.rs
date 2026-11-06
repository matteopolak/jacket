//! Leaf frames are proved from checked operations and composed through calls.
use super::stdlib_allocations::checked_source;
use super::*;

#[test]
fn closed_scalar_helper_frames_cover_writes_without_name_or_note_seeds() {
    let mut c = checked_source(
        r"
        G:s64;
        opaque :: () #foreign;
        native_release :: (p:*s64) #foreign;
        leaf :: (p:*s64) {p.*=0;}
        wrapper :: (p:*s64) {leaf(p);}
        conditional :: (p:*s64, flag:bool) {if flag p.*=0;}
        partial :: (p:*s64) {(cast(*s32)p).*=0;}
        pair :: (left:*s64,right:*s64) {left.*=0;right.*=0;}
        named :: (p:*s64,q:*s64) {pair(right=q,left=p);}
        bad_global :: (p:*s64) {G=1;p.*=0;}
        bad_read :: (p:*s64) {p.*=G;}
        bad_callback :: (p:*s64) {opaque();p.*=0;}
        bad_release :: (p:*s64) {native_release(p);p.*=0;}
        Bigger :: struct {first:s64;second:s64;}
        bad_extent :: (p:*s64) {(cast(*Bigger)p).second=0;}
        recursive :: (p:*s64,flag:bool) {if flag recursive(p,false);p.*=0;}
        unused_leaf :: (p:*s64) {p.*=0;}
        driver :: (a:*[..]s64,p:*s64,q:*s64,flag:bool)->*s64 {
            leaf(p);wrapper(p);conditional(p,flag);partial(p);named(p,q);
            bad_global(p);bad_read(p);bad_callback(p);bad_release(p);bad_extent(p);recursive(p,flag);
            return a.data;
        }
        main :: (){}
    ",
    );
    let id = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    let leaf = id("leaf");
    let wrapper = id("wrapper");
    let all = Summaries::build(&c);
    let slot = super::super::physical_slots::Slot {
        formal: 0,
        offset: 0,
        width: 8,
    };
    for name in ["leaf", "wrapper"] {
        let frame = all.procs[&id(name)]
            .frame
            .as_ref()
            .expect("checked full-write leaf");
        assert!(frame.current(&c));
        assert_eq!(frame.typed().unwrap().writes, vec![slot]);
        assert_eq!(frame.typed().unwrap().must, vec![slot]);
    }
    let conditional = all.procs[&id("conditional")].frame.as_ref().unwrap();
    assert_eq!(conditional.typed().unwrap().writes, vec![slot]);
    assert!(conditional.typed().unwrap().must.is_empty());
    let partial = all.procs[&id("partial")].frame.as_ref().unwrap();
    assert_eq!(
        partial.typed().unwrap().writes,
        vec![super::super::physical_slots::Slot {
            width: 4,
            ..slot
        }]
    );
    assert!(!partial.typed().unwrap().must.contains(&slot));
    let named = all.procs[&id("named")].frame.as_ref().unwrap();
    assert_eq!(named.typed().unwrap().must.len(), 2);
    assert!(named.typed().unwrap().must.contains(&slot));
    assert!(
        named
            .typed()
            .unwrap()
            .must
            .contains(&super::super::physical_slots::Slot {
                formal: 1,
                ..slot
            })
    );
    for name in [
        "bad_global",
        "bad_read",
        "bad_callback",
        "bad_release",
        "bad_extent",
        "recursive",
        "unused_leaf",
    ] {
        assert!(all.procs[&id(name)].frame.is_none(), "{name}");
    }
    let before: HashMap<_, _> = all
        .procs
        .iter()
        .filter_map(|(id, s)| s.frame.clone().map(|frame| (*id, frame)))
        .collect();
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(
        before,
        after
            .procs
            .iter()
            .filter_map(|(id, s)| s.frame.clone().map(|frame| (*id, frame)))
            .collect()
    );
    let retained = before[&leaf].clone();
    let target = c.pr.procs[leaf.0 as usize].target;
    c.pr.procs[leaf.0 as usize].target = c.pr.procs[wrapper.0 as usize].target;
    assert!(!retained.current(&c));
    c.pr.procs[leaf.0 as usize].target = target;
    assert!(retained.current(&c));
    c.pr.procs[leaf.0 as usize].body_state = BodyState::NotNeeded;
    assert!(!retained.current(&c));
    assert!(
        !before[&wrapper].current(&c),
        "a composed frame must reject its stale leaf"
    );
}


#[test]
fn actual_element_initializer_has_a_closed_full_write_after_ir_release() {
    let mut c = checked_source(
        r#"
        #import "Basic";
        probe :: (a:*[..]s64)->*s64 {return array_add(a);}
        main :: (){}
    "#,
    );
    let facts = c.ed.facts().unwrap();
    let probe = facts
        .lifetime_bodies
        .values()
        .find(|b| c.proc(b.procedure).name.as_str() == "probe")
        .unwrap();
    let carrier = probe
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Call {
                callee,
                expands: false,
                ..
            } => Some(*callee),
            _ => None,
        })
        .unwrap();
    let helpers: Vec<_> = facts.lifetime_bodies[&carrier]
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| match e {
            Event::Call {
                callee,
                expands: false,
                ..
            } if c.proc(*callee).sig.as_ref().is_some_and(|s| {
                s.returns.is_empty()
                    && s.params.len() == 1
                    && c.ty.types.pointee(s.params[0].ty) == Some(TypeId::S64)
            }) =>
            {
                Some(*callee)
            }
            _ => None,
        })
        .collect();
    assert_eq!(helpers.len(), 1, "actual checked initialization call");
    let all = Summaries::build(&c);
    let helper = helpers[0];
    let frame = all.procs[&helper]
        .frame
        .as_ref()
        .expect("actual selected helper's closed frame")
        .clone();
    assert_eq!(
        frame.typed().unwrap().writes,
        vec![super::super::physical_slots::Slot {
            formal: 0,
            offset: 0,
            width: 8
        }]
    );
    assert_eq!(frame.typed().unwrap().must, frame.typed().unwrap().writes);
    c.release_lowered_code();
    assert_eq!(
        Summaries::build(&c).procs[&helper].frame.as_ref(),
        Some(&frame)
    );
}


#[test]
fn closed_frames_reject_large_empty_cfgs_and_exhausted_address_queries() {
    let mut c = checked_source(
        r"leaf :: (p:*s64) {p.*=0;} driver :: (a:*[..]s64,p:*s64)->*s64 {leaf(p);return a.data;} main :: (){}",
    );
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "leaf")
            .unwrap()
            .clone();
    let all = Summaries::build(&c);
    assert!(all.procs[&body.procedure].frame.is_some());
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(body.procedure)
            .unwrap(),
        &all,
    );
    let resolver = Resolver::for_prepared(&prepared, &all);
    let incoming = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Parameter {
                incoming, ..
            } => Some(*incoming),
            _ => None,
        })
        .unwrap();
    assert!(resolver.address_object(incoming).is_some());
    let mut query_work = 8192;
    let index = resolver.capture_index(&mut query_work).unwrap();
    assert!(index.address(&resolver, incoming, &mut 0).is_none());
    assert_eq!(
        index.address(&resolver, incoming, &mut query_work),
        resolver.address_object(incoming)
    );
    drop(resolver);
    let mut excessive = body.clone();
    excessive
        .blocks
        .extend(std::iter::repeat_n(excessive.blocks[0].clone(), 129));
    for block in excessive.blocks.iter_mut().skip(1) {
        block.events.clear();
        block.successors.clear();
        block.normal_return = false;
    }
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .insert(body.procedure, excessive);
    assert!(Summaries::build(&c).procs[&body.procedure].frame.is_none());
}

#[test]
fn closed_frame_cancellation_before_publication_discards_read_only_result() {
    if !super::cancellation_child() {
        return;
    }
    let c = checked_source(
        r"noop :: (p:*s64) {} driver :: (a:*[..]s64,p:*s64)->*s64 {noop(p);return a.data;} main :: (){}",
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "noop")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let live = super::super::flow::returning_blocks(body, &all.procs, &all.targets);
    let mut frame = super::super::frames::Builder::new(&r, true, false);
    for (i, b) in body.blocks.iter().enumerate() {
        for (ordinal, e) in b.events.iter().enumerate() {
            frame.observe(&r, i, ordinal, e, &all.procs);
        }
    }
    assert!(frame.active());
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            jaic::interrupt::disarm();
            jaic::interrupt::request(false);
        }
    }
    let _reset = Reset;
    jaic::interrupt::arm();
    jaic::interrupt::request(true);
    assert!(frame.finish(&r, &live, &all.procs).is_none());
    assert!(jaic::interrupt::disarm());
}

#[test]
fn actual_record_initializer_has_a_closed_full_byte_write() {
    let mut c = checked_source(
        r#"#import "Basic"; Item::struct {value:s64;} probe::(a:*[..]Item)->*Item{return array_add(a);} main::(){}"#,
    );
    let all = Summaries::build(&c);
    let facts = c.ed.facts().unwrap();
    let probe = facts
        .lifetime_bodies
        .values()
        .find(|b| c.proc(b.procedure).name.as_str() == "probe")
        .unwrap();
    let carrier = probe
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Call {
                callee,
                expands: false,
                ..
            } => Some(*callee),
            _ => None,
        })
        .unwrap();
    let element =
        c.ty.types
            .pointee(c.proc(carrier).sig.as_ref().unwrap().returns[0])
            .unwrap();
    let helper = facts.lifetime_bodies[&carrier]
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Call {
                callee,
                expands: false,
                ..
            } if c.proc(*callee).sig.as_ref().is_some_and(|s| {
                s.params.len() == 1
                    && s.returns.is_empty()
                    && c.ty.types.pointee(s.params[0].ty) == Some(element)
            }) =>
            {
                Some(*callee)
            }
            _ => None,
        })
        .unwrap();
    let frame = all.procs[&helper]
        .frame
        .as_ref()
        .expect("actual checked record initializer");
    assert_eq!(
        frame.typed().unwrap().must,
        vec![super::super::physical_slots::Slot {
            formal: 0,
            offset: 0,
            width: c.ty.types.size_of(element)
        }]
    );
    let retained = frame.clone();
    let source = facts.lifetime_bodies[&helper].read_only_regions.clone();
    assert!(!source.is_empty());
    for variant in 0..3 {
        let mut changed = source.clone();
        match variant {
            0 => changed[0].bytes -= 1,
            1 => changed[0].ty = TypeId::S64,
            _ => {
                let mut conflict = changed[0].clone();
                conflict.bytes += 1;
                changed.push(conflict);
            }
        }
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .get_mut(&helper)
            .unwrap()
            .read_only_regions = changed;
        assert!(
            Summaries::build(&c).procs[&helper].frame.is_none(),
            "source variant={variant}"
        );
    }
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&helper)
        .unwrap()
        .read_only_regions = source;
    c.release_lowered_code();
    assert_eq!(
        Summaries::build(&c).procs[&helper].frame.as_deref(),
        Some(retained.as_ref())
    );
}

#[test]
fn record_frames_reject_unsupported_sources_callbacks_and_pointer_publication() {
    let c = checked_source(
        r#"#import "Basic";
        Item::struct {value:s64;}
        Pair::struct {left:s64;right:s64;}
        Carrier::struct {pointer:*Item;}
        G:Item;
        opaque::() #foreign;
        full::(p:*Item){p.*=Item.{};}
        conditional::(p:*Item,flag:bool){if flag p.*=Item.{};}
        partial::(p:*Pair){p.left=0;}
        mutable_global::(p:*Item){p.*=G;}
        borrowed::(p:*Item,q:*Item){p.*=q.*;}
        undersized::(p:*Pair){tiny:u8;p.*=(cast(*Pair)*tiny).*;}
        callback::(p:*Item){opaque();p.*=Item.{};}
        publication::(p:*Carrier,value:*Item){tmp:Carrier;tmp.pointer=value;p.*=tmp;}
        private_source::(p:*Item){tmp:Item;p.*=tmp;}
        driver::(a:*[..]Item,p:*Item,q:*Pair,dst:*Carrier,flag:bool)->*Item {
            full(p);conditional(p,flag);partial(q);mutable_global(p);borrowed(p,p);
            undersized(q);callback(p);publication(dst,p);private_source(p);return a.data;
        }
        main::(){}
    "#,
    );
    let id = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    let all = Summaries::build(&c);
    assert!(
        !all.procs[&id("full")]
            .frame
            .as_ref()
            .unwrap()
            .typed()
            .unwrap()
            .must
            .is_empty()
    );
    let conditional = all.procs[&id("conditional")].frame.as_ref().unwrap();
    assert!(
        !conditional.typed().unwrap().writes.is_empty()
            && conditional.typed().unwrap().must.is_empty()
    );
    for name in [
        "partial",
        "mutable_global",
        "borrowed",
        "undersized",
        "callback",
        "publication",
    ] {
        assert!(all.procs[&id(name)].frame.is_none(), "{name}");
    }
    assert!(
        !all.procs[&id("private_source")]
            .frame
            .as_ref()
            .unwrap()
            .typed()
            .unwrap()
            .must
            .is_empty()
    );
}

#[test]
fn pointer_prefix_helpers_require_conditional_signed_footprints() {
    let c = checked_source(
        r"
        opaque::() #foreign;
        G:*void;
        prefix_store::(ptr:*void,region:*void){(cast(**void)(cast(*u8)ptr-8)).*=region;}
        renamed_store::(target:*void,value:*void){(cast(**void)(cast(*u8)target-8)).*=value;}
        captured_store::(ptr:*void,region:*void){saved:=ptr;ptr=null;(cast(**void)(cast(*u8)saved-8)).*=region;}
        reread_store::(ptr:*void,region:*void){ptr=null;(cast(**void)(cast(*u8)ptr-8)).*=region;}
        extra_global::(ptr:*void,region:*void){(cast(**void)(cast(*u8)ptr-8)).*=region;G=region;}
        extra_callback::(ptr:*void,region:*void){(cast(**void)(cast(*u8)ptr-8)).*=region;opaque();}
        unknown_offset::(ptr:*void,region:*void,index:s64){(cast(**void)(cast(*u8)ptr+index)).*=region;}
        prefix_read::(ptr:*void){saved:=(cast(**void)(cast(*u8)ptr-8)).*;}
        bad_append::(a:*[..]s64)->*s64 {slot:=a.data+a.count;prefix_store(cast(*void)slot,null);a.count+=1;return slot;}
        driver::(a:*[..]s64,p:*void,q:*void,index:s64)->*s64 {
            prefix_store(p,q);renamed_store(p,q);captured_store(p,q);reread_store(p,q);
            extra_global(p,q);extra_callback(p,q);unknown_offset(p,q,index);prefix_read(p);return a.data;
        }
        main::(){}
    ",
    );
    let all = Summaries::build(&c);
    let id = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    for name in [
        "prefix_store",
        "renamed_store",
        "captured_store",
        "prefix_read",
    ] {
        let frame = all.procs[&id(name)].frame.as_ref().expect(name);
        assert!(
            frame.typed().is_none(),
            "raw obligation must not be a typed frame"
        );
        let raw = frame.raw().expect("conditional footprint");
        if name == "prefix_read" {
            assert!(!raw.reads.is_empty());
            assert!(
                raw.reads
                    .iter()
                    .all(|a| a.pointer.formal == 0 && a.pointer.displacement == -8 && a.width == 8)
            );
        } else {
            assert_eq!(raw.writes.len(), 1);
            let write = &raw.writes[0];
            assert_eq!(write.access.pointer.formal, 0);
            assert_eq!(write.access.pointer.displacement, -8);
            assert_eq!(write.access.width, 8);
            assert_eq!(write.value.formal, 1);
            assert_eq!(write.value.displacement, 0);
        }
    }
    for name in [
        "reread_store",
        "extra_global",
        "extra_callback",
        "unknown_offset",
    ] {
        assert!(all.procs[&id(name)].frame.is_none(), "{name}");
    }
    assert!(all.procs[&id("bad_append")].no_growth.is_none());
}

#[test]
fn actual_allocator_prefix_store_has_a_conditional_footprint() {
    let mut c = checked_source(
        r#"D::#import "Default_Allocator";
        driver::(a:*[..]s64)->*s64{memory:=D.allocator_proc(.ALLOCATE,64,0,null,null);return a.data;}
        main::(){}"#,
    );
    let all = Summaries::build(&c);
    let helpers: Vec<_> = c
        .ed
        .facts()
        .unwrap()
        .lifetime_bodies
        .values()
        .filter(|b| {
            let p = c.proc(b.procedure);
            p.name.as_str() == "set_region"
                && c.src
                    .sources
                    .get(p.span.file)
                    .path
                    .as_file()
                    .is_some_and(|path| path.ends_with("internal/allocation/aligned-blocks.jai"))
        })
        .collect();
    assert!(
        !helpers.is_empty(),
        "actual selected allocator prefix helper"
    );
    let retained: Vec<_> = helpers
        .into_iter()
        .map(|helper| {
            let frame = all.procs[&helper.procedure]
                .frame
                .as_ref()
                .expect("actual full prefix write");
            assert!(frame.typed().is_none());
            let raw = frame.raw().unwrap();
            assert_eq!(raw.writes.len(), 1);
            assert_eq!(raw.writes[0].access.pointer.displacement, -8);
            assert_eq!(raw.writes[0].access.width, 8);
            (helper.procedure, frame.clone())
        })
        .collect();
    c.release_lowered_code();
    let released = Summaries::build(&c);
    for (procedure, frame) in retained {
        assert_eq!(
            released.procs[&procedure].frame.as_deref(),
            Some(frame.as_ref())
        );
    }
}

#[test]
fn conditional_raw_frames_bound_signature_and_reject_uncovered_sources() {
    let params = (0..128)
        .map(|i| format!("p{i}:*void"))
        .collect::<Vec<_>>()
        .join(",");
    let c = checked_source(&format!(
        r"
        Record::struct {{a:s64;b:s64;}}
        huge::({params}) {{}}
        copy_record::(p:*void,source:*Record) {{local:=source.*;}}
        too_far::(p:*void,q:*void) {{(cast(**void)p-0x7fffffffffffffff).* = q;}}
        driver::(a:*[..]s64,p:*void,q:*void,s:*Record)->*s64 {{copy_record(p,s);too_far(p,q);return a.data;}}
        main::(){{}}
    "
    ));
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "huge")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    for (requested, recursive) in [(false, false), (true, true), (true, false)] {
        let frame = super::super::frames::Builder::new(&r, requested, recursive);
        assert!(!frame.active());
        assert_eq!(
            frame.spent(),
            0,
            "inactive/oversized signature has no frame traversal work"
        );
    }
    for name in ["copy_record", "too_far"] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        assert!(all.procs[&id].frame.is_none(), "{name}");
    }
}

#[test]
fn captured_pointer_leaf_is_neutral_complete_and_depth_bounded() {
    let mut c = checked_source(
        r"
        Record::struct {a:s64;b:s64;}
        aggregate::(record:Record)->Record {return record;}
        equal::(p:*void)->*void {copy:=p;return copy;}
        mixed::(p:*void,q:*void,flag:bool)->*void {copy:=p;if flag copy=q;return copy;}
        numeric::(p:*void)->*void {return cast(*void)cast(u64)p;}
        prefix::(p:*void,q:*void) {(cast(**void)(cast(*u8)p-8)).*=q;}
        driver::(a:*[..]s64,p:*void,q:*void)->*s64 {prefix(p,q);return a.data;}
        main::(){}
    ",
    );
    fn leaf(r: &Resolver<'_>, value: Val, work: &mut usize, depth: usize) -> Option<(Val, usize)> {
        let index = r.capture_index(work)?;
        index.pointer_leaf(r, value, work, depth)
    }
    let all = Summaries::build(&c);
    fn body(c: &Compiler, name: &str) -> LifetimeBody {
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == name)
            .unwrap()
            .clone()
    }
    for (name, valid) in [("equal", true), ("mixed", false), ("numeric", false)] {
        let body = body(&c, name);
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body(body.procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        let value = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .find_map(|e| match e {
                Event::Return {
                    value, ..
                } => Some(*value),
                _ => None,
            })
            .unwrap();
        assert_eq!(leaf(&r, value, &mut 8192, 0).is_some(), valid, "{name}");
        assert!(leaf(&r, value, &mut 0, 0).is_none());
        assert!(leaf(&r, value, &mut 8192, 24).is_none());
        assert_eq!(
            leaf(&r, Val(u32::MAX), &mut 8192, 0),
            Some((Val(u32::MAX), 0)),
            "opaque SSA is not an input certificate"
        );
    }
    let original = body(&c, "prefix");
    let equal = body(&c, "equal");
    let driver = body(&c, "driver");
    let mut wide = equal.clone();
    let mut call = driver
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find(|e| matches!(e, Event::Call { .. }))
        .unwrap()
        .clone();
    let pointer = c.pr.procs[equal.procedure.0 as usize]
        .sig
        .as_ref()
        .unwrap()
        .params[0]
        .ty;
    if let Event::Call {
        results, ..
    } = &mut call
    {
        *results = (0..2048)
            .map(|i| (i, Val(70000 + i as u32), pointer))
            .collect();
    }
    wide.blocks[0].events.push(call);
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .insert(wide.procedure, wide.clone());
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(wide.procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    assert!(
        leaf(&r, Val(u32::MAX), &mut 64, 0).is_none(),
        "result-vector enumeration must exhaust before scanning"
    );
    drop(r);
    drop(prepared);
    let aggregate = body(&c, "aggregate");
    let selected = aggregate
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Load {
                value,
                ty,
                ..
            } if c.ty.types.as_struct(*ty).is_some() => Some(*value),
            _ => None,
        })
        .expect("actual aggregate load");
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(aggregate.procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    assert!(
        leaf(&r, selected, &mut 8192, 0).is_none(),
        "aggregate load is not pointer-value equality"
    );
    drop(r);
    drop(prepared);
    let returned = equal
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Return {
                value, ..
            } => Some(*value),
            _ => None,
        })
        .unwrap();
    let incoming = equal.blocks[0]
        .events
        .iter()
        .find_map(|e| match e {
            Event::Parameter {
                formal: 0,
                incoming,
                ..
            } => Some(*incoming),
            _ => None,
        })
        .unwrap();
    for front in [true, false] {
        let mut conflict = equal.clone();
        let event = Event::Offset {
            value: returned,
            source: incoming,
            stride: TypeId::U8,
            offset: Some(1),
            subscript: None,
            span: c.proc(equal.procedure).span,
        };
        if front {
            conflict.blocks[0].events.insert(0, event);
        } else {
            conflict.blocks[0].events.push(event);
        }
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .insert(conflict.procedure, conflict.clone());
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body(conflict.procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        assert!(
            leaf(&r, returned, &mut 8192, 0).is_none(),
            "conflicting actual producers reject in either source order"
        );
    }
    let input = original.blocks[0]
        .events
        .iter()
        .find_map(|e| match e {
            Event::Parameter {
                formal: 0,
                incoming,
                ..
            } => Some(*incoming),
            _ => None,
        })
        .unwrap();
    let span = c.proc(original.procedure).span;
    for (segments, expected) in [(4, true), (14, false)] {
        let mut changed = original.clone();
        let store = changed.blocks[0]
            .events
            .iter()
            .rposition(|e| matches!(e, Event::Store { .. }))
            .unwrap();
        let mut chain = Vec::new();
        let mut value = input;
        for n in 0..segments {
            let offset = Val(60000 + n * 2);
            let copied = Val(60001 + n * 2);
            chain.push(Event::Offset {
                value: offset,
                source: value,
                stride: TypeId::U8,
                offset: Some(-1),
                subscript: None,
                span,
            });
            chain.push(Event::Copy {
                value: copied,
                source: offset,
            });
            value = copied;
        }
        if let Event::Store {
            place, ..
        } = &mut changed.blocks[0].events[store]
        {
            *place = value;
        }
        changed.blocks[0].events.splice(store..store, chain);
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .insert(original.procedure, changed);
        assert_eq!(
            Summaries::build(&c).procs[&original.procedure]
                .frame
                .is_some(),
            expected,
            "alternating capture/offset depth: {segments}"
        );
    }
}

#[test]
fn closed_frames_preserve_independently_trusted_resource_contracts() {
    let c = super::stdlib_allocations::checked_source_with_contracts(
        r"
        Header::struct {count:s64;cap:s64;}
        opaque::() #foreign;
        contract_free::(p:*s64) {p.*=0;} @FreesArgument
        contract_wrapper::(p:*s64) {contract_free(p);}
        contract_guarded::(p:*Header) {if p.count<p.cap return;opaque();} @FreesArgument
        driver::(a:*[..]s64,p:*s64,h:*Header)->*s64 {contract_free(p);contract_wrapper(p);contract_guarded(h);return a.data;}
        main::(){}
    ",
        Options::host(),
    );
    let all = Summaries::build(&c);
    let mut frames = Vec::new();
    for name in ["contract_free", "contract_wrapper", "contract_guarded"] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        assert!(
            all.procs[&id]
                .guaranteed
                .keys()
                .any(|e| e.kind == Kind::Free),
            "trusted contract must be present: {name}"
        );
        frames.push((name, all.procs[&id].frame.is_some()));
    }
    assert!(
        frames.iter().all(|(_, present)| !*present),
        "memory frame cannot erase independent resource semantics: {frames:?}"
    );
}


#[test]
fn trusted_frame_constraints_reject_stale_transitive_proofs_and_overflow() {
    use super::super::contracts::Trusted;
    let c = checked_source(
        r"
        leaf::(p:*s64){p.*=0;}
        middle::(p:*s64){leaf(p);}
        top::(p:*s64){middle(p);}
        driver::(a:*[..]s64,p:*s64)->*s64{top(p);return a.data;}
        main::(){}
    ",
    );
    let mut all = Summaries::build(&c);
    let id = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    let frame = all.procs[&id("top")]
        .frame
        .clone()
        .expect("positive transitive frame");
    assert!(frame.current(&c) && frame.compatible(&all.procs));
    let free = Effect {
        target: Path {
            root: Root::Parameter(0),
            steps: vec![],
        },
        kind: Kind::Free,
        guard: None,
    };
    all.procs.get_mut(&id("leaf")).unwrap().trusted =
        Trusted::effects(std::iter::once(&free)).map(Box::new);
    assert!(
        frame.current(&c),
        "only constraints changed, not body identity"
    );
    assert!(!frame.compatible(&all.procs));
    all.procs.remove(&id("leaf"));
    assert!(!frame.compatible(&all.procs), "missing is not known-empty");
    let resize = Effect {
        target: Path {
            root: Root::Parameter(0),
            steps: vec![Step::Storage],
        },
        kind: Kind::Resize,
        guard: None,
    };
    let own = Trusted::effects(std::iter::once(&resize)).unwrap();
    assert!(own.covered_by_append(0));
    assert!(!own.covered_by_append(1));
    let mut guarded = resize.clone();
    guarded.guard = Some((1, 1));
    assert!(
        !Trusted::effects(std::iter::once(&guarded))
            .unwrap()
            .covered_by_append(0)
    );
    assert!(
        !Trusted::effects([&resize, &free].into_iter())
            .unwrap()
            .covered_by_append(0)
    );
    let overflow = vec![free.clone(); 33];
    assert_eq!(Trusted::effects(overflow.iter()), Some(Trusted::Opaque));
    let mut long = free;
    long.target.steps = vec![Step::Deref; 9];
    assert_eq!(
        Trusted::effects(std::iter::once(&long)),
        Some(Trusted::Opaque)
    );
}

#[test]
fn actual_append_contract_covers_only_its_selected_receiver_resize() {
    use super::super::contracts::Trusted;
    let c = super::stdlib_allocations::checked_source_with_contracts(
        r#"#import "Basic"; probe::(a:*[..]s64)->*s64{return array_add(a);} main::(){}"#,
        Options::host(),
    );
    let mut all = Summaries::build(&c);
    let (&id, summary) = all
        .procs
        .iter()
        .find(|(id, s)| c.proc(**id).name.as_str() == "array_add" && s.no_growth.is_some())
        .expect("actual contracted array_add witness");
    let witness = summary.no_growth.clone().unwrap();
    assert!(
        summary.trusted.is_some(),
        "actual GrowsArray is an independent seed"
    );
    assert!(witness.compatible(&all.procs));
    let extra = Effect {
        target: Path {
            root: Root::Parameter(1),
            steps: vec![],
        },
        kind: Kind::ResetTemporary,
        guard: None,
    };
    all.procs.get_mut(&id).unwrap().trusted =
        Trusted::effects(std::iter::once(&extra)).map(Box::new);
    assert!(!witness.compatible(&all.procs));
}


#[test]
fn warning_mode_checked_cast_cannot_be_a_closed_write_frame() {
    let mut options = Options::host();
    options.cast_bounds_check = 1;
    let c = super::stdlib_allocations::checked_source_options(
        r"
        checked_write_probe::(p:*u64,value:s64){p.*=cast(u64)value;}
        driver::(a:*[..]u64,p:*u64,value:s64)->*u64{checked_write_probe(p,value);return a.data;}
        main::(){}
    ",
        options,
    );
    let all = Summaries::build(&c);
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "checked_write_probe")
            .unwrap() as u32,
    );
    assert!(
        all.procs[&id].frame.is_none(),
        "a failed warning-mode cast may execute the runtime report handler"
    );
}


#[test]
fn overflow_reporting_callbacks_cannot_form_a_closed_void_frame() {
    for mode in [0, 1, 2] {
        let mut options = Options::host();
        options.arithmetic_overflow_check = mode;
        let c = super::stdlib_allocations::checked_source_options(
            r"
            leaf::(memory:*u64,a:u64,b:u64){memory.*=a+b;}
            driver::(header:*[..]u64,a:u64,b:u64)->*u64{leaf(header.data,a,b);return header.data;}
            main::(){}
        ",
            options,
        );
        let all = Summaries::build(&c);
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == "leaf")
                .unwrap() as u32,
        );
        assert_eq!(
            all.procs[&id].frame.is_some(),
            mode == 0,
            "actual hidden callback mode{mode}"
        );
    }
}

#[test]
fn checked_static_enum_members_do_not_describe_runtime_memory_reads() {
    let mut c = checked_source(
        r"
        Mode::enum { ONE; TWO; }
        select_value::(p:*s64,m:Mode){if m==.ONE p.*=1;}
        driver::(a:*[..]s64,p:*s64,m:Mode)->*s64{select_value(p,m);return a.data;}
        main::(){}
    ",
    );
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "select_value")
            .unwrap() as u32,
    );
    let before = Summaries::build(&c).procs[&id]
        .frame
        .clone()
        .expect("static enum lookup has no runtime access");
    assert_eq!(before.typed().unwrap().writes.len(), 1);
    c.release_lowered_code();
    assert_eq!(
        Summaries::build(&c).procs[&id].frame.as_ref(),
        Some(&before)
    );
    let body =
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .get_mut(&id)
            .unwrap();
    let field = body
        .blocks
        .iter_mut()
        .flat_map(|b| &mut b.events)
        .find(|e| {
            matches!(
                e,
                Event::Field {
                    base: LifetimeOperand::Other,
                    result: LifetimeOperand::Other,
                    base_ty: TypeId::TYPE,
                    ..
                }
            )
        })
        .unwrap();
    let Event::Field {
        base_ty, ..
    } = field
    else {
        unreachable!()
    };
    *base_ty = TypeId::VOID_PTR;
    assert!(
        Summaries::build(&c).procs[&id].frame.is_none(),
        "unknown member access is not a static lookup"
    );
}
