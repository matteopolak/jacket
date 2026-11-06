//! A call captures the current value of a source slot before later mutations.
use super::*;

#[test]
fn helper_slot_sources_are_captured_at_the_call_epoch() {
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
        Pair :: struct { destination: Holder; source: Holder; }
        probe_copy_slot :: (destination: *Holder, source: *Holder) { destination.storage = source.storage; }
        probe_replaced_before_copy :: (pair: *Pair, replacement: *void) {
            pair.source.storage = replacement;
            probe_copy_slot(*pair.destination, *pair.source);
            free(pair.destination.storage);
        }
        probe_replaced_after_copy :: (pair: *Pair, replacement: *void) {
            probe_copy_slot(*pair.destination, *pair.source);
            pair.source.storage = replacement;
            free(pair.destination.storage);
        }
        Cast_A :: struct { left: *void; right: *void; }
        Cast_B :: struct { pad: *void; left: *void; }
        probe_copy_cast_source :: (destination: *Holder, source: *Cast_A) {
            q := cast(*Cast_B)source;
            destination.storage = q.left;
        }
        probe_copied_cast_source :: (destination: *Holder, source: *Cast_A) {
            observed := source.right;
            probe_copy_cast_source(destination, source);
            free(destination.storage);
        }
        probe_capture_private :: (destination: *Holder, source: *void, replacement: *void) {
            current := source;
            old := current;
            current = replacement;
            destination.storage = old;
        }
        probe_free_private_capture :: (destination: *Holder, source: *void, replacement: *void) {
            probe_capture_private(destination, source, replacement);
            free(destination.storage);
        }
        probe_read_cast_B :: (source: *Cast_B) -> *void { return source.left; }
        probe_getter_cast_source :: (destination: *Holder, source: *Cast_A) {
            destination.storage = probe_read_cast_B(cast(*Cast_B)source);
        }
        probe_put_input :: (destination: *Holder, input: *void) {destination.storage = input;}
        probe_input_cast :: (destination: *Holder, source: *Cast_A) {
            q := cast(*Cast_B)source;
            probe_put_input(destination, q.left);
        }
        probe_input_cast_free :: (destination: *Holder, source: *Cast_A) {
            observed := source.right;
            probe_input_cast(destination, source);
            free(destination.storage);
        }
        main :: () {}
    "#.into())]).unwrap();
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
    let getter = get("probe_getter_cast_source");
    assert!(getter.value_writes.values().flatten().all(|source| !matches!(source, value_writes::Installed::Value(path) if !path.steps.is_empty())), "untyped getter fields cannot bypass physical slot certificates: {getter:?}");
    let source_proc =
        c.pr.procs
            .iter()
            .find(|p| p.name.as_str() == "probe_getter_cast_source")
            .unwrap();
    let source_ty =
        c.ty.types
            .pointee(source_proc.sig.as_ref().unwrap().params[1].ty)
            .unwrap();
    let st = c.ty.types.as_struct(source_ty).unwrap();
    let right =
        c.ty.types
            .struct_info(st)
            .fields
            .iter()
            .find(|f| f.name.is_some_and(|n| n.as_str() == "right"))
            .unwrap();
    let expected = super::super::physical_slots::Slot {
        formal: 1,
        offset: right.offset,
        width: c.ty.types.size_of(right.ty),
    };
    assert!(
        getter
            .value_writes
            .values()
            .flatten()
            .any(|source| matches!(source,value_writes::Installed::Entry(slot) if *slot==expected)),
        "getter return retains the actual physical incoming slot: {getter:?}"
    );
    assert!(
        getter
            .value_writes
            .values()
            .flatten()
            .all(|source| !matches!(source, value_writes::Installed::Unknown)),
        "fully modeled getter requires no invented incompleteness: {getter:?}"
    );
    let cast_source = Path {
        root: Root::Parameter(1),
        steps: vec![Step::Field(jaic::intern::Sym::intern("right")), Step::Deref],
    };
    assert!(
        get("probe_copied_cast_source")
            .may
            .keys()
            .any(|e| e.kind == Kind::Free && e.target == cast_source),
        "cast source uses defining byte layout: {:?}",
        get("probe_copied_cast_source")
    );
    assert!(
        get("probe_input_cast_free")
            .may
            .keys()
            .any(|effect| effect.kind == Kind::Free && effect.target == cast_source),
        "formal input actual retains physical field provenance: {:?}",
        get("probe_input_cast_free")
    );
    let private = get("probe_free_private_capture");
    assert!(
        private.may.keys().any(|e| e.kind == Kind::Free
            && e.target
                == Path {
                    root: Root::Parameter(1),
                    steps: Vec::new()
                }),
        "{private:?}"
    );
    assert!(
        !private.may.keys().any(|e| e.kind == Kind::Free
            && e.target
                == Path {
                    root: Root::Parameter(2),
                    steps: Vec::new()
                }),
        "a later private rebind cannot change captured SSA: {private:?}"
    );
    let replacement = Path {
        root: Root::Parameter(1),
        steps: Vec::new(),
    };
    let previous = Path {
        root: Root::Parameter(0),
        steps: vec![
            Step::Field(jaic::intern::Sym::intern("source")),
            Step::Field(jaic::intern::Sym::intern("storage")),
            Step::Deref,
        ],
    };
    let possible_free = |s: &Summary, path: &Path| {
        s.may
            .keys()
            .any(|e| e.kind == Kind::Free && e.target == *path)
    };
    assert!(
        possible_free(get("probe_replaced_before_copy"), &replacement),
        "{:?}",
        get("probe_replaced_before_copy")
    );
    assert!(
        possible_free(get("probe_replaced_after_copy"), &previous),
        "{:?}",
        get("probe_replaced_after_copy")
    );
    assert!(
        !possible_free(get("probe_replaced_after_copy"), &replacement),
        "future stores cannot change captured call input: {:?}",
        get("probe_replaced_after_copy")
    );
}
