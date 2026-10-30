//! A trait struct whose field type fails must not make the editor's full check re-resolve that
//! type without end when a struct uses the trait with `#this`.
use std::sync::mpsc;
use std::time::Duration;
use test_support::lsp::{session, uri};

const PROGRAM: &str = r"SomeTrait :: struct(
    type: Type,
    set: (a: *type, int, int) - ()
) {}
ExSomeTrait :: struct {
    using #as _t: SomeTrait(#this, ex_set);
}
ex_set :: (f: *ExSomeTrait, i: int, val: int) {}
main :: () {}
";

#[test]
fn a_failed_trait_field_type_is_resolved_once_per_layout() {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let mut s = session();
            s.open(uri(), 1, PROGRAM.into()).unwrap();
            let _ = s.diagnostics(&uri());
            let _ = tx.send(());
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(20))
        .expect("the full check returns");
}
