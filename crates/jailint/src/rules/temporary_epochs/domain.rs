use super::*;
use jaic::fxhash::{HashMap, HashSet};
use jaic::ir::Val;
use jaic::types::TypeKind;

pub(super) const STEPS: usize = 8192;
pub(super) const ITEMS: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Root {
    Local(Val),
    Object(InstanceKey),
    Output {
        value: Val,
        ty: TypeId,
        result: usize,
    },
    Context {
        site: Site,
        ty: TypeId,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct InstanceKey(pub Instance);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Address {
    pub root: Root,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Expr {
    Unknown,
    Address(Address),
    Scalar(Operand),
    Read {
        instance: Instance,
        slot: ByteSlot,
        site: Site,
        epoch: u64,
    },
    Instance(Instance),
    Offset {
        base: Box<Expr>,
        index: Box<Expr>,
        stride: u64,
    },
    Add {
        left: Box<Expr>,
        right: Box<Expr>,
        ty: TypeId,
    },
    Sub {
        left: Box<Expr>,
        right: Box<Expr>,
        ty: TypeId,
    },
    Predicate(Predicate),
}

impl Expr {
    /// Check support without allocating a second expression tree.
    pub fn operand_supported(&self) -> bool {
        match self {
            Self::Scalar(value) => operand_supported(value),
            Self::Instance(instance) => instance.pointer != Captured::Unknown,
            Self::Read {
                instance, ..
            } => matches!(instance.pointer, Captured::FormalValue { .. }),
            Self::Add {
                left,
                right,
                ..
            }
            | Self::Sub {
                left,
                right,
                ..
            } => left.operand_supported() && right.operand_supported(),
            _ => false,
        }
    }

    pub fn operand(&self) -> Operand {
        match self {
            Self::Scalar(v) => v.clone(),
            Self::Instance(i) => Operand::Captured(i.pointer.clone()),
            Self::Read {
                instance,
                slot,
                site,
                ..
            } => Operand::Captured(capture(instance, *slot, site.clone())),
            Self::Add {
                left,
                right,
                ty,
            } => Operand::Add {
                left: Box::new(left.operand()),
                right: Box::new(right.operand()),
                ty: *ty,
            },
            Self::Sub {
                left,
                right,
                ty,
            } => Operand::Sub {
                left: Box::new(left.operand()),
                right: Box::new(right.operand()),
                ty: *ty,
            },
            _ => Operand::Unknown,
        }
    }

    pub fn size(&self) -> Option<(usize, usize)> {
        let (n, d) = match self {
            Self::Offset {
                base: left,
                index: right,
                ..
            }
            | Self::Add {
                left,
                right,
                ..
            }
            | Self::Sub {
                left,
                right,
                ..
            } => {
                let (a, b) = left.size()?;
                let (c, d) = right.size()?;
                (a.checked_add(c)?.checked_add(1)?, b.max(d).checked_add(1)?)
            }
            Self::Predicate(p) => return predicate_size(p),
            _ => (1, 1),
        };
        (n <= 128 && d <= 16).then_some((n, d))
    }

    pub fn integer(&self) -> Option<u64> {
        if let Self::Scalar(Operand::Constant {
            bits, ..
        }) = self
        {
            Some(*bits)
        } else {
            None
        }
    }

    pub fn captured(&self) -> Captured {
        match self.operand() {
            Operand::Captured(c) => c,
            Operand::Constant {
                ty,
                bits,
            } => Captured::Constant {
                ty,
                bits,
            },
            _ => Captured::Unknown,
        }
    }
}

pub(super) fn capture(instance: &Instance, slot: ByteSlot, site: Site) -> Captured {
    match &instance.pointer {
        Captured::FormalValue {
            formal, ..
        } => Captured::ReceiverRead {
            formal: *formal,
            receiver_ty: instance.view_ty,
            slot,
            epoch: super::super::allocator_providers::ReadEpoch::Load(site),
        },
        _ => Captured::Unknown,
    }
}

pub(super) fn operand_supported(v: &Operand) -> bool {
    match v {
        Operand::Unknown => false,
        Operand::Add {
            left,
            right,
            ..
        }
        | Operand::Sub {
            left,
            right,
            ..
        } => operand_supported(left) && operand_supported(right),
        _ => true,
    }
}

pub(super) fn predicate_supported(p: &Predicate) -> bool {
    match p {
        Predicate::Unknown => false,
        Predicate::Compare {
            left,
            right,
            ..
        } => operand_supported(left) && operand_supported(right),
        Predicate::Not(a) => predicate_supported(a),
        Predicate::And(a, b) | Predicate::Or(a, b) => {
            predicate_supported(a) && predicate_supported(b)
        }
        Predicate::Constant(_) => true,
    }
}

pub(super) fn predicate_size(p: &Predicate) -> Option<(usize, usize)> {
    let (n, d) = match p {
        Predicate::Not(a) => {
            let (n, d) = predicate_size(a)?;
            (n.checked_add(1)?, d.checked_add(1)?)
        }
        Predicate::And(a, b) | Predicate::Or(a, b) => {
            let (n, d) = predicate_size(a)?;
            let (m, e) = predicate_size(b)?;
            (n.checked_add(m)?.checked_add(1)?, d.max(e).checked_add(1)?)
        }
        Predicate::Compare {
            left,
            right,
            ..
        } => {
            let (n, d) = operand_size(left)?;
            let (m, e) = operand_size(right)?;
            (n.checked_add(m)?.checked_add(1)?, d.max(e).checked_add(1)?)
        }
        _ => (1, 1),
    };
    (n <= 128 && d <= 16).then_some((n, d))
}

pub(super) fn operand_size(p: &Operand) -> Option<(usize, usize)> {
    let (n, d) = match p {
        Operand::Add {
            left,
            right,
            ..
        }
        | Operand::Sub {
            left,
            right,
            ..
        } => {
            let (n, d) = operand_size(left)?;
            let (m, e) = operand_size(right)?;
            (n.checked_add(m)?.checked_add(1)?, d.max(e).checked_add(1)?)
        }
        _ => (1, 1),
    };
    (n <= 128 && d <= 16).then_some((n, d))
}

#[derive(Clone, Debug)]
pub(super) struct Write {
    pub instance: Instance,
    pub slot: ByteSlot,
    pub value: Expr,
    pub site: Site,
}

#[derive(Clone, Debug)]
pub(super) struct Projection {
    pub base: Address,
    pub result: Address,
    pub base_ty: TypeId,
    pub result_ty: TypeId,
    pub offset: u64,
}

#[derive(Clone, Debug)]
pub(super) struct WholeOrigin {
    pub formal: usize,
    pub ty: TypeId,
    pub entry_copy: Site,
    pub last_copy: Site,
    pub fields: std::sync::Arc<[ByteSlot]>,
}

#[derive(Clone, Default)]
pub(super) struct State {
    pub bypass: Option<super::paged::Inputs>,
    pub bypass_failed: bool,
    pub wholes: HashMap<(Address, TypeId, u64), WholeOrigin>,
    pub output_writes: HashSet<usize>,
    pub values: HashMap<Val, Expr>,
    pub projections: HashMap<(usize, usize), Projection>,
    pub addresses: HashMap<Val, Address>,
    pub extents: HashMap<Val, Option<u64>>,
    pub exposed: HashSet<Val>,
    pub slots: HashMap<(Address, ByteSlot), Expr>,
    pub writes: Vec<Write>,
    pub returns: Vec<(usize, TypeId, Expr, Option<Address>, Site)>,
    pub conditions: Vec<Condition>,
    pub visited: HashSet<usize>,
    pub incomplete: bool,
    pub interfered: bool,
    pub epoch: u64,
    pub current_context_address: Option<Address>,
    pub context: Option<(Site, TypeId)>,
    pub context_values: HashMap<Val, (Site, TypeId)>,
}

impl State {
    pub fn value(&self, v: Val) -> Expr {
        self.values.get(&v).cloned().unwrap_or(Expr::Unknown)
    }

    pub fn address(&self, v: Val) -> Option<Address> {
        self.addresses
            .get(&v)
            .cloned()
            .or_else(|| match self.values.get(&v)? {
                Expr::Address(a) => Some(a.clone()),
                Expr::Instance(i) => Some(Address {
                    root: Root::Object(InstanceKey(i.clone())),
                    offset: 0,
                }),
                _ => None,
            })
    }

    pub fn checked(&self, c: &Compiler, a: &Address, width: u64) -> bool {
        let extent = match &a.root {
            Root::Local(v) => self.extents.get(v).copied().flatten(),
            Root::Object(i) => Some(c.ty.types.size_of(i.0.view_ty)),
            Root::Context {
                ty, ..
            }
            | Root::Output {
                ty, ..
            } => Some(c.ty.types.size_of(*ty)),
        };
        a.offset
            .checked_add(width)
            .zip(extent)
            .is_some_and(|(end, extent)| end <= extent)
    }

    /// Keep the original extent, including through a larger pointer reinterpretation.
    pub fn project(&self, c: &Compiler, a: &Address, delta: i128, width: u64) -> Option<Address> {
        let a = Address {
            root: a.root.clone(),
            offset: u64::try_from((a.offset as i128).checked_add(delta)?).ok()?,
        };
        self.checked(c, &a, width).then_some(a)
    }

    /// Return only already captured bytes; an unsupported array copy stays unknown.
    pub fn existing(&self, c: &Compiler, a: &Address, slot: ByteSlot) -> Option<&Expr> {
        let at = self.project(c, a, slot.offset as i128, slot.width)?;
        self.slots.get(&(
            at,
            ByteSlot {
                offset: 0,
                ..slot
            },
        ))
    }

    pub fn read(&mut self, c: &Compiler, a: &Address, slot: ByteSlot, site: &Site) -> Expr {
        self.bypass_access(c, a, slot.offset, slot.width);
        let Some(at) = self.project(c, a, slot.offset as i128, slot.width) else {
            return Expr::Unknown;
        };
        let key = (
            at.clone(),
            ByteSlot {
                offset: 0,
                ..slot
            },
        );
        if let Some(v) = self.slots.get(&key) {
            return v.clone();
        }
        let result = match &at.root {
            Root::Object(i) => Expr::Read {
                instance: i.0.clone(),
                slot: ByteSlot {
                    offset: at.offset,
                    ..slot
                },
                site: site.clone(),
                epoch: self.epoch,
            },
            Root::Context {
                site: selection,
                ty,
            } => Expr::Scalar(Operand::Captured(Captured::ContextRead {
                context_ty: *ty,
                selection: selection.clone(),
                slot: ByteSlot {
                    offset: at.offset,
                    ..slot
                },
                epoch: super::super::allocator_providers::ReadEpoch::Load(site.clone()),
            })),
            _ => Expr::Unknown,
        };
        self.slots.insert(key, result.clone());
        result
    }

    /// Whole typed copies carry an opaque argument origin, not a reconstructed tuple.
    pub fn whole(&self, c: &Compiler, a: &Address, ty: TypeId) -> Option<WholeOrigin> {
        let width = c.ty.types.size_of(ty);
        if !self.checked(c, a, width) {
            return None;
        }
        self.wholes.get(&(a.clone(), ty, width)).cloned()
    }

    pub fn install_whole(&mut self, c: &Compiler, a: &Address, ty: TypeId, origin: WholeOrigin) {
        let width = c.ty.types.size_of(ty);
        if origin.ty == ty && self.checked(c, a, width) {
            self.wholes.insert((a.clone(), ty, width), origin);
        }
    }

    pub fn clear(&mut self, a: &Address, width: u64) {
        self.wholes.retain(|(b, _, bytes), _| {
            a.root != b.root
                || a.offset
                    .checked_add(width)
                    .zip(b.offset.checked_add(*bytes))
                    .is_some_and(|(end, other)| end <= b.offset || other <= a.offset)
        });
        self.slots.retain(|(b, s), _| {
            a.root != b.root
                || a.offset
                    .checked_add(width)
                    .zip(b.offset.checked_add(s.width))
                    .is_some_and(|(end, other)| end <= b.offset || other <= a.offset)
        });
    }

    /// A shared write can alias other borrowed views; only private unexposed roots survive.
    pub fn invalidate_aliases(&mut self, a: &Address) {
        if !matches!(a.root, Root::Local(_))
            || matches!(a.root,Root::Local(v) if self.exposed.contains(&v))
        {
            self.wholes.retain(|(b, _, _), _| {
                b.root == a.root || matches!(b.root,Root::Local(v) if !self.exposed.contains(&v))
            });
            self.slots.retain(|(b, _), _| {
                b.root == a.root || matches!(b.root,Root::Local(v) if !self.exposed.contains(&v))
            });
        }
    }

    pub fn note_write(&mut self, a: &Address) {
        if let Root::Output {
            result, ..
        } = a.root
        {
            self.output_writes.insert(result);
        }
    }

    pub fn write(&mut self, c: &Compiler, a: &Address, slot: ByteSlot, value: Expr, site: &Site) {
        self.bypass_access(c, a, slot.offset, slot.width);
        if self.bypass.is_some() && !value.operand_supported() && !matches!(a.root, Root::Local(_))
        {
            self.bypass_failed = true;
        }
        self.note_write(a);
        let Some(at) = self.project(c, a, slot.offset as i128, slot.width) else {
            self.opaque();
            return;
        };
        if let Root::Object(i) = &at.root {
            self.epoch = self.epoch.saturating_add(1);
            self.writes.push(Write {
                instance: i.0.clone(),
                slot: ByteSlot {
                    offset: at.offset,
                    ..slot
                },
                value: value.clone(),
                site: site.clone(),
            });
            self.slots
                .retain(|(b, _), _| matches!(b.root, Root::Local(_)) || b.root == at.root);
        }
        if (!matches!(at.root, Root::Local(_))
            || matches!(at.root,Root::Local(v) if self.exposed.contains(&v)))
            && let Expr::Address(Address {
                root: Root::Local(v),
                ..
            }) = &value
        {
            self.expose(*v);
        }
        self.invalidate_aliases(&at);
        self.clear(&at, slot.width);
        self.slots.insert(
            (
                at,
                ByteSlot {
                    offset: 0,
                    ..slot
                },
            ),
            value,
        );
    }

    pub fn expose(&mut self, root: Val) {
        let mut pending = vec![root];
        while let Some(v) = pending.pop() {
            if !self.exposed.insert(v) {
                continue;
            }
            for ((a, _), value) in &self.slots {
                if a.root == Root::Local(v)
                    && let Expr::Address(Address {
                        root: Root::Local(other),
                        ..
                    }) = value
                {
                    pending.push(*other);
                }
            }
        }
    }

    /// Preserve private captured bits, but invalidate public or escaped storage and protocol validity.
    pub fn callback(&mut self) {
        // This slice exports no callback transport; keep no stored ghost proof after one.
        self.wholes.clear();
        self.slots
            .retain(|(a, _), _| matches!(a.root,Root::Local(v) if !self.exposed.contains(&v)));
        self.incomplete = true;
        self.interfered = true;
    }

    pub fn opaque(&mut self) {
        self.wholes.clear();
        self.exposed.extend(self.extents.keys().copied());
        self.slots.clear();
        self.incomplete = true;
        self.interfered = true;
    }

    pub fn bypass_access(&mut self, c: &Compiler, a: &Address, offset: u64, width: u64) {
        let Some(input) = &self.bypass else {
            return;
        };
        let allowed = match &a.root {
            Root::Local(root) => !self.exposed.contains(root),
            Root::Object(i) => {
                matches!(&i.0.pointer,Captured::FormalValue{formal,..} if *formal==input.receiver_formal)
                    && i.0.view_ty == input.receiver_ty
            }
            _ => false,
        };
        if !allowed || self.project(c, a, offset as i128, width).is_none() {
            self.bypass_failed = true;
        }
    }

    pub fn simplified(&self, p: &Predicate) -> Predicate {
        if self.bypass.is_some()
            && let Some(value) = super::paged::constant_compare(p)
        {
            return Predicate::Constant(value);
        }
        for condition in &self.conditions {
            if condition.predicate == *p {
                return Predicate::Constant(condition.truth);
            }
            if let Predicate::Not(inner) = &condition.predicate
                && **inner == *p
            {
                return Predicate::Constant(!condition.truth);
            }
        }
        match p {
            Predicate::Not(a) => match self.simplified(a) {
                Predicate::Constant(v) => Predicate::Constant(!v),
                v => Predicate::Not(Box::new(v)),
            },
            Predicate::And(a, b) => match self.simplified(a) {
                Predicate::Constant(false) => Predicate::Constant(false),
                Predicate::Constant(true) => self.simplified(b),
                a => match self.simplified(b) {
                    Predicate::Constant(false) => Predicate::Constant(false),
                    Predicate::Constant(true) => a,
                    b => Predicate::And(Box::new(a), Box::new(b)),
                },
            },
            Predicate::Or(a, b) => match self.simplified(a) {
                Predicate::Constant(true) => Predicate::Constant(true),
                Predicate::Constant(false) => self.simplified(b),
                a => match self.simplified(b) {
                    Predicate::Constant(true) => Predicate::Constant(true),
                    Predicate::Constant(false) => a,
                    b => Predicate::Or(Box::new(a), Box::new(b)),
                },
            },
            _ => p.clone(),
        }
    }

    pub fn bounded(&self) -> bool {
        self.wholes.len() <= 256
            && self.output_writes.len() <= ITEMS
            && self.values.len() <= STEPS
            && self.addresses.len() <= STEPS * 2
            && self.projections.len() <= STEPS
            && self.slots.len() <= 256
            && self.writes.len() <= ITEMS
            && self.conditions.len() <= 32
            && self.returns.len() <= ITEMS
    }
}

pub(super) fn scalar(c: &Compiler, ty: TypeId) -> bool {
    matches!(
        c.ty.types.kind(c.ty.types.repr(ty)),
        TypeKind::Int { .. }
            | TypeKind::Enum(_)
            | TypeKind::Bool
            | TypeKind::Pointer(_)
            | TypeKind::Proc(_)
            | TypeKind::Null
    )
}

pub(super) fn leaves(c: &Compiler, ty: TypeId) -> Option<Vec<ByteSlot>> {
    fn add(
        c: &Compiler,
        ty: TypeId,
        offset: u64,
        depth: usize,
        out: &mut Vec<ByteSlot>,
    ) -> Option<()> {
        if depth > 16 || out.len() >= ITEMS {
            return None;
        }
        if scalar(c, ty) {
            out.push(ByteSlot {
                offset,
                width: c.ty.types.size_of(ty),
                ty,
            });
        } else {
            let st = c.ty.types.as_struct(ty)?;
            let info = c.ty.types.struct_info(st);
            if info.is_union || info.fields.iter().any(|f| f.overlay) {
                return None;
            }
            for f in info.fields.iter() {
                add(c, f.ty, offset.checked_add(f.offset)?, depth + 1, out)?;
            }
        }
        Some(())
    }
    let mut out = Vec::new();
    add(c, ty, 0, 0, &mut out)?;
    Some(out)
}

pub(super) fn site(p: ProcId, b: usize, e: usize) -> Site {
    Site {
        chain: vec![super::super::allocator_providers::EventSite {
            procedure: p,
            block: b,
            event: e,
        }],
    }
}

pub(super) fn literal(c: &Compiler, ty: TypeId, bits: u64) -> Expr {
    let width = c.ty.types.size_of(ty).checked_mul(8);
    match width {
        Some(w) if [8, 16, 32, 64].contains(&w) => Expr::Scalar(Operand::Constant {
            ty,
            bits: bits & (u64::MAX >> (64 - w)),
        }),
        _ => Expr::Unknown,
    }
}
