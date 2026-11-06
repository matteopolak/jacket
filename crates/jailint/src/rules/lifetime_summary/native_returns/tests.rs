//! Native proof authentication keeps its original identity and fuel gates.
use super::*;
use crate::rules::lifetime_summary::tests::stdlib_allocations::checked_source;
fn checked() -> (Compiler, super::super::Summaries, [ProcId; 4]) {
    let c = checked_source(
        r#"
            C::#system_library "libc";
            auth_budget_native::(n:u64)->*void #foreign C "malloc";
            auth_budget_leaf::(n:u64)->*void{return auth_budget_native(n);}
            auth_budget_parent::(n:u64)->*void{return auth_budget_leaf(n);}
            auth_budget_top::(n:u64)->*void{return auth_budget_parent(n);}
            main::(){}
        "#,
    );
    let ids = [
        "auth_budget_native",
        "auth_budget_leaf",
        "auth_budget_parent",
        "auth_budget_top",
    ]
    .map(|name| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    });
    let all = super::super::Summaries::build(&c);
    (c, all, ids)
}

#[test]
fn authentication_charges_shared_child_breadth_and_rejects_stale_deep_leaf() {
    let (mut c, mut all, ids) = checked();
    let proof = all.procs[&ids[3]]
        .native_allocation
        .as_ref()
        .unwrap()
        .clone();
    assert!(proof.current(&c, ids[3], &all.procs, 0, &Work(Cell::new(0))));
    let limited = Work(Cell::new(MAX_WORK - 1));
    assert!(!proof.current(&c, ids[3], &all.procs, 0, &limited));
    assert!(limited.0.get() > MAX_WORK);
    for id in &ids[1..] {
        let mut expanded = (**all.procs[id].native_allocation.as_ref().unwrap()).clone();
        let Allocation::Checked(checked) = &mut expanded else {
            panic!("checked wrapper")
        };
        let path = checked.cases[0].clone();
        checked.cases.resize(MAX_PATHS, path);
        all.procs.get_mut(id).unwrap().native_allocation = Some(std::rc::Rc::new(expanded));
    }
    let proof = all.procs[&ids[3]].native_allocation.as_ref().unwrap();
    let broad = Work(Cell::new(0));
    assert!(!proof.current(&c, ids[3], &all.procs, 0, &broad));
    assert!(
        (MAX_WORK + 1..=MAX_WORK + MAX_NODES + 67).contains(&broad.0.get()),
        "stop on first exhausted visit: {}",
        broad.0.get()
    );
    let jaic::sema::procs::ProcTarget::Foreign(foreign) =
        c.pr.procs[ids[0].0 as usize].target.unwrap()
    else {
        panic!("selected native")
    };
    c.out.program.foreigns[foreign.0 as usize].is_data = true;
    let stale = Work(Cell::new(0));
    assert!(!proof.current(&c, ids[3], &all.procs, 0, &stale));
    assert!(
        stale.0.get() < MAX_WORK,
        "deep ABI rejection precedes broad exhaustion"
    );
}

#[test]
fn completed_native_observation_does_not_publish_after_cancellation() {
    if !super::super::tests::cancellation_child() {
        return;
    }
    let (c, all, ids) = checked();
    let body = &c.ed.facts().unwrap().lifetime_bodies[&ids[1]];
    let prepared = super::super::Prepared::build_requested(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(body.procedure)
            .unwrap(),
        &all.procs,
        &all.targets,
        true,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let mut observer = Builder::new(&r, &all.procs, false).unwrap();
    for (block, b) in body.blocks.iter().enumerate() {
        for (event, e) in b.events.iter().enumerate() {
            observer.observe(
                &r,
                Site {
                    block,
                    event,
                },
                e,
                &all.procs,
            );
        }
    }
    struct Reset;
    impl Drop for Reset {
        fn drop(&mut self) {
            jaic::interrupt::disarm();
            jaic::interrupt::request(false);
        }
    }
    let _reset = Reset;
    jaic::interrupt::arm();
    jaic::interrupt::request(true);
    assert!(observer.finish(&r, &all.procs).is_none());
    assert!(jaic::interrupt::disarm());
}
