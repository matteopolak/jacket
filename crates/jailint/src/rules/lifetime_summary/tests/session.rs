//! Prepared facts must retain the compiler/body that produced their memory epochs.
use super::*;

#[test]
fn prepared_body_identity_cannot_be_repaired_with_a_different_compiler() {
    let first =
        stdlib_allocations::checked_source(r"pick::(p:*void,q:*void)->*void{return p;} main::(){}");
    let second =
        stdlib_allocations::checked_source(r"pick::(p:*void,q:*void)->*void{return q;} main::(){}");
    let first_all = Summaries::build(&first);
    let second_all = Summaries::build(&second);
    fn body(c: &Compiler) -> &LifetimeBody {
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "pick")
            .unwrap()
    }
    let first_body = body(&first);
    let second_body = body(&second);
    assert!(!std::ptr::eq(first_body, second_body));
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&first)
            .body((first_body).procedure)
            .unwrap(),
        &first_all,
    );
    let resolver = Resolver::for_prepared(&prepared, &second_all);
    assert!(
        std::ptr::eq(resolver.body, first_body),
        "Prepared's actual captured-load epochs must retain their original body"
    );
    assert!(
        std::ptr::eq(resolver.compiler, &raw const first),
        "a second compiler cannot rebind prepared SSA/type identities"
    );
}

#[test]
fn checked_session_selects_only_current_keyed_retained_bodies_across_release() {
    use crate::rules::lifetime_facts::CheckedSession;
    let mut c = stdlib_allocations::checked_source(r"pick::(p:*void)->*void{return p;} main::(){}");
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "pick")
            .unwrap() as u32,
    );
    assert!(CheckedSession::new(&c).body(ProcId(u32::MAX)).is_none());
    let original = c.ed.facts().unwrap().lifetime_bodies[&id].clone();
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&id)
        .unwrap()
        .procedure = ProcId(u32::MAX);
    assert!(CheckedSession::new(&c).body(id).is_none());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .insert(id, original.clone());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&id)
        .unwrap()
        .syntax = std::sync::Arc::new((*original.syntax).clone());
    assert!(CheckedSession::new(&c).body(id).is_none());
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .insert(id, original);
    assert!(CheckedSession::new(&c).body(id).is_some());
    c.release_lowered_code();
    assert!(CheckedSession::new(&c).body(id).is_some());
    c.ed.facts_mut().unwrap().lifetime_bodies.remove(&id);
    assert!(CheckedSession::new(&c).body(id).is_none());
}

#[test]
fn captured_index_cannot_pair_with_another_prepared_epoch_map() {
    let c = stdlib_allocations::checked_source(
        r"
        first::(p:*void,q:*void)->*void{saved:=p;return saved;}
        second::(p:*void,q:*void)->*void{saved:=q;return saved;}
        main::(){}
    ",
    );
    let all = Summaries::build(&c);
    let session = crate::rules::lifetime_facts::CheckedSession::new(&c);
    let key = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    let a = Prepared::for_body(session.body(key("first")).unwrap(), &all);
    let b = Prepared::for_body(session.body(key("second")).unwrap(), &all);
    let first = Resolver::for_prepared(&a, &all);
    let second = Resolver::for_prepared(&b, &all);
    let incoming = first
        .body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Parameter {
                incoming,
                formal: 0,
                ..
            } => Some(*incoming),
            _ => None,
        })
        .unwrap();
    let mut work = 8192;
    let index = first.capture_index(&mut work).unwrap();
    assert!(index.pointer_leaf(&first, incoming, &mut work, 0).is_some());
    assert!(
        index
            .pointer_leaf(&second, incoming, &mut work, 0)
            .is_none(),
        "same numeric SSA cannot authenticate another body's captures"
    );
}
