//! Isolated host-workspace evaluation and explicit primitive result transfer.
use super::lower::Operand;
use super::*;
use crate::types::TypeKind;

impl Compiler {
    /// Evaluate the original AST in a separate compiler, with a fresh bootstrap,
    /// module cache, context and type table. Source ids are shared only as source
    /// locations; no target scopes/entities are installed in that compiler.
    pub(super) fn check_isolated_host_run_all(
        &mut self,
        body: &ast::RunBody,
        span: Span,
    ) -> Result<Vec<Operand>> {
        let mut options = self.options.clone();
        (options.os, options.cpu) = compiler_host();
        options.long_double = Options::host().long_double;
        let mut host = Compiler::new(options.clone(), self.fs.clone());
        host.sources = self.sources.clone();
        let source_count = self.sources.len();
        if let Some(parent) = &self.interp.workspaces {
            let (shared, provider) =
                crate::build::isolated_host_environment(parent, options, self.fs.clone());
            host.interp = crate::interp::Interp::new(provider);
            host.interp.workspaces = Some(shared);
        } else {
            let provider = self.interp.host.isolated_host().ok_or_else(|| {
                Box::new(Diagnostic::error(
                    span,
                    "this compiler host does not provide isolated #run,host execution",
                ))
            })?;
            host.interp = crate::interp::Interp::new(provider);
        }
        host.interp.block_budget = self.interp.block_budget;
        let result = (|| {
            host.begin_sources(&[])?;
            let module = host.main_module.unwrap();
            let module_scope = host.modules[module.0 as usize].scope;
            let scope = host.new_scope(
                scope::ScopeKind::File,
                Some(module_scope),
                module,
                Some(span.file),
            );
            if let ast::RunBody::Expr(ast::Expr {
                kind:
                    ast::ExprKind::Call {
                        callee,
                        args,
                        ..
                    },
                ..
            }) = body
                && (!args.is_empty()
                    || matches!(&callee.kind, ast::ExprKind::Proc(procedure) if !procedure.header.params.is_empty()))
            {
                return err(
                    span,
                    "an isolated #run,host procedure must take no arguments",
                );
            }
            let result = host.check_run_inner_all(scope, body, None, span)?;
            if let Some(shared) = host.interp.workspaces.clone() {
                crate::build::finish_all(&shared, &mut host.interp.block_budget)
                    .map_err(|message| Box::new(Diagnostic::error(span, message)))?;
                if shared.borrow().any_failed() {
                    return err(
                        span,
                        "isolated #run,host workspace reported a compilation error",
                    );
                }
            }
            Ok(result)
        })();
        self.interp.block_budget = host.interp.block_budget;
        // Dependency retries must observe real execution in the isolated
        // compiler too. Otherwise an effectful host run followed by a missing
        // target binding is incorrectly classified as an unstarted attempt.
        self.ct_executions += host.ct_executions;
        self.ct_traps += host.ct_traps;
        // Errors from newly loaded host files retain real source locations.
        for index in source_count..host.sources.len() {
            let file = host.sources.get(FileId(index as u32));
            self.sources.add(file.path.clone(), file.text.clone());
        }
        let mut globals = HashMap::default();
        result?
            .into_iter()
            .map(|op| {
                let Operand::Const {
                    ty,
                    value,
                    untyped,
                } = op
                else {
                    return err(
                        span,
                        "#run,host must return a built-in primitive, string or primitive array",
                    );
                };
                let ty = self.import_host_result_type(&host, ty, span)?;
                let value = match value {
                    Value::Bytes(image) => {
                        let mut image = (*image).clone();
                        for relocation in &mut image.relocs {
                            relocation.target = self.import_host_data(
                                &host,
                                relocation.target,
                                &mut globals,
                                span,
                            )?;
                        }
                        Value::Bytes(Rc::new(image))
                    }
                    Value::Int(_) | Value::Float(..) | Value::Bool(_) | Value::String(_) => value,
                    _ => {
                        return err(
                            span,
                            "#run,host cannot transfer this result across workspaces",
                        );
                    }
                };
                Ok(Operand::Const {
                    ty,
                    value,
                    untyped,
                })
            })
            .collect()
    }

    fn import_host_result_type(
        &mut self,
        host: &Compiler,
        ty: TypeId,
        span: Span,
    ) -> Result<TypeId> {
        let kind = host.types.kind(ty).clone();
        match kind {
            TypeKind::Bool
            | TypeKind::Int {
                ..
            }
            | TypeKind::Float {
                ..
            }
            | TypeKind::String => Ok(self.types.intern(kind)),
            TypeKind::Array {
                elem,
                kind,
            } if matches!(
                host.types.kind(elem),
                TypeKind::Bool | TypeKind::Int { .. } | TypeKind::Float { .. }
            ) =>
            {
                let elem = self.import_host_result_type(host, elem, span)?;
                Ok(self.types.array(elem, kind))
            }
            _ => err(
                span,
                "#run,host must return a built-in primitive, string or primitive array",
            ),
        }
    }

    fn import_host_data(
        &mut self,
        host: &Compiler,
        target: ir::RelocTarget,
        globals: &mut HashMap<ir::GlobalId, ir::GlobalId>,
        span: Span,
    ) -> Result<ir::RelocTarget> {
        let ir::RelocTarget::Global(old) = target else {
            return err(
                span,
                "#run,host cannot transfer a procedure or foreign storage address",
            );
        };
        if let Some(&copied) = globals.get(&old) {
            return Ok(ir::RelocTarget::Global(copied));
        }
        let mut global = host.program.globals[old.0 as usize].clone();
        global.name = "host.run.data".into();
        global.export = None;
        let copied = self.program.add_global(global.clone());
        globals.insert(old, copied);
        for relocation in &mut global.relocs {
            relocation.target = self.import_host_data(host, relocation.target, globals, span)?;
        }
        self.program.globals[copied.0 as usize] = global;
        Ok(ir::RelocTarget::Global(copied))
    }
}
