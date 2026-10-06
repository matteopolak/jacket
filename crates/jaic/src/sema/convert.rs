//! Implicit conversions (with overload costs) and explicit casts.
use super::lower::{FnCtx, Operand};
use super::*;
use crate::ir::{ConvOp, Ty};
use crate::types::{ArrayKind, TypeKind};

/// Conversion costs used to rank overloads. Lower is better.
pub const EXACT: u32 = 0;

pub const LITERAL: u32 = 1;
pub const WIDEN: u32 = 2;
pub const POINTER: u32 = 3;
pub const SUBTYPE: u32 = 4;
pub const INT_TO_FLOAT: u32 = 5;
pub const TO_ANY: u32 = 6;

impl Compiler {
    /// Cost of implicitly converting a value of `from` to `to`, if allowed.
    /// `untyped` marks literals, which convert between numeric types freely.
    pub fn implicit_cost(&mut self, from: TypeId, untyped: bool, to: TypeId) -> Option<u32> {
        if from == to {
            return Some(EXACT);
        }
        // A `void` variable boxes too; a call without a value is rejected by the caller.
        if to == TypeId::ANY && from != TypeId::COMPILE_TIME {
            return Some(TO_ANY);
        }
        if self.wide_float(from).is_some() || self.wide_float(to).is_some() {
            return self.wide_implicit_cost(from, untyped, to);
        }
        let fk = self.types.kind(from).clone();
        let tk = self.types.kind(to).clone();
        if untyped {
            let tr = self.types.repr(to);
            if from == TypeId::NULL {
                return matches!(
                    self.types.kind(tr),
                    TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Type
                )
                .then_some(LITERAL);
            }
            // Between float overloads a literal prefers float32 (`log(2)` is `log(float32)`).
            let float_cost = |c: &Self| {
                if c.types.repr(to) == TypeId::F64 {
                    LITERAL + 1
                } else {
                    LITERAL
                }
            };
            if self.types.is_integer(from) {
                if self.types.is_integer(to) {
                    return Some(LITERAL);
                }
                if self.types.is_float(to) {
                    return Some(float_cost(self));
                }
                return None;
            }
            if self.types.is_float(from) {
                return self.types.is_float(to).then(|| float_cost(self));
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
                ) = (self.types.kind(*f), self.types.kind(*t))
                    && fe == te
                {
                    return Some(POINTER);
                }
                // *Derived → *Base through `#as` members.
                self.as_offset(*f, *t).map(|_| SUBTYPE)
            }
            (TypeKind::Null, TypeKind::Pointer(_) | TypeKind::Proc(_)) => Some(LITERAL),
            // `E.loose` and `E` convert to each other (a node's `operator_type` passed as
            // `Operator_Type`).
            (TypeKind::Enum(a), TypeKind::Enum(b))
                if self.types.enums[a.0 as usize].loose_of == Some(to)
                    || self.types.enums[b.0 as usize].loose_of == Some(from) =>
            {
                Some(WIDEN)
            }
            // Loose enums are integer-compatible.
            (
                TypeKind::Enum(_),
                TypeKind::Int {
                    ..
                },
            ) if self.types.is_loose_enum(from) => Some(WIDEN),
            (
                TypeKind::Int {
                    ..
                },
                TypeKind::Enum(_),
            ) if self.types.is_loose_enum(to) => Some(WIDEN),
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
            (TypeKind::Distinct(d), _) if self.types.distincts[d.0 as usize].isa => {
                let base = self.types.distincts[d.0 as usize].base;
                self.implicit_cost(base, false, to).map(|c| c + 1)
            }
            (TypeKind::Struct(_), _) => self.as_offset(from, to).map(|_| SUBTYPE),
            // Auto-dereference: `*Thing` → `Thing` (structs only), also through `#as`.
            (TypeKind::Pointer(f), TypeKind::Struct(_)) => {
                let f = *f;
                if f == to {
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
        let s = self.types.as_struct(from)?;
        if self.layout_struct(s, Span::default()).is_err() {
            return None;
        }
        let fields = self.types.struct_info(s).fields.clone();
        for field in fields {
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

    /// Implicitly convert `op` to `to`, or fail with a type mismatch.
    pub fn convert(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        span: Span,
    ) -> Result<Operand> {
        let from = op.ty();
        if from == to
            && !matches!(
                op,
                Operand::Const {
                    untyped: true,
                    ..
                }
            )
        {
            return Ok(op);
        }
        // A one-character string constant is a `u8` (`case "'";` on a byte).
        if to == TypeId::U8
            && let Operand::Const {
                value: Value::String(s),
                ..
            } = &op
            && s.len() == 1
        {
            return Ok(Operand::int(s[0] as i128, TypeId::U8));
        }
        match op {
            Operand::Type(t) if to == TypeId::TYPE => return Ok(Operand::Type(t)),
            Operand::Type(t) if to == TypeId::ANY => {
                return self.box_any(f, Operand::Type(t), span);
            }
            Operand::Procs(ref procs) => {
                if let TypeKind::Proc(_) = self.types.kind(to) {
                    for &p in procs {
                        if self.proc(p).is_poly {
                            if let Some(inst) = self.instantiate_for_proc_type(p, to, span) {
                                return Ok(Operand::Procs(vec![inst]));
                            }
                            continue;
                        }
                        let pt = self.proc_type(p, span)?;
                        if pt == to || self.proc_types_compatible(pt, to) {
                            return Ok(Operand::Procs(vec![p]));
                        }
                    }
                    return err(
                        span,
                        format!("no overload matches procedure type {}", self.types.name(to)),
                    );
                }
                if to == TypeId::VOID_PTR || (self.types.is_pointer(to) && procs.len() == 1) {
                    let (_, v) = self.rvalue(f, op, span)?;
                    return Ok(Operand::Value {
                        ty: to,
                        val: v,
                    });
                }
                if to == TypeId::ANY {
                    return self.box_any(f, op, span);
                }
            }
            Operand::Const {
                value: Value::String(ref bytes),
                ..
            } if self.types.pointee(to) == Some(TypeId::U8) => {
                // String literals are NUL-terminated: their data pointer is a C string.
                let global = self.string_global(bytes);
                let val = f.b.global_addr(global);
                return Ok(Operand::Value {
                    ty: to,
                    val,
                });
            }
            Operand::Const {
                ty,
                ref value,
                untyped,
            } => {
                if let Some(c) = self.convert_const(ty, value, untyped, to, span)? {
                    return Ok(c);
                }
            }
            _ => {}
        }
        let untyped = matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        if self.implicit_cost(from, untyped, to).is_none() {
            return err(
                span,
                format!(
                    "type mismatch: expected {}, found {}",
                    self.types.name(to),
                    self.types.name(from)
                ),
            );
        }
        self.coerce(f, op, to, span)
    }

    /// Structurally identical procedure types (ignoring parameter names).
    pub(super) fn proc_types_compatible(&self, a: TypeId, b: TypeId) -> bool {
        match (self.types.kind(a), self.types.kind(b)) {
            (TypeKind::Proc(x), TypeKind::Proc(y)) => {
                x.params == y.params && x.returns == y.returns && x.c_call == y.c_call
            }
            _ => false,
        }
    }

    /// Constant-to-constant conversions (returns None when a runtime conversion is needed).
    fn convert_const(
        &mut self,
        ty: TypeId,
        value: &Value,
        untyped: bool,
        to: TypeId,
        span: Span,
    ) -> Result<Option<Operand>> {
        let tr = self.types.repr(to);
        if let Some(fmt) = self.wide_float(to) {
            if self.wide_float(ty).is_none()
                && (untyped || self.implicit_cost(ty, false, to).is_some())
                && let Some(value) = Self::wide_const(fmt, value)
            {
                return Ok(Some(Operand::Const {
                    ty: to,
                    value,
                    untyped: false,
                }));
            }
            return Ok(None);
        }
        let result = match value {
            Value::Int(i)
                if self.types.is_integer(tr)
                    && (untyped || self.implicit_cost(ty, false, to).is_some()) =>
            {
                let (bits, signed) = self.types.int_info(tr).unwrap();
                if untyped && !int_fits(*i, bits, signed) {
                    return err(
                        span,
                        format!("constant {i} does not fit in {}", self.types.name(to)),
                    );
                }
                if matches!(self.types.kind(to), TypeKind::Enum(_)) && untyped {
                    return Ok(None);
                }
                Value::Int(*i)
            }
            Value::Int(i) if self.types.is_float(tr) && untyped => {
                Value::Float(int_to_float(*i, tr))
            }
            Value::Float(x)
                if self.types.is_float(tr)
                    && (untyped || self.implicit_cost(ty, false, to).is_some()) =>
            {
                if tr == TypeId::F32 {
                    Value::Float(*x as f32 as f64)
                } else {
                    Value::Float(*x)
                }
            }
            Value::Null
                if matches!(
                    self.types.kind(tr),
                    TypeKind::Pointer(_) | TypeKind::Proc(_) | TypeKind::Type
                ) =>
            {
                Value::Null
            }
            Value::Bool(b) if tr == TypeId::BOOL => Value::Bool(*b),
            Value::String(s) if tr == TypeId::STRING => Value::String(s.clone()),
            Value::Type(t) if to == TypeId::TYPE => Value::Type(*t),
            Value::Proc(p) if self.implicit_cost(ty, false, to).is_some() => Value::Proc(*p),
            _ => return Ok(None),
        };
        Ok(Some(Operand::Const {
            ty: to,
            value: result,
            untyped: false,
        }))
    }

    /// Perform an allowed conversion at runtime.
    fn coerce(&mut self, f: &mut FnCtx, op: Operand, to: TypeId, span: Span) -> Result<Operand> {
        let from = op.ty();
        if to == TypeId::ANY {
            return self.box_any(f, op, span);
        }
        if let Some(converted) = self.wide_cast(f, op.clone(), to, span)? {
            return Ok(converted);
        }
        let op = if matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        ) {
            self.settle_untyped(op, Some(to))
        } else {
            op
        };
        let from = if op.ty() == from {
            from
        } else {
            op.ty()
        };
        let fk = self.types.kind(from).clone();
        let tk = self.types.kind(to).clone();
        match (&fk, &tk) {
            (
                TypeKind::Array {
                    kind: ArrayKind::Fixed(n),
                    ..
                },
                TypeKind::Array {
                    kind: ArrayKind::View,
                    ..
                },
            ) => {
                let n = *n;
                // A constant literal viewed as `[] T` lives in its own writable global, as in jai:
                // the view outlives the procedure (`return .["a", "b"];`) and may be written.
                let addr = match &op {
                    Operand::Const {
                        value: Value::Bytes(agg),
                        ..
                    } if n > 0 => {
                        let size = self.size_of(from, span)?;
                        let align = self.align_of(from, span)?;
                        let name = format!("array_literal.{}", self.program.globals.len());
                        let global = self.program.add_global(ir::Global {
                            name,
                            size,
                            align,
                            init: agg.bytes.clone(),
                            relocs: agg.relocs.clone(),
                            read_only: false,
                            export: None,
                        });
                        f.b.global_addr(global)
                    }
                    _ => self.address_of(f, op, span)?.1,
                };
                // An empty array views no storage (`.[]` is a null view, not a dangling address).
                let addr = if n == 0 {
                    f.b.iconst(Ty::Ptr, 0)
                } else {
                    addr
                };
                let view = f.b.alloca(16, 8);
                let count = f.b.iconst(Ty::I64, n);
                f.b.store(Ty::I64, view, count);
                let data = f.b.ptr_offset(view, 8);
                f.b.store(Ty::Ptr, data, addr);
                Ok(Operand::Value {
                    ty: to,
                    val: view,
                })
            }
            (
                TypeKind::Array {
                    kind: ArrayKind::Resizable,
                    ..
                },
                TypeKind::Array {
                    kind: ArrayKind::View,
                    ..
                },
            ) => {
                // A resizable array starts with (count, data): reuse its prefix.
                let (_, addr) = self.address_of(f, op, span)?;
                Ok(Operand::Value {
                    ty: to,
                    val: addr,
                })
            }
            (
                TypeKind::Array {
                    kind: ArrayKind::Fixed(_),
                    ..
                },
                TypeKind::Pointer(_),
            ) => {
                let (_, addr) = self.address_of(f, op, span)?;
                Ok(Operand::Value {
                    ty: to,
                    val: addr,
                })
            }
            (TypeKind::Struct(_), _) => {
                let offset = self.as_offset(from, to).unwrap();
                let (_, addr) = self.address_of(f, op, span)?;
                let field = f.b.ptr_offset(addr, offset);
                Ok(Operand::Place {
                    ty: to,
                    addr: field,
                })
            }
            (TypeKind::Pointer(inner), TypeKind::Struct(_)) => {
                let offset = if *inner == to {
                    0
                } else {
                    self.as_offset(*inner, to).unwrap()
                };
                let (_, v) = self.rvalue(f, op, span)?;
                Ok(Operand::Place {
                    ty: to,
                    addr: f.b.ptr_offset(v, offset),
                })
            }
            (TypeKind::Pointer(inner), TypeKind::Pointer(target))
                if *target != TypeId::VOID && *inner != TypeId::VOID =>
            {
                let offset = self.as_offset(*inner, *target).unwrap_or(0);
                let (_, v) = self.rvalue(f, op, span)?;
                Ok(Operand::Value {
                    ty: to,
                    val: f.b.ptr_offset(v, offset),
                })
            }
            _ => self.scalar_convert(f, op, to, span),
        }
    }

    /// Numeric/pointer conversion between scalar register classes.
    fn scalar_convert(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        span: Span,
    ) -> Result<Operand> {
        let from = op.ty();
        let (_, v) = self.rvalue(f, op, span)?;
        let (Some(ft), Some(tt)) = (self.ir_ty(from), self.ir_ty(to)) else {
            if self.types.repr(from) == self.types.repr(to) {
                return Ok(Operand::Value {
                    ty: to,
                    val: v,
                });
            }
            return err(
                span,
                format!(
                    "cannot convert {} to {}",
                    self.types.name(from),
                    self.types.name(to)
                ),
            );
        };
        let fsigned =
            self.types.int_info(from).is_some_and(|(_, s)| s) || from == TypeId::BOOL && false;
        let tsigned = self.types.int_info(to).is_some_and(|(_, s)| s);
        let val = numeric_conv(f, v, ft, tt, fsigned, tsigned);
        Ok(Operand::Value {
            ty: to,
            val,
        })
    }

    /// `cast(T) x` / `xx x`.
    pub fn explicit_cast(
        &mut self,
        f: &mut FnCtx,
        op: Operand,
        to: TypeId,
        flags: ast::CastFlags,
        span: Span,
    ) -> Result<Operand> {
        // A procedure name casts like the procedure value it denotes.
        let op = match op {
            Operand::Procs(_) => {
                let (ty, val) = self.rvalue(f, op, span)?;
                Operand::Value {
                    ty,
                    val,
                }
            }
            op => op,
        };
        let from = op.ty();
        if from == to
            && !matches!(
                op,
                Operand::Const {
                    untyped: true,
                    ..
                }
            )
        {
            return Ok(op);
        }
        if let Some(converted) = self.wide_cast(f, op.clone(), to, span)? {
            return Ok(converted);
        }
        // Constant folding of numeric casts.
        if let Operand::Const {
            value, ..
        } = &op
        {
            let tr = self.types.repr(to);
            let fr = self.types.repr(op.ty());
            let folded = match value {
                // `cast,force` between same-size ints and floats reinterprets the bits.
                Value::Int(i) if flags.force && self.types.is_float(tr) => {
                    Some(Value::Float(if tr == TypeId::F32 {
                        f32::from_bits(*i as u32) as f64
                    } else {
                        f64::from_bits(*i as u64)
                    }))
                }
                Value::Float(x) if flags.force && self.types.is_integer(tr) => {
                    let bits = if fr == TypeId::F32 {
                        (*x as f32).to_bits() as i128
                    } else {
                        x.to_bits() as i128
                    };
                    let (width, signed) = self.types.int_info(tr).unwrap();
                    Some(Value::Int(expr::wrap_int(bits, width, signed)))
                }
                Value::Int(i) if self.types.is_integer(tr) => {
                    let (bits, signed) = self.types.int_info(tr).unwrap();
                    Some(Value::Int(expr::wrap_int(*i, bits, signed)))
                }
                Value::Int(i) if self.types.is_float(tr) => {
                    Some(Value::Float(int_to_float(*i, tr)))
                }
                Value::Float(x) if self.types.is_float(tr) => {
                    Some(Value::Float(if tr == TypeId::F32 {
                        *x as f32 as f64
                    } else {
                        *x
                    }))
                }
                Value::Float(x) if self.types.is_integer(tr) => {
                    let (bits, signed) = self.types.int_info(tr).unwrap();
                    Some(Value::Int(expr::wrap_int(*x as i128, bits, signed)))
                }
                Value::Bool(b) if self.types.is_integer(tr) => Some(Value::Int(*b as i128)),
                Value::Int(i) if tr == TypeId::BOOL => Some(Value::Bool(*i != 0)),
                Value::Null if self.ir_ty(tr) == Some(Ty::Ptr) => Some(Value::Null),
                Value::Int(0) if self.ir_ty(tr) == Some(Ty::Ptr) && self.types.is_pointer(tr) => {
                    Some(Value::Null)
                }
                _ => None,
            };
            if let Some(value) = folded {
                return Ok(Operand::Const {
                    ty: to,
                    value,
                    untyped: false,
                });
            }
        }
        if matches!(
            op,
            Operand::Const {
                value: Value::String(_),
                ..
            }
        ) && self.types.pointee(to) == Some(TypeId::U8)
        {
            return self.convert(f, op, to, span);
        }
        if let Operand::Procs(procs) = &op
            && procs.len() == 1
            && matches!(
                self.types.kind(to),
                TypeKind::Proc(_) | TypeKind::Pointer(_)
            )
        {
            // Casting a procedure to another procedure type (or a pointer) reinterprets its address.
            if let Ok(c) = self.convert(f, op.clone(), to, span) {
                return Ok(c);
            }
            let (_, v) = self.rvalue(f, op, span)?;
            return Ok(Operand::Value {
                ty: to,
                val: v,
            });
        }
        if let Operand::Type(t) = op {
            if to == TypeId::TYPE {
                return Ok(Operand::Type(t));
            }
            // cast(*Type_Info) T: the descriptor address.
            let v = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
            return Ok(Operand::Value {
                ty: to,
                val: v,
            });
        }
        let untyped = matches!(
            op,
            Operand::Const {
                untyped: true,
                ..
            }
        );
        // A number, enum or pointer cast to `bool` tests for non-zero; keeping its low byte
        // would make `cast(bool) 2` a bool that `!` does not negate.
        if self.types.repr(to) == TypeId::BOOL
            && self.types.repr(from) != TypeId::BOOL
            && self.ir_ty(self.types.repr(from)).is_some()
            && !untyped
        {
            let (ty, v) = self.rvalue(f, op, span)?;
            let val = self.truthy(f, ty, v, span)?;
            return Ok(Operand::Value {
                ty: to,
                val,
            });
        }
        if self.implicit_cost(from, untyped, to).is_some()
            && !self.types.is_integer(to)
            && !self.types.is_float(to)
        {
            return self.convert(f, op, to, span);
        }
        // Same-representation reinterpretations (distinct/enum/struct-of-same-size are not allowed except via pointer).
        let fr = self.types.repr(from);
        let tr = self.types.repr(to);
        if flags.force
            && let (Some(fi), Some(ti)) = (self.ir_ty(fr), self.ir_ty(tr))
            && fi.is_float() != ti.is_float()
            && self.size_of(fr, span)? == self.size_of(tr, span)?
        {
            let (_, v) = self.rvalue(f, op, span)?;
            let val = f.b.conv(ConvOp::Bitcast, fi, ti, v);
            return Ok(Operand::Value {
                ty: to,
                val,
            });
        }
        // A string constant cast to a fixed byte array of matching length.
        if let Operand::Const {
            value: Value::String(bytes),
            ..
        } = &op
            && let TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } = self.types.kind(tr).clone()
            && (elem == TypeId::U8 || elem == TypeId::S8)
            && n as usize == bytes.len()
        {
            return Ok(Operand::Const {
                ty: to,
                value: Value::Bytes(std::rc::Rc::new(crate::sema::value::Aggregate {
                    bytes: bytes.to_vec(),
                    relocs: Vec::new(),
                })),
                untyped: false,
            });
        }
        // Strings and arrays cast to `bool` as "is non-empty".
        if tr == TypeId::BOOL && self.ir_ty(fr).is_none() && fr != TypeId::ANY {
            let (ty, v) = self.rvalue(f, op, span)?;
            let val = self.truthy(f, ty, v, span)?;
            return Ok(Operand::Value {
                ty: to,
                val,
            });
        }
        if self.ir_ty(fr).is_some() && self.ir_ty(tr).is_some() {
            let op = self.settle_untyped(op, Some(to));
            return self.scalar_convert(f, op, to, span);
        }
        if fr == tr {
            let (_, v) = self.rvalue(f, op, span)?;
            return Ok(Operand::Value {
                ty: to,
                val: v,
            });
        }
        // Array conversions.
        if let (
            TypeKind::Array {
                ..
            },
            TypeKind::Array {
                kind: ArrayKind::View,
                ..
            },
        ) = (self.types.kind(fr).clone(), self.types.kind(tr).clone())
        {
            let (_, addr) = self.address_of(f, op, span)?;
            return Ok(Operand::Value {
                ty: to,
                val: addr,
            });
        }
        if self.implicit_cost(from, untyped, to).is_some() {
            return self.convert(f, op, to, span);
        }
        // `string` and `[] u8` share the same layout (count, data).
        let is_bytes_view = |s: &Self, t: TypeId| {
            matches!(
                s.types.kind(t),
                TypeKind::Array {
                    elem,
                    kind: ArrayKind::View,
                } if *elem == TypeId::U8
            )
        };
        if (from == TypeId::STRING && is_bytes_view(self, to))
            || (to == TypeId::STRING && is_bytes_view(self, from))
        {
            let (_, addr) = self.address_of(f, op, span)?;
            return Ok(Operand::Place {
                ty: to,
                addr,
            });
        }
        // `xx bytes` where `bytes: [N] u8` / `[..] u8`: view the array, then as a string.
        if to == TypeId::STRING
            && let TypeKind::Array {
                elem: TypeId::U8, ..
            } = self.types.kind(fr)
        {
            let view = self.types.array(TypeId::U8, ArrayKind::View);
            let op = self.explicit_cast(f, op, view, flags, span)?;
            return self.explicit_cast(f, op, to, flags, span);
        }
        // `cast,force` reinterprets an aggregate as another of the same size.
        if flags.force
            && self.ir_ty(fr).is_none()
            && self.ir_ty(tr).is_none()
            && self.size_of(from, span)? == self.size_of(to, span)?
        {
            let (_, addr) = self.address_of(f, op, span)?;
            return Ok(Operand::Place {
                ty: to,
                addr,
            });
        }
        err(
            span,
            format!(
                "cannot cast {} to {}",
                self.types.name(from),
                self.types.name(to)
            ),
        )
    }

    /// Box a value into an `Any` (type info pointer + pointer to a copy).
    pub fn box_any(&mut self, f: &mut FnCtx, op: Operand, span: Span) -> Result<Operand> {
        if op.ty() == TypeId::ANY {
            return Ok(op);
        }
        let (ty, addr) = match op {
            Operand::Type(t) => {
                let v = self.materialize(f, &Value::Type(t), TypeId::TYPE, span)?;
                (TypeId::TYPE, self.spill(f, TypeId::TYPE, v, span)?)
            }
            Operand::Place {
                ty,
                addr,
            } => (ty, addr),
            other => {
                let other = self.settle_untyped(other, None);
                let (ty, v) = self.rvalue(f, other, span)?;
                (ty, self.spill(f, ty, v, span)?)
            }
        };
        let info = self.type_info_global(ty, span)?;
        let any = f.b.alloca(16, 8);
        let ti = f.b.global_addr(info);
        f.b.store(Ty::Ptr, any, ti);
        let vp = f.b.ptr_offset(any, 8);
        f.b.store(Ty::Ptr, vp, addr);
        Ok(Operand::Value {
            ty: TypeId::ANY,
            val: any,
        })
    }
}

pub fn int_fits(v: i128, bits: u8, signed: bool) -> bool {
    if bits >= 64 {
        // Accept both the signed and unsigned spelling of 64-bit constants.
        return v >= i64::MIN as i128 && v <= u64::MAX as i128;
    }
    if signed {
        let max = (1i128 << (bits - 1)) - 1;
        // Also accept bit patterns written as unsigned literals (e.g. 0xFFFF_FFFF for s32).
        v >= -max - 1 && v <= ((1i128 << bits) - 1)
    } else {
        v >= -(1i128 << (bits - 1)) && v < (1i128 << bits)
    }
}

/// Convert a scalar register between classes.
pub fn numeric_conv(
    f: &mut FnCtx,
    v: ir::Val,
    ft: Ty,
    tt: Ty,
    fsigned: bool,
    tsigned: bool,
) -> ir::Val {
    let _ = tsigned;
    if ft == tt {
        return v;
    }
    match (ft.is_float(), tt.is_float()) {
        (true, true) => f.b.conv(
            if tt.size() > ft.size() {
                ConvOp::FExt
            } else {
                ConvOp::FTrunc
            },
            ft,
            tt,
            v,
        ),
        (true, false) => {
            if tt == Ty::Ptr {
                let i = f.b.conv(ConvOp::FToS, ft, Ty::I64, v);
                return f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, i);
            }
            f.b.conv(
                if tsigned {
                    ConvOp::FToS
                } else {
                    ConvOp::FToU
                },
                ft,
                tt,
                v,
            )
        }
        (false, true) => {
            let v = if ft == Ty::Ptr {
                f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v)
            } else {
                v
            };
            let ft = if ft == Ty::Ptr {
                Ty::I64
            } else {
                ft
            };
            f.b.conv(
                if fsigned {
                    ConvOp::SToF
                } else {
                    ConvOp::UToF
                },
                ft,
                tt,
                v,
            )
        }
        (false, false) => {
            if ft == Ty::Ptr || tt == Ty::Ptr {
                // Pointers are 64-bit: widen/narrow through I64.
                let (v, ft) = if ft == Ty::Ptr {
                    (f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v), Ty::I64)
                } else {
                    (v, ft)
                };
                let v = numeric_conv(
                    f,
                    v,
                    ft,
                    if tt == Ty::Ptr {
                        Ty::I64
                    } else {
                        tt
                    },
                    fsigned,
                    tsigned,
                );
                return if tt == Ty::Ptr {
                    f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, v)
                } else {
                    v
                };
            }
            if tt.size() < ft.size() {
                f.b.conv(ConvOp::Trunc, ft, tt, v)
            } else if fsigned {
                f.b.conv(ConvOp::SExt, ft, tt, v)
            } else {
                f.b.conv(ConvOp::ZExt, ft, tt, v)
            }
        }
    }
}

/// An integer constant as a float of type `to`, rounded once to that type's precision: rounding
/// to f64 first and to f32 later would round twice (`0x8000_0080_0000_0001` would end on the
/// even f32 below instead of the one above).
fn int_to_float(i: i128, to: TypeId) -> f64 {
    if to == TypeId::F32 {
        i as f32 as f64
    } else {
        i as f64
    }
}
