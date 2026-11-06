use super::*;
use jaic::fxhash::HashSet;
use jaic::types::TypeKind;

pub(super) const MAX_ITEMS: usize = 64;
pub(super) const MAX_DEPTH: usize = 16;
pub(super) const MAX_STEPS: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Root {
    Private(Val),
    Receiver { formal: usize, ty: TypeId },
    Context { selection: Site, ty: TypeId },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Address {
    pub root: Root,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Value {
    pub ty: TypeId,
    pub capture: Captured,
    pub address: Option<Address>,
    pub source: ValueSource,
    pub predicate: Option<Predicate>,
    pub parameter_pointer: Option<(usize, i64)>,
    pub unsupported_address: bool,
}

impl Value {
    pub fn unknown(ty: TypeId) -> Self {
        Self {
            ty,
            capture: Captured::Unknown,
            address: None,
            source: ValueSource::Unknown,
            predicate: None,
            parameter_pointer: None,
            unsupported_address: false,
        }
    }

    pub fn captured(ty: TypeId, capture: Captured) -> Self {
        Self {
            capture,
            ..Self::unknown(ty)
        }
    }

    pub fn constant(ty: TypeId, bits: u64) -> Self {
        Self {
            source: if bits == 0 {
                ValueSource::Null
            } else {
                ValueSource::Unknown
            },
            ..Self::captured(
                ty,
                Captured::Constant {
                    ty,
                    bits,
                },
            )
        }
    }

    pub fn guard_value(&self) -> GuardValue {
        if matches!(self.source, ValueSource::Callback { .. }) {
            GuardValue::Source(self.source.clone())
        } else if self.capture != Captured::Unknown {
            GuardValue::Captured(self.capture.clone())
        } else {
            GuardValue::Unknown
        }
    }

    pub fn boolean(&self, c: &Compiler) -> Predicate {
        self.predicate
            .clone()
            .unwrap_or_else(|| Predicate::Compare {
                value: self.guard_value(),
                op: CmpOp::Ne,
                constant: 0,
                ty: ir_type(c, self.ty).unwrap_or(Ty::I8),
            })
    }

    pub fn retyped(&self, c: &Compiler, ty: TypeId) -> Self {
        if self.ty == ty {
            return self.clone();
        }
        let pointers = |t| {
            matches!(
                c.ty.types.kind(c.ty.types.repr(t)),
                TypeKind::Pointer(_) | TypeKind::Proc(_)
            )
        };
        if pointers(self.ty)
            && pointers(ty)
            && c.ty.types.size_of(self.ty) == c.ty.types.size_of(ty)
        {
            let mut value = self.clone();
            value.ty = ty;
            value
        } else if let Captured::Constant {
            bits, ..
        } = self.capture
        {
            let bits = if let Some((width, true)) = c.ty.types.int_info(self.ty) {
                if width > 0 && width <= 64 {
                    ((bits << (64 - width)) as i64 >> (64 - width)) as u64
                } else {
                    return Self::unknown(ty);
                }
            } else {
                bits
            };
            literal(c, ty, bits)
        } else {
            Self::unknown(ty)
        }
    }
}

#[derive(Clone, Default)]
pub(super) struct State {
    pub values: HashMap<Val, Value>,
    pub addresses: HashMap<Val, Address>,
    pub unsupported_addresses: HashSet<Val>,
    pub extents: HashMap<Val, Option<u64>>,
    pub slots: HashMap<(Address, ByteSlot), Value>,
    pub exposed: HashSet<Val>,
    pub current_context: Option<Site>,
    pub current_context_address: Option<Address>,
    pub current_context_unsupported: bool,
    pub current_context_valid: bool,
    pub context_valid_values: HashSet<Val>,
    pub context_addresses: HashMap<Val, Address>,
    pub context_values: HashMap<Val, Site>,
    pub guards: Vec<Guard>,
    pub returns: Vec<ReturnSource>,
    pub return_values: HashMap<usize, Value>,
    pub memory_effects: bool,
    pub uses: Vec<Use>,
    pub stores: Vec<StoreSource>,
    pub visited: HashSet<usize>,
    pub incomplete: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Protocol {
    pub operation_formal: usize,
    pub data_formal: usize,
    pub operation_ty: TypeId,
    pub allocate_value: u64,
    pub resize_value: u64,
    pub free_value: u64,
    pub result_position: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Use {
    pub row: ProviderUse,
    pub operation: Value,
    pub procedure: Value,
    pub data: Value,
    pub protocol: Option<Protocol>,
}

#[derive(Clone, Default)]
pub(super) struct Summary {
    pub exits: Vec<State>,
    pub partial: bool,
    pub initial_context: Option<Site>,
    pub parameter_copies: HashMap<usize, Site>,
    pub observed_uses: Vec<Use>,
    pub observed_stores: Vec<StoreSource>,
    pub pointer_results: Vec<usize>,
}

impl State {
    pub fn value(&self, v: Val) -> Value {
        self.values
            .get(&v)
            .cloned()
            .unwrap_or_else(|| Value::unknown(TypeId::VOID))
    }

    pub fn address(&self, v: Val) -> Option<Address> {
        self.addresses
            .get(&v)
            .cloned()
            .or_else(|| self.values.get(&v).and_then(|v| v.address.clone()))
    }

    pub fn extent(&self, c: &Compiler, root: &Root) -> Option<u64> {
        match root {
            Root::Private(v) => self.extents.get(v).copied().flatten(),
            Root::Receiver {
                ty, ..
            }
            | Root::Context {
                ty, ..
            } => Some(c.ty.types.size_of(*ty)),
        }
    }

    pub fn checked(&self, c: &Compiler, a: &Address, width: u64) -> bool {
        width > 0
            && a.offset
                .checked_add(width)
                .is_some_and(|end| self.extent(c, &a.root).is_some_and(|extent| end <= extent))
    }

    pub fn project(&self, c: &Compiler, a: &Address, offset: i128, width: u64) -> Option<Address> {
        i64::try_from(offset).ok()?;
        let offset = (a.offset as i128)
            .checked_add(offset)
            .and_then(|n| i64::try_from(n).ok())
            .and_then(|n| u64::try_from(n).ok())?;
        let result = Address {
            root: a.root.clone(),
            offset,
        };
        self.checked(c, &result, width).then_some(result)
    }

    pub fn clear(&mut self, a: &Address, width: u64) {
        self.slots.retain(|(b, s), _| {
            a.root != b.root
                || b.offset
                    .checked_add(s.width)
                    .is_some_and(|end| end <= a.offset)
                || a.offset
                    .checked_add(width)
                    .is_some_and(|end| end <= b.offset)
        });
    }

    pub fn read(&mut self, c: &Compiler, a: &Address, slot: ByteSlot, epoch: ReadEpoch) -> Value {
        let Some(address) = self.project(c, a, slot.offset as i128, slot.width) else {
            self.incomplete = true;
            return Value::unknown(slot.ty);
        };
        let key = (
            address.clone(),
            ByteSlot {
                offset: 0,
                ..slot
            },
        );
        if let Some(v) = self.slots.get(&key) {
            return v.retyped(c, slot.ty);
        }
        let projected = ByteSlot {
            offset: address.offset,
            ..slot
        };
        let capture = match &address.root {
            Root::Private(_) => Captured::Unknown,
            Root::Receiver {
                formal,
                ty,
            } => Captured::ReceiverRead {
                formal: *formal,
                receiver_ty: *ty,
                slot: projected,
                epoch,
            },
            Root::Context {
                selection,
                ty,
            } => Captured::ContextRead {
                context_ty: *ty,
                selection: selection.clone(),
                slot: projected,
                epoch,
            },
        };
        let value = Value::captured(slot.ty, capture);
        self.slots.insert(key, value.clone());
        value
    }

    pub fn expose(&mut self, root: Val) {
        let mut work = vec![root];
        while let Some(root) = work.pop() {
            if !self.exposed.insert(root) {
                continue;
            }
            for ((a, _), value) in &self.slots {
                if a.root == Root::Private(root)
                    && let Some(Address {
                        root: Root::Private(next),
                        ..
                    }) = &value.address
                {
                    work.push(*next);
                }
            }
        }
    }

    pub fn clobber(&mut self) {
        self.slots
            .retain(|(a, _), _| matches!(a.root,Root::Private(v) if !self.exposed.contains(&v)));
    }

    pub fn write(&mut self, c: &Compiler, a: &Address, slot: ByteSlot, value: Value) {
        let Some(address) = self.project(c, a, slot.offset as i128, slot.width) else {
            self.slots.clear();
            self.incomplete = true;
            return;
        };
        let external = !matches!(address.root, Root::Private(_));
        let escaping =
            external || matches!(address.root,Root::Private(v) if self.exposed.contains(&v));
        if escaping
            && let Some(Address {
                root: Root::Private(v),
                ..
            }) = &value.address
        {
            self.expose(*v);
        }
        if external {
            // Different borrowed roots can denote overlapping storage. Retain only
            // this exact root's non-overlapping cells and unexposed private bytes.
            self.slots.retain(|(b, _), _| {
                b.root == address.root
                    || matches!(b.root,Root::Private(v) if !self.exposed.contains(&v))
            });
        }
        self.clear(&address, slot.width);
        self.slots.insert(
            (
                address,
                ByteSlot {
                    offset: 0,
                    ..slot
                },
            ),
            value.retyped(c, slot.ty),
        );
    }

    pub fn simplified(&self, p: &Predicate) -> Predicate {
        if let Some(g) = self.guards.iter().find(|g| g.predicate == *p) {
            return Predicate::Compare {
                value: GuardValue::Captured(Captured::Constant {
                    ty: TypeId::BOOL,
                    bits: u64::from(g.truth),
                }),
                op: CmpOp::Ne,
                constant: 0,
                ty: Ty::I8,
            };
        }
        match p {
            Predicate::Not(a) => Predicate::not(&self.simplified(a)),
            Predicate::And(a, b) => {
                let a = self.simplified(a);
                if a.known() == Some(false) {
                    a
                } else if a.known() == Some(true) {
                    self.simplified(b)
                } else {
                    Predicate::and(&a, &self.simplified(b))
                }
            }
            Predicate::Or(a, b) => {
                let a = self.simplified(a);
                if a.known() == Some(true) {
                    a
                } else if a.known() == Some(false) {
                    self.simplified(b)
                } else {
                    Predicate::or(&a, &self.simplified(b))
                }
            }
            _ => p.clone(),
        }
    }

    pub fn bounded(&self) -> bool {
        self.slots.len() <= 256
            && self.values.len() <= MAX_STEPS
            && self.addresses.len() <= MAX_STEPS * 2
            && self.guards.len() <= 32
            && self.uses.len() <= MAX_ITEMS
            && self.stores.len() <= MAX_ITEMS
            && self.returns.len() <= MAX_ITEMS
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
        if depth > MAX_DEPTH || out.len() >= MAX_ITEMS {
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

pub(super) fn literal(c: &Compiler, ty: TypeId, bits: u64) -> Value {
    if matches!(
        c.ty.types.kind(c.ty.types.repr(ty)),
        TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Null
    ) {
        return Value::constant(ty, bits);
    }
    let width =
        c.ty.types
            .size_of(ty)
            .checked_mul(8)
            .filter(|w| [8, 16, 32, 64].contains(w));
    width.map_or_else(
        || Value::unknown(ty),
        |w| Value::constant(ty, bits & (u64::MAX >> (64 - w))),
    )
}

pub(super) fn ir_type(c: &Compiler, ty: TypeId) -> Option<Ty> {
    if matches!(
        c.ty.types.kind(c.ty.types.repr(ty)),
        TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Null
    ) {
        Some(Ty::Ptr)
    } else {
        match c.ty.types.size_of(ty) {
            1 => Some(Ty::I8),
            2 => Some(Ty::I16),
            4 => Some(Ty::I32),
            8 => Some(Ty::I64),
            _ => None,
        }
    }
}

pub(super) fn bits(ty: Ty, op: CmpOp, bits: u64) -> Option<u64> {
    let width = match ty {
        Ty::I8 => 8,
        Ty::I16 => 16,
        Ty::I32 => 32,
        Ty::I64 | Ty::Ptr => 64,
        _ => return None,
    };
    let bits = bits & (u64::MAX >> (64 - width));
    Some(
        if matches!(op, CmpOp::SLt | CmpOp::SLe | CmpOp::SGt | CmpOp::SGe) {
            ((bits << (64 - width)) as i64 >> (64 - width)) as u64
        } else {
            bits
        },
    )
}

fn known(p: &Predicate) -> Option<bool> {
    match p {
        Predicate::Compare {
            value:
                GuardValue::Captured(Captured::Constant {
                    bits: v, ..
                }),
            op,
            constant,
            ty,
        } => {
            let v = bits(*ty, *op, *v)?;
            let k = bits(*ty, *op, *constant)?;
            Some(match op {
                CmpOp::Eq => v == k,
                CmpOp::Ne => v != k,
                CmpOp::SLt => (v as i64) < (k as i64),
                CmpOp::SLe => (v as i64) <= (k as i64),
                CmpOp::SGt => (v as i64) > (k as i64),
                CmpOp::SGe => (v as i64) >= (k as i64),
                CmpOp::ULt => v < k,
                CmpOp::ULe => v <= k,
                CmpOp::UGt => v > k,
                CmpOp::UGe => v >= k,
                _ => return None,
            })
        }
        Predicate::Not(a) => Some(!known(a)?),
        Predicate::And(a, b) => match known(a) {
            Some(false) => Some(false),
            Some(true) => known(b),
            None if known(b) == Some(false) => Some(false),
            _ => None,
        },
        Predicate::Or(a, b) => match known(a) {
            Some(true) => Some(true),
            Some(false) => known(b),
            None if known(b) == Some(true) => Some(true),
            _ => None,
        },
        _ => None,
    }
}

impl Predicate {
    pub fn supported(&self) -> bool {
        match self {
            Self::Unknown => false,
            Self::Compare {
                value: GuardValue::Unknown,
                ..
            } => false,
            Self::Not(a) => a.supported(),
            Self::And(a, b) | Self::Or(a, b) => a.supported() && b.supported(),
            Self::Compare {
                ..
            } => true,
        }
    }

    pub fn known(&self) -> Option<bool> {
        known(self)
    }

    fn shape(&self) -> Option<(usize, usize)> {
        match self {
            Self::Not(a) => parent(a, None),
            Self::And(a, b) | Self::Or(a, b) => parent(a, Some(b)),
            _ => Some((1, 1)),
        }
    }

    pub fn not(a: &Self) -> Self {
        if parent(a, None).is_some() {
            Self::Not(Box::new(a.clone()))
        } else {
            Self::Unknown
        }
    }

    pub fn and(a: &Self, b: &Self) -> Self {
        if parent(a, Some(b)).is_some() {
            Self::And(Box::new(a.clone()), Box::new(b.clone()))
        } else {
            Self::Unknown
        }
    }

    pub fn or(a: &Self, b: &Self) -> Self {
        if parent(a, Some(b)).is_some() {
            Self::Or(Box::new(a.clone()), Box::new(b.clone()))
        } else {
            Self::Unknown
        }
    }
}

fn parent(a: &Predicate, b: Option<&Predicate>) -> Option<(usize, usize)> {
    let (an, ad) = a.shape()?;
    let (bn, bd) = b.map_or(Some((0, 0)), Predicate::shape)?;
    let n = an.checked_add(bn)?.checked_add(1)?;
    let d = ad.max(bd).checked_add(1)?;
    (n <= 128 && d <= MAX_DEPTH).then_some((n, d))
}

pub(super) fn site(procedure: ProcId, block: usize, event: usize) -> Site {
    Site {
        chain: vec![EventSite {
            procedure,
            block,
            event,
        }],
    }
}

pub(super) fn prefix(site: &Site, call: &Site) -> Option<Site> {
    if site.chain.len().checked_add(call.chain.len())? > MAX_DEPTH {
        return None;
    }
    let mut chain = call.chain.clone();
    chain.extend_from_slice(&site.chain);
    Some(Site {
        chain,
    })
}

pub(super) fn compare(c: &Compiler, value: &Value, op: CmpOp, constant: u64, ty: Ty) -> Predicate {
    if let Some(p) = &value.predicate {
        let outcome = |v| {
            known(&Predicate::Compare {
                value: GuardValue::Captured(Captured::Constant {
                    ty: TypeId::BOOL,
                    bits: v,
                }),
                op,
                constant,
                ty,
            })
        };
        return match (outcome(0), outcome(1)) {
            (Some(false), Some(true)) => p.clone(),
            (Some(true), Some(false)) => Predicate::not(p),
            (Some(a), Some(b)) if a == b => Value::constant(TypeId::BOOL, u64::from(a)).boolean(c),
            _ => Predicate::Unknown,
        };
    }
    if value.guard_value() == GuardValue::Unknown {
        Predicate::Unknown
    } else {
        Predicate::Compare {
            value: value.guard_value(),
            op,
            constant,
            ty,
        }
    }
}

pub(super) fn displacement(c: &Compiler, mut value: Value, bytes: i64) -> Value {
    value.source = match value.source {
        ValueSource::Callback {
            use_site,
            result,
            byte_offset,
        } => byte_offset
            .checked_add(bytes)
            .map_or(ValueSource::Unknown, |byte_offset| ValueSource::Callback {
                use_site,
                result,
                byte_offset,
            }),
        ValueSource::Null if bytes == 0 => ValueSource::Null,
        _ => ValueSource::Unknown,
    };
    value.parameter_pointer = value
        .parameter_pointer
        .and_then(|(n, k)| k.checked_add(bytes).map(|k| (n, k)));
    if let Some(a) = value.address.take() {
        value.address = (a.offset as i128)
            .checked_add(bytes as i128)
            .and_then(|n| i64::try_from(n).ok())
            .and_then(|n| u64::try_from(n).ok())
            .map(|offset| Address {
                root: a.root,
                offset,
            });
    }
    if bytes != 0 {
        value.capture = Captured::Unknown;
        value.predicate = None;
    }
    let _ = c;
    value
}

pub(super) fn operation(c: &Compiler, value: &Value, protocol: Option<&Protocol>) -> Operation {
    let Some(p) = protocol else {
        return Operation::Unknown;
    };
    if value.ty != p.operation_ty {
        return Operation::Unknown;
    }
    let Captured::Constant {
        bits, ..
    } = value.capture
    else {
        return Operation::Unknown;
    };
    let width = c.ty.types.size_of(p.operation_ty).saturating_mul(8);
    if ![8, 16, 32, 64].contains(&width) {
        return Operation::Unknown;
    }
    let mask = u64::MAX >> (64 - width);
    let bits = bits & mask;
    if bits == p.allocate_value & mask {
        Operation::Allocate
    } else if bits == p.resize_value & mask {
        Operation::Resize
    } else if bits == p.free_value & mask {
        Operation::Free
    } else {
        Operation::OtherConstant {
            ty: p.operation_ty,
            bits,
        }
    }
}
