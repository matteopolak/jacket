//! A body's definitions stay as many as its distinct sources, however often it repeats a use.
use super::*;

#[test]
fn taking_one_places_address_again_adds_no_definition() {
    let calls: String = (0..40).map(|i| format!("    touch(*s, {i});\n")).collect();
    let source = format!(
        "Pair :: struct {{ count: int; }}\n\
         touch :: (s: *Pair, k: int) {{ s.count += k; }}\n\
         work :: () {{\n    s: Pair;\n{calls}}}\n\
         main :: () {{ work(); }}\n"
    );
    let (c, all) = releases::summaries(&source);
    let id = c.pr.procs.iter().position(|p| p.name.as_str() == "work");
    let body = &c.ed.facts().unwrap().lifetime_bodies[&ProcId(id.unwrap() as u32)];
    let prepared = Prepared::for_body(&c, body, &all);
    // Every query about the place walks its definitions: 40 copies would make a long body
    // cost the square of its length.
    assert!(
        prepared.most_definitions() <= 3,
        "{}",
        prepared.most_definitions()
    );
}
