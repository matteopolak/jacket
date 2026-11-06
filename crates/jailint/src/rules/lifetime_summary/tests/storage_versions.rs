//! Storage effects refer to the allocation loaded at the event, not a field spelling.
use super::*;

#[test]
fn field_replacement_preserves_cleanup_storage_versions() {
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
        Holder :: struct { storage: *void; linked: *Holder; }
        replace_then_free :: (p: *Holder, replacement: *void) {
            p.storage = replacement;
            free(p.storage);
        }
        free_then_replace :: (p: *Holder, replacement: *void) {
            free(p.storage);
            p.storage = replacement;
        }
        snapshot_then_replace :: (p: *Holder, replacement: *void) {
            old := p.storage;
            p.storage = replacement;
            free(old);
        }
        conditional_replacement :: (p: *Holder, replacement: *void, choose: bool) {
            if choose p.storage = replacement;
            free(p.storage);
        }
        Descriptor_Pair :: struct { a: [] *void; b: [] *void; }
        indexed_alias_replace :: (p: *Descriptor_Pair, replacement: *void) #no_abc {
            p.b[0] = replacement;
            free(p.a[0]);
        }
        private_parameter_rebind :: (p: *Holder, replacement: *Holder) { p = replacement; }
        helper_replaces :: (p: *Holder, replacement: *void) { p.storage = replacement; }
        helper_then_free :: (p: *Holder, replacement: *void) {
            helper_replaces(p, replacement);
            free(p.storage);
        }
        nested_helper_replaces :: (p: *Holder, replacement: *void) { p.linked.storage = replacement; }
        nested_alias_then_free :: (p: *Holder, replacement: *void) {
            nested_helper_replaces(p, replacement);
            free(p.storage);
        }
        alias_replacement :: (p: *Holder, other: *Holder, replacement: *void) {
            other.storage = replacement;
            free(p.storage);
        }
        Cast_A :: struct { left: *void; right: *void; }
        Cast_B :: struct { pad: *void; left: *void; }
        probe_write_cast_B :: (p: *Cast_B, replacement: *void) { p.left = replacement; }
        probe_cast_outside_helper :: (p: *Cast_A, replacement: *void) {
            probe_write_cast_B(cast(*Cast_B)p, replacement);
            free(p.right);
        }
        probe_cast_inside_helper :: (p: *Cast_A, replacement: *void) {
            q := cast(*Cast_B)p;
            q.left = replacement;
        }
        probe_after_cast_inside :: (p: *Cast_A, replacement: *void) {
            probe_cast_inside_helper(p, replacement);
            free(p.right);
        }
        probe_string_free :: (value: string) { free(value.data); }
        probe_copy_before_replace :: (p: *Holder, replacement: *void) {
            old := p.*;
            p.storage = replacement;
            free(old.storage);
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
    let pair =
        c.pr.procs
            .iter()
            .find(|p| p.name.as_str() == "indexed_alias_replace")
            .unwrap();
    let pair_ty =
        c.ty.types
            .pointee(pair.sig.as_ref().unwrap().params[0].ty)
            .unwrap();
    let indexed = |name| {
        [
            Projection::Field(jaic::intern::Sym::intern(name)),
            Projection::Element(0),
        ]
    };
    assert!(
        !Places::disjoint_paths(&c, pair_ty, &indexed("a"), &indexed("b")),
        "different descriptor slots can have identical backing storage"
    );
    let all = Summaries::build(&c);

    let get = |name: &str| {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        &all.procs[&ProcId(id as u32)]
    };
    let original = Path {
        root: Root::Parameter(0),
        steps: vec![
            Step::Field(jaic::intern::Sym::intern("storage")),
            Step::Deref,
        ],
    };
    let replacement = Path {
        root: Root::Parameter(1),
        steps: vec![],
    };
    let frees = |s: &Summary, target: &Path, must: bool| {
        (if must {
            &s.guaranteed
        } else {
            &s.may
        })
        .keys()
        .any(|e| e.kind == Kind::Free && e.target == *target)
    };
    assert!(
        frees(get("replace_then_free"), &replacement, true),
        "{:?}",
        get("replace_then_free")
    );
    assert!(!frees(get("replace_then_free"), &original, false));
    for name in ["free_then_replace", "snapshot_then_replace"] {
        assert!(frees(get(name), &original, true), "{name}: {:?}", get(name));
        assert!(!frees(get(name), &replacement, false));
    }
    for name in ["probe_cast_outside_helper", "probe_after_cast_inside"] {
        let cast = get(name);
        assert!(
            frees(cast, &replacement, false),
            "typed reinterpretation must retain aliased write: {name}: {cast:?}"
        );
        assert!(
            !cast
                .guaranteed
                .keys()
                .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(0)),
            "{name}: {cast:?}"
        );
    }
    assert!(
        get("probe_string_free")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free),
        "by-value descriptor storage remains exact: {:?}",
        get("probe_string_free")
    );
    assert!(
        frees(get("probe_copy_before_replace"), &original, false),
        "copied aggregate retains old pointer at its write epoch: {:?}",
        get("probe_copy_before_replace")
    );
    assert!(
        !frees(get("probe_copy_before_replace"), &replacement, false),
        "future source writes do not alter aggregate snapshot: {:?}",
        get("probe_copy_before_replace")
    );
    let alias = get("alias_replacement");
    assert!(!frees(alias, &original, true), "{alias:?}");
    let alias_replacement = Path {
        root: Root::Parameter(2),
        steps: vec![],
    };
    assert!(frees(alias, &alias_replacement, false), "{alias:?}");
    assert!(
        get("private_parameter_rebind").value_writes.is_empty(),
        "private pointer slots do not write through the caller's allocation"
    );
    let indexed_alias = get("indexed_alias_replace");
    assert!(
        frees(indexed_alias, &replacement, false),
        "{indexed_alias:?}"
    );
    assert!(
        !indexed_alias
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free && e.target.root == Root::Parameter(0)),
        "{indexed_alias:?}"
    );
    let nested = get("nested_alias_then_free");
    assert!(frees(nested, &replacement, false), "{nested:?}");
    assert!(!frees(nested, &original, true), "{nested:?}");
    let helper = get("helper_then_free");
    for target in [&original, &replacement] {
        assert!(!frees(helper, target, true), "{helper:?}");
    }
    assert!(frees(helper, &replacement, false), "{helper:?}");
    let conditional = get("conditional_replacement");
    for target in [&original, &replacement] {
        assert!(frees(conditional, target, false), "{conditional:?}");
        assert!(!frees(conditional, target, true), "{conditional:?}");
    }
}

#[test]
fn pointer_snapshots_do_not_reinterpret_later_global_or_receiver_stores() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(r#"
        #import "Basic";
        Holder :: struct { storage: *void; }
        G: *void;
        GLOBAL_HOLDER: Holder;
        take_old_global :: (replacement:*void) -> *void { old:=G; G=replacement; return old; }
        free_old_global :: (replacement:*void) { free(take_old_global(replacement)); }
        take_new_global :: (replacement:*void) -> *void { G=replacement; return G; }
        free_new_global :: (replacement:*void) { free(take_new_global(replacement)); }
        global_before_copy :: (p:*Holder,replacement:*void) {
            GLOBAL_HOLDER.storage=replacement;
            copy:=p.*;
            free(copy.storage);
        }
        take_old_receiver :: (p:*Holder,replacement:*void) -> *void {
            old:=p.storage; p.storage=replacement; return old;
        }
        free_old_receiver :: (p:*Holder,replacement:*void) { free(take_old_receiver(p,replacement)); }
        caller_replaces_then_takes_old :: (p:*Holder, first:*void, second:*void) {
            p.storage=first;
            free(take_old_receiver(p,second));
        }
        take_new_receiver :: (p:*Holder,replacement:*void) -> *void {
            p.storage=replacement; return p.storage;
        }
        free_new_receiver :: (p:*Holder,replacement:*void) { free(take_new_receiver(p,replacement)); }
        take_old_local :: (p:*void,q:*void) -> *void { current:=p; old:=current; current=q; return old; }
        free_old_local :: (p:*void,q:*void) { free(take_old_local(p,q)); }
        Cast_A :: struct { left:*void; right:*void; }
        Cast_B :: struct { pad:*void; left:*void; }
        getter_B :: (p:*Cast_B) -> *void { return p.left; }
        free_cast_getter :: (p:*Cast_A) { free(getter_B(cast(*Cast_B)p)); }
        free_cast_getter_after_store :: (p:*Cast_A, replacement:*void) {
            p.right=replacement;
            free(getter_B(cast(*Cast_B)p));
        }
        main :: () {}
    "#.into())]).unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let all = Summaries::build(&c);
    let get = |name: &str| {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        &all.procs[&ProcId(id as u32)]
    };
    let frees = |name: &str, formal: usize, steps: &[Step]| {
        get(name).guaranteed.keys().any(|e| {
            e.kind == Kind::Free
                && e.target.root == Root::Parameter(formal)
                && e.target.steps == steps
        })
    };
    assert!(
        !frees("free_old_global", 0, &[]),
        "old global is not its later replacement: {:?}",
        get("free_old_global").guaranteed
    );
    assert!(frees("free_new_global", 0, &[]));
    let field = [
        Step::Field(jaic::intern::Sym::intern("storage")),
        Step::Deref,
    ];
    assert!(
        !frees("global_before_copy", 0, &field),
        "incoming receiver can alias the written global"
    );
    assert!(
        get("global_before_copy")
            .may
            .keys()
            .any(|e| e.kind == Kind::Free
                && e.target.root == Root::Parameter(1)
                && e.target.steps.is_empty())
    );
    assert!(frees("free_old_receiver", 0, &field));
    assert!(!frees("free_old_receiver", 1, &[]));
    assert!(frees("caller_replaces_then_takes_old", 1, &[]));
    assert!(!frees("caller_replaces_then_takes_old", 0, &field));
    assert!(!frees("caller_replaces_then_takes_old", 2, &[]));
    assert!(frees("free_new_receiver", 1, &[]));
    assert!(!frees("free_new_receiver", 0, &field));
    assert!(frees("free_old_local", 0, &[]));
    assert!(!frees("free_old_local", 1, &[]));
    let right = [Step::Field(jaic::intern::Sym::intern("right")), Step::Deref];
    assert!(frees("free_cast_getter", 0, &right));
    assert!(frees("free_cast_getter_after_store", 1, &[]));
    assert!(!frees("free_cast_getter_after_store", 0, &right));
}


#[test]
fn repeated_allocation_slots_do_not_have_a_single_captured_source() {
    let c = super::stdlib_allocations::checked_source(
        r#"
        libc::#system_library "libc";
        acquire::(bytes:u64)->*void #foreign libc "malloc";
        release::(memory:*void) #foreign libc "free";
        cycle::(again:bool){current:=acquire(8);if !current return;
            while again {saved:=current;next:=acquire(8);if !next return;
                current=next;release(saved);
            }
        }
        main::(){}
    "#,
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "cycle")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let saved = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Local {
                place,
                name,
                ..
            } if name.as_str() == "saved" => Some(*place),
            _ => None,
        })
        .unwrap();
    let loaded = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Store {
                place,
                value,
                ..
            } if *place == saved => Some(*value),
            _ => None,
        })
        .unwrap();
    assert!(r.captured_pointer_value(loaded).is_none());
    assert!(r.captured_pointer_values(loaded, &mut 8).unwrap().count() >= 2);
}

#[test]
fn pointer_constants_require_exact_typed_writes_and_current_read_epochs() {
    let mut c = super::stdlib_allocations::checked_source(
        r#"
        lib::#system_library "not-c-runtime";
        touch::(p:**void) #foreign lib "touch";
        nil_default::()->*void{p:*void;return p;}
        nil_literal::()->*void{p:*void=null;return p;}
        nil_replaced::(q:*void)->*void{p:*void=null;p=q;return p;}
        nil_partial::()->*void{p:*void=null;(cast(*u32)*p).*=1;return p;}
        nil_exposed::()->*void{p:*void=null;touch(*p);return p;}
        main::(){}
    "#,
    );
    let all = Summaries::build(&c);
    let query = |c: &Compiler, body: &jaic::ide::lifetimes::LifetimeBody| {
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        let result = body
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
        r.captured_constant(result)
    };
    let find = |name: &str| {
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == name)
            .unwrap()
    };
    for name in ["nil_default", "nil_literal"] {
        assert!(
            query(&c, find(name)).is_some_and(|(ty, bits)| c.ty.types.is_pointer(ty) && bits == 0),
            "{name}"
        );
    }
    for name in ["nil_replaced", "nil_partial", "nil_exposed"] {
        assert!(query(&c, find(name)).is_none(), "{name}");
    }
    let body = find("nil_default").clone();
    for fault in 0..7 {
        let mut changed = body.clone();
        let image = changed
            .blocks
            .iter_mut()
            .flat_map(|b| &mut b.events)
            .find(|e| matches!(e, Event::DefaultImage { .. }))
            .unwrap();
        let Event::DefaultImage {
            bytes,
            leaves,
            complete,
            ..
        } = image
        else {
            unreachable!()
        };
        match fault {
            0 => leaves.clear(),
            1 => leaves[0].width = 4,
            2 => leaves[0].ty = TypeId::S64,
            3 => *bytes = 64,
            4 => *complete = false,
            5 => leaves[0].value = jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(1),
            6 => {
                for b in &mut changed.blocks {
                    if let Some(at) = b
                        .events
                        .iter()
                        .position(|e| matches!(e, Event::DefaultImage { .. }))
                    {
                        b.events.insert(at, Event::Barrier);
                        break;
                    }
                }
            }
            _ => unreachable!(),
        }
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .insert(body.procedure, changed);
        let changed = &c.ed.facts().unwrap().lifetime_bodies[&body.procedure];
        assert!(
            query(&c, changed).is_none(),
            "malformed pointer image {fault}"
        );
    }
}


#[test]
fn captured_pointer_constants_survive_executable_code_release() {
    fn snapshot(c: &Compiler) -> Vec<Option<(TypeId, u64)>> {
        let all = Summaries::build(c);
        ["nil_default", "nil_literal", "replaced"]
            .map(|name| {
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
                let value = body
                    .blocks
                    .iter()
                    .flat_map(|b| &b.events)
                    .find_map(|e| {
                        if let Event::Return {
                            value, ..
                        } = e
                        {
                            Some(*value)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                r.captured_constant(value)
            })
            .to_vec()
    }
    let mut c = super::stdlib_allocations::checked_source(
        r"
        nil_default::()->*void {p:*void;return p;}
        nil_literal::()->*void {p:*void=null;return p;}
        replaced::(q:*void)->*void {p:*void=null;p=q;return p;}
        main::(){}
    ",
    );
    let before = snapshot(&c);
    assert!(before[0].is_some_and(|(ty, bits)| c.ty.types.is_pointer(ty) && bits == 0));
    assert_eq!(before[0], before[1]);
    assert_eq!(before[2], None);
    c.release_lowered_code();
    assert_eq!(snapshot(&c), before);
}
