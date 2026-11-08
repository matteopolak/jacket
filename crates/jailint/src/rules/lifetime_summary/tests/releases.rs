//! A body that frees storage reachable from a parameter releases that argument.
use super::*;

fn summaries(source: &str) -> (Compiler, Summaries) {
    let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
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
    assert!(c.ide_check_all().is_empty());
    let all = Summaries::build(&c);
    (c, all)
}

fn summary<'a>(c: &Compiler, all: &'a Summaries, name: &str) -> &'a Summary {
    let id =
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == name)
            .unwrap();
    &all.procs[&ProcId(id as u32)]
}

fn contents(c: &Compiler, all: &Summaries, name: &str) -> Option<(Vec<String>, bool)> {
    let released = summary(c, all, name).released_contents(0)?;
    let fields = released
        .fields
        .map(|f| f.iter().map(|n| n.as_str().to_string()).collect())
        .unwrap_or_default();
    Some((fields, released.reset))
}

#[test]
fn bodies_release_what_their_argument_owns() {
    let (c, all) = summaries(
        r#"
        #import "Basic";
        Holder :: struct { name: string; items: [..] int; other: int; ptr: *u8; allocator: Allocator; }
        rel_name :: (h: *Holder) { free(h.name); }
        rel_reset :: (h: *Holder) { free(h.name); h.name = ""; }
        rel_items :: (h: *Holder) { array_reset(*h.items); }
        rel_both :: (h: *Holder) { rel_name(h); array_reset(*h.items); h.* = .{}; }
        rel_via :: (h: *Holder) { rel_name(h); }
        rel_allocator :: (h: *Holder) { h.allocator.proc(.FREE, 0, 0, h.ptr, h.allocator.data); }
        rel_conditional :: (h: *Holder, c: bool) { if c free(h.name); }
        rel_temp :: (h: *Holder) { t := copy_string("x"); free(t); }
        rel_whole :: (h: *Holder) { free(h); }
        main :: () {}
    "#,
    );
    let strings = |names: &[&str]| names.iter().map(|n| n.to_string()).collect::<Vec<_>>();
    assert_eq!(
        contents(&c, &all, "rel_name"),
        Some((strings(&["name"]), false))
    );
    assert_eq!(
        contents(&c, &all, "rel_via"),
        Some((strings(&["name"]), false))
    );
    assert_eq!(
        contents(&c, &all, "rel_items"),
        Some((strings(&["items"]), true))
    );
    assert_eq!(
        contents(&c, &all, "rel_allocator"),
        Some((strings(&["ptr"]), false))
    );
    // Freed and then written over: released, and usable again.
    assert_eq!(
        contents(&c, &all, "rel_reset"),
        Some((strings(&["name"]), true))
    );
    let (fields, reset) = contents(&c, &all, "rel_both").unwrap();
    assert!(reset && fields.len() == 2, "{fields:?}");
    // Not on every path, or not the argument's: nothing to rely on.
    assert_eq!(contents(&c, &all, "rel_conditional"), None);
    assert_eq!(contents(&c, &all, "rel_temp"), None);
    // The argument itself is a plain release.
    assert_eq!(contents(&c, &all, "rel_whole"), None);
}
