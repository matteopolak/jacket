//! `context.stack_trace` as a program sees it while the interpreter runs it: the chain's names,
//! call lines and depths, and its restoration when a call returns.
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::{path::PathBuf, rc::Rc};

fn check(source: &str) {
    let stdlib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    if let Err(e) = c.compile_sources(&[ProgramSource::String(source.into())]) {
        panic!("{}", e.render(&c.src.sources));
    }
}

#[test]
fn the_chain_names_lines_and_depths_of_the_active_calls() {
    check(
        r#"
leaf :: () -> s64 {
    n := context.stack_trace;
    if n.info.name != "leaf" return -1;
    if n.next.info.name != "middle" return -2;
    if n.next.next.info.name != "outer" return -3;
    if n.call_depth != n.next.call_depth + 1 return -4;
    if n.next.call_depth != n.next.next.call_depth + 1 return -5;
    return n.next.line_number;
}
middle :: () -> s64 {
    r := leaf();
    return r;
}
outer :: () -> s64 {
    before := context.stack_trace;
    depth := before.call_depth;
    r := middle();
    after := context.stack_trace;
    if after != before return -10;
    if after.call_depth != depth return -11;
    if after.info.name != "outer" return -12;
    return r;
}
// The call to `leaf` in `middle` is on line 12 of this source.
#assert outer() == 12;
main :: () {}
"#,
    );
}

#[test]
fn recursion_deepens_the_chain_one_node_per_call() {
    check(
        r#"
dive :: (n: s64) -> s64 {
    top := context.stack_trace;
    if n == 0 {
        count := 0;
        node := top;
        while node != null {
            if node.info.name == "dive" count += 1;
            node = node.next;
        }
        return count;
    }
    r := dive(n - 1);
    if context.stack_trace != top return -1;
    return r;
}
#assert dive(40) == 41;
main :: () {}
"#,
    );
}

#[test]
fn a_site_always_hashes_the_same_and_sites_differ() {
    check(
        r#"
probe :: () -> u64 { return context.stack_trace.hash; }
first :: () -> u64 { return probe(); }
second :: () -> u64 { return probe(); }
twice :: () -> bool {
    hashes: [3] u64;
    for 0..1 { hashes[it] = first(); }
    hashes[2] = second();
    return hashes[0] == hashes[1] && hashes[0] != hashes[2];
}
#assert twice();
main :: () {}
"#,
    );
}
