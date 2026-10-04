//! `#modify` blocks: compile-time code that may rewrite a polymorphic call's bindings.
//!
//! The block runs once per call with every polymorphic type variable (`T`, `R`, ...) in scope
//! as an assignable `Type` variable, initialized to the inferred type or to `void` when the
//! arguments did not determine it. Whatever the block leaves in the variables becomes the
//! binding; returning `false` rejects the call.
use super::lower::FnCtx;
use super::scope::{EntityKind, ScopeKind};
use super::*;
use crate::ir::Ty;

impl Compiler {
    pub(super) fn run_modify(
        &mut self,
        proc: ProcId,
        header: &ast::ProcHeader,
        block: &ast::Block,
        mut bindings: Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> Result<Vec<(Sym, Value, TypeId)>> {
        let def_scope = self.proc(proc).scope;
        let module = self.scope(def_scope).module;
        let sig = ir::Sig {
            params: vec![Ty::Ptr],
            returns: vec![Ty::I8],
            conv: ir::Conv::Jai,
            c_varargs: false,
            c_abi: None,
        };
        let mut f = FnCtx::new("modify".into(), sig, self.scope_file(def_scope));
        f.compile_time = true;
        f.context = Some(f.b.param(0));
        f.return_types = vec![TypeId::BOOL];
        f.return_outs = vec![None];
        f.named_results = vec![None];
        let scope = self.new_scope(ScopeKind::Proc, Some(def_scope), module, None);
        let depth = self.scope(scope).proc_depth;

        // Type variables are mutable; other bindings (baked values) are plain constants.
        let mut variables: Vec<(Sym, ir::GlobalId)> = Vec::new();
        let mut type_vars: Vec<Sym> = super::calls::header_poly_names(header);
        for (name, value, ty) in &bindings {
            if *ty == TypeId::TYPE {
                type_vars.push(*name);
            } else {
                self.add_const(scope, *name, span, value.clone(), *ty);
            }
        }
        for name in type_vars {
            if variables.iter().any(|(n, _)| *n == name) {
                continue;
            }
            let initial = match bindings.iter().find(|(n, _, _)| *n == name) {
                Some((_, Value::Type(t), _)) => *t,
                _ => TypeId::VOID,
            };
            let global = self.program.add_global(ir::Global {
                name: format!("modify.{name}"),
                size: 8,
                align: 8,
                init: Vec::new(),
                relocs: Vec::new(),
                read_only: false,
                export: None,
            });
            let addr = f.b.global_addr(global);
            let val = self.materialize(&mut f, &Value::Type(initial), TypeId::TYPE, span)?;
            self.store_value(&mut f, TypeId::TYPE, addr, val, span)?;
            self.add_entity(
                scope,
                name,
                span,
                EntityKind::Local {
                    ty: TypeId::TYPE,
                    addr,
                    depth,
                },
                false,
            );
            variables.push((name, global));
        }

        let inner = self.new_scope(ScopeKind::Block, Some(scope), module, None);
        self.check_block_stmts(&mut f, inner, &block.stmts)?;
        if !f.b.is_terminated() {
            let accept = f.b.iconst(Ty::I8, 1);
            f.b.ret(vec![accept]);
        }
        let results = self.call_thunk(f, span)?;
        if results.first().copied().unwrap_or(0) & 0xff == 0 {
            return err(
                span,
                format!(
                    "#modify rejected the arguments to '{}'",
                    self.proc(proc).name
                ),
            );
        }
        for (name, global) in variables {
            let addr = self
                .interp
                .global_addr(&self.program, global)
                .map_err(|t| Box::new(Diagnostic::error(span, t.message)))?;
            let Value::Type(t) = self.read_value(addr, TypeId::TYPE, span)? else {
                continue;
            };
            if t == TypeId::VOID {
                continue;
            }
            bindings.retain(|(n, _, _)| *n != name);
            bindings.push((name, Value::Type(t), TypeId::TYPE));
        }
        Ok(bindings)
    }
}
