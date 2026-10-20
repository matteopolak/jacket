//! Declaring, reading and writing the operands of an `#asm` instruction.
use super::helpers::{bin, bits, konst, ptr_of, resize};
use super::model::{AsmCtx, AsmReg, Imm, Opd, REG_CLASSES, RegPlace, Rmw};
use crate::ast::{AsmDecl, AsmMem, AsmOperand, AsmPin, CodeBody, Expr, ExprKind as E};
use crate::ir::{BinOp, ConvOp, Intrinsic, Ty, UnOp, Val};
use crate::sema::ProcState;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityKind, Found};
use crate::sema::{
    Compiler, Diagnostic, DiagnosticKind, PreludeType, Result, ScopeId, Span, Sym, TypeId, Value,
    err, scope,
};

impl Compiler {
    /// Canonical bootstrap operand marker, retaining its distinct source type.
    pub(in crate::sema) fn register_operand_type(&mut self, span: Span) -> Result<TypeId> {
        if self.ns.preload.is_some() {
            return self.preload_type(PreludeType::Reg, span);
        }
        let id = self.scope(self.ns.root_scope).names[&Sym::intern("__reg")][0];
        match self.resolve_entity(id)? {
            scope::Resolved::Const {
                value: Value::Type(ty),
                ..
            } => Ok(ty),
            _ => err(span, "bootstrap __reg must be a type"),
        }
    }

    pub(in crate::sema) fn is_register_operand_type(&mut self, ty: TypeId, span: Span) -> bool {
        matches!(self.ty.types.kind(ty), crate::types::TypeKind::Distinct(_))
            && self.register_operand_type(span).ok() == Some(ty)
    }

    /// Operand markers have no standalone runtime storage or procedure ABI.
    pub(in crate::sema) fn ensure_storage_type(&mut self, ty: TypeId, span: Span) -> Result<()> {
        if self.is_register_operand_type(ty, span) {
            return err(
                span,
                "a SIMD register operand has no standalone storage or runtime ABI; use #asm and expanded parameters",
            );
        }
        match self.ty.types.kind(ty).clone() {
            crate::types::TypeKind::Array {
                elem, ..
            } => self.ensure_storage_type(elem, span),
            crate::types::TypeKind::Distinct(d) => {
                self.ensure_storage_type(self.ty.types.distincts[d.0 as usize].base, span)
            }
            _ => Ok(()),
        }
    }

    /// `x: gpr;`, `x: gpr === a;` and `x === a;` as statements.
    pub(super) fn asm_decl_item(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        decl: &AsmDecl,
    ) -> Result<()> {
        if decl.colon {
            self.asm_declare(f, cx, decl, "gpr")?;
            return Ok(());
        }
        // `x === a` pins an existing variable.
        match self.lookup_full(cx.scope, decl.name.name)? {
            Found::Entities(ids) if !ids.is_empty() => {
                if let Some(&id) = ids.last()
                    && self.pr.asm_regs.get(&id) == Some(&AsmReg::Gpr)
                    && let EntityKind::Local {
                        addr, ..
                    } = self.entity(id).kind
                {
                    self.pr.asm_record_pin(decl, addr);
                }
                Ok(())
            }
            _ => Err(Box::new(
                Diagnostic::error(
                    decl.name.span,
                    format!("unknown identifier `{}`", decl.name.name),
                )
                .with_kind(DiagnosticKind::UnknownIdentifier {
                    scope: Some(cx.scope.0),
                }),
            )),
        }
    }

    /// Declare the register named by `decl` in the enclosing scope (reusing an earlier
    /// declaration of the same name in that scope). `default_class` applies to `name:`
    /// without a class (`vec` in a vector operand position).
    pub(super) fn asm_declare(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        decl: &AsmDecl,
        default_class: &str,
    ) -> Result<(AsmReg, Val)> {
        let class = decl.class.map_or(default_class, |c| c.name.as_str());
        let Some(&(_, kind, size)) = REG_CLASSES.iter().find(|(name, ..)| *name == class) else {
            return err(
                decl.class.map_or(decl.name.span, |c| c.span),
                format!("unknown #asm register class `{class}`"),
            );
        };
        let name = decl.name.name;
        if let Some(ids) = self.scope(cx.scope).names.get(&name)
            && let Some(&existing) = ids.last()
            && self.pr.asm_regs.get(&existing) == Some(&kind)
            && let EntityKind::Local {
                addr, ..
            } = self.entity(existing).kind
        {
            if kind == AsmReg::Gpr {
                self.pr.asm_record_pin(decl, addr);
            }
            return Ok((kind, addr));
        }
        let addr = f.b.alloca(size, 8);
        f.b.zero(addr, size);
        if kind == AsmReg::Gpr {
            self.pr.asm_record_pin(decl, addr);
        }
        let depth = self.scope(cx.scope).proc_depth;
        let ty = if kind == AsmReg::Vec {
            self.register_operand_type(decl.name.span)?
        } else {
            TypeId::U64
        };
        let id = self.add_entity(
            cx.scope,
            name,
            decl.name.span,
            EntityKind::Local {
                ty,
                addr,
                depth,
            },
            false,
        );
        self.pr.asm_regs.insert(id, kind);
        Ok((kind, addr))
    }

    /// Errors unless `kind` is a general-purpose register.
    fn asm_require_gpr(kind: AsmReg, span: Span) -> Result<()> {
        match kind {
            AsmReg::Gpr => Ok(()),
            AsmReg::Vec => err(
                span,
                "a vector register is not valid in a general-purpose instruction",
            ),
            AsmReg::Mask => err(
                span,
                "a mask register is only valid in mask (k*) and AVX-512 instructions",
            ),
        }
    }

    pub(super) fn asm_gpr_opd(&self, kind: AsmReg, addr: Val, span: Span) -> Result<Opd> {
        Self::asm_require_gpr(kind, span)?;
        Ok(Opd::Reg(RegPlace {
            addr,
            storage: Ty::I64,
            signed: false,
            natural: false,
        }))
    }

    pub(super) fn asm_operand(
        &mut self,
        f: &mut FnCtx,
        cx: &AsmCtx,
        o: &AsmOperand,
    ) -> Result<Opd> {
        match o {
            AsmOperand::Value(e) => self.asm_value(f, cx.scope, e, 0),
            AsmOperand::Mem(m) => Ok(Opd::Mem(self.asm_mem(f, cx.scope, m)?)),
            AsmOperand::Decl(d) if d.colon => {
                let (kind, addr) = self.asm_declare(f, cx, d, "gpr")?;
                self.asm_gpr_opd(kind, addr, d.name.span)
            }
            AsmOperand::Decl(d) => err(
                d.name.span,
                "a register pin is only valid as a separate statement",
            ),
        }
    }

    /// A variable or constant named in an operand position.
    pub(super) fn asm_value(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        e: &Expr,
        depth: u32,
    ) -> Result<Opd> {
        let mut is_gpr = false;
        if let E::Ident(name) = &e.kind
            && let Found::Entities(ids) = self.lookup_full(scope, *name)?
            && let Some(&id) = ids.last()
            && let Some(&kind) = self.pr.asm_regs.get(&id)
        {
            is_gpr = true;
            Self::asm_require_gpr(kind, e.span)?;
        }
        let op = self.check_expr(f, scope, e, None)?;
        match op {
            Operand::Place {
                ty,
                addr,
            } => self.asm_place(ty, addr, !is_gpr, e.span),
            Operand::Const {
                value: Value::Code(code),
                ..
            } => {
                // An explicitly quoted Code expression keeps its lexical scope.
                if depth > 8 {
                    return err(e.span, "#asm register argument is nested too deeply");
                }
                let body = self.pr.codes[code.0 as usize].clone();
                let code_scope = self.pr.code_scopes[code.0 as usize];
                match &*body {
                    CodeBody::Expr(inner) => self.asm_value(f, code_scope, inner, depth + 1),
                    CodeBody::Block(_) | CodeBody::Stmt(_) => {
                        err(e.span, "an #asm operand must be a variable")
                    }
                }
            }
            Operand::Const {
                value, ..
            } => match value {
                Value::Int(v) => Ok(Opd::Imm(Imm::Int(v))),
                Value::Float(v, _) => Ok(Opd::Imm(Imm::Float(v))),
                Value::Bool(v) => Ok(Opd::Imm(Imm::Int(v as i128))),
                Value::Null => Ok(Opd::Imm(Imm::Int(0))),
                _ => err(e.span, "unsupported constant in an #asm operand"),
            },
            other @ Operand::Value {
                ..
            } => {
                // A computed value: usable as a source (the temporary absorbs writes).
                let span = e.span;
                let (ty, val) = self.rvalue(f, other, span)?;
                let addr = self.spill(f, ty, val, span)?;
                self.asm_place(ty, addr, true, span)
            }
            _ => err(e.span, "an #asm operand must be a variable or a constant"),
        }
    }

    pub(super) fn asm_place(
        &mut self,
        ty: TypeId,
        addr: Val,
        natural: bool,
        span: Span,
    ) -> Result<Opd> {
        let storage = match self.ir_ty(ty) {
            Some(t) if !t.is_float() => t,
            Some(_) => {
                return err(
                    span,
                    "a floating-point variable is a vector register in #asm: it is an operand of vector instructions \
                     (`addss x, y`) and of `movd`/`movq`, not of integer instructions",
                );
            }
            None => {
                return err(
                    span,
                    format!(
                        "#asm operand of type `{}` is not a scalar",
                        self.ty.types.name(ty)
                    ),
                );
            }
        };
        let signed = self.ty.types.int_info(ty).is_some_and(|(_, s)| s);
        Ok(Opd::Reg(RegPlace {
            addr,
            storage,
            signed,
            natural,
        }))
    }

    /// `[base + index*scale +/- disp]` as an `I64` address.
    pub(super) fn asm_mem(&mut self, f: &mut FnCtx, scope: ScopeId, m: &AsmMem) -> Result<Val> {
        let mut acc: Option<Val> = None;
        let mut disp: i128 = 0;
        for term in &m.terms {
            let scale = match &term.scale {
                Some(e) => match self.asm_value(f, scope, e, 0)? {
                    Opd::Imm(Imm::Int(s)) => s,
                    _ => return err(e.span, "the scale of a memory operand must be a constant"),
                },
                None => 1,
            };
            let sign = if term.negate {
                -1
            } else {
                1
            };
            match self.asm_value(f, scope, &term.value, 0)? {
                Opd::Imm(Imm::Int(c)) => disp = disp.wrapping_add(c.wrapping_mul(scale) * sign),
                Opd::Reg(r) => {
                    let mut v = Self::asm_load_i64(f, r);
                    if scale != 1 {
                        let s = konst(f, Ty::I64, scale as u64);
                        v = bin(f, BinOp::Mul, Ty::I64, v, s);
                    }
                    acc = Some(match acc {
                        None if sign < 0 => {
                            let z = konst(f, Ty::I64, 0);
                            bin(f, BinOp::Sub, Ty::I64, z, v)
                        }
                        None => v,
                        Some(a) if sign < 0 => bin(f, BinOp::Sub, Ty::I64, a, v),
                        Some(a) => bin(f, BinOp::Add, Ty::I64, a, v),
                    });
                }
                _ => {
                    return err(
                        term.value.span,
                        "a memory operand term must be an integer variable or constant",
                    );
                }
            }
        }
        Ok(match acc {
            None => konst(f, Ty::I64, disp as u64),
            Some(a) if disp == 0 => a,
            Some(a) => {
                let d = konst(f, Ty::I64, disp as u64);
                bin(f, BinOp::Add, Ty::I64, a, d)
            }
        })
    }

    /// Load a variable as a 64-bit integer (extended by its own signedness).
    fn asm_load_i64(f: &mut FnCtx, r: RegPlace) -> Val {
        let v = f.b.load(r.storage, r.addr);
        if r.storage == Ty::Ptr {
            return f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v);
        }
        resize(f, v, r.storage, Ty::I64, r.signed)
    }

    fn asm_imm_bits(&self, imm: Imm, sz: Ty, span: Span) -> Result<u64> {
        match imm {
            Imm::Int(v) => {
                let n = bits(sz) as u32;
                let min = -(1i128 << (n - 1));
                let max = (1i128 << n) - 1;
                if v < min || v > max {
                    return err(
                        span,
                        format!(
                            "immediate {v} does not fit in {n} bits (no matching instruction form)"
                        ),
                    );
                }
                let mask = if n == 64 {
                    u64::MAX
                } else {
                    (1u64 << n) - 1
                };
                Ok(v as u64 & mask)
            }
            Imm::Float(x) => match sz {
                Ty::I32 => Ok((x as f32).to_bits() as u64),
                Ty::I64 => Ok(x.to_bits()),
                _ => err(
                    span,
                    "floating-point immediates need a 32- or 64-bit operand size",
                ),
            },
        }
    }

    pub(super) fn asm_read(&mut self, f: &mut FnCtx, o: Opd, sz: Ty, span: Span) -> Result<Val> {
        Ok(match o {
            Opd::Reg(r) => {
                let v = f.b.load(r.storage, r.addr);
                let (v, from) = if r.storage == Ty::Ptr {
                    (f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, v), Ty::I64)
                } else {
                    (v, r.storage)
                };
                resize(f, v, from, sz, false)
            }
            Opd::Imm(imm) => {
                let bits = self.asm_imm_bits(imm, sz, span)?;
                konst(f, sz, bits)
            }
            Opd::Mem(addr) => {
                let p = ptr_of(f, addr);
                f.b.load(sz, p)
            }
        })
    }

    pub(super) fn asm_write(
        &mut self,
        f: &mut FnCtx,
        o: Opd,
        sz: Ty,
        v: Val,
        span: Span,
    ) -> Result<()> {
        match o {
            Opd::Reg(r) => {
                let st = if r.storage == Ty::Ptr {
                    Ty::I64
                } else {
                    r.storage
                };
                let new = if sz == st {
                    v
                } else if sz.size() > st.size() {
                    resize(f, v, sz, st, false)
                } else if sz == Ty::I32 {
                    // 32-bit writes clear the upper half of the register.
                    resize(f, v, sz, st, false)
                } else {
                    // 8- and 16-bit writes keep the rest of the register.
                    let old = f.b.load(r.storage, r.addr);
                    let old = if r.storage == Ty::Ptr {
                        f.b.conv(ConvOp::Bitcast, Ty::Ptr, Ty::I64, old)
                    } else {
                        old
                    };
                    let low = resize(f, v, sz, st, false);
                    let mask = konst(f, st, (1u64 << bits(sz)) - 1);
                    let keep = f.b.un(UnOp::Not, st, mask);
                    let kept = bin(f, BinOp::And, st, old, keep);
                    bin(f, BinOp::Or, st, kept, low)
                };
                let new = if r.storage == Ty::Ptr {
                    f.b.conv(ConvOp::Bitcast, Ty::I64, Ty::Ptr, new)
                } else {
                    new
                };
                f.b.store(r.storage, r.addr, new);
                Ok(())
            }
            Opd::Mem(addr) => {
                let p = ptr_of(f, addr);
                f.b.store(sz, p, v);
                Ok(())
            }
            Opd::Imm(_) => err(
                span,
                "an immediate cannot be the destination of an #asm instruction",
            ),
        }
    }

    // Read-modify-write

    /// Start updating `dst`. With `lock` on a memory operand this opens a
    /// compare-and-swap loop that `rmw_end` closes; the value computed in
    /// between must derive only from the returned old value and values
    /// computed before the call.
    pub(super) fn rmw_begin(
        &mut self,
        f: &mut FnCtx,
        dst: Opd,
        sz: Ty,
        lock: bool,
        span: Span,
    ) -> Result<(Rmw, Val)> {
        if lock && let Opd::Mem(addr) = dst {
            let retry = f.b.new_block();
            let exit = f.b.new_block();
            f.b.jump(retry);
            f.b.switch_to(retry);
            let ptr = ptr_of(f, addr);
            let old = f.b.load(sz, ptr);
            return Ok((
                Rmw {
                    dst,
                    sz,
                    cas: Some((ptr, old, retry, exit)),
                },
                old,
            ));
        }
        let old = self.asm_read(f, dst, sz, span)?;
        Ok((
            Rmw {
                dst,
                sz,
                cas: None,
            },
            old,
        ))
    }

    pub(super) fn rmw_end(&mut self, f: &mut FnCtx, rmw: Rmw, new: Val, span: Span) -> Result<()> {
        match rmw.cas {
            Some((ptr, old, retry, exit)) => {
                let width = konst(f, Ty::I64, rmw.sz.size());
                let r = f.b.intrinsic(
                    Intrinsic::CompareAndSwap,
                    vec![ptr, old, new, width],
                    &[Ty::I8, rmw.sz],
                );
                f.b.branch(r[0], exit, retry);
                f.b.switch_to(exit);
                Ok(())
            }
            None => self.asm_write(f, rmw.dst, rmw.sz, new, span),
        }
    }
}

impl ProcState {
    /// Remember that the general-purpose register at `addr` is pinned as `decl` says.
    fn asm_record_pin(&mut self, decl: &AsmDecl, addr: Val) {
        let Some(pin) = decl.pin else {
            return;
        };
        let reg = match pin {
            AsmPin::Index(i) if i < 16 => i as u8,
            AsmPin::Index(_) => return,
            AsmPin::Name(name) => {
                let name = name.name.as_str();
                match name.strip_prefix('r').unwrap_or(name) {
                    "a" | "ax" => 0,
                    "c" | "cx" => 1,
                    "d" | "dx" => 2,
                    "b" | "bx" => 3,
                    "sp" => 4,
                    "bp" => 5,
                    "si" => 6,
                    "di" => 7,
                    other => match other.parse::<u8>() {
                        Ok(n) if (8..16).contains(&n) && name.starts_with('r') => n,
                        _ => return,
                    },
                }
            }
        };
        self.asm_pins.push((reg, addr));
    }
}
