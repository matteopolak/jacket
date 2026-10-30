//! Turning on what the linter reads: the rules look facts up through `jaic::ide`'s queries
//! (`Compiler::ide_entities_at` and the others) and the records of expression types and casts.
use jaic::ide::IdeFacts;
use jaic::sema::Compiler;

/// Record lint facts for the files under `roots` (an existing editor
/// record is extended).
pub fn enable(compiler: &mut Compiler, roots: Vec<jaic::ide::IdeRoot>) {
    if !compiler.ed.is_live() {
        compiler.ed.record(IdeFacts::new(roots));
    }
    if let Some(ide) = compiler.ed.facts_mut() {
        ide.lint = true;
    }
}
