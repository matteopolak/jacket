//! Lowering a procedure body, intrinsic wrappers and instantiating polymorphic procedures.
use super::model::{BodyState, ProcInfo, ProcTarget, Signature};
use crate::ir::Ty;
use crate::sema::lower::{FnCtx, Operand};
use crate::sema::scope::{EntityKind, ScopeKind, UsingEntry};
use crate::sema::{
    Compiler, Diagnostic, HashMap, InstanceKeyValue, ProcId, Rc, Result, Span, Sym, TypeId, Value,
    ast, err, ir,
};

impl Compiler {
    /// Lower one procedure body to IR.
    pub fn lower_body(&mut self, id: ProcId) -> Result<()> {
        // IDE-final checks can enter through a compile-time dependency preflight,
        // not just the lenient queue. Share its failure memo at this common entry.
        if let Some(error) = self.ide_cached_body_failure(id, self.lower_epoch()) {
            return Err(error);
        }
        let misses = (
            self.epochs.placeholder_misses,
            self.epochs.in_progress_misses,
        );
        let result = self.lower_body_uncached(id);
        self.ide_record_body_result(id, self.lower_epoch(), misses, &result);
        result
    }

    fn lower_body_uncached(&mut self, id: ProcId) -> Result<()> {
        let probing = crate::sema::body_probe::enabled();
        if probing {
            crate::sema::body_probe::begin(self, id.0);
        }
        let error_mark = self.ed.error_count();
        let executions = self.ct.executions;
        self.diag.dependencies.begin_body(id);
        self.meta.export.expression_types.remove(&id);
        self.pr.procs[id.0 as usize].body_state = BodyState::Lowering;
        let outermost = (self.pr.lowering_depth == 0).then(|| self.meta.perf.start_lowering());
        let checking = crate::progress::enter(crate::progress::Phase::Checking);
        self.pr.lowering_depth += 1;
        let result = self.lower_body_inner(id);
        self.diag.dependencies.end_body(id, result.is_err());
        if result.is_err()
            && !self.epochs.placeholders_final
            && self.ct.executions == executions
            && !self.unresolved_body_placeholders(id).is_empty()
        {
            // Diagnostics of an unfinished attempt are provisional. A complete
            // retry owns the final diagnostics, including independent errors.
            self.ed.forget_errors(error_mark);
        }
        self.pr.lowering_depth -= 1;
        if result.is_err() {
            self.meta.export.expression_types.remove(&id);
        }
        if let Some(mark) = outermost {
            self.meta.perf.finish_lowering(mark);
        }
        drop(checking);
        self.pr.procs[id.0 as usize].body_state = if result.is_ok() {
            crate::progress::procedure_checked();
            self.pr.lenient_failures.remove(&id);
            BodyState::Done
        } else {
            BodyState::Queued
        };
        if probing {
            crate::sema::body_probe::end(self);
        }
        result
    }

    fn lower_body_inner(&mut self, id: ProcId) -> Result<()> {
        let Some(ProcTarget::Func(func_id)) = self.proc(id).target else {
            return Ok(());
        };
        self.lower_body_code(id, Some(func_id))
    }

    /// Lower a body into `func_id`, or only typecheck it (`None`: the code is dropped).
    fn lower_body_code(&mut self, id: ProcId, func_id: Option<ir::FuncId>) -> Result<()> {
        // Body checking is not a constant's value dependency. Actual emitted
        // body uses are recorded in its FnCtx; #assert/type-only scratch uses
        // must not leak into a surrounding callback/global initializer.
        let captures = std::mem::take(&mut self.out.runtime_storage_dependencies.captures);
        let result = self.lower_body_code_with_runtime_dependencies(id, func_id);
        self.out.runtime_storage_dependencies.captures = captures;
        result
    }

    fn lower_body_code_with_runtime_dependencies(
        &mut self,
        id: ProcId,
        func_id: Option<ir::FuncId>,
    ) -> Result<()> {
        let span = self.proc(id).span;
        let sig = self.signature(id, span)?;
        let lit = self.proc(id).lit.clone();
        let header = lit.header.clone();
        let ir_sig = self.ir_sig(sig.ty, span)?;
        let file = self.scope_file(sig.scope);
        let name = match func_id {
            Some(func_id) => self.out.program.func_names[func_id.0 as usize].clone(),
            None => "typecheck".into(),
        };
        let mut f = FnCtx::new(name, ir_sig, file);
        f.proc = Some(id);
        f.checks_off = header.flags.checks_off();
        if !self.options.array_bounds_check {
            f.checks_off = f.checks_off.with(ast::CheckBits::NO_ABC);
        }
        f.b.func.inline = match header.flags.inline {
            ast::CallHintFlag::Inline => ir::InlineHint::Always,
            ast::CallHintFlag::NoInline => ir::InlineHint::Never,
            ast::CallHintFlag::None => ir::InlineHint::Default,
        };
        if func_id.is_some() && self.options.null_pointer_check && !header.flags.no_abc() {
            f.b.func.null_checks = true;
            self.note_check_handler(span)?;
        }
        if let Some(name) = &self.proc(id).export {
            f.b.func.linkage = ir::Linkage::Export(name.clone());
        }
        f.b.func.source_file = file.0;
        if func_id.is_some() {
            f.b.func.debug_hidden = header.flags.no_debug();
            self.begin_func_debug(&mut f, id, span);
        }
        if sig.has_context
            && func_id.is_some()
            && header.flags.inline != ast::CallHintFlag::Inline
            && !header.flags.no_debug()
        {
            let (line, col) = self.src.sources.get(span.file).line_col(span.start);
            f.b.func.trace = Some(ir::TraceInfo {
                name: self.proc(id).name.to_string(),
                file: span.file.0,
                line,
                col,
            });
        }
        let module = self.scope(sig.scope).module;
        let scope = self.new_scope(ScopeKind::Proc, Some(sig.scope), module, None);
        self.scope_mut(scope).proc = Some(id);
        let Some(body) = &lit.body else {
            // Intrinsic or #compiler without a body: synthesize a forwarding body.
            let Some(func_id) = func_id else {
                return Ok(());
            };
            return self.lower_intrinsic_wrapper(id, func_id, f, &sig, &header);
        };
        let mut next = 0usize;
        if sig.has_context {
            f.set_context(Some(f.b.param(0)));
            f.b.func.trusted.push(f.b.param(0));
            next = 1;
        }
        for param in &sig.params {
            let incoming = f.b.param(next);
            next += 1;
            // An aggregate arrives as the address of the caller's copy, never null.
            if self.is_memory_type(param.ty) {
                f.b.func.trusted.push(incoming);
            }
            if param.discard {
                if let Some(name) = param.name {
                    let e = self.add_const(scope, name, param.span, Value::Int(0), TypeId::S64);
                    self.pr.discard_params.insert(e);
                }
                continue;
            }
            // An aggregate arrives as the address of the caller's value. Parameters are
            // values, so the procedure works on its own copy: `advance(*s, 1)` on a `string`
            // parameter must not move the caller's string.
            let addr = self.spill(&mut f, param.ty, incoming, param.span)?;
            if let Some(name) = param.name {
                self.debug_var(
                    &mut f,
                    scope,
                    name,
                    param.span,
                    (param.ty, addr),
                    next as u32,
                );
                let depth = self.scope(scope).proc_depth;
                let e = self.add_entity(
                    scope,
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
                    self.pr.callable_entities.insert(e, info.clone());
                }
                if param.using {
                    let mut entry = UsingEntry::Place {
                        ty: param.ty,
                        entity: e,
                    };
                    if let Some(filter) = &param.using_filter {
                        entry = self.filter_using_entry(scope, entry, filter, param.span)?;
                    }
                    self.push_using(scope, entry);
                }
            }
        }
        f.return_types.clone_from(&sig.returns);
        for (i, &rt) in sig.returns.iter().enumerate() {
            if self.is_memory_type(rt) {
                // The caller's result slot, never null.
                f.b.func.trusted.push(f.b.param(next));
                f.return_outs.push(Some(f.b.param(next)));
                next += 1;
            } else {
                f.return_outs.push(None);
            }
            // Named results get a slot initialized to their default.
            if let Some(name) = sig.return_names[i] {
                let addr = if let Some(out) = f.return_outs[i] {
                    out
                } else {
                    let size = self.size_of(rt, span)?;
                    let align = self.align_of(rt, span)?;
                    f.b.alloca(size, align)
                };
                let default = header
                    .returns
                    .iter()
                    .filter(|r| r.name.map(|n| n.name) == Some(name))
                    .find_map(|r| r.default.clone());
                match default {
                    Some(d) => {
                        let op = self.check_expr(&mut f, scope, &d, Some(rt))?;
                        let op = self.convert(&mut f, op, rt, d.span)?;
                        let (_, v) = self.rvalue(&mut f, op, d.span)?;
                        self.store_value(&mut f, rt, addr, v, d.span)?;
                    }
                    None => self.init_default(&mut f, rt, addr, span)?,
                }
                // A result named like a parameter is only reachable through `return`.
                if !self.scope(scope).names.contains_key(&name) {
                    self.debug_var(&mut f, scope, name, span, (rt, addr), 0);
                    let depth = self.scope(scope).proc_depth;
                    self.add_entity(
                        scope,
                        name,
                        span,
                        EntityKind::Local {
                            ty: rt,
                            addr,
                            depth,
                        },
                        false,
                    );
                }
                f.named_results.push(Some(addr));
            } else {
                f.named_results.push(None);
            }
        }
        // Parameters and named results live in an outer scope; the body may shadow them.
        let body_scope = self.new_block_scope(scope);
        self.ide_scope_span(body_scope, body.span);
        self.check_block_stmts(&mut f, body_scope, &body.stmts)?;
        self.ide_require_parsed_body(body.span)?;
        // The body is read for what Jai warns about: a result still owed at its end, and
        // statements written after one that leaves the block.
        // Named results are returned at the end (a feature newer than the warning), so a body
        // that names them all is never short of a `return`.
        let reaches_end = self.warn_unreachable(&f, body);
        if f.macro_returned {
            self.ed.macro_returned(id);
        }
        // An editor analysis also drops the warning where the walk shows the end is never
        // reached (`if x >= 0 { return 1; } else if x < 0 { return 2; }` on an unsigned `x`
        // cannot, but the reference compiler would warn, so the batch compiler keeps its rule).
        let owes_result = !f.return_types.is_empty()
            && !f.named_results.iter().all(Option::is_some)
            && !f.macro_returned
            && !self.stmts_return(&f, &body.stmts)
            && (!self.ed.trusts_reachability() || reaches_end);
        if !f.b.is_terminated() {
            self.emit_fallthrough_return(&mut f, body.span)?;
        }
        let storage_constants = f.runtime_storage_constants;
        let func = f.b.finish();
        if owes_result {
            self.warn(
                Diagnostic::warning(header.span, "not all control paths return a value")
                    .with_label("this procedure can reach its end without returning")
                    .with_help(
                        "end the body with a `return`; a switch needs a default `case;` or `#complete`",
                    ),
            );
        }
        if let Some(func_id) = func_id {
            self.out
                .runtime_storage_dependencies
                .functions
                .insert(func_id, (storage_constants, span));
            self.out.program.funcs[func_id.0 as usize] = Some(func);
        }
        Ok(())
    }

    /// Falling off the end of a body: return named results, or nothing.
    pub fn emit_fallthrough_return(&mut self, f: &mut FnCtx, span: Span) -> Result<()> {
        self.emit_defers(f, 0, span)?;
        if f.return_types.is_empty() {
            f.b.ret(Vec::new());
            return Ok(());
        }
        if f.named_results.iter().all(Option::is_some) {
            let mut values = Vec::new();
            for (i, &rt) in f.return_types.clone().iter().enumerate() {
                if f.return_outs[i].is_none() {
                    let t = self.ir_ty(rt).unwrap();
                    let addr = f.named_results[i].unwrap();
                    values.push(f.b.load(t, addr));
                }
            }
            f.b.ret(values);
            return Ok(());
        }
        // Missing return: trap at runtime (Jai reports this only for reachable ends).
        self.emit_trap(f, ir::CheckReason::MissingReturn, span)?;
        f.b.terminate(ir::Term::Unreachable);
        Ok(())
    }

    /// Body for `#intrinsic`/`#compiler` declarations referenced as values.
    fn lower_intrinsic_wrapper(
        &mut self,
        id: ProcId,
        func_id: ir::FuncId,
        mut f: FnCtx,
        sig: &Signature,
        _header: &ast::ProcHeader,
    ) -> Result<()> {
        let name = self.proc(id).name;
        if name.as_str() == crate::sema::private_names::RUNTIME_TEMPORARY_STORAGE_SIZE {
            let Operand::Value {
                val, ..
            } = self.runtime_storage_operand(&mut f, sig, self.proc(id).span)?
            else {
                unreachable!();
            };
            f.b.ret(vec![val]);
            self.out.program.funcs[func_id.0 as usize] = Some(f.b.finish());
            return Ok(());
        }
        let start = usize::from(sig.has_context);
        let args: Vec<ir::Val> = (start..f.b.func.sig.params.len())
            .map(|i| f.b.param(i))
            .collect();
        let returns = f.b.func.sig.returns.clone();
        let op = match name.as_str() {
            "memcpy" => ir::Intrinsic::Memcpy,
            "memset" => ir::Intrinsic::Memset,
            "memcmp" => ir::Intrinsic::Memcmp,
            "debug_break" | "compile_time_debug_break" => ir::Intrinsic::DebugBreak,
            _ => ir::Intrinsic::Trap,
        };
        if op == ir::Intrinsic::Trap {
            // Compile-time-only primitives (`#compiler` hooks) have no runtime body. The
            // interpreter calls the hook itself, so only compiled code reaches this stub: say
            // why it stops, instead of a bare trap.
            let text: Rc<[u8]> = format!(
                "runtime error: `{name}` is a compiler primitive; it runs only at compile time\n"
            )
            .into_bytes()
            .into();
            let data = self.string_global(&text);
            let data = f.b.global_addr(data);
            let count = f.b.iconst(Ty::I64, text.len() as u64);
            let to_stderr = f.b.iconst(Ty::I8, 1);
            f.b.intrinsic(
                ir::Intrinsic::CompilerWrite,
                vec![data, count, to_stderr],
                &[],
            );
            f.b.intrinsic(op, Vec::new(), &[]);
            f.b.terminate(ir::Term::Unreachable);
        } else {
            let results = f.b.intrinsic(op, args, &returns);
            f.b.ret(results);
        }
        self.out.program.funcs[func_id.0 as usize] = Some(f.b.finish());
        Ok(())
    }

    /// Instantiate a polymorphic procedure with bindings (name → value).
    pub fn instantiate(
        &mut self,
        id: ProcId,
        bindings: Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> Result<ProcId> {
        let key: Vec<InstanceKeyValue> = bindings
            .iter()
            .map(|(_, v, t)| crate::sema::instance_key_value(&self.ty.types, v, *t))
            .collect();
        if let Some(&inst) = self.proc(id).instances.get(&key) {
            self.meta.perf.poly_request(id, true, span);
            return Ok(inst);
        }
        self.meta.perf.poly_request(id, false, span);
        let signature_only = self.ide_sketch_refuses(&bindings);
        if self.proc(id).instances.len() >= crate::sema::MAX_INSTANCES {
            return err(span, crate::sema::too_many_instances(self.proc(id).name));
        }
        let parent = self.proc(id).scope;
        let module = self.scope(parent).module;
        let scope = self.new_scope(ScopeKind::Block, Some(parent), module, None);
        let header = self.proc(id).lit.header.clone();
        let binders = self.poly_binders(&header);
        let mut bound = Vec::new();
        for (name, value, ty) in &bindings {
            // Declared at its `$T`, so go to definition from the body lands there.
            let binder = binders.iter().find(|(n, _)| n == name).map(|&(_, s)| s);
            let param = header
                .params
                .iter()
                .find(|p| p.name.is_some_and(|n| n.name == *name));
            let id = match (binder, param) {
                (Some(at), _) => self.add_const(scope, *name, at, value.clone(), *ty),
                // `$T: Type`: its parameter's name.
                (None, Some(p)) if p.baked || p.auto_bake => {
                    let at = p.name.map_or(span, |n| n.span);
                    self.add_const(scope, *name, at, value.clone(), *ty)
                }
                // A runtime parameter typed with a bare polymorphic struct binds its name to
                // the instance for the later parameter types; the parameter itself declares
                // the name the body sees.
                (None, Some(p)) => self.new_const(scope, *name, p.span, value.clone(), *ty),
                (None, None) => self.add_const(scope, *name, span, value.clone(), *ty),
            };
            bound.push((*name, id));
        }
        if self.ed.is_live() {
            self.ide_note_poly_header(&header, &bound, parent);
        }
        let p = self.proc(id);
        let (name, lit, pspan, is_macro) = (p.name, p.lit.clone(), p.span, p.is_macro);
        let inst = ProcId(self.pr.procs.len() as u32);
        self.pr.procs.push(ProcInfo {
            name,
            lit,
            scope: parent,
            span: pspan,
            is_poly: false,
            poly_checked: true,
            is_macro,
            sig: None,
            sig_resolving: false,
            target: None,
            body_state: BodyState::NotNeeded,
            instances: HashMap::default(),
            bindings: Some(scope),
            export: None,
        });
        self.pr.procs[id.0 as usize].instances.insert(key, inst);
        if signature_only {
            self.ed.signature_only(inst);
        }
        self.meta.perf.poly_instance(id, inst, span);
        if let Some(notes) = self.pr.proc_decl_notes.get(&id).cloned() {
            self.pr.proc_decl_notes.insert(inst, notes);
        }
        Ok(inst)
    }
}
