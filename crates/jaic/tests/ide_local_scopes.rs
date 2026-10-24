//! Every local a program declares can be found from its own position: the editor looks names up
//! through the innermost scope recorded at a position, so a block checked without recording its
//! span hides its locals from completion (a `#run { }` block once offered to import a module's
//! `options` in place of the local being typed).
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::scope::{EntityKind, ScopeKind};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource, ScopeId};
use std::path::{Path, PathBuf};
use std::rc::Rc;

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// The locals of `entry` that the scope recorded at their own position cannot reach, as
/// `file:offset name`.
fn hidden_locals(entry: &Path) -> Vec<String> {
    let stdlib = repo().join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    c.ed.record(IdeFacts::new(vec![IdeRoot::under(entry)]));
    let _ = c
        .begin_sources(&[ProgramSource::File(entry.to_path_buf())])
        .and_then(|()| c.finish_program());
    let _ = c.ide_check_all();
    let Some(entry_file) = (0..c.src.files.len() as u32)
        .map(jaic::source::FileId)
        .find(|&f| c.src.files[f.0 as usize].path == entry.into())
    else {
        return Vec::new();
    };
    // A polymorph, a macro or a custom `for` is checked once per use, each copy with its own
    // scopes over the same text: any copy's local with the same name and declaration will do.
    let reaches = |from: ScopeId, local: &jaic::sema::scope::Entity| {
        let mut at = Some(from);
        while let Some(s) = at {
            let found = c
                .scope(s)
                .names
                .get(&local.name)
                .is_some_and(|ids| ids.iter().any(|&id| c.entity(id).span == local.span));
            if found {
                return true;
            }
            at = c.scope(s).parent;
        }
        false
    };
    let mut hidden = Vec::new();
    for e in &c.ns.entities {
        // Parameters belong to the procedure's own scope and are declared in its header,
        // outside the body they are visible in.
        if !matches!(e.kind, EntityKind::Local { .. })
            || c.scope(e.scope).kind != ScopeKind::Block
            || e.span.file != entry_file
            || e.span.end <= e.span.start
            || e.name.as_str().starts_with('\0')
        {
            continue;
        }
        let Some(at) = c.ide_scope_at(entry_file, e.span.end) else {
            continue;
        };
        if !reaches(at, e) {
            hidden.push(format!(
                "{}:{} {}",
                entry.display(),
                e.span.start,
                e.name.as_str()
            ));
        }
    }
    hidden
}

/// Runs `body` on a thread with room for deep checking.
fn big_stack(body: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(body)
        .unwrap()
        .join()
        .unwrap();
}

#[test]
fn locals_of_run_and_modify_blocks_are_in_scope_at_their_position() {
    big_stack(|| {
        let entry = repo().join("crates/jaic/tests/fixtures/ide-local-scopes.jai");
        assert_eq!(hidden_locals(&entry), Vec::<String>::new());
    });
}
