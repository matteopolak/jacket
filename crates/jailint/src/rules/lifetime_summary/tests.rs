use super::*;
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{NativeFs, Options, ProgramSource};
use std::rc::Rc;

#[test]
fn allocator_mode_guards_distinguish_allocate_resize_and_free() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let source = r#"
            #import "Basic";
            #import "Hash_Table";
            Item :: struct { storage: *void; }
            probe_release :: (p: *Item, a: Allocator) { a.proc(.FREE, 0, 0, p.storage, a.data); }
            probe_resize :: (p: *Item, a: Allocator) { a.proc(.RESIZE, 32, 16, p.storage, a.data); }
            probe_allocate :: (p: *Item, a: Allocator) { a.proc(.ALLOCATE, 32, 0, null, a.data); }
            probe_conditional :: (p: *Item, a: Allocator, flag: bool) { if flag probe_release(p, a); }
            main :: () {
                xs: [..] int;
                array_reserve(*xs, 16);
                table: Table(int, int);
                table_add(*table, 1, 2);
                builder: String_Builder;
                append(*builder, "hello");
            }
        "#;
    c.begin_sources(&[ProgramSource::String(source.into())])
        .unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    let summaries = Summaries::build(&c);
    let get = |name: &str| {
        let (i, _) =
            c.pr.procs
                .iter()
                .enumerate()
                .find(|(_, p)| p.name.as_str() == name)
                .unwrap();
        &summaries.procs[&ProcId(i as u32)]
    };
    assert!(
        get("probe_release")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free && e.guard.is_none() && e.target.steps.len() == 2)
    );
    assert!(
        get("probe_resize")
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Resize && e.guard.is_none())
    );
    assert!(get("probe_allocate").may.is_empty());
    assert!(
        get("probe_conditional")
            .may
            .keys()
            .any(|e| e.kind == Kind::Free)
    );
    assert!(get("probe_conditional").guaranteed.is_empty());
    assert!(get("free").guaranteed.keys().any(|e| e.kind == Kind::Free
        && e.guard.is_none()
        && e.target.root == Root::Parameter(0)
        && e.target.steps.is_empty()));
    let table_effects: Vec<_> =
        c.pr.procs
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name.as_str() == "table_add" && !p.is_poly)
            .filter_map(|(i, _)| summaries.procs.get(&ProcId(i as u32)))
            .collect();
    assert!(
        table_effects
            .iter()
            .any(|s| s.may.keys().any(|e| e.kind == Kind::Free)),
        "{table_effects:?}"
    );
    assert!(
        table_effects.iter().any(|s| !s.returns.is_empty()),
        "{table_effects:?}"
    );
    let reserves: Vec<_> =
        c.pr.procs
            .iter()
            .enumerate()
            .filter(|(i, p)| {
                p.name.as_str() == "array_reserve"
                    && !p.is_poly
                    && !c.proc_has_note(ProcId(*i as u32), "GrowsArray")
            })
            .filter_map(|(i, _)| summaries.procs.get(&ProcId(i as u32)))
            .collect();
    assert!(!reserves.is_empty());
    assert!(
        reserves
            .iter()
            .all(|s| s.guaranteed.keys().any(|e| e.kind == Kind::Resize))
    );
}

#[test]
fn prefix_compaction_preserves_independent_allocation_effects() {
    for kind in [Kind::Resize, Kind::Free] {
        let mut effects = HashMap::default();
        let parent = Effect {
            target: Path {
                root: Root::Parameter(0),
                steps: vec![Step::Storage],
            },
            kind,
            guard: None,
        };
        let mut child = parent.clone();
        child.target.steps.extend([Step::Element(0), Step::Storage]);
        insert(&mut effects, child.clone(), Vec::new());
        insert(&mut effects, parent.clone(), Vec::new());
        assert!(effects.contains_key(&parent));
        assert!(
            effects.contains_key(&child),
            "an outer allocation does not subsume a nested allocation"
        );
    }
}

#[test]
fn intrinsic_memory_and_vector_search_have_no_release_effects() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String("#import \"Basic\"; #import \"String\"; probe_search :: (s: string) -> int { return find_index_from_left(s, #char \"x\"); }".into())]).unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    let summaries = Summaries::build(&c);
    let (id, _) =
        c.pr.procs
            .iter()
            .enumerate()
            .find(|(_, p)| p.name.as_str() == "probe_search")
            .unwrap();
    let bad: Vec<_> = summaries.procs[&ProcId(id as u32)]
        .may
        .iter()
        .filter(|(e, _)| matches!(e.kind, Kind::Free | Kind::Unknown))
        .map(|(e, chain)| {
            (
                e,
                chain
                    .iter()
                    .map(|s| {
                        if *s == Span::NONE {
                            "...".to_owned()
                        } else {
                            c.src.sources.snippet(*s).to_owned()
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    assert!(bad.is_empty(), "{bad:#?}");
}

#[test]
fn pointee_resources_preserve_distinct_header_ownership() {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    c.begin_sources(&[ProgramSource::String("release_handle :: (value: s32) {} @FreesArgument\nrelease_field :: (value: *s32) { release_handle(value.*); }".into())]).unwrap();
    c.finish_program().unwrap();
    let _ = c.ide_check_all();
    let summaries = Summaries::build(&c);
    let id =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "release_field")
            .unwrap();
    let summary = &summaries.procs[&ProcId(id as u32)];
    assert!(
        summary
            .guaranteed
            .keys()
            .any(|e| e.kind == Kind::Free && e.target.steps == [Step::Deref]),
        "{summary:?}"
    );
}

#[test]
fn allocator_epoch_requires_returned_context_storage_after_ir_release() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let stdlib = root.join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = true;
    c.ed.record(facts);
    let mut source =
        std::fs::read_to_string(root.join("tests/lint/pointer_lifetime/strict/return-origins.jai"))
            .unwrap();
    source.push_str(r"
independent :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .CAPS return cast(*void) 0x0400_0000;
    return null;
}

incidental :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .CAPS return cast(*void) 0x0400_0000;
    unused := context.temporary_storage;
    return context.default_allocator.proc(mode, size, old_size, old_memory, data);
}

context_owned :: (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, data: *void) -> *void {
    if mode == .CAPS return cast(*void) 0x0400_0000;
    return context.temporary_storage.data;
}
");
    c.begin_sources(&[ProgramSource::String(source)]).unwrap();
    c.finish_program().unwrap();
    let diagnostics = c.ide_check_all();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let before = Summaries::build(&c);
    assert!(!c.ed.facts().unwrap().lifetime_globals.is_empty());
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(before.procs, after.procs);
    let names: HashSet<_> = after
        .allocators
        .temporary
        .iter()
        .map(|p| c.pr.procs[p.0 as usize].name.as_str())
        .collect();
    assert!(names.contains("temporary_allocator_proc"));
    assert!(names.contains("context_owned"), "{names:?}");
    assert!(!names.contains("incidental"));
    assert!(!names.contains("independent"));
    assert!(!after.allocators.heap_functions.is_empty());
}
