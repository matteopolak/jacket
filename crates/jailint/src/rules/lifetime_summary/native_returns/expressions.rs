//! Typed, epoch-captured expressions used by conditional native witnesses.
use super::super::storage_versions::Source;
use super::{
    Event, HashMap, HashSet, MAX_NODES, ProcId, Producer, Resolver, Site, Summary, TypeId, Val,
    Work,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Op {
    Input(usize),
    Constant(u64),
    Add(usize, usize),
    Sub(usize, usize),
    And(usize, usize),
    Not(usize),
    Or(usize, usize),
    Both(usize, usize),
    Compare(jaic::ir::CmpOp, usize, usize),
    Native(ProcId, Site, Vec<usize>),
    Bits(ProcId, Site, Vec<usize>),
}

impl std::hash::Hash for Op {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Self::Input(i) => i.hash(state),
            Self::Constant(n) => n.hash(state),
            Self::Add(a, b)
            | Self::Sub(a, b)
            | Self::And(a, b)
            | Self::Or(a, b)
            | Self::Both(a, b) => (a, b).hash(state),
            Self::Not(a) => a.hash(state),
            Self::Compare(op, a, b) => {
                std::mem::discriminant(op).hash(state);
                (a, b).hash(state);
            }
            Self::Native(p, site, args) | Self::Bits(p, site, args) => (p, site, args).hash(state),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Node {
    pub(super) ty: TypeId,
    pub(super) op: Op,
}

pub(super) struct Parser<'a, 'r> {
    r: &'a Resolver<'r>,
    all: &'a HashMap<ProcId, Summary>,
    loads: &'a super::super::storage_versions::CapturedLoads<'r>,
    producers: &'a super::super::captured_values::QueryIndex,
    pub(super) nodes: Vec<Node>,
    node_ids: HashMap<Node, usize>,
    pub(super) native_dependent: Vec<bool>,
    seen: HashSet<Val>,
    captured: HashMap<(Val, usize), usize>,
    pub(super) work: &'a Work,
}

impl<'a, 'r> Parser<'a, 'r> {
    pub(super) fn new(
        r: &'a Resolver<'r>,
        all: &'a HashMap<ProcId, Summary>,
        loads: &'a super::super::storage_versions::CapturedLoads<'r>,
        producers: &'a super::super::captured_values::QueryIndex,
        work: &'a Work,
    ) -> Option<Self> {
        work.charge(1)?;
        if !r.owns_capture_index(producers) || !loads.matches(r.checked_body()) {
            return None;
        }
        Some(Self {
            r,
            all,
            loads,
            producers,
            nodes: Vec::new(),
            node_ids: HashMap::default(),
            native_dependent: Vec::new(),
            seen: HashSet::default(),
            captured: HashMap::default(),
            work,
        })
    }

    fn add(&mut self, ty: TypeId, op: Op) -> Option<usize> {
        let args = match &op {
            Op::Native(_, _, args) | Op::Bits(_, _, args) => args.len(),
            _ => 0,
        };
        self.work.charge(args.checked_mul(4)?.checked_add(4)?)?;
        let node = Node {
            ty,
            op,
        };
        if let Some(&id) = self.node_ids.get(&node) {
            return Some(id);
        }
        if self.nodes.len() == MAX_NODES {
            return None;
        }
        let dependent = match &node.op {
            Op::Native(..) => true,
            Op::Input(_) | Op::Constant(_) => false,
            Op::Not(a) => *self.native_dependent.get(*a)?,
            Op::Add(a, b)
            | Op::Sub(a, b)
            | Op::And(a, b)
            | Op::Or(a, b)
            | Op::Both(a, b)
            | Op::Compare(_, a, b) => {
                *self.native_dependent.get(*a)? || *self.native_dependent.get(*b)?
            }
            Op::Bits(_, _, args) => args.iter().any(|a| self.native_dependent[*a]),
        };
        let id = self.nodes.len();
        self.node_ids.insert(node.clone(), id);
        self.native_dependent.push(dependent);
        self.nodes.push(node);
        Some(id)
    }
    fn literal(&mut self, ty: TypeId, bits: u64) -> Option<usize> {
        let c = self.r.compiler;
        let width = if ty == TypeId::BOOL {
            if bits > 1 {
                return None;
            }
            8
        } else if c.ty.types.is_pointer(ty) {
            if bits != 0 || c.ty.types.size_of(ty) != 8 {
                return None;
            }
            64
        } else {
            let (width, _) = c.ty.types.int_info(ty)?;
            width
        };
        if width == 0 || width > 64 || (width < 64 && bits >> width != 0) {
            return None;
        }
        self.add(ty, Op::Constant(bits))
    }
    pub(super) fn value(&mut self, value: Val, depth: usize) -> Option<usize> {
        self.work.charge(1)?;
        if let Some(&id) = self.captured.get(&(value, depth)) {
            return Some(id);
        }
        if depth >= 24 || !self.seen.insert(value) {
            return None;
        }
        let result = self.definition(value, depth);
        self.seen.remove(&value);
        if let Some(id) = result {
            self.work.charge(1)?;
            self.captured.insert((value, depth), id);
        }
        result
    }
    fn definition(&mut self, value: Val, depth: usize) -> Option<usize> {
        let c = self.r.compiler;
        match self.producers.get(value)? {
            Producer::Parameter(index, ty) => {
                if c.proc(self.r.body.procedure)
                    .sig
                    .as_ref()?
                    .params
                    .get(index)?
                    .ty
                    != ty
                {
                    return None;
                }
                self.add(ty, Op::Input(index))
            }
            Producer::Load(ty, _) => {
                let sources = self.loads.get(&value)?;
                self.work.charge(sources.len() + 1)?;
                if sources.len() != 1 {
                    return None;
                }
                let id = match *sources.iter().next()? {
                    Source::ScalarValue {
                        ty: actual,
                        value,
                    } if actual == ty && !c.ty.types.is_pointer(ty) => {
                        self.value(value, depth + 1)?
                    }
                    Source::Value(value) if c.ty.types.is_pointer(ty) => {
                        self.value(value, depth + 1)?
                    }
                    Source::Constant {
                        ty: actual,
                        value: jaic::ide::lifetimes::LifetimeDefaultValue::Scalar(bits),
                    } if actual == ty => self.literal(ty, bits)?,
                    _ => return None,
                };
                (self.nodes[id].ty == ty).then_some(id)
            }
            Producer::Copy(source) => self.value(source, depth + 1),
            Producer::Binary(ty, left, right, op, lc, rc)
                if matches!(ty, TypeId::U64 | TypeId::S64) =>
            {
                let a = if let Some(n) = lc {
                    self.literal(ty, n)?
                } else {
                    self.value(left, depth + 1)?
                };
                let b = if let Some(n) = rc {
                    self.literal(ty, n)?
                } else {
                    self.value(right, depth + 1)?
                };
                if c.ir_ty(self.nodes[a].ty) != Some(jaic::ir::Ty::I64)
                    || c.ir_ty(self.nodes[b].ty) != Some(jaic::ir::Ty::I64)
                {
                    return None;
                }
                self.add(
                    ty,
                    match op {
                        jaic::ir::BinOp::Add => Op::Add(a, b),
                        jaic::ir::BinOp::Sub => Op::Sub(a, b),
                        jaic::ir::BinOp::And => Op::And(a, b),
                        _ => return None,
                    },
                )
            }
            Producer::Not(source) => {
                let id = self.value(source, depth + 1)?;
                self.add(TypeId::BOOL, Op::Not(id))
            }
            Producer::And(a, b) | Producer::Or(a, b) => {
                let left = self.value(a, depth + 1)?;
                let right = self.value(b, depth + 1)?;
                let op = if matches!(self.producers.get(value), Some(Producer::And(..))) {
                    Op::Both(left, right)
                } else {
                    Op::Or(left, right)
                };
                self.add(TypeId::BOOL, op)
            }
            Producer::Comparison(index) => {
                let cmp = self.r.body.comparisons.get(index)?.clone();
                let ty = match cmp.ty {
                    jaic::ir::Ty::I8 => TypeId::U8,
                    jaic::ir::Ty::I16 => TypeId::U16,
                    jaic::ir::Ty::I32 => TypeId::U32,
                    jaic::ir::Ty::I64 => TypeId::U64,
                    jaic::ir::Ty::Ptr => TypeId::VOID_PTR,
                    _ => return None,
                };
                let left = if let Some(n) = cmp.left_constant {
                    self.literal(ty, n)?
                } else {
                    self.value(cmp.left, depth + 1)?
                };
                let right = if let Some(n) = cmp.right_constant {
                    self.literal(ty, n)?
                } else {
                    self.value(cmp.right, depth + 1)?
                };
                if c.ir_ty(self.nodes[left].ty) != Some(cmp.ty)
                    || c.ir_ty(self.nodes[right].ty) != Some(cmp.ty)
                {
                    return None;
                }
                self.add(TypeId::BOOL, Op::Compare(cmp.op, left, right))
            }
            Producer::Call(callee, site, index, ty) => {
                if index != 0 || !c.ty.types.is_pointer(ty) {
                    return None;
                }
                let args = self.arguments(callee, site, depth)?;
                if self.all.get(&callee)?.native_allocation.is_some() {
                    self.add(ty, Op::Native(callee, site, args))
                } else {
                    let proof = self.all.get(&callee)?.bit_return.as_ref()?;
                    let count = proof.requirement_count();
                    if count > 33 {
                        return None;
                    }
                    self.work.charge(count * 2 + 1)?;
                    proof.current(c, self.all).then_some(())?;
                    self.add(ty, Op::Bits(callee, site, args))
                }
            }
            _ => None,
        }
    }
    pub(super) fn arguments(
        &mut self,
        callee: ProcId,
        site: Site,
        depth: usize,
    ) -> Option<Vec<usize>> {
        let Event::Call {
            callee: actual,
            args,
            results,
            expands: false,
            ..
        } = self.r.body.blocks.get(site.block)?.events.get(site.event)?
        else {
            return None;
        };
        let signature = self
            .r
            .compiler
            .pr
            .procs
            .get(callee.0 as usize)?
            .sig
            .as_ref()?;
        self.work.charge(args.len().checked_mul(args.len() + 4)?)?;
        if *actual != callee
            || args.len() != signature.params.len()
            || args.len() > 8
            || results.len() != signature.returns.len()
            || results.len() > 1
            || results
                .iter()
                .zip(&signature.returns)
                .enumerate()
                .any(|(i, ((index, _, ty), expected))| *index != i || ty != expected)
        {
            return None;
        }
        let mut out = Vec::with_capacity(args.len());
        for (formal, param) in signature.params.iter().enumerate() {
            let mut selected = args.iter().filter(|a| a.formal == formal);
            let arg = selected.next()?;
            if selected.next().is_some() || arg.spread || param.variadic || arg.ty != param.ty {
                return None;
            }
            let id = if let Some(bits) = arg.constant {
                if let super::super::LifetimeOperand::Value(value) = arg.operand
                    && self.producers.contains(value)
                {
                    let id = self.value(value, depth + 1)?;
                    if constant(&self.nodes, id, self.work) != Some(bits) {
                        return None;
                    }
                    id
                } else {
                    self.literal(arg.ty, bits)?
                }
            } else {
                let super::super::LifetimeOperand::Value(value) = arg.operand else {
                    return None;
                };
                self.value(value, depth + 1)?
            };
            // Selected uses of the same immutable SSA preserve machine bits;
            // no reinterpretation of mutable storage or width conversion is inferred.
            if self.r.compiler.ir_ty(self.nodes[id].ty) != self.r.compiler.ir_ty(arg.ty) {
                return None;
            }
            out.push(id);
        }
        Some(out)
    }
}

pub(super) fn constant(nodes: &[Node], id: usize, work: &Work) -> Option<u64> {
    work.charge(id.checked_add(1)?.checked_mul(2)?)?;
    let mut values: Vec<Option<u64>> = Vec::with_capacity(id + 1);
    for node in nodes.get(..=id)? {
        let get = |id: &usize| values.get(*id).copied().flatten();
        let value = match &node.op {
            Op::Constant(value) => Some(*value),
            Op::Add(a, b) => get(a).zip(get(b)).map(|(a, b)| a.wrapping_add(b)),
            Op::Sub(a, b) => get(a).zip(get(b)).map(|(a, b)| a.wrapping_sub(b)),
            Op::And(a, b) => get(a).zip(get(b)).map(|(a, b)| a & b),
            _ => None,
        };
        values.push(value);
    }
    values.get(id).copied().flatten()
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
        let work = Work(std::cell::Cell::new(8192 - remaining));
        assert!(Parser::new(&r, &all.procs, &r.core().storage_versions, &index, &work).is_some());
        assert!(
            Parser::new(
                &r,
                &all.procs,
                &other.core().storage_versions,
                &index,
                &work
            )
            .is_none()
        );
    }
}
