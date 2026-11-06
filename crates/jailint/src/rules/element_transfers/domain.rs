use super::*;
use jaic::fxhash::HashSet;
use jaic::ir::{CmpOp, Val};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Number {
    Constant(i128),
    Formal(usize),
    Count(usize, i128),
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Place {
    Local(Val),
    Descriptor(usize, TypeId),
    Count(usize),
    Data(usize, TypeId),
    Buffer(usize, TypeId),
    Element(usize, Number, TypeId),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Value {
    Number(Number),
    Pointer(Place),
    Payload(usize, Number, TypeId),
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

#[derive(Clone, Default)]
pub(super) struct State {
    pub values: HashMap<Val, Value>,
    pub slots: HashMap<Val, Value>,
    pub places: HashMap<(Val, TypeId), Place>,
    pub roots: Option<Rc<StackRoots>>,
    pub addresses: HashMap<Val, Address>,
    pub count_delta: i128,
    pub count_writes: usize,
    pub count_site: Option<Site>,
    pub move_site: Option<Site>,
    pub moved: Option<usize>,
    pub visited: HashSet<usize>,
}

impl State {
    pub fn local(&self, v: Val) -> Val {
        self.roots.as_ref().map_or(v, |roots| roots.canonical(v))
    }

    pub fn reference(&self, v: Val) -> &Value {
        self.values.get(&v).unwrap_or(&Value::Unknown)
    }

    pub fn value(&self, v: Val) -> Value {
        self.values.get(&v).cloned().unwrap_or(Value::Unknown)
    }

    pub fn place(&self, c: &Compiler, v: Val, ty: TypeId) -> Place {
        if let Some(place) = self.places.get(&(v, ty)) {
            return *place;
        }
        if let Value::Pointer(place) = self.value(v) {
            return place;
        }
        let v = self.local(v);
        if self.addresses.get(&v).is_some_and(|a| {
            super::super::lifetime_facts::ByteRange {
                offset: a.offset,
                width: c.ty.types.size_of(ty),
            }
            .within(
                self.roots
                    .as_ref()
                    .and_then(|roots| roots.extent(a.root))
                    .flatten(),
            )
        }) {
            Place::Local(v)
        } else {
            Place::Unknown
        }
    }

    pub fn write_local(&mut self, v: Val, value: Value) {
        let root = self.addresses.get(&v).map(|a| a.root);
        self.slots
            .retain(|other, _| self.addresses.get(other).map(|a| a.root) != root);
        self.slots.insert(v, value);
    }

    pub fn project(&mut self, value: Val, source: Val, delta: i128, width: u64) {
        let source_address = self.addresses.get(&self.local(source)).copied();
        if value != source {
            self.addresses.remove(&value);
        }
        if let Some(a) = source_address
            && let Some(projected) = a.project(
                delta,
                width,
                self.roots
                    .as_ref()
                    .and_then(|roots| roots.extent(a.root))
                    .flatten(),
            )
        {
            self.addresses.insert(value, projected);
        }
    }

    pub fn operand(&self, c: &Compiler, v: Val, ty: TypeId) -> Value {
        if let Some(pointee) = c.ty.types.pointee(ty)
            && let Some(place) = self.places.get(&(v, pointee))
        {
            return Value::Pointer(*place);
        }
        self.value(v)
    }
}

pub(super) fn number(value: &Value) -> Number {
    if let Value::Number(n) = value {
        *n
    } else {
        Number::Unknown
    }
}

pub(super) fn truth(value: &Value, receiver: usize, index: usize) -> Option<bool> {
    let Value::Bool(predicate) = value else {
        return if let Value::Number(Number::Constant(n)) = value {
            Some(*n != 0)
        } else {
            None
        };
    };
    match &**predicate {
        Predicate::Not(a) => Some(!truth(a, receiver, index)?),
        Predicate::And(a, b) => match truth(a, receiver, index) {
            Some(false) => Some(false),
            Some(true) => truth(b, receiver, index),
            None => {
                if truth(b, receiver, index) == Some(false) {
                    Some(false)
                } else {
                    None
                }
            }
        },
        Predicate::Or(a, b) => match truth(a, receiver, index) {
            Some(true) => Some(true),
            Some(false) => truth(b, receiver, index),
            None => {
                if truth(b, receiver, index) == Some(true) {
                    Some(true)
                } else {
                    None
                }
            }
        },
        Predicate::Compare(a, op, b) => {
            let scalar = |v: &Value| {
                if let Value::Bool(_) = v {
                    truth(v, receiver, index)
                        .map_or(Number::Unknown, |b| Number::Constant(i128::from(b)))
                } else {
                    number(v)
                }
            };
            let (left, right) = (scalar(a), scalar(b));
            if let (Number::Constant(a), Number::Constant(b)) = (left, right) {
                return compare(a, *op, b);
            }
            // These are explicit query preconditions over well-defined calls, not
            // assumptions that an assertion handler cannot return.
            use Number::{Constant as C, Count as N, Formal as F};
            match (left, *op, right) {
                (F(i), CmpOp::SGe, C(0)) | (C(0), CmpOp::SLe, F(i)) if i == index => Some(true),
                (F(i), CmpOp::SLt, C(0)) | (C(0), CmpOp::SGt, F(i)) if i == index => Some(false),
                (F(i), CmpOp::SLt, N(r, 0)) | (N(r, 0), CmpOp::SGt, F(i))
                    if i == index && r == receiver =>
                {
                    Some(true)
                }
                (F(i), CmpOp::SGe, N(r, 0)) | (N(r, 0), CmpOp::SLe, F(i))
                    if i == index && r == receiver =>
                {
                    Some(false)
                }
                (N(r, 0), CmpOp::SGt, C(0)) | (C(0), CmpOp::SLt, N(r, 0)) if r == receiver => {
                    Some(true)
                }
                _ => None,
            }
        }
    }
}

fn compare(a: i128, op: CmpOp, b: i128) -> Option<bool> {
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

const MAX_BOOL_NODES: usize = 128;
const MAX_BOOL_DEPTH: usize = 16;

impl Value {
    fn shape(&self) -> Option<(usize, usize)> {
        let Value::Bool(predicate) = self else {
            return Some((1, 1));
        };
        match &**predicate {
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
        (nodes <= MAX_BOOL_NODES && depth <= MAX_BOOL_DEPTH).then_some((nodes, depth))
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
}
