//! Runtime conditional values retain their selected branches and captured result slot.
use super::{bodies, compile, events};
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event};
use jaic::ir::Val;
use jaic::sema::Compiler;
use jaic::types::TypeId;

const SOURCE: &str = r#"
    #import "Basic";
    D::#import "Default_Allocator";
    left::()->s64{return 56;}
    right::()->s64{return 0;}
    selected::(flag:bool)->s64{return ifx flag then 56 else 0;}
    effectful::(flag:bool)->s64{return ifx flag then left() else right();}
    missing::(flag:bool)->s64{return ifx flag then 56;}
    folded::()->s64{return ifx true then 56 else 0;}
    allocation::()->*void{return D.allocator_proc(.ALLOCATE,64,0,null,null);}
    main::(){flag:bool;present:=#exists(ifx flag then 1 else 2);}
"#;

fn body<'a>(c: &'a Compiler, name: &str) -> &'a LifetimeBody {
    bodies(c)
        .find(|b| c.proc(b.procedure).name.as_str() == name)
        .unwrap()
}

fn canonical(b: &LifetimeBody, value: Val) -> Val {
    b.slot_aliases
        .iter()
        .find_map(|(alias, root)| (*alias == value).then_some(*root))
        .unwrap_or(value)
}

#[test]
fn runtime_ifx_retains_selected_branch_and_effectful_arm_order() {
    let mut c = compile(SOURCE, true, false);
    let snapshot = |c: &Compiler| {
        let mut rows = Vec::new();
        for name in ["selected", "effectful", "missing", "obtain_contents"] {
            let b = body(c, name);
            let branches: Vec<_> = b
                .blocks
                .iter()
                .enumerate()
                .flat_map(|(block, bb)| {
                    bb.events.iter().filter_map(move |e| match e {
                        Event::Branch {
                            condition,
                            then_block,
                            else_block,
                            span,
                        } if c.src.sources.snippet(*span)
                            == if name == "obtain_contents" {
                                "wide"
                            } else {
                                "flag"
                            } =>
                        {
                            Some((block, *condition, *then_block, *else_block))
                        }
                        _ => None,
                    })
                })
                .collect();
            assert_eq!(branches.len(), 1, "{name}: actual runtime ifx branch");
            let &(block, _, yes, no) = branches.first().unwrap();
            assert_eq!(b.blocks[block].successors, vec![yes, no]);
            if name == "effectful" {
                for (arm, callee) in [(yes, "left"), (no, "right")] {
                    let calls: Vec<_> = b.blocks[arm]
                        .events
                        .iter()
                        .filter_map(|e| match e {
                            Event::Call {
                                callee, ..
                            } => Some(c.proc(*callee).name.as_str()),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(calls, vec![callee]);
                }
            }
            rows.push((b.procedure, branches));
        }
        rows
    };
    let before = snapshot(&c);
    c.release_lowered_code();
    assert_eq!(snapshot(&c), before);
}

#[test]
fn runtime_ifx_result_load_captures_the_actual_shared_slot() {
    let mut c = compile(SOURCE, true, false);
    let snapshot = |c: &Compiler| {
        let mut rows = Vec::new();
        for name in ["selected", "effectful", "missing"] {
            let b = body(c, name);
            let result = events(b)
                .find_map(|e| match e {
                    Event::Return {
                        index: 0,
                        value,
                        ty: TypeId::S64,
                        ..
                    } => Some(*value),
                    _ => None,
                })
                .unwrap();
            let loads: Vec<_> = events(b)
                .filter_map(|e| match e {
                    Event::Load {
                        place,
                        value,
                        ty: TypeId::S64,
                        ..
                    } if *value == result => Some(*place),
                    _ => None,
                })
                .collect();
            assert_eq!(loads.len(), 1, "{name}: actual scalar ifx result load");
            let root = canonical(b, loads[0]);
            assert!(
                b.stack_roots
                    .iter()
                    .any(|r| r.value == root && r.bytes == Some(8))
            );
            let writes: Vec<_> = events(b)
                .filter_map(|e| match e {
                    Event::Store {
                        place,
                        ty: TypeId::S64,
                        ..
                    }
                    | Event::DefaultImage {
                        place,
                        ty: TypeId::S64,
                        ..
                    } if canonical(b, *place) == root => Some(*place),
                    _ => None,
                })
                .collect();
            assert_eq!(
                writes.len(),
                2,
                "both actual arms write the same physical result slot"
            );
            rows.push((b.procedure, root, loads, writes));
        }
        let b = body(c, "obtain_contents");
        assert!(
            events(b).any(|e| matches!(e,Event::Load{ty:TypeId::S64,span,..}
            if c.src.sources.snippet(*span)=="ifx wide then SLACK else 0")),
            "the real initial allocation request captures its selected padding"
        );
        rows
    };
    let before = snapshot(&c);
    c.release_lowered_code();
    assert_eq!(snapshot(&c), before);
}

#[test]
fn constant_and_type_only_ifx_add_no_runtime_branch_or_result_load() {
    let c = compile(SOURCE, true, false);
    for name in ["folded", "main"] {
        let b = body(&c, name);
        assert!(
            !events(b).any(|e| matches!(e, Event::Branch { .. })),
            "{name}"
        );
        assert!(
            !events(b).any(|e| matches!(e,Event::Load{span,..}
            if c.src.sources.snippet(*span).starts_with("ifx"))),
            "{name}"
        );
    }
    let c = compile(SOURCE, false, false);
    assert!(bodies(&c).next().is_none());
}
