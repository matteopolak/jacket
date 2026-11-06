//! A separate test binary isolates the compiler's process-wide cancellation countdown.
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

#[test]
fn retained_execution_cancellation_after_initial_validation_drops_owned_storage() {
    for hold_reader in [false, true] {
        let stdlib = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
        let mut options = Options::host();
        options.import_paths = vec![stdlib.clone()];
        options.set_stdlib(&NativeFs, &stdlib);
        let mut c = Compiler::new(options, Rc::new(NativeFs));
        let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
        facts.lint = true;
        c.ed.record(facts);
        c.compile_sources(&[ProgramSource::String(
            "value :: () -> int {return 7;} VALUE :: #run value(); main :: () {}".into(),
        )])
        .unwrap();
        assert!(c.ide_check_all().is_empty());
        c.release_lowered_code_with_lifetime_execution(true);
        assert!(c.lifetime_execution_usage().0 > 0);
        let held = hold_reader.then(|| c.lifetime_execution().unwrap());
        let held_node = held.as_ref().map(|v| {
            (0..v.function_count())
                .map(|i| jaic::ir::FuncId(i as u32))
                .find(|id| v.function(*id).is_some())
                .unwrap()
        });
        jaic::interrupt::request(false);
        jaic::interrupt::arm();
        jaic::interrupt::stop_after(2);
        let available = c.lifetime_execution().is_some();
        let fired = jaic::interrupt::disarm();
        jaic::interrupt::stop_after(0);
        jaic::interrupt::request(false);
        assert!(
            !available && fired,
            "post-initial cancellation: certificate={available}, observed stop={fired}"
        );
        assert!(
            c.lifetime_execution().is_none(),
            "a cancelled certificate cannot resurrect while an earlier reader remains alive"
        );
        if let (Some(view), Some(id)) = (&held, held_node) {
            assert!(
                !view.visit_edges(jaic::ide::execution::Node::Function(id), |_| {}),
                "an invalidated reader cannot authorize traversal"
            );
        }
        drop(held);
        assert_eq!(c.lifetime_execution_usage(), (0, 0));
    }
}
