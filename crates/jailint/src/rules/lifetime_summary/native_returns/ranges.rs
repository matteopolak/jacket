//! Invocation-specific range and alignment obligations for a selected path.
use super::{
    Allocation, Case, Cell, Compiler, HashMap, Op, ProcId, Summary, TypeId, Work, native_test,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::rules::lifetime_summary) struct Range {
    pub bytes: u64,
    pub alignment: u64,
    original: u64,
    min: u64,
    max: u64,
    base: bool,
}

impl Range {
    /// Validate signed accesses relative to every possible returned offset.
    /// This proves extent only, conditional on the same original live object.
    pub(in crate::rules::lifetime_summary) fn covers_relative(
        self,
        (start, end, alignment): (i128, i128, u64),
    ) -> bool {
        start <= end
            && alignment != 0
            && self.alignment % alignment == 0
            && i128::from(self.min)
                .checked_add(start)
                .is_some_and(|at| at >= 0)
            && i128::from(self.max)
                .checked_add(end)
                .is_some_and(|at| at <= i128::from(self.original))
    }
}

impl Allocation {
    /// Conditional on this actual call returning nonnull; the caller proves that edge.
    pub(in crate::rules::lifetime_summary) fn apply_nonnull(
        &self,
        c: &Compiler,
        all: &HashMap<ProcId, Summary>,
        (id, inputs): (ProcId, &[Option<u64>]),
        spent: &mut usize,
    ) -> Option<Range> {
        let work = Work(Cell::new(*spent));
        let result = self.apply(c, all, id, inputs, &work, 0);
        *spent = work.0.get();
        result
    }
    fn apply(
        &self,
        c: &Compiler,
        all: &HashMap<ProcId, Summary>,
        id: ProcId,
        inputs: &[Option<u64>],
        work: &Work,
        depth: usize,
    ) -> Option<Range> {
        work.charge(8)?;
        if depth >= 8 || !self.current(c, id, all, depth, work) {
            return None;
        }
        match self {
            Self::Foreign(contract) => {
                if inputs.len() != 1 {
                    return None;
                }
                let bytes = inputs[0]?;
                (bytes > 0 && i64::try_from(bytes).is_ok()).then_some(Range {
                    bytes,
                    alignment: contract.alignment,
                    original: bytes,
                    min: 0,
                    max: 0,
                    base: true,
                })
            }
            Self::Checked(checked) => {
                if inputs.len() != checked.params.len() {
                    return None;
                }
                let mut result: Option<Range> = None;
                for case in &checked.cases {
                    work.charge(case.nodes.len() + case.guards.len() + 1)?;
                    let mut eval = Evaluation {
                        c,
                        all,
                        case,
                        inputs,
                        work,
                        depth,
                        numbers: vec![None; case.nodes.len()],
                        queried: vec![false; case.nodes.len()],
                    };
                    if !case
                        .guards
                        .iter()
                        .all(|&(id, when)| eval.number(id).map(|v| v != 0) == Some(when))
                    {
                        continue;
                    }
                    let range = eval.range(case.result)?;
                    let mut valid = true;
                    for write in &case.prefix {
                        let Some(at) = eval.range(write.pointer) else {
                            valid = false;
                            break;
                        };
                        let lo = i128::from(at.min) + i128::from(write.offset);
                        let hi = i128::from(at.max) + i128::from(write.offset);
                        if write.stored_value.is_some_and(|value| value != case.native)
                            || !write.alignment.is_power_of_two()
                            || at.alignment < write.alignment
                            || i128::from(write.offset) % i128::from(write.alignment) != 0
                            || lo < 0
                            || hi < 0
                            || u64::try_from(hi)
                                .ok()
                                .and_then(|v| v.checked_add(write.width))
                                .is_none_or(|end| end > at.original)
                        {
                            valid = false;
                            break;
                        }
                    }
                    if !valid {
                        return None;
                    }
                    result = Some(match result {
                        None => range,
                        Some(old) => Range {
                            bytes: old.bytes.min(range.bytes),
                            alignment: old.alignment.min(range.alignment),
                            original: old.original.min(range.original),
                            min: old.min.min(range.min),
                            max: old.max.max(range.max),
                            base: old.base && range.base,
                        },
                    });
                }
                work.charge(1)?;
                result
            }
        }
    }
}

/// Conditional range of this captured call-result expression. This does not
/// establish acquisition success, intervening liveness or memory preservation.
pub(in crate::rules::lifetime_summary) fn captured_address_range<'c>(
    r: &super::Resolver<'c>,
    all: &HashMap<ProcId, Summary>,
    loads: &super::super::storage_versions::CapturedLoads<'c>,
    index: &super::super::captured_values::QueryIndex,
    value: super::Val,
    spent: &mut usize,
) -> Option<(super::Val, Range)> {
    let (root, nodes, result) =
        super::super::address_arithmetic::bit_returns::captured_address_bits(
            r, loads, index, value, spent,
        )?;
    let work = Work(Cell::new(*spent));
    let out = (|| {
        work.charge(8)?;
        let super::Producer::Call(callee, site, 0, ty) = index.get(root)? else {
            return None;
        };
        if !r.compiler.ty.types.is_pointer(ty) || r.compiler.ty.types.size_of(ty) != 8 {
            return None;
        }
        let super::Event::Call {
            callee: actual,
            args,
            results,
            expands: false,
            ..
        } = r.body.blocks.get(site.block)?.events.get(site.event)?
        else {
            return None;
        };
        if *actual != callee || args.len() != 1 || results.len() != 1 || results[0] != (0, root, ty)
        {
            return None;
        }
        let arg = &args[0];
        if arg.formal != 0 || arg.spread || arg.ty != TypeId::U64 {
            return None;
        }
        let allocation = all.get(&callee)?.native_allocation.as_ref()?;
        // This first object query accepts an authenticated direct acquisition;
        // checked wrappers retain their separate guarded-return obligations.
        allocation.foreign()?;
        let mut parser = super::Parser::new(r, all, loads, index, &work)?;
        let args = parser.arguments(callee, site, 0)?;
        let bytes = super::expressions::constant(&parser.nodes, *args.first()?, &work)?;
        let base = allocation.apply(r.compiler, all, callee, &[Some(bytes)], &work, 0)?;
        let range = bit_range(&nodes, &[0], &[None], 0, base, result, &work)?;
        work.charge(1)?;
        Some((root, range))
    })();
    *spent = work.0.get();
    out
}

struct Evaluation<'a> {
    c: &'a Compiler,
    all: &'a HashMap<ProcId, Summary>,
    case: &'a Case,
    inputs: &'a [Option<u64>],
    work: &'a Work,
    depth: usize,
    numbers: Vec<Option<u64>>,
    queried: Vec<bool>,
}

impl Evaluation<'_> {
    fn number(&mut self, id: usize) -> Option<u64> {
        self.work.charge(1)?;
        if *self.queried.get(id)? {
            return self.numbers[id];
        }
        self.queried[id] = true;
        let value = if let Some(present) = native_test(&self.case.nodes, id, self.case.native) {
            Some(u64::from(present))
        } else {
            match self.case.nodes.get(id)?.op.clone() {
                Op::Input(formal) => *self.inputs.get(formal)?,
                Op::Constant(bits) => Some(bits),
                Op::Add(a, b) => Some(self.number(a)?.wrapping_add(self.number(b)?)),
                Op::Sub(a, b) => Some(self.number(a)?.wrapping_sub(self.number(b)?)),
                Op::And(a, b) => Some(self.number(a)? & self.number(b)?),
                Op::Not(a) => Some(u64::from(self.number(a)? == 0)),
                Op::Or(a, b) => Some(u64::from(self.number(a)? != 0 || self.number(b)? != 0)),
                Op::Both(a, b) => Some(u64::from(self.number(a)? != 0 && self.number(b)? != 0)),
                Op::Compare(op, a, b) => {
                    let (left, right) = (self.number(a)?, self.number(b)?);
                    compare(self.c, self.case.nodes[a].ty, op, left, right).map(u64::from)
                }
                _ => None,
            }
        };
        self.numbers[id] = value;
        value
    }
    fn range(&mut self, id: usize) -> Option<Range> {
        self.work.charge(1)?;
        match self.case.nodes.get(id)?.op.clone() {
            Op::Native(callee, _, args) => {
                self.work.charge(args.len() + 1)?;
                let inputs: Vec<_> = args.into_iter().map(|a| self.number(a)).collect();
                self.all.get(&callee)?.native_allocation.as_ref()?.apply(
                    self.c,
                    self.all,
                    callee,
                    &inputs,
                    self.work,
                    self.depth + 1,
                )
            }
            Op::Bits(callee, _, args) => {
                let proof = self.all.get(&callee)?.bit_return.as_ref()?;
                let count = proof.requirement_count();
                if count > 33 {
                    return None;
                }
                self.work.charge(count * 2 + 1)?;
                if !proof.current(self.c, self.all) {
                    return None;
                }
                let native = self.range(self.case.native)?;
                if !native.base {
                    return None;
                }
                self.work.charge(args.len() + proof.nodes.len() + 1)?;
                let mut values = Vec::with_capacity(args.len());
                for &arg in &args {
                    values.push(if arg == self.case.native {
                        None
                    } else {
                        Some(self.number(arg)?)
                    });
                }
                let when = usize::from(*values.get(proof.formal)?.as_ref()? != 0);
                bit_range(
                    &proof.nodes,
                    &args,
                    &values,
                    self.case.native,
                    native,
                    proof.arms[when],
                    self.work,
                )
            }
            _ => None,
        }
    }
}

fn compare(c: &Compiler, ty: TypeId, op: jaic::ir::CmpOp, a: u64, b: u64) -> Option<bool> {
    use jaic::ir::CmpOp;
    let bits = c.ty.types.size_of(ty).checked_mul(8)?;
    if !matches!(bits, 8 | 16 | 32 | 64) {
        return None;
    }
    let mask = if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    let (a, b) = (a & mask, b & mask);
    let signed = |v: u64| ((v << (64 - bits)) as i64) >> (64 - bits);
    Some(match op {
        CmpOp::Eq => a == b,
        CmpOp::Ne => a != b,
        CmpOp::SLt => signed(a) < signed(b),
        CmpOp::SLe => signed(a) <= signed(b),
        CmpOp::SGt => signed(a) > signed(b),
        CmpOp::SGe => signed(a) >= signed(b),
        CmpOp::ULt => a < b,
        CmpOp::ULe => a <= b,
        CmpOp::UGt => a > b,
        CmpOp::UGe => a >= b,
        _ => return None,
    })
}

#[derive(Clone, Copy)]
enum BitValue {
    Number(u64),
    Address(u64),
}

fn bit_range(
    nodes: &[super::super::address_arithmetic::bit_returns::BitNode],
    args: &[usize],
    numbers: &[Option<u64>],
    native: usize,
    range: Range,
    result: usize,
    work: &Work,
) -> Option<Range> {
    use super::super::address_arithmetic::bit_returns::BitNode;
    if jacket_config::env::LIFETIME_STATS.is_set() {
        eprintln!(
            "lifetime-native-bit-range native={} numbers={:?} nodes={:?} original={:?}",
            native, numbers, nodes, range
        );
    }
    let mut period = range.alignment;
    if !period.is_power_of_two() || period > 256 {
        return None;
    }
    // Validate the acyclic graph before the mask prepass indexes its operands.
    work.charge(nodes.len() + 1)?;
    if nodes.is_empty() || nodes.len() > super::MAX_NODES || result >= nodes.len() {
        return None;
    }
    for (id, node) in nodes.iter().enumerate() {
        if let BitNode::Add(a, b) | BitNode::Sub(a, b) | BitNode::And(a, b) = *node
            && (a >= id || b >= id)
        {
            return None;
        }
    }
    // Masks are constants in this first bounded alignment composition.
    for node in nodes {
        if let BitNode::And(a, b) = *node {
            let mask = match (&nodes[a], &nodes[b]) {
                (_, BitNode::Constant(mask)) | (BitNode::Constant(mask), _) => *mask,
                _ => return None,
            };
            let alignment = (!mask).checked_add(1)?;
            if !alignment.is_power_of_two() || alignment > 256 {
                return None;
            }
            period = period.max(alignment);
        }
    }
    let residues = period.checked_div(range.alignment)?.max(1);
    work.charge(
        usize::try_from(residues)
            .ok()?
            .checked_mul(nodes.len() + 1)?,
    )?;
    let mut minimum = u64::MAX;
    let mut maximum = 0;
    let mut alignment = period;
    for residue in (0..period).step_by(usize::try_from(range.alignment).ok()?) {
        let mut values: Vec<BitValue> = Vec::with_capacity(nodes.len());
        for node in nodes {
            let value = match *node {
                BitNode::Pointer(formal) => {
                    if *args.get(formal)? != native {
                        return None;
                    }
                    BitValue::Address(0)
                }
                BitNode::Unsigned(formal) => BitValue::Number(*numbers.get(formal)?.as_ref()?),
                BitNode::Constant(bits) => BitValue::Number(bits),
                BitNode::Add(a, b) => match (*values.get(a)?, *values.get(b)?) {
                    (BitValue::Number(a), BitValue::Number(b)) => {
                        BitValue::Number(a.wrapping_add(b))
                    }
                    (BitValue::Address(at), BitValue::Number(n))
                    | (BitValue::Number(n), BitValue::Address(at)) => {
                        let at = at.checked_add(n)?;
                        if at > range.original {
                            return None;
                        }
                        BitValue::Address(at)
                    }
                    _ => return None,
                },
                BitNode::Sub(a, b) => match (*values.get(a)?, *values.get(b)?) {
                    (BitValue::Number(a), BitValue::Number(b)) => {
                        BitValue::Number(a.wrapping_sub(b))
                    }
                    (BitValue::Address(at), BitValue::Number(n)) => {
                        let at = at.checked_sub(n)?;
                        if at > range.original {
                            return None;
                        }
                        BitValue::Address(at)
                    }
                    _ => return None,
                },
                BitNode::And(a, b) => match (*values.get(a)?, *values.get(b)?) {
                    (BitValue::Number(a), BitValue::Number(b)) => BitValue::Number(a & b),
                    (BitValue::Address(at), BitValue::Number(mask))
                    | (BitValue::Number(mask), BitValue::Address(at)) => {
                        let at = (residue.checked_add(at)? & mask).checked_sub(residue)?;
                        if at > range.original {
                            return None;
                        }
                        BitValue::Address(at)
                    }
                    _ => return None,
                },
            };
            values.push(value);
        }
        let BitValue::Address(offset) = *values.get(result)? else {
            return None;
        };
        minimum = minimum.min(offset);
        maximum = maximum.max(offset);
        let address = residue.checked_add(offset)?;
        if address != 0 {
            alignment = alignment.min(1u64 << address.trailing_zeros());
        }
    }
    Some(Range {
        bytes: range.original.checked_sub(maximum)?,
        alignment,
        original: range.original,
        min: minimum,
        max: maximum,
        base: false,
    })
}

#[cfg(test)]
mod tests {
    use super::Range;

    #[test]
    fn signed_accesses_must_fit_every_returned_offset() {
        let range = Range {
            bytes: 535,
            alignment: 16,
            original: 559,
            min: 16,
            max: 24,
            base: false,
        };
        assert!(range.covers_relative((-16, 528, 8)));
        assert!(!range.covers_relative((-17, 528, 8)));
        assert!(!range.covers_relative((-16, 536, 8)));
        assert!(!range.covers_relative((8, 7, 8)));
        assert!(!range.covers_relative((-16, 528, 32)));
    }
}
