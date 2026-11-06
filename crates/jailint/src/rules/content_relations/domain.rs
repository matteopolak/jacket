use super::*;
use jaic::fxhash::HashSet;
use jaic::ir::CmpOp;

pub(super) const MAX_DEPTH: usize = 16;
pub(super) const MAX_ITEMS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Scalar {
    Formal(usize),
    StringCount(usize),
    Constant(i128),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Base {
    InputData(usize),
    Parameter(usize),
    Call(Site, usize),
    Slot(Val),
    Null,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct Pointer {
    pub base: Base,
    pub offset: i128,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Value {
    Scalar(Scalar),
    Pointer(Pointer),
    Bool(Box<Predicate>),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Predicate {
    Compare(Value, CmpOp, Value),
    Not(Value),
    And(Value, Value),
    Or(Value, Value),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Descriptor {
    pub count: Scalar,
    pub data: Option<Pointer>,
}

impl Default for Descriptor {
    fn default() -> Self {
        Self {
            count: Scalar::Unknown,
            data: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Address {
    pub root: Val,
    pub offset: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Fact {
    pub destination: Pointer,
    pub source: Pointer,
    pub length: Scalar,
    pub site: Site,
    pub requires_disjoint: bool,
    pub separations: Vec<Separation>,
}

#[derive(Clone, Default)]
pub(super) struct State {
    pub values: HashMap<Val, Value>,
    pub slots: HashMap<(Address, ByteSlot), Value>,
    pub addresses: HashMap<Val, Address>,
    pub snapshots: HashMap<Val, Descriptor>,
    pub invalid_snapshots: HashSet<Val>,
    pub roots: HashMap<Val, u64>,
    pub allocation_extents: HashMap<Val, Option<u64>>,
    pub owned: HashSet<Val>,
    pub exposed: HashSet<Val>,
    pub headers: HashSet<Address>,
    pub copies: Vec<Fact>,
    pub returns: HashMap<usize, Descriptor>,
    pub visited: HashSet<usize>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct Exit {
    pub returns: HashMap<usize, Descriptor>,
    pub copies: Vec<Fact>,
}

#[derive(Clone, Default)]
pub(super) struct Summary {
    pub exits: Vec<Exit>,
}

#[derive(Clone, Copy)]
pub(super) struct Layout {
    pub data: ByteSlot,
    pub count: ByteSlot,
}

impl Layout {
    pub fn of(c: &Compiler) -> Option<Self> {
        let data_ty = (0..c.ty.types.type_count())
            .map(|i| TypeId(i as u32))
            .find(|t| c.ty.types.pointee(*t) == Some(TypeId::U8))?;
        Some(Self {
            count: ByteSlot {
                offset: jaic::layout::view::COUNT,
                width: 8,
                ty: TypeId::S64,
            },
            data: ByteSlot {
                offset: jaic::layout::view::DATA,
                width: 8,
                ty: data_ty,
            },
        })
    }
}

impl State {
    pub fn value(&self, v: Val) -> Value {
        self.reference(v).clone()
    }

    pub fn reference(&self, v: Val) -> &Value {
        self.values.get(&v).unwrap_or(&Value::Unknown)
    }

    pub fn scalar(&self, v: Val) -> Scalar {
        if let Value::Scalar(s) = self.value(v) {
            s
        } else {
            Scalar::Unknown
        }
    }

    pub fn pointer(&self, v: Val) -> Option<Pointer> {
        if let Some(a) = self.addresses.get(&v) {
            return Some(Pointer {
                base: Base::Slot(a.root),
                offset: a.offset as i128,
            });
        }
        if let Value::Pointer(p) = self.value(v) {
            Some(p)
        } else {
            None
        }
    }

    pub fn address(&self, v: Val) -> Option<Address> {
        if let Some(a) = self.addresses.get(&v) {
            return Some(*a);
        }
        let Pointer {
            base: Base::Slot(root),
            offset,
        } = self.pointer(v)?
        else {
            return None;
        };
        Some(Address {
            root,
            offset: u64::try_from(offset).ok()?,
        })
    }

    pub fn root(&mut self, v: Val, width: u64, owned: bool) {
        let root = self.addresses.get(&v).map_or(v, |a| a.root);
        if let Some(extent) = self.allocation_extents.get(&root) {
            if let Some(bytes) = extent {
                self.roots.insert(root, *bytes);
            }
        } else {
            self.roots.entry(root).or_insert(width);
        }
        self.addresses.entry(v).or_insert(Address {
            root: v,
            offset: 0,
        });
        if owned
            && self
                .allocation_extents
                .get(&root)
                .is_some_and(std::option::Option::is_some)
        {
            self.owned.insert(root);
        }
    }

    pub fn checked(&self, a: Address, width: u64) -> bool {
        width > 0
            && a.offset
                .checked_add(width)
                .is_some_and(|end| self.roots.get(&a.root).is_some_and(|extent| end <= *extent))
    }

    pub fn clear(&mut self, a: Address, width: Option<u64>) {
        let invalid: Vec<_> = self
            .snapshots
            .keys()
            .copied()
            .filter(|v| self.addresses.get(v).is_some_and(|b| b.root == a.root))
            .collect();
        for v in invalid {
            self.snapshots.remove(&v);
            self.invalid_snapshots.insert(v);
        }
        self.slots.retain(|(b, s), _| {
            b.root != a.root
                || width.is_some_and(|width| {
                    b.offset
                        .checked_add(s.width)
                        .is_some_and(|end| end <= a.offset)
                        || a.offset
                            .checked_add(width)
                            .is_some_and(|end| end <= b.offset)
                })
        });
    }

    pub fn put(&mut self, a: Address, slot: ByteSlot, value: Value) {
        let Some(offset) = a.offset.checked_add(slot.offset) else {
            return;
        };
        let a = Address {
            offset,
            ..a
        };
        if !self.checked(a, slot.width) {
            return;
        }
        self.clear(a, Some(slot.width));
        self.slots.insert(
            (
                a,
                ByteSlot {
                    offset: 0,
                    ..slot
                },
            ),
            value,
        );
    }

    pub fn get(&self, a: Address, slot: ByteSlot) -> Value {
        let Some(offset) = a.offset.checked_add(slot.offset) else {
            return Value::Unknown;
        };
        self.slots
            .get(&(
                Address {
                    offset,
                    ..a
                },
                ByteSlot {
                    offset: 0,
                    ..slot
                },
            ))
            .cloned()
            .unwrap_or(Value::Unknown)
    }

    pub fn descriptor(&self, v: Val, layout: Layout) -> Descriptor {
        if self.invalid_snapshots.contains(&v) {
            return Descriptor::default();
        }
        if let Some(s) = self.snapshots.get(&v) {
            return s.clone();
        }
        self.current_descriptor(v, layout)
    }

    pub fn current_descriptor(&self, v: Val, layout: Layout) -> Descriptor {
        let Some(a) = self.address(v) else {
            return self.snapshots.get(&v).cloned().unwrap_or_default();
        };
        Descriptor {
            count: if let Value::Scalar(n) = self.get(a, layout.count) {
                n
            } else {
                Scalar::Unknown
            },
            data: if let Value::Pointer(p) = self.get(a, layout.data) {
                Some(p)
            } else {
                None
            },
        }
    }

    pub fn descriptor_at(&mut self, v: Val, descriptor: &Descriptor, layout: Layout) {
        self.root(v, jaic::layout::view::SIZE, false);
        let a = self.address(v).unwrap();
        self.put(a, layout.count, Value::Scalar(descriptor.count));
        self.put(
            a,
            layout.data,
            descriptor
                .data
                .clone()
                .map_or(Value::Unknown, Value::Pointer),
        );
        self.headers.insert(a);
        self.snapshots.insert(v, descriptor.clone());
        self.invalid_snapshots.remove(&v);
    }

    pub fn opaque(&mut self, args: &[jaic::ide::lifetimes::LifetimeArgument]) {
        self.copies.clear();
        let mut inspect = Vec::new();
        for arg in args {
            let (jaic::ide::lifetimes::LifetimeOperand::Value(v)
            | jaic::ide::lifetimes::LifetimeOperand::Place(v)) = arg.operand
            else {
                continue;
            };
            if let Some(Pointer {
                base: Base::Slot(root),
                ..
            }) = self.pointer(v)
            {
                // A string header is passed by value, but pointers stored inside it
                // can still expose other private objects to the callee.
                if arg.ty != TypeId::STRING {
                    self.exposed.insert(root);
                }
                inspect.push(root);
            }
        }
        let mut seen = HashSet::default();
        while let Some(root) = inspect.pop() {
            if !seen.insert(root) {
                continue;
            }
            for ((address, _), value) in &self.slots {
                if address.root == root
                    && let Value::Pointer(Pointer {
                        base: Base::Slot(target),
                        ..
                    }) = value
                {
                    self.exposed.insert(*target);
                    inspect.push(*target);
                }
            }
        }
        self.invalidate_ambiguous();
    }

    pub fn invalidate_ambiguous(&mut self) {
        self.slots
            .retain(|(a, _), _| self.owned.contains(&a.root) && !self.exposed.contains(&a.root));
        let invalid: Vec<_> = self
            .snapshots
            .keys()
            .copied()
            .filter(|v| {
                self.addresses.get(v).is_some_and(|a| {
                    !self.owned.contains(&a.root) || self.exposed.contains(&a.root)
                })
            })
            .collect();
        for v in invalid {
            self.snapshots.remove(&v);
            self.invalid_snapshots.insert(v);
        }
    }

    pub fn bounded(&self) -> bool {
        self.headers.len() <= MAX_ITEMS
            && self.copies.len() <= MAX_ITEMS
            && self.slots.len() <= MAX_ITEMS * 4
            && self.copies.iter().all(|f| f.separations.len() <= MAX_ITEMS)
    }
}

impl Value {
    fn shape(&self) -> Option<(usize, usize)> {
        let Value::Bool(p) = self else {
            return Some((1, 1));
        };
        match &**p {
            Predicate::Not(a) => Self::parent(a, None),
            Predicate::And(a, b) | Predicate::Or(a, b) | Predicate::Compare(a, _, b) => {
                Self::parent(a, Some(b))
            }
        }
    }

    fn parent(a: &Value, b: Option<&Value>) -> Option<(usize, usize)> {
        let (an, ad) = a.shape()?;
        let (bn, bd) = if let Some(b) = b {
            b.shape()?
        } else {
            (0, 0)
        };
        let nodes = an.checked_add(bn)?.checked_add(1)?;
        let depth = ad.max(bd).checked_add(1)?;
        (nodes <= 128 && depth <= MAX_DEPTH).then_some((nodes, depth))
    }

    pub fn not(a: &Value) -> Value {
        if Self::parent(a, None).is_none() {
            Value::Unknown
        } else {
            Value::Bool(Box::new(Predicate::Not(a.clone())))
        }
    }

    pub fn and(a: &Value, b: &Value) -> Value {
        if Self::parent(a, Some(b)).is_none() {
            Value::Unknown
        } else {
            Value::Bool(Box::new(Predicate::And(a.clone(), b.clone())))
        }
    }

    pub fn or(a: &Value, b: &Value) -> Value {
        if Self::parent(a, Some(b)).is_none() {
            Value::Unknown
        } else {
            Value::Bool(Box::new(Predicate::Or(a.clone(), b.clone())))
        }
    }

    pub fn compare(a: &Value, op: CmpOp, b: &Value) -> Value {
        if Self::parent(a, Some(b)).is_none() {
            Value::Unknown
        } else {
            Value::Bool(Box::new(Predicate::Compare(a.clone(), op, b.clone())))
        }
    }

    pub fn truth(&self) -> Option<bool> {
        match self {
            Value::Scalar(Scalar::Constant(k)) => Some(*k != 0),
            Value::Bool(p) => match &**p {
                Predicate::Not(a) => Some(!a.truth()?),
                Predicate::And(a, b) => match a.truth() {
                    Some(false) => Some(false),
                    Some(true) => b.truth(),
                    None => {
                        if b.truth() == Some(false) {
                            Some(false)
                        } else {
                            None
                        }
                    }
                },
                Predicate::Or(a, b) => match a.truth() {
                    Some(true) => Some(true),
                    Some(false) => b.truth(),
                    None => {
                        if b.truth() == Some(true) {
                            Some(true)
                        } else {
                            None
                        }
                    }
                },
                Predicate::Compare(a, op, b) => {
                    let scalar = |v: &Value| match v {
                        Value::Scalar(Scalar::Constant(k)) => Some(*k),
                        Value::Bool(_) => v.truth().map(i128::from),
                        _ => None,
                    };
                    let (a, b) = (scalar(a)?, scalar(b)?);
                    Some(match op {
                        CmpOp::Eq => a == b,
                        CmpOp::Ne => a != b,
                        CmpOp::SLt => a < b,
                        CmpOp::SLe => a <= b,
                        CmpOp::SGt => a > b,
                        CmpOp::SGe => a >= b,
                        _ => return None,
                    })
                }
            },
            _ => None,
        }
    }
}
