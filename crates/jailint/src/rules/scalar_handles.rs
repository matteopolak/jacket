//! Fresh scalar acquisitions, separate from pointer-allocation origins.
use super::lifetime_facts::{
    Address, ByteRange, ByteSlot as Slot, EventSite as Site, StackRoots, current_body,
};
use jaic::fxhash::{HashMap, HashSet};
use jaic::ide::lifetimes::{
    LifetimeBody, LifetimeBoolean, LifetimeEvent as Event, LifetimeOperand as Operand,
};
use jaic::ir::{CmpOp, Ty, Val};
use jaic::sema::{Compiler, ProcId, procs::ProcTarget};
use jaic::source::FileId;
use jaic::types::{TypeId, TypeKind};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum NativeKind {
    Descriptor,
    Stream,
    WindowsHandle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) enum Validity {
    NonNegative,
    NonNull,
    NeitherNullNorMinusOne,
}

impl Validity {
    fn accepts(self, value: u64) -> bool {
        match self {
            Self::NonNegative => value & (1 << 31) == 0,
            Self::NonNull => value != 0,
            Self::NeitherNullNorMinusOne => value != 0 && value != u64::MAX,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(in crate::rules) struct ResultGuard {
    pub result: usize,
    pub valid_when: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::rules) struct ReturnCapability {
    pub result: usize,
    pub slot: Option<Slot>,
    pub kind: NativeKind,
    pub validity: Validity,
    pub guard: Option<ResultGuard>,
    pub template: Vec<Site>,
    /// Each invocation must preserve this acquisition before the result is proved live.
    pub requires_noninterference: Vec<Site>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::rules) struct Opening {
    pub alternatives: Vec<ReturnCapability>,
    pub incomplete: bool,
}

#[derive(Default)]
pub(in crate::rules) struct CallOpenings {
    calls: HashMap<(ProcId, usize, usize), Vec<usize>>,
    probed_bodies: usize,
}

impl CallOpenings {
    /// Results still unhandled at every applicable acquired exit of this caller.
    pub fn positions(&self, procedure: ProcId, block: usize, event: usize) -> Option<&[usize]> {
        self.calls
            .get(&(procedure, block, event))
            .map(Vec::as_slice)
    }
}

#[derive(Default)]
pub(in crate::rules) struct Analysis {
    openings: HashMap<ProcId, Opening>,
}

impl Analysis {
    /// Build only from current, successfully checked bodies. No source notes seed freshness.
    /// For example, build `let analysis = Analysis::build(compiler);`, then query
    /// `analysis.opening(id)` while that run-local analysis is alive.
    #[cfg(test)]
    pub fn build(c: &Compiler) -> Self {
        Self::build_internal(c, false, &HashSet::default()).0
    }

    /// Capture actual returned acquisitions in caller CFGs and follow native
    /// releases, aliases and liveness frames. For example, closing the first
    /// result can preserve a separately acquired second result's obligation.
    pub fn caller_openings(c: &Compiler, files: &HashSet<FileId>) -> CallOpenings {
        Self::build_internal(c, true, files).1
    }

    fn build_internal(
        c: &Compiler,
        include_callers: bool,
        files: &HashSet<FileId>,
    ) -> (Self, CallOpenings) {
        if jaic::interrupt::should_stop() {
            return (Self::default(), CallOpenings::default());
        }
        let Some(facts) = c.ed.facts() else {
            return (Self::default(), CallOpenings::default());
        };
        let relations_timer = super::lifetime_stats::Timer::start();
        let bodies = facts
            .lifetime_bodies
            .iter()
            .filter(|(id, b)| current_body(c, **id, b))
            .map(|(id, b)| (*id, b))
            .collect();
        let mut solver = Solver {
            c,
            bodies,
            cache: HashMap::default(),
            active: HashSet::default(),
        };
        let mut out = Self::default();
        for i in 0..c.pr.procs.len() {
            if jaic::interrupt::should_stop() {
                return (Self::default(), CallOpenings::default());
            }
            let id = ProcId(i as u32);
            if let Some(r) = solver.relation(id) {
                if jaic::interrupt::should_stop() {
                    return (Self::default(), CallOpenings::default());
                }
                let opening = capabilities(&r);
                if !opening.alternatives.is_empty() {
                    out.openings.insert(id, opening);
                }
            }
        }
        if jaic::interrupt::should_stop() {
            return (Self::default(), CallOpenings::default());
        }
        relations_timer.finish("scalar_relations", None, solver.cache.len());
        let calls = if include_callers {
            let timer = super::lifetime_stats::Timer::start();
            let calls = solver.caller_openings(&out, files);
            timer.finish("scalar_callers", None, calls.probed_bodies);
            calls
        } else {
            CallOpenings::default()
        };
        if jaic::interrupt::should_stop() {
            (Self::default(), CallOpenings::default())
        } else {
            (out, calls)
        }
    }

    /// Possible fresh origins, conditional on each capability's noninterference sites.
    /// `None` is unknown, never an absence-of-effects proof.
    pub fn opening(&self, id: ProcId) -> Option<&Opening> {
        self.openings.get(&id)
    }
}

#[cfg(test)]
#[path = "scalar_handles/tests.rs"]
mod tests;

const MAX_STATES: usize = 256;
const MAX_STEPS: usize = 8192;
const MAX_DEPTH: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Token {
    kind: NativeKind,
    validity: Validity,
    sites: Vec<Site>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expr {
    Unknown,
    Constant(u64),
    Input(usize, Slot),
    Fresh(Token),
    Compare(Box<Expr>, CmpOp, u64, Ty),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct HandleEffect {
    value: Expr,
    kind: Option<NativeKind>,
}

#[derive(Clone, Default, PartialEq, Eq)]
struct State {
    values: HashMap<Val, Expr>,
    addresses: HashMap<Val, Address>,
    roots: Option<Rc<StackRoots>>,
    borrowed: HashMap<Val, u64>,
    exposed: HashSet<Val>,
    slots: HashMap<(Address, Slot), Expr>,
    returns: Vec<(usize, Option<Slot>, Expr)>,
    probes: HashMap<usize, Expr>,
    conditions: Vec<(Expr, bool)>,
    invalid: HashSet<Token>,
    consumed: Vec<HandleEffect>,
    uncertain: Vec<HandleEffect>,
    global_consumed: HashSet<NativeKind>,
    global_sites: Vec<Site>,
    global_unknown: bool,
    frames: HashMap<Token, Vec<Site>>,
    liveness_work: usize,
    liveness_exhausted: bool,
    requirements: Vec<(Expr, NativeKind)>,
    visited: HashSet<usize>,
    interferes: bool,
}

/// The exported relation contains no private memory or traversal working state.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
struct Exit {
    returns: Vec<(usize, Option<Slot>, Expr)>,
    probes: HashMap<usize, Expr>,
    conditions: Vec<(Expr, bool)>,
    invalid: HashSet<Token>,
    consumed: Vec<HandleEffect>,
    uncertain: Vec<HandleEffect>,
    global_consumed: HashSet<NativeKind>,
    global_sites: Vec<Site>,
    global_unknown: bool,
    frames: HashMap<Token, Vec<Site>>,
    requirements: Vec<(Expr, NativeKind)>,
    interferes: bool,
}

impl State {
    /// Move captured return/effect facts out at the actual normal-return boundary.
    fn into_exit(mut self) -> Option<Exit> {
        if self.liveness_exhausted {
            return None;
        }
        self.conditions.retain(|(x, _)| related(x));
        Some(Exit {
            returns: self.returns,
            probes: self.probes,
            conditions: self.conditions,
            invalid: self.invalid,
            consumed: self.consumed,
            uncertain: self.uncertain,
            global_consumed: self.global_consumed,
            global_sites: self.global_sites,
            global_unknown: self.global_unknown,
            frames: self.frames,
            requirements: self.requirements,
            interferes: self.interferes,
        })
    }
}

#[derive(Clone, Default)]
struct Relation {
    exits: Vec<Exit>,
}

struct Solver<'a> {
    c: &'a Compiler,
    bodies: HashMap<ProcId, &'a LifetimeBody>,
    cache: HashMap<ProcId, Option<Relation>>,
    active: HashSet<ProcId>,
}

mod calls;
mod events;
#[path = "scalar_handles/guards.rs"]
mod guards;
#[path = "scalar_handles/native.rs"]
mod native;
#[path = "scalar_handles/slots.rs"]
mod slots;
#[path = "scalar_handles/transfer.rs"]
mod transfer;
use guards::capabilities;
use native::native;
use slots::{comparison_constant, leaves, related, reverse, scalar, substitute};

#[path = "scalar_handles/expression.rs"]
mod expression;

#[path = "scalar_handles/liveness.rs"]
mod liveness;

#[path = "scalar_handles/caller.rs"]
mod caller;

#[path = "scalar_handles/proof.rs"]
mod proof;
