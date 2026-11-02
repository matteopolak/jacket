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
