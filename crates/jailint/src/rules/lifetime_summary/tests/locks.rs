//! Native lock protocol composition, independent of procedure and field names.
use super::*;

#[test]
fn recursive_lock_transfers_preserve_order_requirements_and_uncertainty() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        lib :: #library,system "kernel32";
        enter_native :: (p: *void) #foreign lib "EnterCriticalSection";
        wrong_system :: #library,system "c";
        not_system :: #library "kernel32";
        wrong_enter :: (p:*void) #foreign wrong_system "EnterCriticalSection";
        local_enter :: (p:*void) #foreign not_system "EnterCriticalSection";
        probe_wrong_native_library :: (p:*void) {wrong_enter(p);}
        probe_non_system_native_library :: (p:*void) {local_enter(p);}
        leave_native :: (p: *void) #foreign lib "LeaveCriticalSection";
        status_native :: (p: *void) -> s32 #foreign lib "pthread_mutex_lock";
        unknown_native :: (p: *void) #foreign lib "unknown_lock_effect";
        probe_acquire :: (p: *void) { enter_native(p); }
        probe_release :: (p: *void) { leave_native(p); }
        probe_nested :: (p: *void) { probe_acquire(p); probe_acquire(p); probe_release(p); }
        probe_balanced :: (p: *void) { probe_acquire(p); probe_release(p); }
        probe_requires_held :: (p: *void) { probe_release(p); probe_acquire(p); }
        probe_alias_balanced :: (p: *void) { alias := p; probe_acquire(alias); probe_release(p); }
        Probe_Pointer_Lock :: struct { current: *Probe_Lock; }
        probe_replace_current :: (p: *Probe_Pointer_Lock, replacement: *Probe_Lock) { p.current = replacement; }
        probe_acquire_current :: (p: *Probe_Pointer_Lock) { probe_enter_typed(p.current); }
        probe_leave_typed :: (p: *Probe_Lock) #foreign lib "LeaveCriticalSection";
        probe_release_current :: (p: *Probe_Pointer_Lock) { probe_leave_typed(p.current); }
        probe_replaced_object :: (p: *Probe_Pointer_Lock, replacement: *Probe_Lock) {
            probe_acquire_current(p);
            probe_replace_current(p, replacement);
            probe_release_current(p);
        }
        probe_distinct :: (p: *void, q: *void) { probe_acquire(p); probe_release(q); }
        probe_conditional :: (p: *void, choose: bool) { if choose probe_acquire(p); }
        probe_changed_unknown :: (p: *void) { probe_acquire(p); unknown_native(p); }
        probe_status_acquire :: (p: *void) { status_native(p); }
        probe_repeated :: (p: *void, count: int) { for 0..count probe_acquire(p); }
        probe_recursive :: (p: *void, count: int) { if count > 0 probe_recursive(p, count - 1); probe_acquire(p); }
        Flag :: struct { held: bool; }
        probe_boolean_write :: (p: *Flag) { p.held = true; }
        Probe_Lock :: struct { storage: [64] u8; }
        probe_enter_typed :: (p: *Probe_Lock) #foreign lib "EnterCriticalSection";
        Probe_Pair :: struct { first: Probe_Lock; second: Probe_Lock; }
        probe_by_value_acquire :: (copy: Probe_Lock) { probe_enter_typed(*copy); }
        probe_field_acquire :: (p: *Probe_Pair) { probe_enter_typed(*p.first); }
        probe_void_field_acquire :: (p: *Probe_Pair) { enter_native(*p.first); }
        probe_second_acquire :: (p:*Probe_Pair) { probe_enter_typed(*p.second); }
        probe_local_extent_bad :: () { tiny:u8; probe_enter_typed(*((cast(*Probe_Pair)*tiny).second)); }
        probe_local_extent_good :: () { pair:Probe_Pair; probe_enter_typed(*pair.second); }
        probe_local_extent_forwarded_bad :: () { tiny:u8; probe_second_acquire(cast(*Probe_Pair)*tiny); }
        probe_local_extent_forwarded_good :: () { pair:Probe_Pair; probe_second_acquire(*pair); }
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let all = Summaries::build(&c);
    let copied_id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "probe_by_value_acquire")
            .unwrap() as u32,
    );
    let copied_body = &c.ed.facts().unwrap().lifetime_bodies[&copied_id];
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((copied_body).procedure)
            .unwrap(),
        &all,
    );
    let resolver = Resolver::for_prepared(&prepared, &all);
    let incoming = copied_body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|event| match event {
            Event::Parameter {
                incoming,
                formal: 0,
                ..
            } => Some(*incoming),
            _ => None,
        })
        .unwrap();
    assert!(
        resolver.address_object(incoming).is_none(),
        "a by-value formal snapshot is not a caller pointer address"
    );
    for (name, expected) in [
        ("probe_local_extent_bad", false),
        ("probe_local_extent_good", true),
        ("probe_local_extent_forwarded_bad", false),
        ("probe_local_extent_forwarded_good", true),
    ] {
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
        let mut checked = false;
        for (block, b) in body.blocks.iter().enumerate() {
            for (event, e) in b.events.iter().enumerate() {
                let Event::Call {
                    callee,
                    args,
                    ..
                } = e
                else {
                    continue;
                };
                let object = match c.pr.procs[callee.0 as usize].name.as_str() {
                    "probe_enter_typed" => resolver.address_object(value(&args[0]).unwrap()),
                    "probe_second_acquire" => resolver.instantiate_lock_object(
                        all.procs[callee].locks.as_ref().unwrap().object.unwrap(),
                        args,
                        crate::rules::lifetime_movement::facts::Site {
                            block,
                            event,
                        },
                        &all.procs,
                    ),
                    _ => continue,
                };
                checked = true;
                assert_eq!(
                    object.is_some(),
                    expected,
                    "{name}: cast views cannot enlarge the actual local extent: {object:?}"
                );
            }
        }
        assert!(checked, "{name}: expected checked call");
    }
    let get = |name: &str| {
        let id =
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap();
        all.procs[&ProcId(id as u32)].locks.as_ref()
    };
    for (name, required, delta) in [
        ("probe_acquire", 0, 1),
        ("probe_release", 1, -1),
        ("probe_nested", 0, 1),
        ("probe_balanced", 0, 0),
        ("probe_requires_held", 1, 0),
        ("probe_alias_balanced", 0, 0),
    ] {
        let transfer = get(name).unwrap_or_else(|| panic!("missing {name}"));
        assert_eq!(
            (transfer.balance.required, transfer.balance.delta),
            (required, delta),
            "{name}: {transfer:?}"
        );
    }
    for name in [
        "probe_wrong_native_library",
        "probe_non_system_native_library",
        "probe_by_value_acquire",
        "probe_distinct",
        "probe_replace_current",
        "probe_replaced_object",
        "probe_conditional",
        "probe_changed_unknown",
        "probe_status_acquire",
        "probe_repeated",
        "probe_recursive",
    ] {
        assert!(get(name).is_none(), "{name}: {:?}", get(name));
    }
    assert!(get("probe_boolean_write").is_none_or(|t| t.target.is_none()));
    let typed = get("probe_field_acquire").expect("typed field receiver must be retained");
    assert!(
        typed
            .target
            .as_ref()
            .is_some_and(|p| p.root == Root::Parameter(0)
                && p.steps == [Step::Field(jaic::intern::Sym::intern("first"))]),
        "{typed:?}"
    );
    assert!(
        get("probe_void_field_acquire").is_none_or(|t| t
            .target
            .as_ref()
            .is_some_and(|p| p.root == Root::Parameter(0)
                && (p.steps.is_empty()
                    || p.steps.first() == Some(&Step::Field(jaic::intern::Sym::intern("first")))))),
        "void coercion must not select a different object"
    );
}

#[test]
fn windows_recursive_mutex_release_is_inferred_from_checked_native_wrapper() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual, IdeRoot::under(&stdlib)]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        #import "Thread";
        main :: () { mutex: Mutex; init(*mutex); lock(*mutex); unlock(*mutex); destroy(*mutex); }
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    without_body_notes(&mut c);
    let all = Summaries::build(&c);
    let mut count = 0;
    for (index, proc) in c.pr.procs.iter().enumerate() {
        if proc.name.as_str() != "unlock"
            || proc.body_state != jaic::sema::procs::BodyState::Done
            || !c
                .src
                .sources
                .get(proc.span.file)
                .path
                .to_string()
                .ends_with("internal/thread/native-windows.jai")
        {
            continue;
        }
        count += 1;
        let transfer = all.procs[&ProcId(index as u32)].locks.as_ref().unwrap();
        assert_eq!((transfer.balance.required, transfer.balance.delta), (1, -1));
        assert!(
            transfer
                .target
                .as_ref()
                .is_some_and(|p| p.root == Root::Parameter(0))
        );
    }
    assert!(count > 0, "no checked Windows release carrier");
}

#[test]
fn native_lock_boundary_requires_the_actual_windows_target() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.os = jaic::target::TargetOs::Linux;
    options.cpu = jaic::target::TargetCpu::X64;
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"
        lib :: #library,system "kernel32";
        enter :: (p:*void) #foreign lib "EnterCriticalSection";
        probe :: (p:*void) { enter(p); }
        main :: () {}
    "#
        .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    assert!(c.ide_check_all().is_empty());
    let id =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "enter")
            .unwrap();
    assert!(super::super::locks::native(&c, ProcId(id as u32)).is_none());
}

#[test]
fn native_boundaries_reject_variadic_lock_prototypes() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.os = jaic::target::TargetOs::Windows;
    options.cpu = jaic::target::TargetCpu::X64;
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String(
        r#"lib :: #system_library "kernel32";
        enter :: (p:*void) #foreign lib "EnterCriticalSection";
        leave :: (p:..*void) #foreign lib "LeaveCriticalSection";
        probe :: (p:*void) {enter(p);leave(p);}
        main :: (){}"#
            .into(),
    )])
    .unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    for (name, expected) in [("enter", true), ("leave", false)] {
        let id = ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        );
        assert_eq!(
            super::super::locks::native(&c, id).is_some(),
            expected,
            "{name}"
        );
    }
}
