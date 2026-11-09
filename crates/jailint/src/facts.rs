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


/// Keep what the lowered code says about which procedures only compile-time code reaches, before
/// `Compiler::release_lowered_code` drops that code; lints answer from it afterwards.
pub fn retain_compile_time_reach(compiler: &mut Compiler) {
    crate::rules::retain_compile_time_reach(compiler);
}

/// Destructive profiling after all users of a compiler's lint facts have finished.
/// The example installs `CountingAllocator` and uses one worker.
pub(crate) fn profile_retained(compiler: &mut Compiler, root: &std::path::Path, complete: bool) {
    if !jacket_config::env::LIFETIME_MEMORY.is_set() {
        return;
    }
    if jaic::memory_limit::limit().is_none() {
        return;
    }
    let Some(mut facts) = compiler.ed.stop() else {
        return;
    };
    let bodies = facts.lifetime_bodies.len();
    let inline = 2 * std::mem::size_of::<jaic::fxhash::HashMap<usize, usize>>();
    let before = jaic::memory_limit::live();
    facts.clear_lifetimes();
    let middle = jaic::memory_limit::live();
    drop(facts);
    let after = jaic::memory_limit::live();
    if let Some(sidecar) = before
        .checked_sub(middle)
        .and_then(|b| b.checked_add(inline))
        && let Some(other) = middle
            .checked_sub(after)
            .and_then(|b| b.checked_sub(inline))
    {
        eprintln!(
            "lifetime-memory {}",
            serde_json::json!({"root": root, "complete": complete, "bodies": bodies, "sidecar_bytes": sidecar, "other_lint_fact_bytes": other})
        );
    } else {
        eprintln!(
            "lifetime-memory unstable allocation counters for {}",
            root.display()
        );
    }
}
