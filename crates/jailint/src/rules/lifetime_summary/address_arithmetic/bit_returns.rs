//! Typed machine-bit expressions, using the existing ordered storage transfer.
use super::super::storage_versions::{CapturedLoads, Source};
use super::super::{
    Compiler, Event, HashMap, HashSet, ProcId, Resolver, Summary, TypeId, Val, flow, frames,
};
use crate::rules::lifetime_movement::facts::Site;

const MAX_WORK: usize = 8192;
const MAX_NODES: usize = 32;

/// U64 machine operations: Add and Sub both use modulo 2^64 arithmetic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules) enum BitNode {
    Pointer(usize),
    Unsigned(usize),
    Constant(u64),
    Add(usize, usize),
    Sub(usize, usize),
    And(usize, usize),
}

/// Complete guarded bit returns, with no ownership, range or nonwrap implication.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct BitReturn {
    frame: frames::Frame,
    pub formal: usize,
    pub read: Site,
    pub branch: Site,
    pub then_block: usize,
    pub else_block: usize,
    pub inverted: bool,
    pub nodes: Vec<BitNode>,
    /// Indexed by the actual captured Boolean formal, false then true.
    pub arms: [usize; 2],
    pub captures: Vec<Site>,
}

impl BitReturn {
    /// Inline graph and owned vector capacities, excluding shared syntax and allocator rounding.
    pub fn payload_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.frame.owned_buffer_bytes()
            + self.nodes.capacity() * std::mem::size_of::<BitNode>()
            + self.captures.capacity() * std::mem::size_of::<Site>()
    }

    pub(in crate::rules::lifetime_summary) fn requirements(
        &self,
    ) -> impl Iterator<Item = &super::super::frames::Requirement> {
        self.frame.requirements()
    }

    pub(in crate::rules::lifetime_summary) fn requirement_count(&self) -> usize {
        self.frame.requirement_count()
    }

    /// Valid only for this checked compilation and its current declaration constraints.
    pub fn current(&self, c: &Compiler, all: &HashMap<ProcId, Summary>) -> bool {
        self.frame.current(c) && self.frame.compatible(all)
    }
}

/// A graph is validated before the shared Frame observer admits its exact outputs.
pub(in crate::rules::lifetime_summary) struct PendingBits {
    formal: usize,
    read: Site,
    branch: Site,
    then_block: usize,
    else_block: usize,
    inverted: bool,
    nodes: Vec<BitNode>,
    arms: [usize; 2],
    captures: Vec<Site>,
    outputs: Vec<Val>,
    work: usize,
    capture_index: QueryIndex,
}

impl PendingBits {
    pub(in crate::rules::lifetime_summary) fn observer_work(&self) -> usize {
        self.work
    }
    pub(in crate::rules::lifetime_summary) fn observer_index(&self) -> &QueryIndex {
        &self.capture_index
    }
    pub(in crate::rules::lifetime_summary) fn observer_outputs(&self) -> &[Val] {
        &self.outputs
    }
    pub fn observer(&self, r: &Resolver<'_>) -> frames::Builder {
        frames::Builder::bit_return(r, self)
    }

    pub fn finish(self, frame: frames::Frame) -> BitReturn {
        BitReturn {
            frame,
            formal: self.formal,
            read: self.read,
            branch: self.branch,
            then_block: self.then_block,
            else_block: self.else_block,
            inverted: self.inverted,
            nodes: self.nodes,
            arms: self.arms,
            captures: self.captures,
        }
    }
}

use super::producers::{Producer, QueryIndex};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Goal {
    Integer,
    PointerFormal,
    Boolean,
    Return,
}

#[derive(Clone, Copy)]
enum Answer {
    Node(usize),
    Formal {
        index: usize,
        read: Option<Site>,
        inverted: bool,
    },
}

struct Graph<'r, 'a> {
    r: &'r Resolver<'a>,
    loads: &'r CapturedLoads<'a>,
    nodes: &'r mut Vec<BitNode>,
    captures: &'r mut Vec<Site>,
    work: &'r mut usize,
    seen: HashSet<(Val, Goal)>,
    producers: &'r QueryIndex,
    // Query-local opaque native SSA binding; never an ownership certificate.
    captured_root: Option<Option<Val>>,
}

fn charge(work: &mut usize, amount: usize) -> Option<()> {
    *work = work.checked_add(amount)?;
    (*work <= MAX_WORK && !jaic::interrupt::should_stop()).then_some(())
}

impl<'r, 'a> Graph<'r, 'a> {
    fn new(
        r: &'r Resolver<'a>,
        loads: &'r CapturedLoads<'a>,
        nodes: &'r mut Vec<BitNode>,
        captures: &'r mut Vec<Site>,
        work: &'r mut usize,
        producers: &'r QueryIndex,
    ) -> Option<Self> {
        charge(work, 1)?;
        if !r.owns_capture_index(producers) || !loads.matches(r.checked_body()) {
            return None;
        }
        Some(Self {
            r,
            loads,
            nodes,
            captures,
            work,
            seen: HashSet::default(),
            producers,
            captured_root: None,
        })
    }

    fn producer(&mut self, value: Val) -> Option<Producer> {
        charge(self.work, 1)?;
        self.producers.get(value)
    }

    fn node(&mut self, node: BitNode) -> Option<Answer> {
        charge(self.work, self.nodes.len() + 1)?;
        let index = if let Some(index) = self.nodes.iter().position(|old| *old == node) {
            index
        } else {
            if self.nodes.len() >= MAX_NODES {
                return None;
            }
            self.nodes.push(node);
            self.nodes.len() - 1
        };
        Some(Answer::Node(index))
    }

    fn capture(&mut self, site: Site) -> Option<()> {
        charge(self.work, self.captures.len() + 1)?;
        if !self.captures.contains(&site) {
            self.captures.push(site);
        }
        Some(())
    }

    fn query(&mut self, value: Val, goal: Goal, depth: usize) -> Option<Answer> {
        charge(self.work, 1)?;
        if depth >= 24 || !self.seen.insert((value, goal)) {
            return None;
        }
        let result = self.query_inner(value, goal, depth);
        if result.is_none()
            && self.captured_root.is_some()
            && jacket_config::env::LIFETIME_STATS.is_set()
        {
            let producer = self.producers.get(value);
            eprintln!(
                "lifetime-object-bit-stop value={:?} depth={} producer={} loads={:?}",
                value,
                depth,
                match producer {
                    Some(Producer::Load(..)) => "load",
                    Some(Producer::Copy(..)) => "copy",
                    Some(Producer::Cast(..)) => "cast",
                    Some(Producer::Binary(..)) => "binary",
                    Some(Producer::Call(..)) => "call",
                    _ => "other",
                },
                self.loads.get(&value)
            );
        }
        self.seen.remove(&(value, goal));
        result
    }

    fn query_inner(&mut self, value: Val, goal: Goal, depth: usize) -> Option<Answer> {
        match self.producer(value)? {
            Producer::Copy(source) => self.query(source, goal, depth + 1),
            Producer::Load(ty, site) => self.load(value, ty, site, goal, depth),
            Producer::Call(_, _, _, ty)
                if goal == Goal::PointerFormal
                    && self.captured_root.is_some()
                    && self.r.compiler.ty.types.is_pointer(ty)
                    && self.r.compiler.ty.types.size_of(ty) == 8 =>
            {
                let root = self.captured_root.as_mut()?;
                if root.is_some_and(|root| root != value) {
                    return None;
                }
                *root = Some(value);
                Some(Answer::Formal {
                    index: 0,
                    read: None,
                    inverted: false,
                })
            }
            Producer::Parameter(index, ty) => {
                if self.captured_root.is_some() {
                    return None;
                }
                let sig = self.r.compiler.proc(self.r.body.procedure).sig.as_ref()?;
                if sig.params.get(index)?.ty != ty {
                    return None;
                }
                match goal {
                    Goal::Integer if ty == TypeId::U64 => self.node(BitNode::Unsigned(index)),
                    Goal::Boolean if ty == TypeId::BOOL => Some(Answer::Formal {
                        index,
                        read: None,
                        inverted: false,
                    }),
                    Goal::PointerFormal
                        if self.r.compiler.ty.types.is_pointer(ty)
                            && self.r.compiler.ty.types.size_of(ty) == 8 =>
                    {
                        Some(Answer::Formal {
                            index,
                            read: None,
                            inverted: false,
                        })
                    }
                    _ => None,
                }
            }
            Producer::Not(source) if goal == Goal::Boolean => {
                let Answer::Formal {
                    index,
                    read,
                    inverted,
                } = self.query(source, goal, depth + 1)?
                else {
                    return None;
                };
                Some(Answer::Formal {
                    index,
                    read,
                    inverted: !inverted,
                })
            }
            Producer::Cast(source, false, 8) if goal == Goal::Integer => {
                let Answer::Formal {
                    index, ..
                } = self.query(source, Goal::PointerFormal, depth + 1)?
                else {
                    return None;
                };
                self.node(BitNode::Pointer(index))
            }
            Producer::Cast(source, true, 8) if goal == Goal::Return => {
                self.query(source, Goal::Integer, depth + 1)
            }
            Producer::Binary(TypeId::U64, left, right, op, lc, rc) if goal == Goal::Integer => {
                if !matches!(
                    op,
                    jaic::ir::BinOp::Add | jaic::ir::BinOp::And | jaic::ir::BinOp::Sub
                ) {
                    return None;
                }
                if op == jaic::ir::BinOp::Sub && rc.is_none() {
                    return None;
                }
                let left = if let Some(n) = lc {
                    self.node(BitNode::Constant(n))?
                } else {
                    self.query(left, goal, depth + 1)?
                };
                let right = if let Some(n) = rc {
                    self.node(BitNode::Constant(n))?
                } else {
                    self.query(right, goal, depth + 1)?
                };
                let (Answer::Node(left), Answer::Node(right)) = (left, right) else {
                    return None;
                };
                self.node(match op {
                    jaic::ir::BinOp::And => BitNode::And(left, right),
                    jaic::ir::BinOp::Sub => BitNode::Sub(left, right),
                    _ => BitNode::Add(left, right),
                })
            }
            _ => None,
        }
    }

    fn load(
        &mut self,
        value: Val,
        ty: TypeId,
        site: Site,
        goal: Goal,
        depth: usize,
    ) -> Option<Answer> {
        let valid = match goal {
            Goal::Integer => ty == TypeId::U64,
            Goal::Boolean => ty == TypeId::BOOL,
            Goal::PointerFormal | Goal::Return => {
                self.r.compiler.ty.types.is_pointer(ty) && self.r.compiler.ty.types.size_of(ty) == 8
            }
        };
        if !valid {
            return None;
        }
        let sources = self.loads.get(&value)?;
        charge(self.work, sources.len() + 1)?;
        if sources.len() != 1 {
            return None;
        }
        let source = *sources.iter().next()?;
        self.capture(site)?;
        let result = match source {
            Source::ScalarValue {
                ty: actual,
                value,
            } if actual == ty && matches!(goal, Goal::Integer | Goal::Boolean) => {
                self.query(value, goal, depth + 1)
            }
            Source::Constant {
                ty: TypeId::U64,
                value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(bits),
            } if goal == Goal::Integer => self.node(BitNode::Constant(bits)),
            Source::Value(value) if matches!(goal, Goal::PointerFormal | Goal::Return) => {
                self.query(value, goal, depth + 1)
            }
            _ => None,
        }?;
        Some(match result {
            Answer::Formal {
                index,
                inverted,
                ..
            } if goal == Goal::Boolean => Answer::Formal {
                index,
                read: Some(site),
                inverted,
            },
            _ => result,
        })
    }
}

/// Normalize one captured pointer expression into machine-bit operations rooted
/// at one actual call-result SSA. Authentication, nonnull and extent are separate
/// caller obligations; this query supplies no allocation or generation fact.
pub(in crate::rules::lifetime_summary) fn captured_address_bits<'c>(
    r: &Resolver<'c>,
    loads: &CapturedLoads<'c>,
    producers: &QueryIndex,
    value: Val,
    work: &mut usize,
) -> Option<(Val, Vec<BitNode>, usize)> {
    if r.compiler.ty.types.size_of(TypeId::VOID_PTR) != 8 {
        return None;
    }
    let mut nodes = Vec::new();
    let mut captures = Vec::new();
    let mut graph = Graph::new(r, loads, &mut nodes, &mut captures, work, producers)?;
    graph.captured_root = Some(None);
    let Answer::Node(result) = graph.query(value, Goal::Return, 0)? else {
        return None;
    };
    let root = graph.captured_root.flatten()?;
    charge(graph.work, 1)?;
    Some((root, nodes, result))
}

impl Resolver<'_> {
    /// Build both actual Boolean alternatives once, without interpreting resource state.
    pub(in crate::rules::lifetime_summary) fn bit_return_plan(
        &self,
        recursive: bool,
    ) -> Option<PendingBits> {
        if recursive {
            return None;
        }
        let requests = self.core().scalar_requests.as_ref()?;
        let mut work = requests.work_bound;
        let body = self.body;
        if body.blocks.is_empty() || body.blocks.len() > 32 {
            return None;
        }
        let mut scan = body.blocks.len().checked_add(body.address_casts.len())?;
        charge(
            &mut work,
            requests.reads.len() * (requests.reads.len() + 4) + body.blocks.len() + 1,
        )?;
        let sig = self.compiler.proc(body.procedure).sig.as_ref()?;
        if sig.returns.len() != 1
            || !self.compiler.ty.types.is_pointer(sig.returns[0])
            || self.compiler.ty.types.size_of(sig.returns[0]) != 8
        {
            return None;
        }
        let mut branch = None;
        for (block, b) in body.blocks.iter().enumerate() {
            charge(&mut work, b.events.len() + b.successors.len() + 1)?;
            scan = scan.checked_add(b.events.len())?;
            for e in &b.events {
                if let Event::Call {
                    results, ..
                }
                | Event::IndirectCall {
                    results, ..
                } = e
                {
                    scan = scan.checked_add(results.len())?;
                }
            }
            for (event, e) in b.events.iter().enumerate() {
                if let Event::Branch {
                    condition,
                    then_block,
                    else_block,
                    ..
                } = e
                    && branch
                        .replace((
                            Site {
                                block,
                                event,
                            },
                            *condition,
                            *then_block,
                            *else_block,
                        ))
                        .is_some()
                {
                    return None;
                }
            }
        }
        let mut remaining = MAX_WORK.checked_sub(work)?;
        let producers = self.capture_index(&mut remaining)?;
        work = MAX_WORK - remaining;
        let (site, condition, yes, no) = branch?;
        if yes == no
            || yes >= body.blocks.len()
            || no >= body.blocks.len()
            || body.blocks[site.block].successors.len() != 2
            || !body.blocks[site.block].successors.contains(&yes)
            || !body.blocks[site.block].successors.contains(&no)
            || body.blocks.iter().enumerate().any(|(i, b)| {
                b.successors.iter().any(|n| *n >= body.blocks.len())
                    || (i != site.block && b.successors.len() > 1)
            })
        {
            return None;
        }
        let mut nodes = Vec::new();
        let mut captures = Vec::new();
        let Answer::Formal {
            index: formal,
            read: Some(read),
            inverted,
        } = Graph::new(
            self,
            &self.core().storage_versions,
            &mut nodes,
            &mut captures,
            &mut work,
            &producers,
        )?
        .query(condition, Goal::Boolean, 0)?
        else {
            return None;
        };
        let probes = super::super::call_slots::Probes::for_scalar_requests(requests);
        let mut arms = [0; 2];
        let mut outputs = Vec::new();
        let mut covered = vec![false; body.blocks.len()];
        for (value, arm) in arms.iter_mut().enumerate() {
            let excluded = if (value == 1) ^ inverted {
                no
            } else {
                yes
            };
            let cuts = HashSet::from_iter([(site.block, excluded)]);
            charge(&mut work, (body.blocks.len() + 1) * (body.blocks.len() + 1))?;
            let mut repeated = false;
            let states = flow::states_except(
                body,
                &flow::Live::all(body),
                (),
                &cuts,
                |(), ()| {
                    repeated = true;
                    false
                },
                |(), _, _| {},
            )?;
            if repeated || states[site.block].is_none() {
                return None;
            }
            let (loads, _) = super::super::storage_versions::loads_except(
                &self.places,
                self.core(),
                &probes,
                &cuts,
            );
            let mut returned = None;
            charge(&mut work, scan)?;
            for (block, b) in body
                .blocks
                .iter()
                .enumerate()
                .filter(|(i, _)| states[*i].is_some())
            {
                covered[block] = true;
                let mut count = 0;
                for e in &b.events {
                    if let Event::Return {
                        index,
                        value,
                        ty,
                        ..
                    } = e
                    {
                        if !b.normal_return {
                            return None;
                        }
                        if *index != 0 || *ty != sig.returns[0] {
                            return None;
                        }
                        count += 1;
                        let Answer::Node(node) = Graph::new(
                            self,
                            &loads,
                            &mut nodes,
                            &mut captures,
                            &mut work,
                            &producers,
                        )?
                        .query(*value, Goal::Return, 0)?
                        else {
                            return None;
                        };
                        if returned.is_some_and(|prior| prior != node) {
                            return None;
                        }
                        returned = Some(node);
                        charge(&mut work, outputs.len() + 1)?;
                        if !outputs.contains(value) {
                            outputs.push(*value);
                        }
                    }
                }
                if b.normal_return && count != 1 {
                    return None;
                }
            }
            *arm = returned?;
        }
        if !covered.first().copied().unwrap_or(false)
            || body
                .blocks
                .iter()
                .enumerate()
                .any(|(i, b)| covered[i] && b.successors.iter().any(|s| !covered[*s]))
        {
            return None;
        }
        charge(&mut work, nodes.len() + captures.len() + outputs.len() + 1)?;
        Some(PendingBits {
            formal,
            read,
            branch: site,
            then_block: yes,
            else_block: no,
            inverted,
            nodes,
            arms,
            captures,
            outputs,
            work,
            capture_index: producers,
        })
    }
}

#[cfg(test)]
mod capture_ownership_tests {
    use super::*;
    #[test]
    fn reader_rejects_another_bodys_captured_loads() {
        let c = crate::rules::lifetime_summary::tests::stdlib_allocations::checked_source(
            "first::(p:*void)->*void{return p;} second::(p:*void)->*void{return p;} main::(){}",
        );
        let all = crate::rules::lifetime_summary::Summaries::build(&c);
        let session = crate::rules::lifetime_facts::CheckedSession::new(&c);
        let key = |name: &str| {
            ProcId(
                c.pr.procs
                    .iter()
                    .position(|p| p.name.as_str() == name)
                    .unwrap() as u32,
            )
        };
        let a = crate::rules::lifetime_summary::Prepared::for_body(
            session.body(key("first")).unwrap(),
            &all,
        );
        let b = crate::rules::lifetime_summary::Prepared::for_body(
            session.body(key("second")).unwrap(),
            &all,
        );
        let r = Resolver::for_prepared(&a, &all);
        let other = Resolver::for_prepared(&b, &all);
        let mut remaining = 8192;
        let index = r.capture_index(&mut remaining).unwrap();
        let mut nodes = Vec::new();
        let mut captures = Vec::new();
        let mut work = 8192 - remaining;
        assert!(
            Graph::new(
                &r,
                &r.core().storage_versions,
                &mut nodes,
                &mut captures,
                &mut work,
                &index
            )
            .is_some()
        );
        assert!(
            Graph::new(
                &r,
                &other.core().storage_versions,
                &mut nodes,
                &mut captures,
                &mut work,
                &index
            )
            .is_none()
        );
    }
}
