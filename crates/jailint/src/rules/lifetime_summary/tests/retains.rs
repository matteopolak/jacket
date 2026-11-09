//! A body that leaves a parameter in a field of another parameter retains it.
use super::releases::{summaries, summary};
use super::*;

/// The retained fields of `name` as (holder parameter, member names, held parameter).
fn retained(c: &Compiler, all: &Summaries, name: &str) -> Vec<(usize, Vec<String>, usize)> {
    let mut found: Vec<_> = summary(c, all, name)
        .retains
        .iter()
        .filter_map(|(path, held)| {
            let Root::Parameter(holder) = path.root else {
                return None;
            };
            let members = path
                .steps
                .iter()
                .map(|s| match s {
                    Step::Field(name) => name.as_str().to_string(),
                    other => format!("{other:?}"),
                })
                .collect();
            Some((holder, members, *held))
        })
        .collect();
    found.sort();
    found
}

#[test]
fn stores_of_a_parameter_into_a_field_are_retained_through_wrappers() {
    let (c, all) = summaries(
        r#"
        #import "Basic";
        Node :: struct { value: int; }
        Registry :: struct { item: *Node; other: *Node; }
        keep :: (r: *Registry, n: *Node) { r.item = n; }
        keep_twice :: (r: *Registry, n: *Node) { keep(r, n); }
        keep_swapped :: (n: *Node, r: *Registry) { keep(r, n); }
        keep_both :: (r: *Registry, a: *Node, b: *Node) { r.item = a; r.other = b; }
        keep_if :: (r: *Registry, n: *Node, c: bool) { if c r.item = n; }
        keep_then_clear :: (r: *Registry, n: *Node) { r.item = n; r.item = null; }
        keep_other_local :: (r: *Registry) { n := New(Node); r.item = n; }
        main :: () {}
    "#,
    );
    let item = vec!["item".to_string()];
    assert_eq!(retained(&c, &all, "keep"), [(0, item.clone(), 1)]);
    assert_eq!(retained(&c, &all, "keep_twice"), [(0, item.clone(), 1)]);
    assert_eq!(retained(&c, &all, "keep_swapped"), [(1, item.clone(), 0)]);
    assert_eq!(
        retained(&c, &all, "keep_both"),
        [(0, item.clone(), 1), (0, vec!["other".to_string()], 2)]
    );
    // Not on every path, overwritten, or not a parameter: nothing to rely on.
    assert_eq!(retained(&c, &all, "keep_if"), []);
    assert_eq!(retained(&c, &all, "keep_then_clear"), []);
    assert_eq!(retained(&c, &all, "keep_other_local"), []);
}
