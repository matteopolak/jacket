//! `while`, `for`, `break`/`continue`/`remove` and for-expansion.
use crate::ast::ExprKind as E;
use crate::ir::{CmpOp, Ty};
use crate::sema::calls::CallArg;
use crate::sema::lower::{FnCtx, ForBody, LoopFrame, LoopRemove, Operand};
use crate::sema::scope::{EntityKind, Resolved};
use crate::sema::{
    Compiler, PreludeType, ProcId, Rc, Result, ScopeId, Span, Sym, TypeId, Value, ast, err, ir,
    procs,
};
use crate::types::{ArrayKind, TypeKind};

/// What the two forms of a `for` loop share once its names and scope are set up.
#[derive(Clone, Copy)]
struct ForLoop<'a> {
    for_: &'a ast::For,
    span: Span,
    loop_scope: ScopeId,
    depth: u32,
    it_name: Sym,
    index_name: Sym,
}

impl Compiler {
    pub(super) fn check_while(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        while_: &ast::WhileStmt,
    ) -> Result<()> {
        let ast::WhileStmt {
            label,
            bind_label,
            backtick_label,
            cond,
            body,
        } = while_;
        let head = f.b.new_block();
        let body_block = f.b.new_block();
        let exit = f.b.new_block();
        f.b.jump(head);
        f.b.switch_to(head);
        let (scope, c) = match label.filter(|_| *bind_label) {
            Some(name) => {
                // `while s := next()`: the value is a local of the loop, besides being its label.
                let op = self.check_expr(f, scope, cond, Some(TypeId::BOOL))?;
                let op = self.settle_untyped(op, None);
                let (ty, v) = self.rvalue(f, op, cond.span)?;
                let size = self.size_of(ty, cond.span)?;
                let align = self.align_of(ty, cond.span)?;
                let addr = f.b.alloca(size.max(1), align);
                self.store_value(f, ty, addr, v, cond.span)?;
                let inner = self.new_block_scope(scope);
                self.ide_scope_span(inner, cond.span.to(body.span));
                let depth = self.scope(inner).proc_depth;
                self.declare_loop_var(
                    f,
                    inner,
                    *backtick_label,
                    (
                        name.name,
                        EntityKind::Local {
                            ty,
                            addr,
                            depth,
                        },
                    ),
                    (name.span, Some(name.span)),
                );
                (inner, self.truthy(f, ty, v, cond.span)?)
            }
            None => (scope, self.check_condition(f, scope, cond)?),
        };
        // `while true` only ends through `break`, which the missing-return warning relies on.
        if matches!(cond.kind, ast::ExprKind::Bool(true)) {
            f.b.jump(body_block);
        } else {
            f.b.branch(c, body_block, exit);
        }
        f.b.switch_to(body_block);
        let defer_depth = f.mark_defers();
        f.loops.push(LoopFrame {
            label: label.map(|l| l.name),
            break_block: exit,
            continue_block: head,
            defer_depth,
            remove: LoopRemove::Unsupported(
                "remove needs a for loop over an array, not a while loop".into(),
            ),
        });
        let result = self.check_scoped(f, scope, body);
        f.loops.pop();
        result?;
        f.b.jump(head);
        f.b.switch_to(exit);
        Ok(())
    }

    pub(super) fn check_break(
        &mut self,
        f: &mut FnCtx,
        label: Option<Sym>,
        is_break: bool,
        span: Span,
    ) -> Result<()> {
        let frame = match label {
            Some(l) => f.loops.iter().rev().find(|lp| lp.label == Some(l)).cloned(),
            None => f.loops.last().cloned(),
        };
        let Some(frame) = frame else {
            let what = if is_break {
                "break"
            } else {
                "continue"
            };
            return match label {
                Some(l) => err(span, format!("{what}: no enclosing loop named `{l}`")),
                None => err(span, format!("{what} outside of a loop")),
            };
        };
        self.emit_defers(f, frame.defer_depth.position(), span)?;
        f.b.jump(if is_break {
            frame.break_block
        } else {
            frame.continue_block
        });
        Ok(())
    }

    pub(super) fn check_remove(
        &mut self,
        f: &mut FnCtx,
        label: Option<Sym>,
        span: Span,
    ) -> Result<()> {
        let frame = match label {
            Some(l) => f.loops.iter().rev().find(|lp| lp.label == Some(l)).cloned(),
            None => f.loops.last().cloned(),
        };
        let Some(frame) = frame else {
            return match label {
                Some(l) => err(span, format!("remove: no enclosing loop named `{l}`")),
                None => err(span, "remove outside of a for loop"),
            };
        };
        let (container, index_slot, elem, reverse) = match frame.remove {
            LoopRemove::Array(container, index_slot, elem, reverse) => {
                (container, index_slot, elem, reverse)
            }
            LoopRemove::Unsupported(message) => return err(span, message),
        };
        // array[it_index] = array[count-1]; count -= 1; it_index -= 1 (going forward: the
        // moved element is visited next; a reverse loop has already seen it).
        let Operand::Place {
            addr, ..
        } = container
        else {
            return err(span, "remove needs an addressable array");
        };
        let count = f.b.load(Ty::I64, addr);
        let dp = f.b.ptr_offset(addr, 8);
        let data = f.b.load(Ty::Ptr, dp);
        let size = self.size_of(elem, span)?;
        let s = f.b.iconst(Ty::I64, size);
        let one = f.b.iconst(Ty::I64, 1);
        let last = f.b.bin(ir::BinOp::Sub, Ty::I64, count, one);
        let idx = f.b.load(Ty::I64, index_slot);
        let off_i = f.b.bin(ir::BinOp::Mul, Ty::I64, idx, s);
        let off_l = f.b.bin(ir::BinOp::Mul, Ty::I64, last, s);
        let dst = f.b.ptr_add(data, off_i);
        let src = f.b.ptr_add(data, off_l);
        f.b.copy(dst, src, size);
        f.b.store(Ty::I64, addr, last);
        if !reverse {
            let idx2 = f.b.bin(ir::BinOp::Sub, Ty::I64, idx, one);
            f.b.store(Ty::I64, index_slot, idx2);
        }
        Ok(())
    }

    /// Declare a loop's iterator and index. ``for `it, `it_index`` inside a macro also declares
    /// them in the scope that called the macro, where the inserted loop body can see them.
    fn declare_loop_vars(
        &mut self,
        f: &mut FnCtx,
        loop_scope: ScopeId,
        backtick: bool,
        vars: [(Sym, EntityKind); 2],
        for_: &ast::For,
        span: Span,
    ) {
        let written = [for_.it.map(|i| i.span), for_.index.map(|i| i.span)];
        let first = vars[0].0;
        for (i, var) in vars.into_iter().enumerate() {
            // `for it_index: 0..n`: the named value hides the default index.
            if i == 1 && var.0 == first {
                continue;
            }
            self.declare_loop_var(f, loop_scope, backtick, var, (span, written[i]));
        }
    }

    /// Publish the same local storage to an expansion caller for a backtick loop name.
    fn declare_loop_var(
        &mut self,
        f: &mut FnCtx,
        loop_scope: ScopeId,
        backtick: bool,
        (name, kind): (Sym, EntityKind),
        (span, written): (Span, Option<Span>),
    ) {
        // The entity's span is the whole loop; the editor's declaration is the name, where the
        // loop writes it, and none for the `it` and `it_index` it declares unwritten.
        if backtick
            && let Some(caller) = f.macros.last().map(|m| m.caller_scope)
            && caller != loop_scope
        {
            let id = self.new_entity(caller, name, span, kind.clone(), false);
            self.ide_note_entity_at(id, written);
        }
        self.debug_local_kind(f, loop_scope, name, span, &kind);
        let id = self.new_entity(loop_scope, name, span, kind, false);
        self.ide_note_entity_at(id, written);
    }

    pub(super) fn check_for(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        for_: &ast::For,
        span: Span,
    ) -> Result<()> {
        // `for *=cond` / `for <=cond`: by pointer / in reverse when the constant holds.
        if for_.pointer_if.is_some() || for_.reverse_if.is_some() {
            let mut resolved = for_.clone();
            if let Some(cond) = resolved.pointer_if.take() {
                resolved.by_pointer |= self.eval_static_condition(scope, &cond)?;
            }
            if let Some(cond) = resolved.reverse_if.take() {
                resolved.reverse |= self.eval_static_condition(scope, &cond)?;
            }
            return self.check_for(f, scope, &resolved, span);
        }
        let flagged = crate::parser::check_bits(&for_.flags);
        if flagged.adds_to(f.checks_off) {
            let saved = f.checks_off;
            f.checks_off = saved.with(flagged);
            let result = self.check_for(f, scope, for_, span);
            f.checks_off = saved;
            return result;
        }
        let it_name = for_.it.map_or_else(|| Sym::intern("it"), |i| i.name);
        let index_name = for_
            .index
            .map_or_else(|| Sym::intern("it_index"), |i| i.name);
        if let Some(index) = for_.index
            && index.name == it_name
        {
            return err(
                index.span,
                format!("the loop's value and index are both named `{it_name}`"),
            );
        }
        let (lo, hi, collection) = match &for_.over {
            ast::ForOver::Range(a, b) => (Some(a), Some(b), None),
            ast::ForOver::Collection(c) => (None, None, Some(c)),
        };
        let loop_scope = self.new_block_scope(scope);
        self.ide_scope_span(loop_scope, span);
        let depth = self.scope(loop_scope).proc_depth;
        let lp = ForLoop {
            for_,
            span,
            loop_scope,
            depth,
            it_name,
            index_name,
        };
        match (lo, hi, collection) {
            (Some(a), Some(b), _) => self.check_for_range(f, scope, &lp, a, b),
            (_, _, Some(c)) => self.check_for_collection(f, scope, &lp, c),
            _ => unreachable!("a `for` iterates a range or a collection"),
        }
    }

    /// `for a..b`: an inclusive integer range.
    fn check_for_range(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        lp: &ForLoop,
        a: &ast::Expr,
        b: &ast::Expr,
    ) -> Result<()> {
        let ForLoop {
            for_,
            span,
            loop_scope,
            depth,
            it_name,
            index_name,
        } = *lp;
        // Integer range, inclusive.
        let av = self.check_expr(f, scope, a, None)?;
        let bv = self.check_expr(f, scope, b, None)?;
        let ty = match (&av, &bv) {
            (
                Operand::Const {
                    untyped: true, ..
                },
                Operand::Const {
                    untyped: true, ..
                },
            ) => TypeId::S64,
            (
                Operand::Const {
                    untyped: true, ..
                },
                other,
            ) => other.ty(),
            (other, _) => other.ty(),
        };
        let ty = if self.ty.types.is_integer(ty) {
            ty
        } else {
            TypeId::S64
        };
        let av = self.convert(f, av, ty, a.span)?;
        let bv = self.convert(f, bv, ty, b.span)?;
        let (_, start) = self.rvalue(f, av, a.span)?;
        let (_, end) = self.rvalue(f, bv, b.span)?;
        let t = self.ir_ty(ty).unwrap();
        let signed = self.ty.types.int_info(ty).is_some_and(|i| i.1);
        let it = f.b.alloca(t.size(), t.size());
        let idx = f.b.alloca(8, 8);
        f.b.store(
            t,
            it,
            if for_.reverse {
                end
            } else {
                start
            },
        );
        let zero = f.b.iconst(Ty::I64, 0);
        f.b.store(Ty::I64, idx, zero);
        let head = f.b.new_block();
        let body_block = f.b.new_block();
        let step = f.b.new_block();
        let exit = f.b.new_block();
        f.b.jump(head);
        f.b.switch_to(head);
        let cur = f.b.load(t, it);
        let op = match (for_.reverse, signed) {
            (false, true) => CmpOp::SLe,
            (false, false) => CmpOp::ULe,
            (true, true) => CmpOp::SGe,
            (true, false) => CmpOp::UGe,
        };
        let c = f.b.cmp(
            op,
            t,
            cur,
            if for_.reverse {
                start
            } else {
                end
            },
        );
        f.b.branch(c, body_block, exit);
        f.b.switch_to(body_block);
        let names = [it_name, index_name];
        self.declare_loop_vars(
            f,
            loop_scope,
            for_.backtick_names,
            [
                (
                    names[0],
                    EntityKind::Local {
                        ty,
                        addr: it,
                        depth,
                    },
                ),
                (
                    names[1],
                    EntityKind::Local {
                        ty: TypeId::S64,
                        addr: idx,
                        depth,
                    },
                ),
            ],
            for_,
            span,
        );
        self.ide_note_implicit_loop_names(for_, a.span, [ty, TypeId::S64]);
        let defer_depth = f.mark_defers();
        f.loops.push(LoopFrame {
            label: Some(it_name),
            break_block: exit,
            continue_block: step,
            defer_depth,
            remove: LoopRemove::Unsupported(
                "remove needs a for loop over an array, not a range of numbers".into(),
            ),
        });
        let result = self.check_scoped(f, loop_scope, &for_.body);
        f.loops.pop();
        result?;
        f.b.jump(step);
        f.b.switch_to(step);
        // Stop at (or past, if the body moved `it`) the bound, before stepping could wrap.
        // Otherwise the next value is still in range, so the step goes straight back to the
        // body and only the first iteration runs the test in `head`.
        let cur = f.b.load(t, it);
        let past = match (for_.reverse, signed) {
            (false, true) => CmpOp::SGe,
            (false, false) => CmpOp::UGe,
            (true, true) => CmpOp::SLe,
            (true, false) => CmpOp::ULe,
        };
        let at_end = f.b.cmp(
            past,
            t,
            cur,
            if for_.reverse {
                start
            } else {
                end
            },
        );
        let advance = f.b.new_block();
        f.b.branch(at_end, exit, advance);
        f.b.switch_to(advance);
        let one = f.b.iconst(t, 1);
        let next = f.b.bin(
            if for_.reverse {
                ir::BinOp::Sub
            } else {
                ir::BinOp::Add
            },
            t,
            cur,
            one,
        );
        f.b.store(t, it, next);
        let i = f.b.load(Ty::I64, idx);
        let one64 = f.b.iconst(Ty::I64, 1);
        let i2 = f.b.bin(ir::BinOp::Add, Ty::I64, i, one64);
        f.b.store(Ty::I64, idx, i2);
        f.b.jump(body_block);
        f.b.switch_to(exit);
        Ok(())
    }

    /// `for x: collection`: an array, a view, or a struct with a `for_expansion`.
    fn check_for_collection(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        lp: &ForLoop,
        c: &ast::Expr,
    ) -> Result<()> {
        let ForLoop {
            for_,
            span,
            loop_scope,
            depth,
            it_name,
            index_name,
        } = *lp;
        // `for iterate(x)`: a call to a for_expansion-shaped macro (body: Code second).
        if for_.iterator.is_none()
            && let ast::ExprKind::Call {
                callee,
                args,
                ..
            } = &c.kind
            && let Ok(Operand::Procs(procs)) = self.check_expr_no_emit(scope, callee)
            && !procs.is_empty()
            && procs.iter().all(|&p| self.proc_takes_for_body(p))
        {
            let leading = self.precheck_args(f, scope, args)?;
            return self.check_for_expansion(
                f,
                scope,
                for_,
                (procs, leading),
                (it_name, index_name),
                span,
            );
        }
        let iterated = c.span;
        let op = self.check_expr(f, scope, c, None)?;
        let cty = op.ty();
        let target = self.ty.types.pointee(cty).unwrap_or(cty);
        // A sketch's stand-in iterates like an array of stand-ins.
        let stand_in_elem = (for_.iterator.is_none() && self.is_stand_in(cty))
            .then(|| self.ide_stand_in_member(target, Sym::intern("[]"), c.span));
        let is_struct = stand_in_elem.is_none()
            && matches!(
                self.ty.types.kind(self.ty.types.repr_struct(target)),
                TypeKind::Struct(_)
            );
        if for_.iterator.is_some() || is_struct {
            let procs = self.for_expansion_procs(scope, for_, &op, span)?;
            // `for_expansion :: (x: *T, ...)` iterates a struct value through its address.
            // Unless an overload takes this very type by value (overloads for other types
            // in the same set do not count).
            let mut by_value = false;
            let mut by_pointer = false;
            for &p in &procs {
                let Some(t) = self
                    .proc(p)
                    .lit
                    .header
                    .params
                    .first()
                    .and_then(|q| q.ty.clone())
                else {
                    continue;
                };
                if matches!(t.kind, E::Unary(ast::UnOp::Star, _)) {
                    by_pointer = true;
                } else {
                    let home = self.proc(p).scope;
                    let takes_it = if procs::has_poly(&t)
                        || self.mentions_poly_struct(home, &t).unwrap_or(false)
                    {
                        // A polymorphic pattern counts only when it matches this very type.
                        self.match_pattern(&t, cty, &mut Vec::new(), home).is_ok()
                    } else {
                        self.eval_type(home, &t).ok() == Some(cty)
                    };
                    by_value |= takes_it;
                }
            }
            let wants_pointer = by_pointer && !by_value;
            let op = if wants_pointer && !self.ty.types.is_pointer(cty) {
                let (_, addr) = self.address_of(f, op, span)?;
                Operand::Value {
                    ty: self.ty.types.pointer(cty),
                    val: addr,
                }
            } else {
                op
            };
            let leading = vec![CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(op),
                span,
                scope,
            }];
            return self.check_for_expansion(
                f,
                scope,
                for_,
                (procs, leading),
                (it_name, index_name),
                span,
            );
        }
        let (elem, fixed) = match self.ty.types.kind(self.ty.types.repr(cty)).clone() {
            _ if let Some(elem) = stand_in_elem => (elem, None),
            TypeKind::Array {
                elem,
                kind: ArrayKind::Fixed(n),
            } => (elem, Some(n)),
            TypeKind::Array {
                elem, ..
            } => (elem, None),
            TypeKind::String => (TypeId::U8, None),
            _ => {
                return err(
                    c.span,
                    format!(
                        "cannot iterate over a value of type {}",
                        self.ty.types.name(cty)
                    ),
                );
            }
        };
        let container = match op {
            p @ Operand::Place {
                ..
            } => p,
            other => {
                let (ty, addr) = self.address_of(f, other, c.span)?;
                Operand::Place {
                    ty,
                    addr,
                }
            }
        };
        let Operand::Place {
            addr: caddr, ..
        } = container
        else {
            unreachable!()
        };
        let esize = self.size_of(elem, span)?;
        let idx = f.b.alloca(8, 8);
        let load_count = |f: &mut FnCtx| match fixed {
            Some(n) => f.b.iconst(Ty::I64, n),
            None => f.b.load(Ty::I64, caddr),
        };
        let start = if for_.reverse {
            let n = load_count(f);
            let one = f.b.iconst(Ty::I64, 1);
            f.b.bin(ir::BinOp::Sub, Ty::I64, n, one)
        } else {
            f.b.iconst(Ty::I64, 0)
        };
        f.b.store(Ty::I64, idx, start);
        let head = f.b.new_block();
        let body_block = f.b.new_block();
        let step = f.b.new_block();
        let exit = f.b.new_block();
        f.b.jump(head);
        f.b.switch_to(head);
        // The bound test, here and again after each step (so a step goes straight to the body
        // or out, without a jump back up). The count is read each time: the body may grow or
        // shrink the array.
        let in_range = |f: &mut FnCtx| {
            let i = f.b.load(Ty::I64, idx);
            if for_.reverse {
                let zero = f.b.iconst(Ty::I64, 0);
                f.b.cmp(CmpOp::SGe, Ty::I64, i, zero)
            } else {
                let n = load_count(f);
                f.b.cmp(CmpOp::SLt, Ty::I64, i, n)
            }
        };
        let c = in_range(f);
        f.b.branch(c, body_block, exit);
        f.b.switch_to(body_block);
        let data = if fixed.is_some() {
            caddr
        } else {
            let p = f.b.ptr_offset(caddr, 8);
            f.b.load(Ty::Ptr, p)
        };
        let i = f.b.load(Ty::I64, idx);
        let s = f.b.iconst(Ty::I64, esize);
        let off = f.b.bin(ir::BinOp::Mul, Ty::I64, i, s);
        let elem_addr = f.b.ptr_add(data, off);
        let (it_ty, it_addr) = if for_.by_pointer {
            let pt = self.ty.types.pointer(elem);
            (pt, self.spill(f, pt, elem_addr, span)?)
        } else {
            // A by-value `it` names the element itself, as in jai: `*it` is the element's
            // address and stays valid after the loop (toml-jai, Photon rely on it).
            (elem, elem_addr)
        };
        self.declare_loop_vars(
            f,
            loop_scope,
            for_.backtick_names,
            [
                (
                    it_name,
                    EntityKind::Local {
                        ty: it_ty,
                        addr: it_addr,
                        depth,
                    },
                ),
                (
                    index_name,
                    EntityKind::Local {
                        ty: TypeId::S64,
                        addr: idx,
                        depth,
                    },
                ),
            ],
            for_,
            span,
        );
        self.ide_note_implicit_loop_names(for_, iterated, [it_ty, TypeId::S64]);
        let remove = if fixed.is_none() {
            LoopRemove::Array(container.clone(), idx, elem, for_.reverse)
        } else {
            let elem_name = self.ty.types.name(elem);
            LoopRemove::Unsupported(format!(
                "remove cannot shrink the fixed-size array `{}`; loop over a resizable array \
                 (`[..] {elem_name}`) or an array view (`[] {elem_name}`) to remove elements",
                self.ty.types.name(cty)
            ))
        };
        let defer_depth = f.mark_defers();
        f.loops.push(LoopFrame {
            label: Some(it_name),
            break_block: exit,
            continue_block: step,
            defer_depth,
            remove,
        });
        let result = self.check_scoped(f, loop_scope, &for_.body);
        f.loops.pop();
        result?;
        f.b.jump(step);
        f.b.switch_to(step);
        let i = f.b.load(Ty::I64, idx);
        let one = f.b.iconst(Ty::I64, 1);
        let next = f.b.bin(
            if for_.reverse {
                ir::BinOp::Sub
            } else {
                ir::BinOp::Add
            },
            Ty::I64,
            i,
            one,
        );
        f.b.store(Ty::I64, idx, next);
        let c = in_range(f);
        f.b.branch(c, body_block, exit);
        f.b.switch_to(exit);
        Ok(())
    }

    /// `for x: collection` over a struct: expand its `for_expansion` macro.
    /// `for_expansion` (or the `for :name` iterator) visible at the loop.
    fn for_expansion_procs(
        &mut self,
        scope: ScopeId,
        for_: &ast::For,
        collection: &Operand,
        span: Span,
    ) -> Result<Vec<ProcId>> {
        let macro_name = for_
            .iterator
            .map_or_else(|| Sym::intern("for_expansion"), |i| i.name);
        let mut ids = self.lookup(scope, macro_name)?;
        if ids.is_empty()
            && for_.iterator.is_none()
            && let Some(home) = self.ty.struct_home_scope(collection.ty())
        {
            ids = self.lookup(home, macro_name)?;
        }
        let mut procs = Vec::new();
        for id in ids {
            if let Resolved::Proc(p) = self.resolve_entity(id)? {
                procs.push(p);
            }
        }
        if let Some(iterator) = &for_.iterator {
            self.ide_note_for_iterator(iterator, &procs);
        }
        if procs.is_empty() {
            return err(
                span,
                format!(
                    "no `{macro_name}` is visible for iterating over {}",
                    self.ty.types.name(collection.ty())
                ),
            );
        }
        Ok(procs)
    }

    /// A macro whose second parameter is the loop body (`(x, body: Code, flags)`).
    fn proc_takes_for_body(&mut self, p: ProcId) -> bool {
        let info = self.proc(p);
        if !info.is_macro {
            return false;
        }
        let lit = info.lit.clone();
        lit.header
            .params
            .get(1)
            .is_some_and(|param| self.param_is_code(p, param))
    }

    /// Expand a for loop through `procs`, called with `leading` arguments
    /// followed by the body as Code and the `For_Flags`.
    fn check_for_expansion(
        &mut self,
        f: &mut FnCtx,
        scope: ScopeId,
        for_: &ast::For,
        (procs, leading): (Vec<ProcId>, Vec<CallArg>),
        (it_name, index_name): (Sym, Sym),
        span: Span,
    ) -> Result<()> {
        // The body becomes a Code value; names declared with backticks land in the loop scope.
        let loop_scope = self.new_block_scope(scope);
        let body_stmt = Rc::new((*for_.body).clone());
        let code = self.add_code(
            Rc::new(ast::CodeBody::Block(ast::Block {
                stmts: vec![(*for_.body).clone()],
                span: for_.body.span,
                checks_off: crate::ast::CheckBits::NONE,
            })),
            loop_scope,
        );
        let flags_value = (for_.by_pointer as i128) | ((for_.reverse as i128) << 1);
        // A program without the bootstrap prelude has no `For_Flags`; its loop macros that
        // take the flags take a plain byte.
        let flags_ty = self
            .preload_type(PreludeType::ForFlags, span)
            .unwrap_or(TypeId::U8);
        let mut args = leading;
        args.extend([
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(Operand::Const {
                    ty: TypeId::CODE,
                    value: Value::Code(code),
                    untyped: false,
                }),
                span,
                scope,
            },
            CallArg {
                name: None,
                spread: false,
                expr: None,
                op: Some(Operand::Const {
                    ty: flags_ty,
                    value: Value::Int(flags_value),
                    untyped: false,
                }),
                span,
                scope,
            },
        ]);
        f.pending_for_body = Some(ForBody {
            call_span: span,
            code,
            body: body_stmt,
            scope: loop_scope,
            it_name,
            index_name,
            name_spans: [for_.it.map(|i| i.span), for_.index.map(|i| i.span)],
            label: Some(it_name),
        });
        let result = self.call_procs(f, loop_scope, &procs, args, None, span);
        f.pending_for_body = None;
        result.map(|_| ())
    }
}
