//! Borrowed worker lifetimes. Contracts identify a receiver and callback; the callback's
//! parameter-relative access summary selects captured fields without container-name rules.
use super::{
    Cell, Cells, Cx, Event, Facts, HashMap, LifetimeBody, LifetimeOperand, Places, Resolver, Site,
    Span, Summaries,
};
use crate::rules::lifetime_movement::{self, Selector};
use crate::rules::lifetime_summary::{Path, Root};
use jaic::ir::Val;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub(super) struct Token {
    pub region: usize,
    pub header: bool,
}

#[derive(Clone)]
pub(super) enum Op {
    Start {
        token: Token,
        cells: Vec<usize>,
        possible: bool,
        span: Span,
    },
    End(Token),
    MayEnd(Token),
}

struct Registration {
    sites: Vec<Site>,
    reads: Vec<Vec<crate::rules::lifetime_summary::Step>>,
    opaque: bool,
}

pub(super) struct Builder {
    registrations: HashMap<usize, Registration>,
    aliases: HashMap<usize, usize>,
    started: Vec<usize>,
    completions: HashMap<Val, usize>,
    successors: Vec<Vec<usize>>,
}

fn parameter(cx: &Cx, event: &Event, name: &str) -> Option<Vec<String>> {
    let Event::Call {
        callee, ..
    } = event
    else {
        return None;
    };
    lifetime_movement::notes(cx.compiler, *callee).find_map(|n| {
        lifetime_movement::parameters(&n.text, name)
            .map(|a| a.into_iter().map(str::to_owned).collect())
    })
}

fn argument(event: &Event, formal: usize) -> Option<&jaic::ide::lifetimes::LifetimeArgument> {
    let Event::Call {
        args, ..
    } = event
    else {
        return None;
    };
    args.iter().find(|a| a.formal == formal)
}

fn receiver(
    event: &Event,
    formal: usize,
    places: &mut Places,
    resolver: &mut Resolver<'_>,
) -> Option<usize> {
    let p = resolver.object(argument(event, formal)?)?;
    Some(places.import(&resolver.places, p))
}

impl Builder {
    pub fn new(
        cx: &Cx,
        body: &LifetimeBody,
        summaries: &Summaries,
        places: &mut Places,
        resolver: &mut Resolver<'_>,
    ) -> Self {
        let mut this = Self {
            registrations: HashMap::default(),
            aliases: HashMap::default(),
            started: Vec::new(),
            completions: HashMap::default(),
            successors: body.blocks.iter().map(|b| b.successors.clone()).collect(),
        };
        for event in body.blocks.iter().flat_map(|b| &b.events) {
            for note in [
                "RegistersBorrower",
                "CompletesBorrower",
                "EndsBorrower",
                "StartsBorrower",
            ] {
                let Some(formal) =
                    parameter(cx, event, note).and_then(|a| a.first().and_then(|n| n.parse().ok()))
                else {
                    continue;
                };
                let Some(p) = receiver(event, formal, places, resolver) else {
                    continue;
                };
                let own = resolver.places.import(places, p);
                if let Some(owner) = resolver.local_destination_of(own, &summaries.procs)
                    && resolver.places.types.get(&owner).is_some_and(|&ty| {
                        matches!(
                            cx.compiler.ty.types.kind(ty),
                            jaic::types::TypeKind::Array {
                                kind: jaic::types::ArrayKind::Fixed(_),
                                ..
                            }
                        )
                    })
                {
                    let owner = places.import(&resolver.places, owner);
                    let index = resolver.places.paths[own].0.0;
                    let element = places.project(
                        owner,
                        crate::rules::lifetime_places::Projection::DynamicElement(index),
                    );
                    this.aliases.insert(p, element);
                }
            }
        }
        for (at, event) in body.blocks.iter().enumerate().flat_map(|(block, b)| {
            b.events.iter().enumerate().map(move |(event, e)| {
                (
                    Site {
                        block,
                        event,
                    },
                    e,
                )
            })
        }) {
            if let Some(formal) = parameter(cx, event, "StartsBorrower")
                .and_then(|a| a.first().and_then(|n| n.parse().ok()))
                && let Some(region) = this.receiver(event, formal, places, resolver)
                && !this.started.contains(&region)
            {
                this.started.push(region);
            }
            if let Some(args) = parameter(cx, event, "RegistersBorrower") {
                let (Some(receiver_formal), Some(callback_formal), Some(callback_input)) = (
                    args.first().and_then(|a| a.parse().ok()),
                    args.get(1).and_then(|a| a.parse().ok()),
                    args.get(2).and_then(|a| a.parse().ok()),
                ) else {
                    continue;
                };
                let Some(region) = this.receiver(event, receiver_formal, places, resolver) else {
                    continue;
                };
                let Some(arg) = argument(event, callback_formal) else {
                    continue;
                };
                let LifetimeOperand::Value(value) = arg.operand else {
                    continue;
                };
                let targets = summaries.targets.get(body.procedure, value);
                let mut registration = Registration {
                    sites: vec![at],
                    reads: Vec::new(),
                    opaque: targets.unknown,
                };
                let mut common =
                    None::<jaic::fxhash::HashSet<Vec<crate::rules::lifetime_summary::Step>>>;
                for proc in targets.procedures {
                    let Some(s) = summaries.procs.get(&proc) else {
                        registration.opaque = true;
                        continue;
                    };
                    registration.opaque |= !s.stable_captures(
                        cx.compiler,
                        cx.compiler
                            .ed
                            .facts()
                            .and_then(|facts| facts.lifetime_bodies.get(&proc)),
                    );
                    let reads = if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
                        &s.may_reads
                    } else {
                        &s.reads
                    };
                    let current = reads
                        .iter()
                        .filter(|p| p.root == Root::Parameter(callback_input))
                        .map(|p| p.steps.clone())
                        .collect::<jaic::fxhash::HashSet<_>>();
                    if let Some(old) = &mut common {
                        if cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
                            old.extend(current);
                        } else {
                            old.retain(|p| current.contains(p));
                        }
                    } else {
                        common = Some(current);
                    }
                }
                registration.reads.extend(common.into_iter().flatten());
                if let Some(old) = this.registrations.get_mut(&region) {
                    old.opaque |= registration.opaque;
                    old.sites.extend(registration.sites);
                    old.reads.retain(|p| registration.reads.contains(p));
                } else {
                    this.registrations.insert(region, registration);
                }
            }
            if let Some(args) = parameter(cx, event, "CompletesBorrower")
                && let Some(formal) = args.first().and_then(|a| a.parse().ok())
                && let Some(region) = this.receiver(event, formal, places, resolver)
                && let Event::Call {
                    results, ..
                } = event
                && let Some(&(_, value, _)) = results.first()
            {
                this.completions.insert(value, region);
            }
        }
        this
    }

    fn receiver(
        &self,
        event: &Event,
        formal: usize,
        places: &mut Places,
        resolver: &mut Resolver<'_>,
    ) -> Option<usize> {
        let p = receiver(event, formal, places, resolver)?;
        Some(self.aliases.get(&p).copied().unwrap_or(p))
    }

    pub fn operations(
        &self,
        cx: &Cx,
        event: &Event,
        at: Site,
        facts: &Facts,
        storage: (&mut Places, &mut Cells),
        resolver: &mut Resolver<'_>,
    ) -> Vec<Op> {
        let (places, cells) = storage;
        let mut out = Vec::new();
        if let Some(args) = parameter(cx, event, "EndsBorrower")
            && let Some(formal) = args.first().and_then(|a| a.parse().ok())
            && let Some(region) = self.receiver(event, formal, places, resolver)
        {
            out.extend([
                Op::End(Token {
                    region,
                    header: false,
                }),
                Op::End(Token {
                    region,
                    header: true,
                }),
            ]);
            for &started in &self.started {
                if may_alias(places, region, started) {
                    for header in [false, true] {
                        out.push(Op::MayEnd(Token {
                            region: started,
                            header,
                        }));
                    }
                }
            }
        }
        if let Some(args) = parameter(cx, event, "StartsBorrower")
            && let Some(formal) = args.first().and_then(|a| a.parse().ok())
            && let Some(region) = self.receiver(event, formal, places, resolver)
            && let Some(arg) = argument(event, formal)
            && let LifetimeOperand::Value(v) = arg.operand
            && let Event::Call {
                span, ..
            } = event
        {
            out.push(Op::Start {
                token: Token {
                    region,
                    header: true,
                },
                cells: vec![cells.id(Cell::Value(v))],
                possible: false,
                span: *span,
            });
            if let Some(registration) = self.registrations.get(&region)
                && (cx.lifetime_mode == jacket_config::LifetimeMode::Strict
                    || registration
                        .sites
                        .iter()
                        .any(|&site| self.dominates(site, at)))
            {
                let mut captured = Vec::new();
                for steps in &registration.reads {
                    let own = resolver.places.import(places, region);
                    if let Some(p) = resolver.materialize(&Path {
                        root: Root::Local(own),
                        steps: steps.clone(),
                    }) {
                        captured.push(cells.id(Cell::Place(places.import(&resolver.places, p))));
                    }
                }
                if registration.opaque && cx.lifetime_mode == jacket_config::LifetimeMode::Strict {
                    captured.extend(
                        places
                            .descendants(region)
                            .into_iter()
                            .map(|p| cells.id(Cell::Place(p))),
                    );
                }
                captured.sort_unstable();
                captured.dedup();
                out.push(Op::Start {
                    token: Token {
                        region,
                        header: false,
                    },
                    cells: captured,
                    possible: registration.opaque,
                    span: *span,
                });
            }
        }
        if let Event::Call {
            callee,
            args,
            ..
        } = event
            && cx.compiler.proc_has_note(*callee, "AssertsArgument")
            && let Some(arg) = args.iter().find(|a| a.formal == 0)
            && let LifetimeOperand::Value(v) = arg.operand
            && let Selector::Local(v) = facts.selector(v, None, at)
            && let Some(&region) = self.completions.get(&v)
        {
            out.push(Op::End(Token {
                region,
                header: false,
            }));
        }
        out
    }

    fn dominates(&self, earlier: Site, later: Site) -> bool {
        if earlier.block == later.block {
            return earlier.event < later.event;
        }
        let mut seen = jaic::fxhash::HashSet::default();
        let mut queue = std::collections::VecDeque::from([0]);
        while let Some(block) = queue.pop_front() {
            if block == earlier.block || !seen.insert(block) {
                continue;
            }
            if block == later.block {
                return false;
            }
            queue.extend(&self.successors[block]);
        }
        true
    }

    pub fn completion(&self, value: Val, facts: &Facts, at: Site) -> Option<Token> {
        let Selector::Local(value) = facts.selector(value, None, at) else {
            return None;
        };
        self.completions.get(&value).map(|&region| Token {
            region,
            header: false,
        })
    }
}

// Distinct dynamic selectors can overlap. Completing one makes retention of the
// other uncertain; it does not prove every element completed (strict still checks it).
fn may_alias(places: &Places, a: usize, b: usize) -> bool {
    use crate::rules::lifetime_places::Projection;
    let (root_a, path_a) = &places.paths[a];
    let (root_b, path_b) = &places.paths[b];
    a != b
        && root_a == root_b
        && path_a.len() == path_b.len()
        && path_a.iter().zip(path_b).all(|(a, b)| {
            a == b
                || matches!(
                    (a, b),
                    (
                        Projection::DynamicElement(_),
                        Projection::DynamicElement(_) | Projection::Element(_)
                    ) | (Projection::Element(_), Projection::DynamicElement(_))
                )
        })
}
