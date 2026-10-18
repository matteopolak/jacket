//! IR functions for procedures: entry-point wrappers, naming and library resolution.
use super::model::{BodyState, ProcTarget};
use crate::ir::Ty;
use crate::sema::lower::FnCtx;
use crate::sema::*;

impl Compiler {
    /// The callable target of a procedure, reserving it (and queueing its body) on first use.
    pub fn proc_func(&mut self, id: ProcId, span: Span) -> Result<ProcTarget> {
        if let Some(t) = self.proc(id).target {
            return Ok(t);
        }
        if self.proc(id).is_macro {
            return err(
                span,
                format!(
                    "macro `{}` cannot be used as a procedure value",
                    self.proc(id).name
                ),
            );
        }
        let sig = self.signature(id, span)?;
        let ir_sig = self.ir_sig(sig.ty, span)?;
        let header = self.proc(id).lit.header.clone();
        let name = self.proc(id).name;
        // `#entry_point` declarations stand for the program's `main`.
        if lit_is_entry_point(&header) && self.proc(id).lit.body.is_none() {
            let func = self.entry_point_wrapper(id, span)?;
            let target = ProcTarget::Func(func);
            self.set_proc_target(id, target);
            return Ok(target);
        }
        let target = if let Some(foreign) = &header.foreign {
            let scope = self.proc(id).scope;
            let library = match &foreign.library {
                Some(lib_ident) => self.resolve_library(scope, lib_ident)?,
                None => None,
            };
            let symbol = match &foreign.name {
                ast::ForeignName::Named(s) => s.to_string(),
                ast::ForeignName::Default => name.to_string(),
            };
            ProcTarget::Foreign(self.program.add_foreign(ir::Foreign {
                symbol,
                library,
                sig: ir_sig,
                is_data: false,
            }))
        } else if let Some(llvm) =
            llvm_intrinsic(&header).filter(|_| self.proc(id).lit.body.is_none())
        {
            // `#intrinsic "llvm.x"`: a call to the LLVM intrinsic itself. It exists only in
            // native code; the interpreter cannot resolve the symbol.
            ProcTarget::Foreign(self.program.add_foreign(ir::Foreign {
                symbol: llvm.to_string(),
                library: None,
                sig: ir_sig,
                is_data: false,
            }))
        } else if self.proc(id).lit.body.is_none() {
            if header.flags.intrinsic || header.flags.compiler {
                // Intrinsics are expanded at call sites; a reference as a value gets a wrapper later.
                let func = self.program.reserve_func(name.to_string());
                self.procs[id.0 as usize].body_state = BodyState::Queued;
                crate::sema::body_probe::enqueued(id.0);
                self.body_queue.push(id);
                ProcTarget::Func(func)
            } else {
                // `#elsewhere` or a declaration without body: resolve as a process symbol.
                ProcTarget::Foreign(self.program.add_foreign(ir::Foreign {
                    symbol: name.to_string(),
                    library: None,
                    sig: ir_sig,
                    is_data: false,
                }))
            }
        } else {
            let display = self.proc_display_name(id);
            let func = self.program.reserve_func_with_sig(display, ir_sig);
            self.procs[id.0 as usize].body_state = BodyState::Queued;
            crate::sema::body_probe::enqueued(id.0);
            self.body_queue.push(id);
            ProcTarget::Func(func)
        };
        self.set_proc_target(id, target);
        if self.runtime_interpreter_assertion == Some(id) {
            let ProcTarget::Func(func) = target else {
                return err(
                    span,
                    "private interpreter assertion handler must be defined in Jai",
                );
            };
            let hook = crate::interp::Hook::AssertionFailed {
                context: sig.has_context,
                location: self.location_layout(span)?,
            };
            self.interp
                .hooks
                .resize(self.interp.hooks.len().max(func.0 as usize + 1), None);
            self.interp.hooks[func.0 as usize] = Some(hook);
        }
        if let ProcTarget::Func(func) = target {
            self.func_procs.insert(func, id);
            if header.flags.compiler {
                let hook = match name.as_str() {
                    "write_string" => Some(crate::interp::Hook::WriteString),
                    "write_strings" => Some(crate::interp::Hook::WriteStrings),
                    "compile_time_debug_break" => Some(crate::interp::Hook::DebugBreak),
                    "compile_time_host_monotonic_nanoseconds" => {
                        Some(crate::interp::Hook::HostMonotonicNanos)
                    }
                    other => crate::build::MetaOp::from_name(other)
                        .map(|op| crate::interp::Hook::Meta(op, !header.flags.no_context)),
                };
                if let Some(hook) = hook {
                    if matches!(
                        hook,
                        crate::interp::Hook::Meta(crate::build::MetaOp::TypeTable, _)
                    ) {
                        self.type_table_func = Some(func);
                        if let Some(workspaces) = &self.interp.workspaces {
                            crate::build::Workspaces::note_type_table_used(workspaces);
                        }
                    }
                    let i = func.0 as usize;
                    if self.interp.hooks.len() <= i {
                        self.interp.hooks.resize(i + 1, None);
                    }
                    self.interp.hooks[i] = Some(hook);
                }
            }
        }
        Ok(target)
    }

    /// Body of an `#entry_point` declaration: call the program's `main` and
    /// convert its result (if any) to the declared return type.
    fn entry_point_wrapper(&mut self, id: ProcId, span: Span) -> Result<ir::FuncId> {
        let sig = self.signature(id, span)?;
        let ir_sig = self.ir_sig(sig.ty, span)?;
        let func = self.program.reserve_func("__entry_point".into());
        self.set_proc_target(id, ProcTarget::Func(func));
        let main = match self.program_main(span) {
            Ok(main) => main,
            // The editor checks every body in the project's files, also a vendored
            // `Runtime_Support` calling the entry point of a program that has no `main` (a
            // build metaprogram). No build reaches that call: it stops the program.
            Err(_) if self.ide.is_some() => {
                let mut f = FnCtx::new("__entry_point".into(), ir_sig.clone(), FileId(0));
                f.b.intrinsic(ir::Intrinsic::Trap, Vec::new(), &[]);
                let rets = ir_sig.returns.iter().map(|&t| f.b.iconst(t, 0)).collect();
                f.b.ret(rets);
                self.program.funcs[func.0 as usize] = Some(f.b.finish());
                return Ok(func);
            }
            Err(e) => return Err(e),
        };
        let main_sig = self.signature(main, span)?;
        if !main_sig.params.is_empty() {
            let name = &self.options.entry_point;
            return err(
                self.proc(main).span,
                format!("`{name}` must not take parameters"),
            );
        }
        let ProcTarget::Func(main_func) = self.proc_func(main, span)? else {
            return err(
                span,
                format!("`{}` must have a body", self.options.entry_point),
            );
        };
        let main_ir = self.ir_sig(main_sig.ty, span)?;
        let mut f = FnCtx::new("__entry_point".into(), ir_sig, FileId(0));
        let mut args = Vec::new();
        if main_sig.has_context {
            args.push(if sig.has_context {
                f.b.param(0)
            } else {
                f.b.iconst(Ty::Ptr, 0)
            });
        }
        let results =
            f.b.call(ir::Callee::Func(main_func), args, &main_ir.returns);
        let mut rets = Vec::new();
        if let Some(&rt) = sig.returns.first() {
            let want = self.ir_ty(rt).unwrap_or(Ty::I32);
            let v = match (results.first(), main_sig.returns.first()) {
                (Some(&r), Some(&mt)) if self.types.is_integer(mt) => {
                    let have = f.b.val_ty(r);
                    if have.size() > want.size() {
                        f.b.conv(ir::ConvOp::Trunc, have, want, r)
                    } else if have.size() < want.size() {
                        let signed = self.types.int_info(mt).is_some_and(|i| i.1);
                        f.b.conv(
                            if signed {
                                ir::ConvOp::SExt
                            } else {
                                ir::ConvOp::ZExt
                            },
                            have,
                            want,
                            r,
                        )
                    } else {
                        r
                    }
                }
                _ => f.b.iconst(want, 0),
            };
            rets.push(v);
        }
        f.b.ret(rets);
        self.program.funcs[func.0 as usize] = Some(f.b.finish());
        Ok(func)
    }

    fn proc_display_name(&self, id: ProcId) -> String {
        let p = self.proc(id);
        if p.bindings.is_some() {
            format!("{}#{}", p.name, id.0)
        } else {
            p.name.to_string()
        }
    }

    pub(in crate::sema) fn resolve_library(
        &mut self,
        scope: ScopeId,
        ident: &ast::Ident,
    ) -> Result<Option<usize>> {
        let ids = self.lookup(scope, ident.name)?;
        let Some(&e) = ids.first() else {
            let name = ident.name;
            return Err(Box::new(
                Diagnostic::error(ident.span, format!("unknown library `{name}`"))
                    .with_kind(DiagnosticKind::UnknownLibrary)
                    .with_label("no library of this name is declared here")
                    .with_help(format!(
                        "declare it with `{name} :: #library \"path/to/lib{name}\";` (or `#system_library`); a declaration under `#if OS == ...` exists only for that OS"
                    )),
            ));
        };
        match self.resolve_entity(e)? {
            scope::Resolved::Library(l) => Ok(Some(self.libraries[l.0 as usize].ir)),
            _ => Err(Box::new(
                Diagnostic::error(ident.span, format!("`{}` is not a library", ident.name))
                    .with_help(
                        "`#foreign` names a library declared with `name :: #library \"...\";`",
                    ),
            )),
        }
    }
}

fn lit_is_entry_point(header: &ast::ProcHeader) -> bool {
    header
        .flags
        .other
        .iter()
        .any(|f| f.name.as_str() == "entry_point")
}

/// The LLVM intrinsic an `#intrinsic "llvm.name"` declaration binds to.
pub(in crate::sema) fn llvm_intrinsic(header: &ast::ProcHeader) -> Option<&str> {
    let name = header.flags.builtin_name.as_deref()?;
    (header.flags.intrinsic && name.starts_with("llvm.")).then_some(name)
}
