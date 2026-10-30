//! A `for_expansion` whose body expands itself twice must not keep the editor's recovering check
//! busy once the expansion depth limit is reached: every expansion inside that chain fails at
//! once instead of doubling the work at each level.
use std::sync::mpsc;
use std::time::Duration;
use test_support::lsp::{session, uri};

const PROGRAM: &str = r"Range :: struct { first: int; count: int; }
Walk :: (range: *Range, body: Code, flags: For_Flags) #expand {
    for i: 0..range.count - 1 {
        `it_index := range.first + i;
        `it := i;
        for: Walk range { }
        for: Walk range { }
        #insert body;
    }
}
main :: () {
    range := Range.{ 1, 3 };
    total := 0;
    for: Walk range total += it;
}
";

#[test]
fn a_self_expanding_for_expansion_stops_at_the_depth_limit() {
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let mut s = session();
            s.open(uri(), 1, PROGRAM.into()).unwrap();
            let diagnostics = s.diagnostics(&uri()).unwrap();
            let _ = tx.send(
                diagnostics
                    .iter()
                    .any(|d| d.message.contains("too deeply nested")),
            );
        })
        .unwrap();
    let reported = rx
        .recv_timeout(Duration::from_secs(20))
        .expect("the full check returns");
    assert!(reported, "the depth limit is reported");
}
