//! Exact machine-bit returns never imply live allocation ranges.
use super::stdlib_allocations::checked_source;
use super::*;

#[test]
fn aligned_prefix_cast_checks_stay_at_the_selected_call_boundary() {
    for mode in [0, 1, 2] {
        let mut options = Options::host();
        options.cast_bounds_check = mode;
        let mut c = super::stdlib_allocations::checked_source_options(
            r#"
            D::#import "Default_Allocator";
            driver::()->*void{return D.allocator_proc(.ALLOCATE,64,0,null,null);}
            main::(){}
        "#,
            options,
        );
        let snapshot = |c: &Compiler| {
            let bodies = &c.ed.facts().unwrap().lifetime_bodies;
            let selected = |name: &str| {
                bodies
                    .values()
                    .find(|b| {
                        let p = c.proc(b.procedure);
                        p.name.as_str() == name
                            && c.src
                                .sources
                                .get(p.span.file)
                                .path
                                .as_file()
                                .is_some_and(|p| {
                                    p.ends_with("internal/allocation/aligned-blocks.jai")
                                })
                    })
                    .unwrap()
            };
            let helper = selected("contents_in");
            assert_eq!(
                c.proc(helper.procedure).sig.as_ref().unwrap().params[1].ty,
                TypeId::U64
            );
            assert!(
                !helper
                    .blocks
                    .iter()
                    .flat_map(|b| &b.events)
                    .any(|e| matches!(e, Event::Barrier))
            );
            let all = Summaries::build(c);
            let prepared = Prepared::for_body(
                crate::rules::lifetime_facts::CheckedSession::new(c)
                    .body((helper).procedure)
                    .unwrap(),
                &all,
            );
            let r = Resolver::for_prepared(&prepared, &all);
            assert!(
                r.core().scalar_requests.is_some(),
                "actual bit helper uses the existing scalar capture admission"
            );
            let mut retained = Vec::new();
            for name in ["obtain_contents", "regrow_contents"] {
                let parent = selected(name);
                let checks: Vec<_> = parent
                    .blocks
                    .iter()
                    .enumerate()
                    .flat_map(|(block, b)| {
                        b.events.iter().filter_map(move |e| match e {
                            Event::Branch {
                                span,
                                then_block,
                                else_block,
                                ..
                            } if c.src.sources.snippet(*span) == "cast(u64) prefix" => {
                                Some((block, *then_block, *else_block))
                            }
                            _ => None,
                        })
                    })
                    .collect();
                assert_eq!(checks.len(), usize::from(mode != 0), "{name} mode{mode}");
                if let Some(&(block, fail, pass)) = checks.first() {
                    let condition = parent.blocks[block]
                        .events
                        .iter()
                        .find_map(|e| match e {
                            Event::Branch {
                                condition,
                                span,
                                ..
                            } if c.src.sources.snippet(*span) == "cast(u64) prefix" => {
                                Some(*condition)
                            }
                            _ => None,
                        })
                        .unwrap();
                    assert!(
                        parent.comparisons.iter().any(|cmp| cmp.value == condition
                            && cmp.ty == jaic::ir::Ty::I64
                            && cmp.op == jaic::ir::CmpOp::SLt
                            && cmp.right_constant == Some(0)),
                        "negative signed prefixes still reach the failure callback"
                    );
                    assert!(
                        parent.blocks[fail]
                            .events
                            .iter()
                            .any(|e| matches!(e, Event::Barrier))
                    );
                    assert!(
                        parent.blocks[pass].events.iter().any(
                            |e| matches!(e,Event::Call{callee,..} if *callee==helper.procedure)
                        )
                    );
                    if name == "regrow_contents" {
                        assert!(parent.blocks.iter().flat_map(|b|&b.events).any(|e|matches!(e,
                            Event::Branch{span,then_block,..} if *then_block==block && c.src.sources.snippet(*span)=="wide")));
                    }
                }
                retained.push((parent.procedure, checks));
            }
            retained
        };
        let before = snapshot(&c);
        c.release_lowered_code();
        assert_eq!(snapshot(&c), before);
    }
}


#[test]
fn actual_aligned_allocations_keep_payload_bytes_across_width_changes() {
    let _c = checked_source(
        r#"
        D::#import "Default_Allocator";
        #import "Basic";
        check_size::(size:s64) {
            memory:=D.allocator_proc(.ALLOCATE,size,0,null,null);
            assert(memory!=null);
            if size>=64 assert(cast(u64)memory%64==0);
            bytes:=cast(*u8)memory;
            bytes[0]=17;bytes[size-1]=29;
            assert(bytes[0]==17 && bytes[size-1]==29);
            D.allocator_proc(.FREE,0,size,memory,null);
        }
        exercise::() {
            check_size(8);check_size(64);check_size(65);check_size(128);
            memory:=D.allocator_proc(.ALLOCATE,8,0,null,null);
            assert(memory!=null);
            (cast(*u8)memory)[0]=41;
            wide:=D.allocator_proc(.RESIZE,128,8,memory,null);
            assert(wide!=null && cast(u64)wide%64==0);
            assert((cast(*u8)wide)[0]==41);
            small:=D.allocator_proc(.RESIZE,16,128,wide,null);
            assert(small!=null && (cast(*u8)small)[0]==41);
            D.allocator_proc(.FREE,0,16,small,null);
        }
        #run exercise();
        main::(){}
    "#,
    );
}


#[test]
fn actual_aligned_bit_helper_has_a_complete_machine_return_relation() {
    let c = checked_source(
        r#"
        D::#import "Default_Allocator";
        driver::()->*void{return D.allocator_proc(.ALLOCATE,64,0,null,null);}
        main::(){}
    "#,
    );
    let all = Summaries::build(&c);
    let helper =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| {
                let p = c.proc(b.procedure);
                p.name.as_str() == "contents_in"
                    && c.src
                        .sources
                        .get(p.span.file)
                        .path
                        .as_file()
                        .is_some_and(|p| p.ends_with("internal/allocation/aligned-blocks.jai"))
            })
            .unwrap();
    assert!(all.procs[&helper.procedure].bit_return.is_some());
    assert!(
        all.procs[&helper.procedure].frame.is_none(),
        "the pointer output is not an ordinary void frame"
    );
}

#[test]
fn generic_private_integer_copies_keep_both_machine_return_arms() {
    let c = checked_source(
        r"
        transform::(base:*void,prefix:u64,wide:bool)->*void {
            start:=cast(u64)base+prefix;
            copied:=wide;
            if copied start=(start+63)&~cast(u64)63;
            return cast(*void)start;
        }
        main::(){}
    ",
    );
    let all = Summaries::build(&c);
    let id = ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "transform")
            .unwrap() as u32,
    );
    assert!(all.procs[&id].bit_return.is_some());
    assert!(all.procs[&id].frame.is_none());
}


const BIT_SOURCE: &str = r"
    transform::(base:*void,prefix:u64,wide:bool)->*void {
        start:=cast(u64)base+prefix;
        if wide start=(start+63)&~cast(u64)63;
        return cast(*void)start;
    }
    main::(){}
";

fn transform_id(c: &Compiler) -> ProcId {
    ProcId(
        c.pr.procs
            .iter()
            .position(|p| p.name.as_str() == "transform")
            .unwrap() as u32,
    )
}

fn evaluate(
    proof: &super::super::address_arithmetic::BitReturn,
    args: [u64; 3],
    wide: bool,
) -> u64 {
    use super::super::address_arithmetic::bit_returns::BitNode;
    let mut values: Vec<u64> = Vec::new();
    for node in &proof.nodes {
        values.push(match *node {
            BitNode::Pointer(formal) | BitNode::Unsigned(formal) => args[formal],
            BitNode::Constant(n) => n,
            BitNode::Add(a, b) => values[a].wrapping_add(values[b]),
            BitNode::Sub(a, b) => values[a].wrapping_sub(values[b]),
            BitNode::And(a, b) => values[a] & values[b],
        });
    }
    values[proof.arms[usize::from(wide)]]
}

#[test]
fn bit_returns_keep_machine_wrap_and_actual_boolean_polarity() {
    let mut c = checked_source(BIT_SOURCE);
    let id = transform_id(&c);
    let all = Summaries::build(&c);
    let proof = all.procs[&id].bit_return.as_ref().unwrap();
    assert!(proof.current(&c, &all.procs));
    assert_eq!(proof.formal, 2);
    for (base, prefix) in [(0x1000, 16), (u64::MAX - 7, 16)] {
        assert_eq!(
            evaluate(proof, [base, prefix, 0], false),
            base.wrapping_add(prefix)
        );
        assert_eq!(
            evaluate(proof, [base, prefix, 0], true),
            base.wrapping_add(prefix).wrapping_add(63) & !63
        );
    }
    let expected = proof.clone();
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(after.procs[&id].bit_return.as_ref(), Some(&expected));
    let inverted = checked_source(&BIT_SOURCE.replace("if wide", "if !wide"));
    let inverse = Summaries::build(&inverted);
    let inverse = inverse.procs[&transform_id(&inverted)]
        .bit_return
        .as_ref()
        .unwrap();
    assert!(inverse.inverted);
    assert_eq!(evaluate(inverse, [0x1000, 16, 0], true), 0x1010);
    assert_eq!(evaluate(inverse, [0x1000, 16, 0], false), 0x1040);
}

#[test]
fn bit_returns_reject_unclassified_effects_and_lost_guard_origins() {
    for mutation in [
        BIT_SOURCE.replace("if wide", "wide=false;if wide"),
        BIT_SOURCE.replace("if wide", "wide=cast(bool)prefix;if wide"),
        BIT_SOURCE.replace("start:=", "G=base;start:=") + " G:*void;",
        BIT_SOURCE.replace("start:=", "opaque();start:=") + " opaque::() #foreign;",
        BIT_SOURCE
            .replace("prefix:u64", "prefix:u32")
            .replace("+prefix", "+cast(u64)prefix"),
        BIT_SOURCE.replace("start+63", "start*63"),
        BIT_SOURCE.replace("if wide", "while wide"),
        BIT_SOURCE.replace("return cast(*void)start", "return helper(start)")
            + " helper::(n:u64)->*void{return cast(*void)n;}",
    ] {
        let c = checked_source(&mutation);
        let all = Summaries::build(&c);
        assert!(
            all.procs[&transform_id(&c)].bit_return.is_none(),
            "{mutation}"
        );
    }
    let overwritten =
        checked_source(&BIT_SOURCE.replace("return cast", "start=prefix;return cast"));
    let all = Summaries::build(&overwritten);
    let proof = all.procs[&transform_id(&overwritten)]
        .bit_return
        .as_ref()
        .unwrap();
    assert_eq!(evaluate(proof, [0x1000, 16, 0], false), 16);
    assert_eq!(evaluate(proof, [0x1000, 16, 0], true), 16);
}

#[test]
fn bit_return_query_rejects_conflicting_producers_in_either_order() {
    let mut c = checked_source(BIT_SOURCE);
    let all = Summaries::build(&c);
    let key = transform_id(&c);
    let body = c.ed.facts().unwrap().lifetime_bodies[&key].clone();
    for first in [false, true] {
        for fault in 0..4 {
            let mut changed = body.clone();
            let (value, ty, span) = changed
                .blocks
                .iter()
                .flat_map(|b| &b.events)
                .find_map(|e| match e {
                    Event::Load {
                        value,
                        ty,
                        span,
                        ..
                    } if *ty == TypeId::U64 => Some((*value, *ty, *span)),
                    _ => None,
                })
                .unwrap();
            let extra = match fault {
                0 => Event::View {
                    value,
                    source: value,
                    from: ty,
                    to: ty,
                    span,
                },
                1 => Event::Procedure {
                    value,
                    callee: body.procedure,
                    ty,
                    span,
                },
                2 => Event::EmptyDescriptor {
                    value,
                    ty,
                },
                3 => Event::IntegerBinary {
                    value,
                    ty,
                    left: value,
                    right: value,
                    op: jaic::ir::BinOp::Add,
                    left_constant: None,
                    right_constant: None,
                },
                _ => unreachable!(),
            };
            if first {
                changed.blocks[0].events.insert(0, extra);
            } else {
                changed.blocks[0].events.push(extra);
            }
            c.ed.facts_mut()
                .unwrap()
                .lifetime_bodies
                .insert(key, changed);
            let checked = crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body(key)
                .unwrap();
            let prepared = Prepared::for_body(checked, &all);
            let r = Resolver::for_prepared(&prepared, &all);
            assert!(
                r.bit_return_plan(false).is_none(),
                "fault{fault} first{first}"
            );
        }
    }
    for width in [4, 0] {
        let mut changed = body.clone();
        changed.address_casts[0].integer_bytes = width;
        c.ed.facts_mut()
            .unwrap()
            .lifetime_bodies
            .insert(key, changed);
        let checked = crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body(key)
            .unwrap();
        let prepared = Prepared::for_body(checked, &all);
        let r = Resolver::for_prepared(&prepared, &all);
        assert!(r.bit_return_plan(false).is_none());
    }
}

#[test]
fn bit_returns_budget_and_cancellation_discard_complete_output() {
    if !super::cancellation_child() {
        return;
    }
    let c = checked_source(BIT_SOURCE);
    let all = Summaries::build(&c);
    let body = &c.ed.facts().unwrap().lifetime_bodies[&transform_id(&c)];
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let mut r = Resolver::for_prepared(&prepared, &all);
    drop(prepared);
    assert!(r.bit_return_plan(false).is_some());
    let reserve = r.core().scalar_requests.as_ref().unwrap().work_bound;
    for extra in [8192, 8191, 8176] {
        r.set_scalar_work_bound(extra);
        assert!(r.bit_return_plan(false).is_none());
    }
    r.set_scalar_work_bound(reserve);
    let plan = r.bit_return_plan(false).unwrap();
    let mut observer = plan.observer(&r);
    for (block, b) in body.blocks.iter().enumerate() {
        for (ordinal, event) in b.events.iter().enumerate() {
            observer.observe(&r, block, ordinal, event, &all.procs);
        }
    }
    jaic::interrupt::arm();
    jaic::interrupt::request(true);
    let result = observer.finish(&r, &super::super::flow::Live::all(body), &all.procs);
    assert!(jaic::interrupt::disarm());
    jaic::interrupt::request(false);
    assert!(result.is_none());
}


#[test]
fn bit_returns_keep_formal_roles_and_all_selected_pointer_outputs() {
    let reordered = checked_source(
        r"
        transform::(prefix:u64,wide:bool,base:*u8)->*void {
            if wide return cast(*void)(cast(u64)base+prefix+64);
            return cast(*void)(cast(u64)base+prefix);
        }
        main::(){}
    ",
    );
    let all = Summaries::build(&reordered);
    let proof = all.procs[&transform_id(&reordered)]
        .bit_return
        .as_ref()
        .unwrap();
    assert_eq!(proof.formal, 1);
    assert_eq!(evaluate(proof, [16, 0, 0x1000], false), 0x1010);
    assert_eq!(evaluate(proof, [16, 0, 0x1000], true), 0x1050);
    let extra = checked_source(
        r"
        transform::(base:*void,prefix:u64,wide:bool)->(*void,*void) {
            start:=cast(u64)base+prefix;
            if wide start=(start+63)&~cast(u64)63;
            return cast(*void)start,base;
        }
        main::(){}
    ",
    );
    assert!(
        Summaries::build(&extra).procs[&transform_id(&extra)]
            .bit_return
            .is_none()
    );
}

#[test]
fn bit_return_identity_and_independent_contracts_are_rechecked() {
    let mut c = checked_source(BIT_SOURCE);
    let id = transform_id(&c);
    let mut all = Summaries::build(&c);
    let proof = all.procs[&id].bit_return.clone().unwrap();
    assert!(proof.current(&c, &all.procs));
    all.procs.get_mut(&id).unwrap().trusted =
        Some(Box::new(super::super::contracts::Trusted::Opaque));
    assert!(!proof.current(&c, &all.procs));
    all.procs.get_mut(&id).unwrap().trusted = None;
    let target = c.pr.procs[id.0 as usize].target.take();
    assert!(!proof.current(&c, &all.procs));
    c.pr.procs[id.0 as usize].target = target;
    assert!(proof.current(&c, &all.procs));
    c.ed.facts_mut()
        .unwrap()
        .lifetime_bodies
        .get_mut(&id)
        .unwrap()
        .procedure = ProcId(u32::MAX);
    assert!(!proof.current(&c, &all.procs));
    let trusted = super::stdlib_allocations::checked_source_with_contracts(
        r"
        transform::(base:*void,prefix:u64,wide:bool)->*void {
            start:=cast(u64)base+prefix;
            if wide start=(start+63)&~cast(u64)63;
            return cast(*void)start;
        } @FreesArgument
        main::(){}
    ",
        Options::host(),
    );
    let all = Summaries::build(&trusted);
    let summary = &all.procs[&transform_id(&trusted)];
    assert!(summary.trusted.is_some());
    assert!(summary.bit_return.is_none());
}


#[test]
fn checked_addition_failure_paths_cannot_be_published_as_plain_modular_returns() {
    for mode in [1, 2] {
        let mut options = Options::host();
        options.arithmetic_overflow_check = mode;
        let c = super::stdlib_allocations::checked_source_options(BIT_SOURCE, options);
        assert!(
            Summaries::build(&c).procs[&transform_id(&c)]
                .bit_return
                .is_none()
        );
    }
}

#[test]
fn bit_return_reads_the_actual_ifx_result_epoch() {
    let mut c = checked_source(
        r"
        opaque::() #foreign;
        side::()->u64{opaque();return 56;}
        conditional_bits::(base:*void,prefix:u64,wide:bool)->*void {
            padding:=ifx wide then cast(u64)56 else cast(u64)0;
            return cast(*void)(cast(u64)base+prefix+padding);
        }
        callback_bits::(base:*void,prefix:u64,wide:bool)->*void {
            padding:=ifx wide then side() else cast(u64)0;
            return cast(*void)(cast(u64)base+prefix+padding);
        }
        main::(){}
    ",
    );
    let id = |name: &str| {
        ProcId(
            c.pr.procs
                .iter()
                .position(|p| p.name.as_str() == name)
                .unwrap() as u32,
        )
    };
    let good = id("conditional_bits");
    let bad = id("callback_bits");
    let all = Summaries::build(&c);
    let before = all.procs[&good]
        .bit_return
        .clone()
        .expect("selected actual ifx stores feed its captured load");
    assert_ne!(before.arms[0], before.arms[1]);
    assert!(
        before
            .nodes
            .contains(&super::super::address_arithmetic::bit_returns::BitNode::Constant(56))
    );
    assert!(
        all.procs[&bad].bit_return.is_none(),
        "an effectful arm is not a closed bit transform"
    );
    c.release_lowered_code();
    let after = Summaries::build(&c);
    assert_eq!(
        after.procs[&good].bit_return.as_deref(),
        Some(before.as_ref())
    );
    assert!(after.procs[&bad].bit_return.is_none());
}
