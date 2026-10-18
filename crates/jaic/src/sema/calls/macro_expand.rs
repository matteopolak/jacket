//! Expanding `#expand` macro calls.

use super::*;

/// What the parameters of one macro expansion share.
#[derive(Clone, Copy)]
struct MacroCall<'a> {
    caller: ScopeId,
    mscope: ScopeId,
    sig: &'a Signature,
    args: &'a [CallArg],
    body: &'a ast::Block,
    header: &'a ast::ProcHeader,
    proc: ProcId,
    span: Span,
}

impl Compiler {
    /// Whether a block's source text may assign to (or take the address of) `name`:
    /// `name =`, `name op=`, `name.field =` or `*name`. Conservative: comments and strings
    /// count too.
    pub(super) fn text_may_write(&self, body: &ast::Block, name: Sym) -> bool {
        let text = &self.sources.get(body.span.file).text;
        let text = &text.as_bytes()[body.span.start as usize..body.span.end as usize];
        // `#asm` operands may write any name they mention.
        if text.windows(4).any(|w| w == b"#asm") {
            return true;
        }
        let name = name.as_str();
        let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
        let skip_ws = |mut i: usize| {
            while i < text.len() && text[i].is_ascii_whitespace() {
                i += 1;
            }
            i
        };
        let assigns = |i: usize| {
            let i = skip_ws(i);
            let rest = &text[i.min(text.len())..];
            match rest {
                [b'=', b'=', ..] => false,
                [b'=', ..] => true,
                [op, b'=', ..] => b"+-*/%|&^".contains(op),
                [b'<' | b'>', b'<' | b'>', b'=', ..] => true,
                _ => false,
            }
        };
        let mut from = 0;
        while let Some(pos) = text[from..]
            .windows(name.len())
            .position(|w| w == name.as_bytes())
        {
            let start = from + pos;
            let end = start + name.len();
            from = end;
            if (start > 0 && word(text[start - 1])) || (end < text.len() && word(text[end])) {
                continue;
            }
            let mut before = start;
            while before > 0 && text[before - 1].is_ascii_whitespace() {
                before -= 1;
            }
            if before > 0 && text[before - 1] == b'*' {
                return true;
            }
            if assigns(end) {
                return true;
            }
            // `name.field = ...`
            let mut i = skip_ws(end);
            if i < text.len() && text[i] == b'.' {
                i = skip_ws(i + 1);
                while i < text.len() && word(text[i]) {
                    i += 1;
                }
                if assigns(i) {
                    return true;
                }
            }
        }
        false
    }

    /// Binds one parameter of a macro in the expansion scope.
    fn bind_macro_param(
        &mut self,
        f: &mut FnCtx,
        call: &MacroCall,
        i: usize,
        slot: &Slot,
        param: procs::ParamInfo,
    ) -> Result<()> {
        let MacroCall {
            caller,
            mscope,
            sig,
            args,
            body,
            header,
            proc,
            span,
        } = *call;
        let Some(name) = param.name else {
            return Ok(());
        };
        if param.discard {
            let e = self.add_const(mscope, name, param.span, Value::Int(0), TypeId::S64);
            self.discard_params.insert(e);
            return Ok(());
        }
        if self.is_register_operand_type(param.ty, param.span) {
            let Slot::Arg(a) = slot else {
                return err(param.span, "register operands need an explicit argument");
            };
            let Some(expr) = &args[*a].expr else {
                return err(param.span, "register operands need a caller register");
            };
            let Some((kind, addr)) = self.asm_reg_named(args[*a].scope, expr)? else {
                return err(args[*a].span, "expected an #asm register operand");
            };
            // A general-purpose or mask register is a 64-bit local, so the parameter is
            // seen as such inside the body; only a vector register carries the marker type.
            let ty = if kind == asm::AsmReg::Vec {
                param.ty
            } else {
                TypeId::U64
            };
            let depth = self.scope(mscope).proc_depth;
            let entity = self.add_entity(
                mscope,
                name,
                param.span,
                EntityKind::Local {
                    ty,
                    addr,
                    depth,
                },
                false,
            );
            self.asm_regs.insert(entity, kind);
            return Ok(());
        }
        if param.ty == TypeId::CODE {
            // Code arguments are passed unevaluated, bound to the caller's scope.
            let code = match slot {
                Slot::Arg(a)
                    if let Some(Operand::Const {
                        value: Value::Code(code),
                        ..
                    }) = &args[*a].op =>
                {
                    *code
                }
                Slot::Arg(a) => {
                    let Some(expr) = args[*a].expr.clone() else {
                        return err(args[*a].span, "Code parameter needs a code argument");
                    };
                    // A name of a `#code` constant passes that code; a name of a
                    // `Code` variable passes its value.
                    let mut runtime_code = false;
                    let named = match &expr.kind {
                        E::Ident(_) | E::Member(..) => {
                            match self.check_expr_no_emit(args[*a].scope, &expr) {
                                Ok(Operand::Const {
                                    value: Value::Code(code),
                                    ..
                                }) => Some(code),
                                Ok(op) if op.ty() == TypeId::CODE => {
                                    runtime_code = true;
                                    None
                                }
                                _ => None,
                            }
                        }
                        _ => None,
                    };
                    match named {
                        Some(code) => code,
                        None if runtime_code => {
                            let v = self.param_value(f, sig, &param, slot, args, span)?;
                            let addr = self.spill(f, param.ty, v, span)?;
                            let depth = self.scope(mscope).proc_depth;
                            self.add_entity(
                                mscope,
                                name,
                                param.span,
                                EntityKind::Local {
                                    ty: param.ty,
                                    addr,
                                    depth,
                                },
                                false,
                            );
                            return Ok(());
                        }
                        None => {
                            let body = match &expr.kind {
                                E::Code(c) => c.clone(),
                                _ => Rc::new(ast::CodeBody::Expr(expr)),
                            };
                            self.add_code(body, caller)
                        }
                    }
                }
                Slot::Default
                    if param
                        .default
                        .as_ref()
                        .is_some_and(|d| matches!(d.kind, E::CallerCode)) =>
                {
                    let d = param.default.as_ref().unwrap();
                    match self.check_expr(f, sig.scope, d, Some(TypeId::CODE))? {
                        Operand::Const {
                            value: Value::Code(code),
                            ..
                        } => code,
                        _ => return err(d.span, "#caller_code did not produce Code"),
                    }
                }
                _ => return err(span, "Code parameter needs an argument"),
            };
            self.add_const(mscope, name, param.span, Value::Code(code), TypeId::CODE);
            return Ok(());
        }
        // A for_expansion's `flags` is a compile-time constant (`#assert(flags == 0)`).
        if let Slot::Arg(a) = slot
            && let Some(Operand::Const {
                value: value @ Value::Int(_),
                ..
            }) = &args[*a].op
            && self.preload_type(PreludeType::ForFlags, span).ok() == Some(param.ty)
        {
            self.add_const(mscope, name, param.span, value.clone(), param.ty);
            return Ok(());
        }
        // `v: $T` given an untyped literal stays a literal inside the macro, so
        // `` `return ERR, v `` adapts `0` to the caller's (e.g. distinct) result type.
        if let Slot::Arg(a) = slot
            && let Some(Operand::Const {
                value,
                untyped: true,
                ..
            }) = &args[*a].op
            && matches!(
                header.params[i].ty.as_ref().map(|t| &t.kind),
                Some(E::PolyVar { .. })
            )
        {
            let e = self.add_const(mscope, name, param.span, value.clone(), param.ty);
            self.entity_mut(e).untyped_const = true;
            return Ok(());
        }
        let v = self.param_value(f, sig, &param, slot, args, span)?;
        let addr = if self.is_memory_type(param.ty) {
            v
        } else {
            self.spill(f, param.ty, v, span)?
        };
        let depth = self.scope(mscope).proc_depth;
        let e = self.add_entity(
            mscope,
            name,
            param.span,
            EntityKind::Local {
                ty: param.ty,
                addr,
                depth,
            },
            false,
        );
        if let Some(info) = &param.callable {
            self.callable_entities.insert(e, info.clone());
        }
        // The argument's value when it is a constant: a literal or constant given, a
        // procedure, or a literal default (`var_name := "var"`).
        let constant = match slot {
            Slot::Arg(a) => match &args[*a].op {
                Some(Operand::Const {
                    value,
                    ty,
                    untyped,
                }) => Some((value.clone(), *ty, *untyped)),
                Some(Operand::Procs(p)) if p.len() == 1 => {
                    Some((Value::Proc(p[0]), param.ty, false))
                }
                // A type argument (`gen(R = type_of(r))`): `#if R.FLAG` in the body reads it.
                Some(Operand::Type(t)) if param.ty == TypeId::TYPE => {
                    Some((Value::Type(*t), TypeId::TYPE, false))
                }
                _ => None,
            },
            Slot::Default => match &header.params[i].default {
                Some(d) if matches!(d.kind, E::Str(_) | E::Int(_) | E::Float(_) | E::Bool(_)) => {
                    let pscope = self.proc(proc).scope;
                    match self.eval_const(pscope, d, Some(param.ty)) {
                        Ok(Operand::Const {
                            value,
                            ty,
                            untyped,
                        }) => Some((value, ty, untyped)),
                        _ => None,
                    }
                }
                _ => None,
            },
            _ => None,
        };
        if let Some((value, ty, untyped)) = constant
            && (ty == param.ty
                || (untyped && matches!(value, Value::Int(_)) && self.types.is_integer(param.ty)))
        {
            self.local_consts.insert(e, (value.clone(), param.ty));
            // A constant argument the body never writes stays a constant.
            if matches!(
                value,
                Value::String(_)
                    | Value::Int(_)
                    | Value::Bool(_)
                    | Value::Float(_, _)
                    | Value::Proc(_)
                    | Value::Type(_)
            ) && !self.text_may_write(body, name)
            {
                self.const_macro_params.insert(e, (value, param.ty));
            }
        }
        if param.using {
            let mut entry = scope::UsingEntry::Place {
                ty: param.ty,
                entity: e,
            };
            if let Some(filter) = &param.using_filter {
                entry = self.filter_using_entry(mscope, entry, filter, param.span)?;
            }
            self.push_using(mscope, entry);
        }
        Ok(())
    }

    pub(super) fn expand_macro(
        &mut self,
        f: &mut FnCtx,
        caller: ScopeId,
        proc: ProcId,
        slots: &[Slot],
        args: Vec<CallArg>,
        span: Span,
    ) -> Result<Operand> {
        let sig = self.signature(proc, span)?;
        let header = self.proc(proc).lit.header.clone();
        let Some(body) = self.proc(proc).lit.body.clone() else {
            return err(span, "macro has no body");
        };
        if let Some(ide) = self.ide.as_mut() {
            ide.expanded_macros.insert(proc);
        }
        if f.macros.len() > 64 {
            return err(span, "macro expansion is too deeply nested");
        }
        let module = self.scope(sig.scope).module;
        let mscope = self.new_scope(ScopeKind::Macro, Some(sig.scope), module, None);
        // Locals of the macro belong to the caller's procedure.
        self.scopes[mscope.0 as usize].proc_depth = self.scope(caller).proc_depth;
        let call = MacroCall {
            caller,
            mscope,
            sig: &sig,
            args: &args,
            body: &body,
            header: &header,
            proc,
            span,
        };
        let mut k = 0;
        for (i, slot) in slots.iter().enumerate() {
            if header.params[i].baked {
                continue;
            }
            let param = sig.params[k].clone();
            k += 1;
            self.bind_macro_param(f, &call, i, slot, param)?;
        }
        if self.ide.is_some() {
            self.ide_note_macro(proc, slots, &args, &body, span);
        }
        let exit = f.b.new_block();
        let mut result_slots = Vec::new();
        for &rt in &sig.returns {
            let size = self.size_of(rt, span)?;
            let align = self.align_of(rt, span)?;
            result_slots.push((rt, f.b.alloca(size.max(1), align)));
        }
        let caller_cleanup_serial = f.current_cleanup_serial();
        let defer_depth = f.mark_defers();
        f.macros.push(MacroFrame {
            caller_scope: caller,
            caller_cleanup_serial,
            exit_block: exit,
            result_slots: result_slots.clone(),
            defer_depth,
            loop_depth: f.loops.len(),
            caller_macro_depth: f.macros.len(),
            for_body: f.pending_for_body.take(),
        });
        // Backticks inside this expansion refer to its caller, not to a running `` `defer ``.
        let saved_backtick = f.backtick_scope.take();
        let result = self.check_block_stmts(f, mscope, &body.stmts);
        f.backtick_scope = saved_backtick;
        let frame = f.macros.pop().unwrap();
        result?;
        self.ide_require_parsed_body(body.span)?;
        if !f.b.is_terminated() {
            self.emit_defers(f, frame.defer_depth.position(), span)?;
            f.b.jump(exit);
        }
        f.truncate_defers(frame.defer_depth.position());
        f.b.switch_to(exit);
        let mut values = Vec::new();
        for (i, (rt, addr)) in result_slots.into_iter().enumerate() {
            let v = match self.ir_ty(rt) {
                Some(t) => f.b.load(t, addr),
                None => addr,
            };
            values.push(
                Operand::Value {
                    ty: rt,
                    val: v,
                }
                .with_callable(sig.return_callables.get(i).cloned().flatten()),
            );
        }
        Ok(match values.len() {
            0 => Operand::Void,
            1 => values.remove(0),
            _ => Operand::Multi(values),
        })
    }
}
