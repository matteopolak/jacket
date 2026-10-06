//! Compile-time values.
use crate::ir::Reloc;
use crate::types::{TypeId, Types};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ProcId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct ModuleId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct PolyStructId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct LibraryId(pub u32);
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct CodeId(pub u32);

/// A constant. Scalars are stored canonically: integers sign- or zero-extended
/// according to their type, floats as f64.
#[derive(Clone, Debug)]
pub enum Value {
    Int(i128),
    Float(f64),
    Bool(bool),
    String(Rc<[u8]>),
    Type(TypeId),
    Null,
    Proc(ProcId),
    /// Raw bytes of an aggregate (struct/array/Any...) plus pointer relocations.
    Bytes(Rc<Aggregate>),
    Code(CodeId),
    Void,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Aggregate {
    pub bytes: Vec<u8>,
    pub relocs: Vec<Reloc>,
}

/// The largest compile-time value the compiler builds in memory (a constant, a global's
/// initializer, a default struct image). Types may be far larger (`types::MAX_SIZE`): this
/// bound keeps an initializer of such a type a diagnostic instead of an allocation failure.
pub const MAX_IMAGE: u64 = 1 << 32;

impl Aggregate {
    /// `size` zero bytes with no relocations, or an error at `span` past `MAX_IMAGE`.
    pub fn zeroed(size: u64, span: crate::source::Span) -> super::Result<Aggregate> {
        if size > MAX_IMAGE {
            return super::err(
                span,
                format!(
                    "a compile-time value of {size} bytes is too large (the limit is {MAX_IMAGE} bytes)"
                ),
            );
        }
        Ok(Aggregate {
            bytes: vec![0; size as usize],
            relocs: Vec::new(),
        })
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        use Value::*;
        match (self, other) {
            (Int(a), Int(b)) => a == b,
            (Float(a), Float(b)) => a.to_bits() == b.to_bits(),
            (Bool(a), Bool(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Type(a), Type(b)) => a == b,
            (Null, Null) | (Void, Void) => true,
            (Proc(a), Proc(b)) => a == b,
            (Bytes(a), Bytes(b)) => a.bytes == b.bytes && a.relocs.len() == b.relocs.len(),
            (Code(a), Code(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Value {
}
impl std::hash::Hash for Value {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            Value::Int(v) => v.hash(state),
            Value::Float(v) => v.to_bits().hash(state),
            Value::Bool(v) => v.hash(state),
            Value::String(v) => v.hash(state),
            Value::Type(v) => v.hash(state),
            Value::Proc(v) => v.hash(state),
            Value::Bytes(v) => v.bytes.hash(state),
            Value::Code(v) => v.hash(state),
            Value::Null | Value::Void => {}
        }
    }
}

impl Value {
    pub fn as_int(&self) -> Option<i128> {
        match self {
            Value::Int(v) => Some(*v),
            Value::Bool(b) => Some(*b as i128),
            _ => None,
        }
    }
    pub fn as_type(&self) -> Option<TypeId> {
        match self {
            Value::Type(t) => Some(*t),
            _ => None,
        }
    }
    pub fn render(&self, types: &Types) -> String {
        match self {
            Value::Int(v) => v.to_string(),
            Value::Float(v) => format!("{v}"),
            Value::Bool(v) => v.to_string(),
            Value::String(s) => format!("\"{}\"", String::from_utf8_lossy(s)),
            Value::Type(t) => types.name(*t),
            Value::Null => "null".into(),
            Value::Proc(p) => format!("proc#{}", p.0),
            Value::Bytes(_) => "{...}".into(),
            Value::Code(_) => "#code".into(),
            Value::Void => "void".into(),
        }
    }
}
