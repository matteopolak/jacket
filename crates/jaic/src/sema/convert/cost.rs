//! The cost of an implicit conversion and the offset of a subtype.
use super::{EXACT, INT_TO_FLOAT, LITERAL, POINTER, SUBTYPE, TO_ANY, WIDEN};
use crate::sema::{Compiler, Span, TypeId};
use crate::types::{ArrayKind, TypeKind};

impl Compiler {
    /// Whether an argument known only by its type fact may convert to `to`, for tools that
    /// resolve a call without the operand itself (`literal`: an untyped literal; `constant`: a
    /// compile-time value). Only a proven mismatch answers `false`: the conversions that depend
    /// on the operand (code quotation, procedure values, auto-dereference, contextual aggregates,
    /// constant arithmetic and one-byte strings) answer `true`.
    pub fn may_convert_argument(
        &mut self,
        from: TypeId,
        literal: bool,
        constant: bool,
        to: TypeId,
    ) -> bool {
        let scalar = |kind: &TypeKind| {
            matches!(
                kind,
                TypeKind::Bool | TypeKind::Int { .. } | TypeKind::Float { .. } | TypeKind::String
            )
        };
        if !scalar(self.ty.types.kind(from)) || !scalar(self.ty.types.kind(to)) {
            return true;
        }
        let number =
            |kind: &TypeKind| matches!(kind, TypeKind::Int { .. } | TypeKind::Float { .. });
        if constant {
            let numeric = number(self.ty.types.kind(from)) && number(self.ty.types.kind(to));
            if (numeric && !literal) || (from == TypeId::STRING && to == TypeId::U8) {
                return true;
            }
        }
        self.implicit_cost(from, literal, to).is_some()
    }

    /// Cost of implicitly converting a value of `from` to `to`, if allowed.
    /// `untyped` marks literals, which convert between numeric types freely.
    pub fn implicit_cost(&mut self, from: TypeId, untyped: bool, to: TypeId) -> Option<u32> {
        if from == to {
            return Some(EXACT);
        }
        if self.is_stand_in(from) || self.is_stand_in(to) {
            // A sketch's stand-in: anything converts, at the cost of the worst conversion.
            return Some(TO_ANY);
        }
        // A `void` variable boxes too; a call without a value is rejected by the caller.
        if to == TypeId::ANY && from != TypeId::COMPILE_TIME {
            return Some(TO_ANY);
        }
        if self.ty.types.wide_float(from).is_some() || self.ty.types.wide_float(to).is_some() {
            return self.wide_implicit_cost(from, untyped, to);
        }
        let fk = self.ty.types.kind(from).clone();
        let tk = self.ty.types.kind(to).clone();
        if untyped {
            let tr = self.ty.types.repr(to);
            if from == TypeId::NULL {
                return matches!(
                    self.ty.types.kind(tr),
                    TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Type
                )
                .then_some(LITERAL);
            }
            // Between float overloads a literal prefers float32 (`log(2)` is `log(float32)`).
            let float_cost = |c: &Self| {
                if c.ty.types.repr(to) == TypeId::F64 {
                    LITERAL + 1
                } else {
                    LITERAL
                }
            };
            if self.ty.types.is_integer(from) {
                if self.ty.types.is_integer(to) {
                    return Some(LITERAL);
                }
                if self.ty.types.is_float(to) {
                    return Some(float_cost(self));
                }
                return None;
            }
            if self.ty.types.is_float(from) {
                return self.ty.types.is_float(to).then(|| float_cost(self));
            }
        }
        match (&fk, &tk) {
            (
                TypeKind::Int {
                    bits: fb,
                    signed: fs,
                },
                TypeKind::Int {
                    bits: tb,
                    signed: ts,
                },
            ) => {
                if (fs == ts && tb >= fb) || (!fs && *ts && tb > fb) {
                    Some(WIDEN)
                } else {
                    None
                }
            }
            (
                TypeKind::Float {
                    bits: 32,
                },
                TypeKind::Float {
                    bits: 64,
                },
            ) => Some(WIDEN),
            // Integers convert to a float only when every value is exactly representable.
            (
                TypeKind::Int {
                    bits: fb, ..
                },
                TypeKind::Float {
                    bits: tb,
                },
            ) if *fb
                <= if *tb == 64 {
                    32
                } else {
                    16
                } =>
            {
                Some(INT_TO_FLOAT)
            }
            (TypeKind::Pointer(_), TypeKind::Pointer(t)) if *t == TypeId::VOID => Some(POINTER),
            (TypeKind::Pointer(f), TypeKind::Pointer(_)) if *f == TypeId::VOID => Some(POINTER),
            // A `*void` holding an address (a looked-up `WNDPROC`) is a procedure pointer too.
            (TypeKind::Pointer(f), TypeKind::Proc(_)) if *f == TypeId::VOID => Some(POINTER),
            (TypeKind::Pointer(f), TypeKind::Pointer(t)) => {
                // `*[..] T` → `*[] T`: a resizable array starts with its view.
                if let (
                    TypeKind::Array {
                        elem: fe,
                        kind: ArrayKind::Resizable,
                    },
                    TypeKind::Array {
                        elem: te,
                        kind: ArrayKind::View,
                    },
                ) = (self.ty.types.kind(*f), self.ty.types.kind(*t))
                    && fe == te
                {
                    return Some(POINTER);
                }
                // *Derived → *Base through `#as` members.
                if self.as_offset(*f, *t).is_some() {
                    return Some(SUBTYPE);
                }
                // `*Wrapper` → the pointer type of its `#as` member (`#as handle: VkDevice`).
                self.as_offset(*f, to).map(|_| SUBTYPE + 1)
            }
            (TypeKind::Null, TypeKind::Pointer(_) | TypeKind::Proc(_)) => Some(LITERAL),
            // `E.loose` and `E` convert to each other (a node's `operator_type` passed as
            // `Operator_Type`).
            (TypeKind::Enum(a), TypeKind::Enum(b))
                if self.ty.types.enums[a.0 as usize].loose_of == Some(to)
                    || self.ty.types.enums[b.0 as usize].loose_of == Some(from) =>
            {
                Some(WIDEN)
            }
            // Loose enums are integer-compatible.
            (
                TypeKind::Enum(_),
                TypeKind::Int {
                    ..
                },
            ) if self.ty.types.is_loose_enum(from) => Some(WIDEN),
            (
                TypeKind::Int {
                    ..
                },
                TypeKind::Enum(_),
            ) if self.ty.types.is_loose_enum(to) => Some(WIDEN),
            (
                TypeKind::Array {
                    elem: fe,
                    kind: ArrayKind::Fixed(_) | ArrayKind::Resizable,
                },
                TypeKind::Array {
                    elem: te,
                    kind: ArrayKind::View,
                },
            ) if fe == te => Some(POINTER),
            // A fixed array decays to a pointer to its first element (`to_string(props.name)` on a
            // C `char[256]` field); ranked below view conversions but above boxing into `Any`.
            (
                TypeKind::Array {
                    elem: fe,
                    kind: ArrayKind::Fixed(_),
                },
                TypeKind::Pointer(te),
            ) if fe == te => Some(INT_TO_FLOAT),
            (TypeKind::Distinct(d), _) if self.ty.types.distincts[d.0 as usize].isa => {
                let base = self.ty.types.distincts[d.0 as usize].base;
                self.implicit_cost(base, false, to).map(|c| c + 1)
            }
            (TypeKind::Struct(_), _) => self.as_offset(from, to).map(|_| SUBTYPE),
            // Auto-dereference: `*Thing` → `Thing` (structs only), and `*Thing` → the type of
            // any of its `#as` members.
            (TypeKind::Pointer(f), _) => {
                let f = *f;
                if f == to && matches!(tk, TypeKind::Struct(_)) {
                    return Some(SUBTYPE + 1);
                }
                self.as_offset(f, to).map(|_| SUBTYPE + 1)
            }
            (TypeKind::Proc(a), TypeKind::Proc(b)) => (a == b).then_some(EXACT),
            _ => None,
        }
    }

    /// Offset of a `#as` member of type `to` inside struct `from` (searching recursively).
    pub fn as_offset(&mut self, from: TypeId, to: TypeId) -> Option<u64> {
        let s = self.ty.types.as_struct(from)?;
        if self.layout_struct(s, Span::NONE).is_err() {
            return None;
        }
        let fields = self.ty.types.struct_info(s).fields.clone();
        for field in fields.iter() {
            if field.as_ {
                if field.ty == to {
                    return Some(field.offset);
                }
                if let Some(inner) = self.as_offset(field.ty, to) {
                    return Some(field.offset + inner);
                }
            }
        }
        None
    }
}
