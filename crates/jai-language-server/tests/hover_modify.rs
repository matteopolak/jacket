//! A procedure's hover names its `#modify` block without the block's code, however the block
//! is laid out.
use test_support::lsp::{at, open, uri};

const PROGRAM: &str = "one_line :: ($n: s64) -> s64 #modify { return n > 0; } { return n; }

split :: ($n: s64) -> s64 #modify {
    return n > 0;
} {
    return n;
}

main :: () { x := one_line(1) + split(2); }
";

#[test]
fn a_modify_block_is_named_not_quoted() {
    let s = open(PROGRAM);
    for name in ["one_line ::", "split ::"] {
        let hover = s.hover(&uri(), at(PROGRAM, name)).unwrap().expect("hover");
        let text = hover.contents.value;
        assert!(text.contains("-> s64 #modify"), "{text}");
        assert!(!text.contains("return"), "{text}");
    }
}
