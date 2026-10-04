//! `#modify` blocks: compile-time code that may rewrite a polymorphic call's bindings.
//!
//! The block runs once per call with every polymorphic type variable (`T`, `R`, ...) in scope
//! as an assignable `Type` variable, initialized to the inferred type or to `void` when the
//! arguments did not determine it. Whatever the block leaves in the variables becomes the
//! binding; returning `false` rejects the call. On a polymorphic struct every parameter
//! (types and values) is assignable, and the final values select the instance.
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
        // Type variables are mutable; other bindings (baked values) are plain constants.
        let mut variables: Vec<(Sym, Value, TypeId)> = Vec::new();
        let mut constants = Vec::new();
        let mut type_vars: Vec<Sym> = super::calls::header_poly_names(header);
        for (name, value, ty) in &bindings {
            if *ty == TypeId::TYPE {
                type_vars.push(*name);
            } else {
                constants.push((*name, value.clone(), *ty));
            }
        }
        for name in type_vars {
            if variables.iter().any(|(n, _, _)| *n == name) {
                continue;
            }
            let initial = match bindings.iter().find(|(n, _, _)| *n == name) {
                Some((_, Value::Type(t), _)) => *t,
                _ => TypeId::VOID,
            };
            variables.push((name, Value::Type(initial), TypeId::TYPE));
        }
        let scope = self.proc(proc).scope;
        let what = format!("'{}'", self.proc(proc).name);
        let results = self.run_modify_block(scope, block, variables, constants, &what, span)?;
        for (name, value, ty) in results {
            // A type variable left `void` stays unbound.
            if matches!(value, Value::Type(TypeId::VOID)) {
                continue;
            }
            bindings.retain(|(n, _, _)| *n != name);
            bindings.push((name, value, ty));
        }
        Ok(bindings)
    }

    /// A polymorphic struct's `#modify`: every parameter is a variable the block may change
    /// (`if N < 8 then N = 8;`); the final values pick the instance.
    pub(super) fn run_struct_modify(
        &mut self,
        def_scope: ScopeId,
        name: Sym,
        block: &ast::Block,
        bindings: Vec<(Sym, Value, TypeId)>,
        span: Span,
    ) -> Result<Vec<(Sym, Value, TypeId)>> {
        let what = format!("'{name}'");
        self.run_modify_block(def_scope, block, bindings, Vec::new(), &what, span)
    }

    /// Run a `#modify` block with `variables` assignable and `constants` in scope, returning
    /// the variables' final values. `what` names the procedure or struct in the rejection error.
    fn run_modify_block(
        &mut self,
        def_scope: ScopeId,
        block: &ast::Block,
        variables: Vec<(Sym, Value, TypeId)>,
        constants: Vec<(Sym, Value, TypeId)>,
        what: &str,
        span: Span,
    ) -> Result<Vec<(Sym, Value, TypeId)>> {
        let module = self.scope(def_scope).module;
        let sig = ir::Sig {
            params: vec![Ty::Ptr],
            returns: vec![Ty::I8],
            conv: ir::Conv::Jai,
            c_varargs: false,
            c_fixed: 0,
            c_abi: None,
        };
        let mut f = FnCtx::new("modify".into(), sig, self.scope_file(def_scope));
        f.compile_time = true;
        f.context = Some(f.b.param(0));
        // `return false, "message"`: the optional message goes to a global (empty by default).
        let message = self.program.add_global(ir::Global {
            name: "modify.message".into(),
            size: 16,
            align: 8,
            init: Vec::new(),
            relocs: Vec::new(),
            read_only: false,
            export: None,
        });
        let message_addr = f.b.global_addr(message);
        f.return_types = vec![TypeId::BOOL, TypeId::STRING];
        f.return_outs = vec![None, Some(message_addr)];
        f.named_results = vec![None, Some(message_addr)];
        let scope = self.new_scope(ScopeKind::Proc, Some(def_scope), module, None);
        let depth = self.scope(scope).proc_depth;
        for (name, value, ty) in constants {
            self.add_const(scope, name, span, value, ty);
        }
        let mut globals = Vec::new();
        for (name, value, ty) in variables {
            let size = self.size_of(ty, span)?.max(8);
            let global = self.program.add_global(ir::Global {
                name: format!("modify.{name}"),
                size,
                align: 8,
                init: Vec::new(),
                relocs: Vec::new(),
                read_only: false,
                export: None,
            });
            let addr = f.b.global_addr(global);
            let val = self.materialize(&mut f, &value, ty, span)?;
            self.store_value(&mut f, ty, addr, val, span)?;
            self.add_entity(
                scope,
                name,
                span,
                EntityKind::Local {
                    ty,
                    addr,
                    depth,
                },
                false,
            );
            globals.push((name, ty, global));
        }

        let inner = self.new_scope(ScopeKind::Block, Some(scope), module, None);
        self.check_block_stmts(&mut f, inner, &block.stmts)?;
        if !f.b.is_terminated() {
            let accept = f.b.iconst(Ty::I8, 1);
            f.b.ret(vec![accept]);
        }
        let results = self.call_thunk(f, span)?;
        if results.first().copied().unwrap_or(0) & 0xff == 0 {
            let addr = self
                .interp
                .global_addr(&self.program, message)
                .map_err(|t| Box::new(Diagnostic::error(span, t.message)))?;
            let text = match self.read_value(addr, TypeId::STRING, span)? {
                Value::String(s) if !s.is_empty() => format!(": {}", String::from_utf8_lossy(&s)),
                _ => String::new(),
            };
            return err(
                span,
                format!("#modify rejected the arguments to {what}{text}"),
            );
        }
        let mut out = Vec::new();
        for (name, ty, global) in globals {
            let addr = self
                .interp
                .global_addr(&self.program, global)
                .map_err(|t| Box::new(Diagnostic::error(span, t.message)))?;
            out.push((name, self.read_value(addr, ty, span)?, ty));
        }
        Ok(out)
    }
}
