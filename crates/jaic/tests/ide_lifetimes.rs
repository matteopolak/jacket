//! The sidecar records checked instances and lowering choices, never sketches or failures.
use jaic::ide::lifetimes::{LifetimeBody, LifetimeEvent as Event, LifetimeOperand};
use jaic::ide::{IdeFacts, IdeRoot};
use jaic::sema::{Compiler, NativeFs, Options, ProgramSource};
use std::rc::Rc;

fn compile(source: &str, lint: bool, broken: bool) -> Compiler {
    let stdlib = test_support::repo::root().join("stdlib");
    let mut options = Options::host();
    options.import_paths = vec![stdlib.clone()];
    options.set_stdlib(&NativeFs, &stdlib);
    let mut c = Compiler::new(options, Rc::new(NativeFs));
    let mut facts = IdeFacts::new(vec![IdeRoot::Virtual]);
    facts.lint = lint;
    c.ed.record(facts);
    let start = c.begin_sources(&[ProgramSource::String(source.to_owned())]);
    assert!(start.is_ok(), "{start:?}");
    let result = c.finish_program();
    assert_eq!(result.is_err(), broken, "{result:?}");
    let _ = c.ide_check_all();
    c
}

fn bodies(c: &Compiler) -> impl Iterator<Item = &LifetimeBody> {
    c.ed.facts().unwrap().lifetime_bodies.values()
}

fn events(body: &LifetimeBody) -> impl Iterator<Item = &Event> {
    body.blocks.iter().flat_map(|b| &b.events)
}

const POLY: &str = r"
grow :: (xs: *[..] int) {} @GrowsArray
poly :: ($G: bool) {
    xs: [..] int;
    p := *xs[0];
    #if G grow(*xs);
    p.* = 1;
}
unused :: (value: $T) { local := value; }
main :: () { poly(true); poly(false); }
";

#[test]
fn real_instantiations_keep_independent_choices_and_identities() {
    let mut c = compile(POLY, true, false);
    c.ide_sketch_polymorphs();
    let selected: Vec<_> = bodies(&c)
        .filter(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "poly")
        .collect();
    assert_eq!(selected.len(), 2);
    let mut choices = Vec::new();
    let mut identities = Vec::new();
    for b in selected {
        let taken = events(b)
            .find_map(|e| match e {
                Event::StaticChoice {
                    taken, ..
                } => Some(*taken),
                _ => None,
            })
            .unwrap();
        let grows = events(b)
            .filter(|e| matches!(e, Event::Grow { .. }))
            .count();
        assert_eq!(grows, usize::from(taken));
        choices.push(taken);
        identities.push(
            events(b)
                .find_map(|e| match e {
                    Event::Expression {
                        entity: Some(id), ..
                    } if c.entity(*id).name.as_str() == "xs" => Some(*id),
                    _ => None,
                })
                .unwrap(),
        );
        assert!(
            b.blocks
                .iter()
                .flat_map(|b| &b.successors)
                .all(|&i| i < b.blocks.len())
        );
    }
    choices.sort_unstable();
    assert_eq!(choices, [false, true]);
    assert_ne!(identities[0], identities[1]);
    assert!(!bodies(&c).any(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "unused"));
}

#[test]
fn lint_facts_off_allocate_no_sidecars() {
    let c = compile(POLY, false, false);
    assert_eq!(bodies(&c).count(), 0);
}

#[test]
fn failed_body_never_publishes_partial_events() {
    let c = compile("main :: () { n := 1; missing(); }", true, true);
    assert!(!bodies(&c).any(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main"));
}

#[test]
fn named_spread_and_deferred_arguments_keep_formal_mapping() {
    let c = compile(
        r"
        grow :: (xs: *[..] int, other: int, rest: ..int) {} @GrowsArray
        main :: () {
            xs: [..] int; rest: [2] int;
            grow(other = 3, xs = xx *xs, rest = ..rest);
        }
    ",
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    let args = events(b)
        .find_map(|e| match e {
            Event::Call {
                callee,
                args,
                ..
            } if c.pr.procs[callee.0 as usize].name.as_str() == "grow" => Some(args),
            _ => None,
        })
        .unwrap();
    assert_eq!(args.len(), 3);
    assert!(args.iter().all(|a| a.named));
    assert!(args.iter().any(|a| a.formal == 2 && a.spread));
    let region = events(b)
        .find_map(|e| match e {
            Event::Grow {
                region, ..
            } => Some(*region),
            _ => None,
        })
        .unwrap();
    assert!(args.iter().any(|a| a.formal == 0 && matches!(a.operand, LifetimeOperand::Value(v) if v == region)));
}

#[test]
fn expansions_and_cleanup_remain_in_the_actual_blocks() {
    let c = compile(
        r#"
        grow :: (xs: *[..] int) {} @GrowsArray
        marker :: () #expand { n := 1; }
        main :: () {
            xs: [..] int;
            marker();
            { defer grow(*xs); }
            #insert "grow(*xs);";
        }
    "#,
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    assert!(events(b).any(|e| matches!(
        e,
        Event::Call {
            expands: true,
            ..
        }
    )));
    assert!(events(b).any(|e| matches!(
        e,
        Event::Cleanup {
            deferred: Some(_),
            ..
        }
    )));
    assert!(events(b).any(|e| matches!(e, Event::Expansion { output, .. } if !output.is_empty())));
    assert_eq!(
        events(b)
            .filter(|e| matches!(e, Event::Grow { .. }))
            .count(),
        2
    );
}


#[test]
fn type_only_probe_does_not_emit_runtime_effects() {
    let c = compile(
        r"
        grow :: (xs: *[..] int) {} @GrowsArray
        main :: () { xs: [..] int; exists := #exists(grow(*xs)); }
    ",
        true,
        false,
    );
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    assert!(!events(b).any(|e| matches!(e, Event::Grow { .. } | Event::Call { .. })));
}


#[test]
fn assignment_targets_and_comparisons_are_not_reads_but_their_operands_are() {
    let source = r"
        main :: () {
            p: *int; q: *int; pp := *p;
            p = q;
            p, q = q, p;
            p += 1;
            pp.* = q;
            same := p == q;
            other := p.* == q.*;
        }
    ";
    let c = compile(source, true, false);
    let b = bodies(&c)
        .find(|b| c.pr.procs[b.procedure.0 as usize].name.as_str() == "main")
        .unwrap();
    for (fragment, expected) in [
        ("p = q", 1),
        ("p, q = q, p", 2),
        ("p += 1", 1),
        ("pp.* = q", 2),
        ("p == q", 0),
        ("p.* == q.*", 2),
    ] {
        let start = source.find(fragment).unwrap() as u32;
        let count = events(b).filter(|e| matches!(e, Event::Use { span, .. } if span.start >= start && span.end <= start + fragment.len() as u32)).count();
        assert_eq!(count, expected, "{fragment}");
    }
}
