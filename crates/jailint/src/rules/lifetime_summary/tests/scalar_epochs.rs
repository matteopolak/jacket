//! Typed scalar captures share the pointer storage transfer, never its meaning.
use super::super::storage_versions::{self, Source};
use super::stdlib_allocations::checked_source;
use super::*;

#[test]
fn scalar_loads_capture_checked_ssa_before_later_replacement() {
    let c = checked_source(
        r"
        scalar_epoch_probe::(region:*void,prefix:u64,wide:bool)->*void {
            start:=cast(u64)region+prefix;
            before:=start;start=77;after:=start;
            if wide return cast(*void)before;
            return cast(*void)after;
        }
        main::(){}
    ",
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "scalar_epoch_probe")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let initial = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::IntegerBinary {
                ty,
                value,
                op: jaic::ir::BinOp::Add,
                ..
            } if *ty == TypeId::U64 => Some(*value),
            _ => None,
        })
        .unwrap();
    let start = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Store {
                place,
                value,
                ty,
                ..
            } if *value == initial => r.places.existing(*place, *ty),
            _ => None,
        })
        .unwrap();
    let loads: Vec<_> = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .filter_map(|e| match e {
            Event::Load {
                place,
                value,
                ty,
                ..
            } if *ty == TypeId::U64 && r.places.existing(*place, *ty) == Some(start) => {
                Some(*value)
            }
            _ => None,
        })
        .collect();
    assert_eq!(loads.len(), 2);
    assert_eq!(
        r.storage_versions.get(&loads[0]),
        Some(&HashSet::from_iter([Source::ScalarValue {
            ty: TypeId::U64,
            value: initial
        }])),
        "checked carrier events: {:?}; private={:?}; exposed={:?}",
        body.blocks,
        r.private_bytes,
        r.exposed
    );
    assert_eq!(
        r.storage_versions.get(&loads[1]),
        Some(&HashSet::from_iter([Source::Constant {
            ty: TypeId::U64,
            value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(77)
        }]))
    );
    assert!(
        prepared
            .captured_pointer_values(loads[0], &mut 128)
            .is_none()
    );
}

#[test]
fn explicit_storage_edge_cuts_preserve_empty_cut_and_separate_alternatives() {
    let c = checked_source(
        r"
        scalar_cut_probe::(p:*void,q:*void,wide:bool)->*void {
            selected:=p;if wide selected=q;return selected;
        }
        main::(){}
    ",
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "scalar_cut_probe")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let mut places = r.places.clone();
    let probes = super::super::call_slots::Probes::collect(&c, body, &mut places);
    let ordinary = storage_versions::loads(&places, &r, &probes).0;
    let empty = storage_versions::loads_except(&places, &r, &probes, &HashSet::default()).0;
    assert_eq!(ordinary, empty);
    let (block, then_block, else_block) = body
        .blocks
        .iter()
        .enumerate()
        .find_map(|(i, b)| {
            b.events.iter().find_map(|e| match e {
                Event::Branch {
                    then_block,
                    else_block,
                    ..
                } => Some((i, *then_block, *else_block)),
                _ => None,
            })
        })
        .unwrap();
    let result = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Return {
                value: v, ..
            } => Some(*v),
            _ => None,
        })
        .unwrap();
    let yes = storage_versions::loads_except(
        &places,
        &r,
        &probes,
        &HashSet::from_iter([(block, else_block)]),
    )
    .0;
    let no = storage_versions::loads_except(
        &places,
        &r,
        &probes,
        &HashSet::from_iter([(block, then_block)]),
    )
    .0;
    assert_ne!(yes.get(&result), no.get(&result));
    assert_eq!(yes[&result].len(), 1);
    assert_eq!(no[&result].len(), 1);
}

#[test]
fn requested_boolean_reads_keep_actual_input_and_replacement_identities() {
    let c = checked_source(
        r"
        bool_input_probe::(region:*void,prefix:s64,wide:bool)->*void {
            if wide return region;return cast(*void)prefix;
        }
        bool_replaced_probe::(region:*void,prefix:s64,wide:bool)->*void {
            wide=!wide;if wide return region;return cast(*void)prefix;
        }
        bool_integer_probe::(region:*void,prefix:s64,wide:bool)->*void {
            wide=cast(bool)prefix;if wide return region;return cast(*void)prefix;
        }
        main::(){}
    ",
    );
    let all = Summaries::build(&c);
    for name in [
        "bool_input_probe",
        "bool_replaced_probe",
        "bool_integer_probe",
    ] {
        let body =
            c.ed.facts()
                .unwrap()
                .lifetime_bodies
                .values()
                .find(|b| c.proc(b.procedure).name.as_str() == name)
                .unwrap();
        let prepared = Prepared::for_body(
            crate::rules::lifetime_facts::CheckedSession::new(&c)
                .body((body).procedure)
                .unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&prepared, &all);
        let incoming = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .find_map(|e| match e {
                Event::Parameter {
                    incoming,
                    ty,
                    ..
                } if *ty == TypeId::BOOL => Some(*incoming),
                _ => None,
            })
            .unwrap();
        let condition = body
            .blocks
            .iter()
            .flat_map(|b| &b.events)
            .find_map(|e| match e {
                Event::Branch {
                    condition, ..
                } => Some(*condition),
                _ => None,
            })
            .unwrap();
        let input = HashSet::from_iter([Source::ScalarValue {
            ty: TypeId::BOOL,
            value: incoming,
        }]);
        if name == "bool_input_probe" {
            assert_eq!(r.storage_versions.get(&condition), Some(&input));
        } else {
            assert_ne!(r.storage_versions.get(&condition), Some(&input), "{name}");
        }
        assert!(
            prepared
                .captured_pointer_values(condition, &mut 128)
                .is_none()
        );
    }
}

#[test]
fn optional_scalar_requests_reject_cycles_and_preserve_ordinary_pointer_capture() {
    let c = checked_source(
        r"
        scalar_cycle_probe::(region:*void,prefix:u64,wide:bool)->*void {
            saved:=region;start:=cast(u64)region+prefix;
            while wide start+=prefix;
            return saved;
        }
        main::(){}
    ",
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "scalar_cycle_probe")
            .unwrap();
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    let returned = body
        .blocks
        .iter()
        .flat_map(|b| &b.events)
        .find_map(|e| match e {
            Event::Return {
                value, ..
            } => Some(*value),
            _ => None,
        })
        .unwrap();
    assert!(
        prepared
            .captured_pointer_values(returned, &mut 128)
            .is_some()
    );
    assert!(
        r.scalar_requests.is_none(),
        "unsupported cyclic optional work must not enter preparation"
    );
}


#[test]
fn signed_scalar_check_failure_is_not_a_pure_transform() {
    let c = checked_source(
        r"
        signed_scalar_epoch::(region:*void,prefix:s64,wide:bool)->*void {
            start:=cast(u64)region+cast(u64)prefix;
            before:=start;start=77;after:=start;
            if wide return cast(*void)before;
            return cast(*void)after;
        }
        main::(){}
        ",
    );
    let all = Summaries::build(&c);
    let body =
        c.ed.facts()
            .unwrap()
            .lifetime_bodies
            .values()
            .find(|b| c.proc(b.procedure).name.as_str() == "signed_scalar_epoch")
            .unwrap();
    assert!(
        body.blocks
            .iter()
            .flat_map(|b| &b.events)
            .any(|e| matches!(e, Event::Barrier)),
        "the actual check failure can invoke the handler"
    );
    assert!(
        body.blocks
            .iter()
            .flat_map(|b| &b.events)
            .filter(|e| matches!(e, Event::Branch { .. }))
            .count()
            >= 2,
        "checked cast and semantic Boolean branch remain distinct"
    );
    let prepared = Prepared::for_body(
        crate::rules::lifetime_facts::CheckedSession::new(&c)
            .body((body).procedure)
            .unwrap(),
        &all,
    );
    let r = Resolver::for_prepared(&prepared, &all);
    assert!(
        r.scalar_requests.is_none(),
        "without a proved cast-fit premise, neither callback arm may be ignored"
    );
}
