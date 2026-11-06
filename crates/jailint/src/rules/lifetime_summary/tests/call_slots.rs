//! Actual call prestates share the pointer storage pass's epochs and clobbers.
use super::stdlib_allocations::checked_without_contracts;
use super::*;

#[test]
fn descriptor_call_slots_capture_typed_zero_constants_and_alias_clobbers() {
    let (c, summaries) = checked_without_contracts(
        r#"
        #import "Basic";
        observer :: (a:*[..]s64, marker:s64=0)->*s64 {return a.data;}
        opaque :: (p:*void) #foreign;
        mutate_count :: (a:*[..]s64)->s64 {a.count=3;return 0;}
        Alias :: struct {tag:s64; #overlay(tag) other:s64;}
        before :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;observer(*a);a.count=3;}
        after :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;a.count=3;observer(*a);}
        replaced :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;a.count=3;old:=a.count;a.count=1;observer(*a);}
        negative :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;a.count=-1;observer(*a);}
        ambiguous :: (flag:bool) {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;if flag a.count=1;observer(*a);}
        partial :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;(cast(*u32)*a).*=1;observer(*a);}
        whole :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;a=.{};observer(*a);}
        overlay :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;(cast(*Alias)*a).other=2;observer(*a);}
        unknown :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;opaque(*a);observer(*a);}
        late_argument :: () {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;observer(*a,mutate_count(*a));}
        wrong_extent :: () {tiny:u8;observer(cast(*[..]s64)*tiny);}
        borrowed_header :: (a:*[..]s64) {observer(a);}
        real_carrier :: () -> *s64 {back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.allocated=4;return array_add(*a);}
        main :: () {}
    "#,
    );
    let states = |name: &str| {
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|body| c.proc(body.procedure).name.as_str() == name)
                .unwrap();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &summaries,
        );
        let resolver = Resolver::for_prepared(&prepared, &summaries);

        resolver
            .call_slots
            .values()
            .map(|snapshot| {
                assert_eq!(snapshot.request.formal, 0);
                assert_eq!(snapshot.request.element, TypeId::S64);
                assert!(matches!(
                    c.ty.types.kind(snapshot.request.ty),
                    TypeKind::Array {
                        kind: ArrayKind::Resizable,
                        ..
                    }
                ));
                assert!(resolver.owned.contains(&snapshot.request.header));
                (snapshot.bounds(), snapshot.data().is_some())
            })
            .collect::<Vec<_>>()
    };
    for (name, want) in [
        ("before", (0, 4)),
        ("real_carrier", (0, 4)),
        ("after", (3, 4)),
        ("replaced", (1, 4)),
        ("negative", (-1, 4)),
    ] {
        assert_eq!(states(name), vec![(Some(want), true)], "{name}");
    }
    for name in [
        "ambiguous",
        "partial",
        "whole",
        "overlay",
        "unknown",
        "late_argument",
    ] {
        let snapshots = states(name);
        assert_eq!(snapshots.len(), 1, "{name}");
        assert_eq!(snapshots[0].0, None, "{name}");
    }
    for name in ["wrong_extent", "borrowed_header"] {
        assert!(states(name).is_empty(), "{name}");
    }
}

#[test]
fn private_slot_exposure_applies_to_all_typed_views_of_its_actual_root() {
    let (c, summaries) = checked_without_contracts(
        r"opaque :: (p:*void) #foreign; escaped_view :: (p:*void) {item:*void=p;opaque(cast(*void)*item);(cast(*s64)*item).*=0;} main :: (){}",
    );
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|body| c.proc(body.procedure).name.as_str() == "escaped_view")
            .unwrap();
    let address = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|event| match event {
            Event::Address {
                place,
                ty,
                ..
            } if c.ty.types.is_pointer(*ty) => Some(*place),
            _ => None,
        })
        .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &summaries,
    );
    let resolver = Resolver::for_prepared(&prepared, &summaries);
    let view = resolver.places.existing(address, TypeId::S64).unwrap();
    assert!(
        resolver.owned.contains(&resolver.places.root_of(view)),
        "actual original slot exists"
    );
    assert!(
        !resolver.immutable_private_slot(view),
        "escaping one address view exposes that same allocation through every typed view"
    );
}

#[test]
fn summary_registry_rejects_mismatched_recorded_procedure() {
    let (mut c, summaries) = checked_without_contracts(
        r"body_key_probe_identity :: (p:*void)->*void{return p;} body_key_probe_other :: (p:*void)->*void{return null;} main :: (){}",
    );
    let chosen = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    let identity = chosen("body_key_probe_identity");
    let other = chosen("body_key_probe_other");
    assert!(
        matches!(summaries.procs[&identity].returns.get(&0),Some((Path {root:Root::Parameter(0),steps},None)) if steps.is_empty())
    );
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&identity)
        .unwrap()
        .procedure = other;
    let after = Summaries::build(&c);
    assert!(
        !after.procs[&identity].returns.contains_key(&0),
        "a valid syntax Arc cannot authenticate facts recorded for another procedure"
    );
}

#[test]
fn repeated_expanded_append_calls_keep_distinct_checked_event_identities() {
    let (c, all) = checked_without_contracts(
        r#"#import "Basic";
    append_one::(a:*[..]s64)->*s64 #expand {return array_add(a);}
    probe::(){back:[4]s64;a:[..]s64;a.data=cast(*s64)*back;a.count=2;a.allocated=4;
        first:=array_add(*a);second:=append_one(*a);third:=append_one(*a);print("%\n",first.*);}
    main::(){}"#,
    );
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "probe")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let calls = super::super::no_growth::calls::Calls::collect(&r, &all);
    let mut spans = HashMap::default();
    for block in &body.blocks {
        for event in &block.events {
            if let Event::Call {
                span,
                expands: false,
                ..
            } = event
            {
                *spans.entry(*span).or_insert(0usize) += 1;
            }
        }
    }
    assert!(
        spans.values().any(|n| *n == 2),
        "the actual macro body must supply the shared-span pair"
    );
    assert!(r.call_slots.keys().any(|(s, _)| calls.get(*s).is_some()));
}

#[test]
fn append_content_aliases_have_an_actual_call_prestate() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/lint/pointer_lifetime/resources/stack-array-add-contents.jai"),
    )
    .unwrap();
    let (c, all) = checked_without_contracts(&source);
    for name in ["overwritten", "captured", "unrelated"] {
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        let calls = super::super::no_growth::calls::Calls::collect(&r, &all);
        assert!(
            r.call_slots
                .keys()
                .any(|(site, _)| calls.get(*site).is_some()),
            "{name}"
        );
    }
}

#[test]
fn actual_stack_append_requires_original_bounds_and_header_separation() {
    let (c, all) = checked_without_contracts(
        r#"#import "Basic";
    good::()->*s64{b:[4]s64;a:[..]s64;a.data=cast(*s64)*b;a.allocated=4;return array_add(*a);}
    last::()->*s64{b:[4]s64;a:[..]s64;a.data=cast(*s64)*b;a.count=3;a.allocated=4;return array_add(*a);}
    full::()->*s64{b:[4]s64;a:[..]s64;a.data=cast(*s64)*b;a.count=4;a.allocated=4;return array_add(*a);}
    negative::()->*s64{b:[4]s64;a:[..]s64;a.data=cast(*s64)*b;a.count=-1;a.allocated=4;return array_add(*a);}
    oversized::()->*s64{b:[4]s64;a:[..]s64;a.data=cast(*s64)*b;a.allocated=5;return array_add(*a);}
    overflow::()->*s64{b:[4]s64;a:[..]s64;a.data=cast(*s64)*b;a.count=0x7ffffffffffffffe;a.allocated=0x7fffffffffffffff;return array_add(*a);}
    self_overlap::()->*s64{a:[..]s64;a.data=cast(*s64)*a;a.allocated=4;return array_add(*a);}
    partial::()->*s64{b:[4]s64;a:[..]s64;a.data=cast(*s64)*b;a.allocated=4;(cast(*u32)*a).*=1;return array_add(*a);}
    main::(){}"#,
    );
    for (name, want) in [
        ("good", true),
        ("last", true),
        ("full", false),
        ("negative", false),
        ("oversized", false),
        ("overflow", false),
        ("self_overlap", false),
        ("partial", false),
    ] {
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        let calls = super::super::no_growth::calls::Calls::collect(&r, &all);
        assert_eq!(
            r.call_slots
                .keys()
                .any(|(site, _)| calls.get(*site).is_some()),
            want,
            "{name}"
        );
    }
}

#[test]
fn native_backed_append_requires_actual_success_extent_and_live_epoch() {
    let (mut c, all) = checked_without_contracts(
        r#"#import "Basic";
        libc::#system_library "libc";
        allocation::(bytes:u64)->*void #foreign libc "malloc";
        release::(memory:*void) #foreign libc "free";
        opaque::() #foreign;
        Wide::struct #align 64 {value:s64;}
        good::() {
            memory:=allocation(64); if !memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        too_short::() {
            memory:=allocation(8); if !memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        released::() {
            memory:=allocation(64); if !memory return;
            release(memory);
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        callback::() {
            memory:=allocation(64); if !memory return;
            opaque();
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        unguarded::() {
            memory:=allocation(64);
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        captured_old::() {
            memory:=allocation(64);if !memory return;saved:=memory;memory=null;
            values:[..]s64;values.data=cast(*s64)saved;values.allocated=8;
            slot:=array_add(*values);
        }
        wrong_acquisition::() {
            old:=allocation(64);replacement:=allocation(64);if !old return;
            values:[..]s64;values.data=cast(*s64)replacement;values.allocated=8;
            slot:=array_add(*values);
        }
        null_reaches::() {
            memory:=allocation(64);if memory {tag:=1;}
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        failure_only::() {
            memory:=allocation(64);if memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        wrong_guard::(other:*void) {
            memory:=allocation(64);if !other return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);
        }
        after_callback::() {
            memory:=allocation(64);if !memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);opaque();
        }
        after_release::() {
            memory:=allocation(64);if !memory return;
            values:[..]s64;values.data=cast(*s64)memory;values.allocated=8;
            slot:=array_add(*values);release(memory);
        }
        over_aligned::() {
            memory:=allocation(128);if !memory return;
            values:[..]Wide;values.data=cast(*Wide)memory;values.allocated=2;
            slot:=array_add(*values);
        }
        main::(){}
    "#,
    );
    for (name, expected) in [
        ("good", true),
        ("too_short", false),
        ("released", false),
        ("callback", false),
        ("unguarded", false),
        ("captured_old", true),
        ("wrong_acquisition", false),
        ("null_reaches", false),
        ("failure_only", false),
        ("wrong_guard", false),
        ("after_callback", true),
        ("after_release", true),
        ("over_aligned", false),
    ] {
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        assert!(
            !r.call_slots.is_empty(),
            "actual descriptor snapshot: {name}"
        );
        if name == "over_aligned" {
            assert!(r.call_slots.keys().any(|(site,_)|matches!(&body.blocks[site.block].events[site.event],Event::Call {callee,..} if all.procs[callee].no_growth.is_some())),"alignment control requires a supported callee witness");
        }
        let calls = super::super::no_growth::calls::Calls::collect(&r, &all);
        assert!(calls.work <= 8192);
        for (site, _) in r.call_slots.keys() {
            if let Some(call) = calls.get(*site) {
                let native = call
                    .native
                    .as_ref()
                    .expect("native certificate distinct from stack roots");
                assert_eq!(native.bytes, 64);
                assert_eq!(native.alignment, 8);
                assert!(
                    calls
                        .addresses
                        .as_ref()
                        .unwrap()
                        .roots
                        .extent(native.root)
                        .is_none()
                );
            }
        }
        assert_eq!(
            r.call_slots
                .keys()
                .any(|(site, _)| calls.get(*site).is_some()),
            expected,
            "{name}"
        );
    }
    let original =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "good")
            .unwrap()
            .clone();
    for corrupt in ["argument_type", "result_type", "branch_edge"] {
        let mut changed = original.clone();
        if corrupt == "branch_edge" {
            let event = changed.blocks[0]
                .events
                .iter_mut()
                .find(|e| matches!(e, Event::Branch { .. }))
                .unwrap();
            if let Event::Branch {
                then_block, ..
            } = event
            {
                *then_block = usize::MAX;
            }
        } else {
            let event=changed.blocks[0].events.iter_mut().find(|e|matches!(e,Event::Call {callee,..} if all.procs[callee].native_allocation.is_some())).unwrap();
            if let Event::Call {
                args,
                results,
                ..
            } = event
            {
                if corrupt == "argument_type" {
                    args[0].ty = TypeId::S64;
                } else {
                    results[0].2 = TypeId::S64;
                }
            }
        }
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .insert(original.procedure, changed);
        let body = &c.ed.facts().unwrap().lifetime_bodies[&original.procedure];
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        assert!(
            super::super::no_growth::calls::Calls::collect(&r, &all).is_empty(),
            "{corrupt}"
        );
    }
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .insert(original.procedure, original.clone());
    c.release_lowered_code();
    let body = &c.ed.facts().unwrap().lifetime_bodies[&original.procedure];
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    assert!(
        !super::super::no_growth::calls::Calls::collect(&r, &all).is_empty(),
        "actual native call applicability survives IR release"
    );
}

/// The original diagnostic carriers must enter initial growth with exact bits.
fn initial_append_data(fixture: &str, procedure: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let source = std::fs::read_to_string(root.join(fixture)).unwrap();
    let (c, all) = checked_without_contracts(&source);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == procedure)
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let initial: Vec<_> = r
        .call_slots
        .iter()
        .filter(|(_, snapshot)| snapshot.bounds() == Some((0, 0)))
        .collect();
    assert_eq!(initial.len(), 1, "{fixture}: exact initial bounds");
    assert!(
        initial[0].1.null_data(&c),
        "{fixture}: initial growth requires captured typed null, not an unknown DATA source"
    );
    let calls = super::super::no_growth::calls::Calls::collect(&r, &all);
    assert!(
        calls.get(initial[0].0.0).is_none(),
        "typed zero bits alone cannot suppress initial growth or certify a provider"
    );
}

#[test]
fn original_initial_append_record_captures_exact_empty_data() {
    initial_append_data(
        "tests/lint/pointer_into_resizable/bad.jai",
        "added_then_added",
    );
}

#[test]
fn original_initial_append_scalar_captures_exact_empty_data() {
    initial_append_data(
        "tests/lint/pointer_lifetime/regions-bad.jai",
        "returned_growth_pointer",
    );
}

#[test]
fn initial_append_null_bits_keep_overlap_exposure_and_release_boundaries() {
    let mut c = super::stdlib_allocations::checked_source(
        r#"
        #import "Basic";
        Partial::struct { count:s64; half:u32; }
        opaque::(p:*void) #foreign;
        observer::(p:*[..]s64)->*s64{return p.data;}
        fresh::(){a:[..]s64;observer(*a);}
        replaced::(){back:[8]s64;a:[..]s64;a.data=cast(*s64)*back;observer(*a);}
        partial::(){a:[..]s64;(cast(*Partial)*a).half=0;observer(*a);}
        exposed::(){a:[..]s64;opaque(*a);observer(*a);}
        main::(){}
    "#,
    );
    let capture = |c: &Compiler| {
        let all = Summaries::build(c);
        ["fresh", "replaced", "partial", "exposed"].map(|name| {
            let body =
                c.ed.facts()
                    .unwrap()
                    .lifetime_bodies
                    .values()
                    .find(|b| c.proc(b.procedure).name.as_str() == name)
                    .unwrap();
            let prepared = Prepared::for_body(
                crate::rules::lifetime_facts::CheckedSession::new(c)
                    .body((body).procedure)
                    .unwrap(),
                &all,
            );
            let r = Resolver::for_prepared(&prepared, &all);
            assert_eq!(r.call_slots.len(), 1, "{name}");
            r.call_slots.values().next().unwrap().null_data(c)
        })
    };
    assert_eq!(capture(&c), [true, false, false, false]);
    c.release_lowered_code();
    assert_eq!(capture(&c), [true, false, false, false]);
}

#[test]
fn initial_append_null_bits_require_the_exact_original_zero_write() {
    let mut c = super::stdlib_allocations::checked_source(
        r#"
        #import "Basic";
        observer::(p:*[..]s64)->*s64{return p.data;}
        probe::(){a:[..]s64;observer(*a);}
        main::(){}
    "#,
    );
    let original =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "probe")
            .unwrap()
            .clone();
    let query = |c: &Compiler| {
        let all = Summaries::build(c);
        let body = &c.ed.facts().unwrap().lifetime_bodies[&original.procedure];
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        assert_eq!(r.call_slots.len(), 1);
        r.call_slots.values().next().unwrap().null_data(c)
    };
    assert!(query(&c));
    for corruption in ["partial", "oversized", "copied", "unpaired"] {
        let mut body = original.clone();
        let (block, event) = body
            .blocks
            .iter()
            .enumerate()
            .find_map(|(b, block)| {
                block
                    .events
                    .iter()
                    .position(|e| {
                        matches!(e, Event::DefaultImage{ty,..}
                if matches!(c.ty.types.kind(*ty), TypeKind::Array{kind:ArrayKind::Resizable,..}))
                    })
                    .map(|e| (b, e))
            })
            .unwrap();
        if corruption == "unpaired" {
            let Event::Initialize {
                place, ..
            } = &mut body.blocks[block].events[event - 1]
            else {
                panic!()
            };
            *place = Val(u32::MAX);
        } else {
            let Event::DefaultImage {
                bytes,
                zeroed,
                ..
            } = &mut body.blocks[block].events[event]
            else {
                panic!()
            };
            match corruption {
                "partial" => *bytes = 15,
                "oversized" => *bytes += 1,
                "copied" => *zeroed = false,
                _ => unreachable!(),
            }
        }
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .insert(original.procedure, body);
        assert!(!query(&c), "{corruption}");
    }
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .insert(original.procedure, original.clone());
    c.release_lowered_code();
    assert!(query(&c));
}
