//! Certified ranges stay conservative for overlap, width and shifted recursion.
use super::*;

#[test]
fn physical_write_slots_preserve_overlap_width_and_finite_forwarding() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Basic";
        Overlap :: union { a: *void; b: *void; }
        Holder :: struct { storage: *void; }
        Pair :: struct { left: *void; right: *void; }
        probe_union_write :: (p: *Overlap, replacement: *void) {
            p.b = replacement;
            free(p.a);
        }
        probe_partial_write :: (p: *Holder) {
            bytes := cast(*u8)*p.storage;
            bytes.* = 1;
            free(p.storage);
        }
        probe_shift_recursive :: (p: *Pair, replacement: *void, more: bool) {
            p.left = replacement;
            if more probe_shift_recursive(cast(*Pair)*p.right, replacement, false);
        }
        probe_shift_a :: (p: *Pair, replacement: *void, more: bool) {
            p.left = replacement;
            if more probe_shift_b(cast(*Pair)*p.right, replacement, false);
        }
        probe_shift_b :: (p: *Pair, replacement: *void, more: bool) {
            if more probe_shift_a(cast(*Pair)*p.right, replacement, false);
        }
        probe_redirect :: (slot: **Pair, replacement: *Pair) { slot.* = replacement; }
        probe_write_left :: (p: *Pair, replacement: *void) { p.left = replacement; }
        probe_exposed_local :: (p: *Pair, next: *Pair, replacement: *void) {
            current := p;
            probe_redirect(*current, next);
            probe_write_left(current, replacement);
            free(p.right);
        }
        probe_write_pointer :: (slot: **void, replacement: *void) { slot.* = replacement; }
        probe_offset_alias :: (p: *Pair, replacement: *void) {
            probe_write_pointer(cast(**void)p + 1, replacement);
            free(p.right);
        }
        Overlay :: struct { a: *void; #overlay(a) b: *void; }
        probe_local_overlay :: (p: *void, replacement: *void) {
            item: Overlay;
            item.a = p;
            item.b = replacement;
            free(item.a);
        }
        probe_local_reset :: (p: *void) {
            item: Holder;
            item.storage = p;
            item = .{};
            free(item.storage);
        }
        probe_global: *void;
        probe_set_global :: (value: *void) { probe_global = value; }
        probe_global_after_call :: (p: *void, replacement: *void) {
            probe_global = p;
            probe_set_global(replacement);
            free(probe_global);
        }
        probe_global_alias_slot :: (slot: **void, replacement: *void) {
            probe_global = replacement;
            free(slot.*);
        }
        probe_negative_offset :: (p: *Pair, replacement: *void) {
            probe_write_pointer(cast(**void)*p.right - 1, replacement);
            free(p.left);
        }
        probe_inline_stack_alias :: (p: *void, replacement: *void) {
            pair: Pair;
            pair.right = p;
            anchor: u8;
            delta := cast(s64)*pair.right - cast(s64)*anchor;
            (cast(**void)(*anchor + delta)).* = replacement;
            free(pair.right);
        }
        probe_dynamic_offset :: (p: *Pair, replacement: *void, index: int) {
            probe_write_pointer(cast(**void)p + index, replacement);
            free(p.right);
        }
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    without_body_notes(&mut c);
    let all = Summaries::build(&c);
    let get = |name: &str| {
        let index =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        &all.procs[&ProcId(index as u32)]
    };
    let alias_id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "probe_inline_stack_alias")
            .unwrap() as u32,
    );
    let alias_body = &c.ed.facts().unwrap().lifetime_bodies[&alias_id];
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((alias_body).procedure)
            .unwrap(),
        &all,
    );
    let resolver = Resolver::for_prepared(&prepared, &all);
    let mut checked = 0;
    for (place, ((value, _), steps)) in resolver.places.paths.iter().enumerate() {
        if steps.is_empty()
            && alias_body.stack_places.contains(value)
            && !alias_body
                .stack_roots
                .iter()
                .any(|root| root.value == *value)
        {
            checked += 1;
            assert!(
                !resolver.immutable_private_slot(place),
                "derived stack ancestry cannot certify a new private allocation: {value:?}"
            );
        }
    }
    assert!(
        checked > 0,
        "fixture must retain a derived address for the ownership invariant"
    );
    let replacement = Path {
        root: Root::Parameter(1),
        steps: Vec::new(),
    };
    assert!(
        get("probe_union_write")
            .may
            .keys()
            .any(|effect| effect.kind == Kind::Free && effect.target == replacement)
    );
    assert!(
        !get("probe_partial_write")
            .guaranteed
            .keys()
            .any(|effect| effect.kind == Kind::Free),
        "partial pointer bytes are not a whole pointer source"
    );
    for (name, formal) in [
        ("probe_exposed_local", 2),
        ("probe_offset_alias", 1),
        ("probe_global_alias_slot", 1),
        ("probe_negative_offset", 1),
        ("probe_dynamic_offset", 1),
    ] {
        assert!(
            get(name).may.keys().any(|effect| effect.kind == Kind::Free
                && effect.target
                    == Path {
                        root: Root::Parameter(formal),
                        steps: Vec::new()
                    }),
            "uncertified address may alias the loaded slot: {name}: {:?}",
            get(name)
        );
    }
    for name in [
        "probe_local_overlay",
        "probe_local_reset",
        "probe_global_after_call",
        "probe_inline_stack_alias",
    ] {
        assert!(
            !get(name)
                .guaranteed
                .keys()
                .any(|effect| effect.kind == Kind::Free
                    && effect.target
                        == Path {
                            root: Root::Parameter(0),
                            steps: Vec::new()
                        }),
            "local byte overlap/reset replaces the initial pointer: {name}: {:?}",
            get(name)
        );
    }
    let global_alias = get("probe_global_alias_slot");
    assert!(
        !global_alias
            .guaranteed
            .keys()
            .any(|effect| effect.kind == Kind::Free),
        "a global write can replace the incoming slot's contents: {global_alias:?}"
    );
    for name in ["probe_shift_recursive", "probe_shift_a", "probe_shift_b"] {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        let body = &c.ed.facts().unwrap().lifetime_bodies[&ProcId(id as u32)];
        let prepared = prepared::Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let resolver = Resolver::for_prepared(&prepared, &all);
        for (&place, ty) in &resolver.places.types {
            if let Some(slot) = resolver.physical_slot(place, c.ty.types.size_of(*ty)) {
                assert!(
                    slot.offset + slot.width
                        <= 2 * c.ty.types.size_of(jaic::types::TypeId::VOID_PTR)
                );
            }
        }
        assert!(
            get(name).value_writes.len() < 16,
            "offset growth must widen: {name}: {:?}",
            get(name)
        );
    }
}

#[test]
fn recursive_write_footprints_do_not_enumerate_large_pointee_extents() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r"
        Wide :: struct { first: *void; rest: [128] *void; }
        shifted :: (p: *Wide, value: *void, more: bool) {
            p.first = value;
            if more shifted(cast(*Wide)*p.rest, value, false);
        }
        mutual_a :: (p: *Wide, value: *void, more: bool) {
            p.first = value;
            if more mutual_b(cast(*Wide)*p.rest, value, false);
        }
        mutual_b :: (p: *Wide, value: *void, more: bool) {
            if more mutual_a(cast(*Wide)*p.rest, value, false);
        }
        main :: () {}
    "
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let all = Summaries::build(&c);
    for name in ["shifted", "mutual_a", "mutual_b"] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        let summary = &all.procs[&id];
        assert!(summary.recursive);
        assert!(
            // One direct checked write and one widened recursive footprint.
            summary.value_writes.len() <= 2,
            "recursive forwarding must widen composed offsets, not enumerate the pointee extent: {name}: {:?}",
            summary.value_writes
        );
        assert!(summary.value_writes.values().flatten().any(|source| matches!(source,
            super::super::value_writes::Installed::Value(Path {root:Root::Parameter(1),steps}) if steps.is_empty())),
            "widening must retain the installed source as a may effect");
    }
}

#[test]
fn physical_private_formal_slots_are_distinct_from_incoming_pointer_values() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r"
        Item :: struct { storage:*void; extra:[32]u8; }
        pointer_slot :: (p:*Item)->*void { return p.storage; }
        address_slot :: (p:*Item)->**Item { return *p; }
        aggregate_slot :: (item:Item)->*void { return item.storage; }
        main :: () {}
        "
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let all = Summaries::build(&c);
    for name in ["pointer_slot", "address_slot", "aggregate_slot"] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        let body = &c.ed.facts().unwrap().lifetime_bodies[&id];
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let resolver = Resolver::for_prepared(&prepared, &all);
        let (place, incoming, ty) = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .find_map(|event| match event {
                Event::Parameter {
                    formal: 0,
                    place,
                    incoming,
                    ty,
                    ..
                } => Some((*place, *incoming, *ty)),
                _ => None,
            })
            .unwrap();
        let slot = resolver.places.existing(place, ty).unwrap();
        assert!(
            resolver.owned.contains(&resolver.places.root_of(slot)),
            "{name}: actual formal spill is private storage"
        );
        assert!(super::super::storage_versions::private_range(
            &c,
            body,
            &resolver.places,
            slot,
            c.ty.types.size_of(ty),
            &resolver.private_bytes
        ));
        if name == "aggregate_slot" {
            assert!(
                resolver.address_object(incoming).is_none(),
                "by-value ABI buffer is not an incoming pointer identity"
            );
        } else {
            assert_eq!(
                resolver.address_object(incoming).unwrap().base,
                super::super::address_objects::Base::Input(0)
            );
        }
        if name == "pointer_slot" {
            assert!(resolver.immutable_private_slot(slot));
            let pointee_places: Vec<_> = resolver
                .places
                .paths
                .iter()
                .enumerate()
                .filter(|(_, (_, steps))| steps.contains(&Projection::Deref))
                .map(|(p, _)| p)
                .collect();
            assert!(!pointee_places.is_empty());
            assert!(
                pointee_places
                    .into_iter()
                    .all(|p| !resolver.immutable_private_slot(p)),
                "pointed storage is never the formal variable's spill"
            );
        }
        if name == "address_slot" {
            assert!(
                matches!(
                    resolver.address_object(place).map(|object| object.base),
                    Some(super::super::address_objects::Base::Local(_))
                ),
                "address of the private formal slot is local, not Input"
            );
            assert!(
                !resolver.immutable_private_slot(slot),
                "address-taking exposes the slot"
            );
        }
    }
}

#[test]
fn retention_locations_distinguish_selected_bytes_under_pointer_views() {
    let c = super::stdlib_allocations::checked_source(
        r"
        A::struct {slot:*s64;pad:s64;}
        B::struct {pad:s64;slot:*s64;}
        retention_views::(p:*A,v:*s64){p.slot=v;(cast(*B)p).slot=null;}
        main::(){}
        ",
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "retention_views")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let mut work = 8192;
    let index = r.capture_index(&mut work).unwrap();
    let locations: Vec<_> = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| {
            let Event::Store {
                place,
                ty,
                span,
                ..
            } = e
            else {
                return None;
            };
            c.src
                .sources
                .snippet(*span)
                .contains(".slot")
                .then(|| index.write_location(&r, *place, *ty, &mut work, 0).unwrap())
        })
        .collect();
    assert_eq!(locations.len(), 2);
    assert_ne!(
        locations[0], locations[1],
        "same owner and field spelling do not identify the same bytes"
    );
}


#[test]
fn retention_locations_keep_partial_write_width() {
    let c = super::stdlib_allocations::checked_source(
        r"
        A::struct {slot:*s64;pad:s64;}
        retention_widths::(p:*A,v:*s64){p.slot=v;(cast(*u32)*p.slot).*=0;}
        main::(){}
        ",
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "retention_widths")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let mut work = 8192;
    let index = r.capture_index(&mut work).unwrap();
    let locations: Vec<_> = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| {
            let Event::Store {
                place,
                ty,
                span,
                ..
            } = e
            else {
                return None;
            };
            c.src
                .sources
                .snippet(*span)
                .contains(".slot")
                .then(|| index.write_location(&r, *place, *ty, &mut work, 0).unwrap())
        })
        .collect();
    assert_eq!(locations.len(), 2);
    assert_eq!(locations[0].0, locations[1].0);
    assert_eq!((locations[0].1.offset, locations[1].1.offset), (0, 0));
    assert_eq!((locations[0].1.width, locations[1].1.width), (8, 4));
    assert_ne!(locations[0], locations[1]);
}


#[test]
fn physical_slot_separation_requires_both_checked_ends() {
    use super::super::physical_slots::Slot;
    let byte = Slot {
        formal: 0,
        offset: 0,
        width: 1,
    };
    let overflow = Slot {
        formal: 0,
        offset: u64::MAX,
        width: 1,
    };
    assert!(!overflow.disjoint(byte));
    assert!(!byte.disjoint(overflow));
    assert!(byte.disjoint(Slot {
        formal: 0,
        offset: 1,
        width: 1
    }));
    assert!(!byte.disjoint(Slot {
        formal: 1,
        offset: 1,
        width: 1
    }));
    assert!(!byte.disjoint(byte));
}
