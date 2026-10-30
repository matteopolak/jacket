//! The lookups by source span (`Compiler::ide_entities_at` and the others): each answers from
//! the record of one small program, and none of them sees what a sketch of a polymorphic
//! procedure recorded.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::scope::EntityKind;
use jaic::sema::{Compiler, NativeFs, Options, ProcId, ProgramSource};
use jaic::source::{FileId, Span};
use std::rc::Rc;

const SOURCE: &str = "\
main :: () {
    r := twice(3);
}

twice :: (x: int) -> int {
    y := x * 2;
    return y;
}

broken :: () {
    z: int = \"text\";
}

unused_poly :: (a: $T) -> T {
    return a;
}
";

struct Program {
    compiler: Compiler,
    file: FileId,
}

fn new_compiler() -> Compiler {
    let stdlib = test_support::repo::root().join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    Compiler::new(options, Rc::new(NativeFs))
}

fn compile(sketch: bool) -> Program {
    let mut compiler = new_compiler();
    let mut ide = IdeFacts::new(vec![IdeRoot::Virtual]);
    ide.lint = true;
    compiler.ed.record(ide);
    let started = compiler
        .begin_sources(&[ProgramSource::String(SOURCE.to_owned())])
        .and_then(|()| compiler.finish_program());
    drop(started);
    let _ = compiler.ide_check_all();
    if sketch {
        compiler.ide_sketch_polymorphs();
    }
    let file = (0..compiler.src.sources.len() as u32)
        .map(FileId)
        .find(|&f| &*compiler.src.sources.get(f).text == SOURCE)
        .expect("the program's file");
    Program {
        compiler,
        file,
    }
}

impl Program {
    /// The span of the `n`th (from zero) occurrence of `text`.
    fn span(&self, text: &str, n: usize) -> Span {
        let start = SOURCE.match_indices(text).nth(n).expect("occurrence").0 as u32;
        Span {
            file: self.file,
            start,
            end: start + text.len() as u32,
        }
    }

    /// The span of `word` inside the first occurrence of `context`.
    fn within(&self, context: &str, word: &str) -> Span {
        let outer = self.span(context, 0);
        let start = outer.start + context.find(word).expect("word in context") as u32;
        Span {
            start,
            end: start + word.len() as u32,
            ..outer
        }
    }

    /// The first procedure declared as `name :: (...)`.
    fn proc_named(&self, name: &str) -> ProcId {
        let declared = format!("{name} :: ");
        let at = self
            .compiler
            .pr
            .procs
            .iter()
            .position(|p| {
                let header = p.lit.header.span;
                header.file == self.file && SOURCE[..header.start as usize].ends_with(&declared)
            })
            .unwrap_or_else(|| panic!("no procedure {name}"));
        ProcId(at as u32)
    }
}

#[test]
fn a_call_finds_its_procedure_and_a_procedure_its_calls() {
    let p = compile(false);
    let c = &p.compiler;
    let call = p.span("twice", 0);
    // The declaration's own name is not a use.
    assert!(c.ide_entities_at(p.span("twice", 1)).is_empty());
    assert!(c.ide_procs_named_at(p.span("twice", 1)).is_empty());

    let twice = p.proc_named("twice");
    assert_eq!(&*c.ide_procs_named_at(call), &[twice]);
    assert_eq!(&*c.ide_uses_of_proc(twice), &[call]);
    assert!(c.ide_uses_of_proc(p.proc_named("broken")).is_empty());
    assert!(c.ide_procs_named_at(p.span("main", 0)).is_empty());
}

#[test]
fn a_header_finds_the_procedures_made_from_it() {
    let p = compile(false);
    let c = &p.compiler;
    let twice = p.proc_named("twice");
    let header = c.pr.procs[twice.0 as usize].lit.header.span;
    assert_eq!(&*c.ide_procs_with_header(header), &[twice]);
    assert!(c.ide_procs_with_header(p.span("twice", 1)).is_empty());
}

#[test]
fn a_local_is_found_by_the_span_of_its_declaration() {
    let p = compile(false);
    let c = &p.compiler;
    let y = p.span("y", 0);
    let locals = c.ide_locals_declared_at(y);
    assert_eq!(locals.len(), 1);
    let local = c.entity(locals[0]);
    assert_eq!(local.name.as_str(), "y");
    assert!(matches!(local.kind, EntityKind::Local { .. }));
    // The use in `return y` resolves to that local.
    assert_eq!(&*c.ide_entities_at(p.within("return y", "y")), &*locals);
    assert!(c.ide_locals_declared_at(p.span("twice", 0)).is_empty());
}

#[test]
fn only_a_body_checked_to_the_end_is_clean() {
    let p = compile(false);
    let c = &p.compiler;
    let body = |name: &str| {
        c.pr.procs[p.proc_named(name).0 as usize]
            .lit
            .body
            .as_ref()
            .unwrap()
            .span
    };
    assert!(c.ide_body_is_clean(body("twice")));
    assert!(!c.ide_body_is_clean(body("broken")));
    assert!(!c.ide_body_is_clean(p.span("twice", 1)));
}

#[test]
fn what_a_sketch_recorded_is_left_out() {
    let plain = compile(false);
    let sketched = compile(true);
    assert!(
        sketched.compiler.pr.procs.len() > plain.compiler.pr.procs.len(),
        "the sketch should have made an instance"
    );
    for p in [&plain, &sketched] {
        let c = &p.compiler;
        let header = c.pr.procs[p.proc_named("unused_poly").0 as usize]
            .lit
            .header
            .span;
        assert_eq!(c.ide_procs_with_header(header).len(), 1);
        let body = c.pr.procs[p.proc_named("unused_poly").0 as usize]
            .lit
            .body
            .as_ref()
            .unwrap()
            .span;
        assert!(!c.ide_body_is_clean(body));
    }
    // The lookups are built before and after the sketch from different extents of the record.
    let mut c = compile(false).compiler;
    let header = c.pr.procs[0].lit.header.span;
    let before = c.ide_procs_with_header(header).len();
    c.ide_sketch_polymorphs();
    assert_eq!(c.ide_procs_with_header(header).len(), before);
}

#[test]
fn nothing_recorded_means_nothing_found() {
    let mut c = new_compiler();
    let _ = c
        .begin_sources(&[ProgramSource::String(SOURCE.to_owned())])
        .and_then(|()| c.finish_program());
    let at = Span {
        file: FileId(0),
        start: 0,
        end: 5,
    };
    assert!(c.ide_entities_at(at).is_empty());
    assert!(c.ide_locals_declared_at(at).is_empty());
    assert!(c.ide_procs_with_header(at).is_empty());
    assert!(c.ide_procs_named_at(at).is_empty());
    assert!(c.ide_uses_of_proc(ProcId(0)).is_empty());
    assert!(!c.ide_body_is_clean(at));
}
