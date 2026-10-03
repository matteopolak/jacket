//! Exported macro cleanups retain definition bindings and caller lifetimes.
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new(source: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "jai-caller-defer-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("main.jai"), source).unwrap();
        Self(root)
    }
    fn program(&self) -> Result<jai_ir::Program, String> {
        let graph = jai_modules::ModuleGraph::load(&self.0.join("main.jai"), Default::default())
            .map_err(|error| error.to_string())?;
        jai_sema::resolve_graph_with_options(
            &graph,
            &jai_sema::ResolveOptions {
                layout: Some(jai_types::LayoutPolicy::lp64()),
                ..Default::default()
            },
            &mut jai_vm::NoEffects,
        )
        .map_err(|error| error.to_string())
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn check(source: &str) {
    let program = Fixture::new(source).program().unwrap();
    let outcome = jai_vm::execute(&program, jai_vm::Limits::default()).outcome;
    let jai_vm::Outcome::Complete(values) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(values[0].integer().unwrap().value(), 42);
}

#[test]
fn allocator_procedure_values_restore_at_caller_block_exit() {
    check(include_str!("fixtures/caller-defer-allocator.jai"));
}

#[test]
fn return_values_are_captured_before_reverse_order_caller_cleanups() {
    check(include_str!("fixtures/caller-defer-lifecycle.jai"));
}

#[test]
fn caller_context_cleanup_uses_the_actual_pushed_context() {
    check(include_str!("fixtures/caller-defer-context.jai"));
}

#[test]
fn nested_macros_keep_the_original_caller_cleanup_scope() {
    check(
        "total:int; register::()#expand{`defer total=42;} wrapper::()#expand{register();} main::()->int{{wrapper();if total!=0 return 1;}return total;}",
    );
}

#[test]
fn caller_cleanups_read_hygienic_macro_storage_at_cleanup_time() {
    check(
        "total:int; register::()#expand{old:=1;`defer total=old;old=42;} main::()->int{old:=100;{register();if total!=0 return 1;}return total+old-100;}",
    );
}

#[test]
fn continue_and_break_run_the_callers_iteration_cleanup() {
    check(
        "order:int; register::(digit:int)#expand{`defer order=order*10+digit;} main::()->int{for i:0..2{register(i+1);if i==0 continue;if i==1 break;}return order+30;}",
    );
}

#[test]
fn nested_runtime_macro_blocks_cannot_export_a_defer() {
    let error = Fixture::new(
        "total:int; register::()#expand{{`defer total=42;}} main::()->int{register();return 0;}",
    )
    .program()
    .unwrap_err();
    assert!(
        error.contains("top-level statement list of a macro"),
        "{error}"
    );
}

#[test]
fn caller_defer_requires_an_active_expansion() {
    let error = Fixture::new("main::()->int{`defer{}return 0;}")
        .program()
        .unwrap_err();
    assert!(error.contains("active #expand"), "{error}");
}

#[test]
fn caller_defer_does_not_fabricate_an_unavailable_context() {
    let error=Fixture::new("#add_context marker:int; register::()#expand{`defer context.marker=42;} main::()->int #no_context{register();return 0;}").program().unwrap_err();
    assert!(error.contains("context is unavailable"), "{error}");
}
