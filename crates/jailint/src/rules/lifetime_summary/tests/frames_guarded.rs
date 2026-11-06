//! Entry guards are conditional effects, never helper-name contracts.
use super::stdlib_allocations::checked_source;
use super::*;

#[test]
fn guarded_read_only_effects_require_exact_entry_reads_and_all_selected_paths() {
    let mut c = checked_source(
        r"
        H::struct {count:s64;cap:s64;}
        opaque::() #foreign;
        quiet::(h:*H) {if h.count<h.cap return;opaque();}
        swapped::(h:*H) {if h.cap>h.count return;opaque();}
        inverted::(h:*H) {if h.count>=h.cap opaque();}
        mutation::(h:*H) {h.count=1;if h.count<h.cap return;opaque();}
        callback::(h:*H) {opaque();if h.count<h.cap return;opaque();}
        alternate::(h:*H,flag:bool) {if flag h.count=1;if h.count<h.cap return;opaque();}
        unsigned_guard::(h:*H) {if cast(u64)h.count<cast(u64)h.cap return;opaque();}
        cycle::(h:*H,flag:bool) {if h.count<h.cap {while flag {}} else opaque();}
        driver::(a:*[..]s64,h:*H,flag:bool)->*s64 {
            quiet(h);swapped(h);inverted(h);mutation(h);callback(h);alternate(h,flag);unsigned_guard(h);cycle(h,flag);return a.data;
        }
        main::(){}
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
    let all = Summaries::build(&c);
    for name in ["quiet", "swapped", "inverted"] {
        let frame = all.procs[&id(name)].frame.as_ref().expect(name);
        let guard = frame.condition.as_ref().expect("conditional result");
        assert!(frame.current(&c));
        assert!(frame.typed().unwrap().writes.is_empty() && frame.typed().unwrap().must.is_empty());
        let values = |count, cap| {
            move |slot: super::super::physical_slots::Slot| match (
                slot.formal,
                slot.offset,
                slot.width,
            ) {
                (0, 0, 8) => Some(count),
                (0, 8, 8) => Some(cap),
                _ => None,
            }
        };
        assert_eq!(guard.evaluate(values(0, 4)), Some(true), "{name}");
        assert_eq!(guard.evaluate(values(4, 4)), Some(false), "{name}");
        assert_eq!(guard.evaluate(values(5, 4)), Some(false), "{name}");
        assert_eq!(guard.evaluate(|_| None), None);
    }
    for name in [
        "mutation",
        "callback",
        "alternate",
        "unsigned_guard",
        "cycle",
    ] {
        assert!(all.procs[&id(name)].frame.is_none(), "{name}");
    }
    let before: HashMap<_, _> = all
        .procs
        .iter()
        .filter_map(|(id, s)| s.frame.clone().map(|f| (*id, f)))
        .collect();
    c.release_lowered_code();
    assert_eq!(
        before,
        Summaries::build(&c)
            .procs
            .iter()
            .filter_map(|(id, s)| s.frame.clone().map(|f| (*id, f)))
            .collect()
    );
}

#[test]
fn actual_growth_guard_proves_only_the_selected_spare_capacity_path() {
    let mut c = checked_source(
        r#"#import "Basic"; probe::(a:*[..]s64)->*s64 {return array_add(a);} main::(){}"#,
    );
    let facts = c.ed.facts().unwrap();
    let probe = facts
        .lifetime_bodies
        .values()
        .find(|b| c.proc(b.procedure).name.as_str() == "probe")
        .unwrap();
    let call = |body: &LifetimeBody| {
        body.blocks
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
            .unwrap()
    };
    let carrier = call(probe);
    let growth = call(&facts.lifetime_bodies[&carrier]);
    let all = Summaries::build(&c);
    let body = &facts.lifetime_bodies[&growth];
    let frame = all.procs[&growth].frame.as_ref().unwrap_or_else(|| {
        panic!(
            "actual guard: {} blocks {} events",
            body.blocks.len(),
            body.blocks.iter().map(|b| b.events.len()).sum::<usize>()
        )
    });
    let guard = frame.condition.as_ref().expect("actual conditional effect");
    assert!(frame.typed().unwrap().writes.is_empty());
    let values = |count, cap| {
        move |slot: super::super::physical_slots::Slot| match (slot.formal, slot.offset, slot.width)
        {
            (0, jaic::layout::view::COUNT, 8) => Some(count),
            (0, jaic::layout::resizable::ALLOCATED, 8) => Some(cap),
            _ => None,
        }
    };
    assert_eq!(guard.evaluate(values(0, 4)), Some(true));
    assert_eq!(guard.evaluate(values(4, 4)), Some(false));
    assert_eq!(guard.evaluate(values(5, 4)), Some(false));
    let retained = frame.clone();
    c.release_lowered_code();
    assert_eq!(
        Summaries::build(&c).procs[&growth].frame.as_ref(),
        Some(&retained)
    );
    c.pr.procs[growth.0 as usize].body_state = BodyState::NotNeeded;
    assert!(!retained.current(&c));
}


#[test]
fn guarded_frames_reject_incomplete_and_inconsistent_control_metadata() {
    let mut c = checked_source(
        r"H::struct {count:s64;cap:s64;} opaque::() #foreign; quiet::(h:*H) {if h.count<h.cap return;opaque();} driver::(a:*[..]s64,h:*H)->*s64 {quiet(h);return a.data;} main::(){}",
    );
    let all = Summaries::build(&c);
    let original =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "quiet")
            .unwrap()
            .clone();
    let guard = all.procs[&original.procedure]
        .frame
        .as_ref()
        .unwrap()
        .condition
        .as_ref()
        .unwrap()
        .clone();
    let key = original.procedure;
    fn prove(
        c: &mut Compiler,
        key: ProcId,
        body: &LifetimeBody,
        all: &Summaries,
        work: usize,
    ) -> Option<super::super::frames::Frame> {
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .insert(key, body.clone());
        let checked = crate::rules::lifetime_facts::CheckedSession::new(c).body(key)?;
        let prepared = Prepared::for_body(checked, all);
        let r = Resolver::for_prepared(&prepared, all);
        let mut prior = super::super::frames::Builder::new(&r, true, false);
        if work != 0 {
            prior.resume_work(work);
        }
        super::super::frames::read_only::infer(&r, &all.procs, &prior)
    }
    assert!(prove(&mut c, key, &original, &all, 0).is_some());
    assert!(
        prove(&mut c, key, &original, &all, 8192).is_none(),
        "a guarded fallback cannot reset an exhausted body budget"
    );
    let mut changed = original.clone();
    changed.blocks.clear();
    assert!(prove(&mut c, key, &changed, &all, 0).is_none());
    let mut changed = original.clone();
    changed.blocks[0].successors.clear();
    assert!(prove(&mut c, key, &changed, &all, 0).is_none());
    let mut changed = original.clone();
    let Event::Branch {
        then_block, ..
    } = &mut changed.blocks[guard.branch.block].events[guard.branch.event]
    else {
        panic!()
    };
    *then_block = usize::MAX;
    assert!(prove(&mut c, key, &changed, &all, 0).is_none());
    let mut changed = original.clone();
    for block in &mut changed.blocks {
        block.normal_return = false;
    }
    assert!(prove(&mut c, key, &changed, &all, 0).is_none());
    let mut changed = original.clone();
    changed.comparisons[guard.comparison].ty = jaic::ir::Ty::I32;
    assert!(prove(&mut c, key, &changed, &all, 0).is_none());
    let mut changed = original.clone();
    let Event::Load {
        ty, ..
    } = &mut changed.blocks[guard.left.site.block].events[guard.left.site.event]
    else {
        panic!()
    };
    *ty = TypeId::S32;
    assert!(prove(&mut c, key, &changed, &all, 0).is_none());
}


#[test]
fn checked_append_body_composes_guarded_growth_and_full_element_write() {
    let c = checked_source(
        r#"#import "Basic"; probe::(a:*[..]s64)->*s64 {return array_add(a);} main::(){}"#,
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
    let all = Summaries::build(&c);
    let borrow = all.procs[&carrier]
        .no_growth
        .as_ref()
        .expect("checked no-growth append relation");
    assert!(borrow.current(&c));
    assert!(borrow.guards(0, 4));
    assert!(borrow.guards(3, 4));
    assert!(!borrow.guards(4, 4));
    assert!(!borrow.guards(5, 4));
    assert!(
        borrow.offset.event < borrow.initialize.event
            && borrow.initialize.event < borrow.count_write.event
    );
}

#[test]
fn append_private_default_write_uses_frozen_byte_width() {
    let mut c = checked_source(
        r#"#import "Basic";
        write_leaf::(p:*s64){p.*=0;}
        initialize_slot::(p:*s64){write_leaf(p);}
        append_checked::(a:*[..]s64)->*s64 {
            scratch:*s64;scratch=null;
            slot:=a.data+a.count;initialize_slot(slot);a.count+=1;return slot;
        }
        main::(){a:[..]s64;append_checked(*a);}
    "#,
    );
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "append_checked")
            .unwrap() as u32,
    );
    let initial = Summaries::build(&c);
    assert!(
        initial.procs[&id].no_growth.is_some(),
        "matched private write positive"
    );
    let leaf = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "write_leaf")
            .unwrap() as u32,
    );
    let state = c.pr.procs[leaf.0 as usize].body_state;
    c.pr.procs[leaf.0 as usize].body_state = BodyState::NotNeeded;
    assert!(
        !initial.procs[&id].no_growth.as_ref().unwrap().current(&c),
        "stale initializer grandchild"
    );
    c.pr.procs[leaf.0 as usize].body_state = state;
    let body =
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .get_mut(&id)
            .unwrap();
    let image = body
        .blocks
        .iter_mut()
        .flat_map(|b| &mut b.events)
        .find(|e| matches!(e, Event::DefaultImage { .. }))
        .unwrap();
    let Event::DefaultImage {
        bytes, ..
    } = image
    else {
        unreachable!()
    };
    *bytes = 64;
    assert!(
        Summaries::build(&c).procs[&id].no_growth.is_none(),
        "cast/current type cannot hide an oversized selected write"
    );
}
