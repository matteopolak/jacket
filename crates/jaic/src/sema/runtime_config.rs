//! Target runtime options are private data, finalized after workspace settlement.
use super::lower::{FnCtx, Operand};
use super::scope::Resolved;
use super::*;

impl Compiler {
    /// Compiler-owned transport extensions live in the selected runtime's private
    /// file scopes. They are optional; application imports retain normal visibility.
    pub(super) fn private_runtime_procedure(
        &mut self,
        name: &str,
        span: Span,
    ) -> Result<Option<ProcId>> {
        let Some(runtime) = self.runtime_support else {
            return Ok(None);
        };
        let name = Sym::intern(name);
        let mut found = Vec::new();
        for file in &self.files {
            if file.module == runtime
                && let Some(ids) = self.scope(file.scope).names.get(&name)
            {
                found.extend(ids.iter().copied());
            }
        }
        let entity = match found.as_slice() {
            [] => return Ok(None),
            [entity] => *entity,
            _ => {
                return err(
                    span,
                    format!("runtime defines multiple private extensions `{name}`"),
                );
            }
        };
        self.ide_note_use(entity);
        let Resolved::Proc(proc) = self.resolve_entity(entity)? else {
            return err(
                span,
                format!("private runtime extension `{name}` must be a procedure"),
            );
        };
        Ok(Some(proc))
    }

    pub(super) fn prepare_interpreter_assertion(&mut self, span: Span) -> Result<()> {
        self.runtime_interpreter_assertion = None;
        let Some(proc) = self.private_runtime_procedure("__jaic_interpreter_assertion", span)?
        else {
            return Ok(());
        };
        let signature = self.signature(proc, span)?;
        let location = self.preload_type("Source_Code_Location", span)?;
        if !signature.has_context
            || signature.c_call
            || signature.c_varargs
            || signature.returns != [TypeId::BOOL]
            || signature.params.len() != 2
            || signature.params.iter().any(|p| p.variadic || p.discard)
            || signature.params[0].ty != location
            || signature.params[1].ty != TypeId::STRING
        {
            return err(
                span,
                "private interpreter assertion handler requires (Source_Code_Location, string) -> bool with Jai context",
            );
        }
        let header = &self.proc(proc).lit.header;
        if self.proc(proc).lit.body.is_none()
            || header.flags.compiler
            || header.flags.intrinsic
            || self.proc(proc).is_macro
        {
            return err(
                span,
                "private interpreter assertion handler must have an ordinary Jai body for native execution",
            );
        }
        self.runtime_interpreter_assertion = Some(proc);
        Ok(())
    }

    fn runtime_storage_settings(&self) -> (bool, i64) {
        self.interp
            .workspaces
            .as_ref()
            .and_then(|workspaces| {
                crate::build::runtime_storage_setting(
                    workspaces,
                    self.workspace,
                    self.options.temporary_storage_size,
                )
            })
            .unwrap_or((true, self.options.temporary_storage_size))
    }

    pub(super) fn runtime_storage_operand(
        &mut self,
        f: &mut FnCtx,
        sig: &procs::Signature,
        span: Span,
    ) -> Result<Operand> {
        if sig.has_context || !sig.params.is_empty() || sig.returns != [TypeId::S64] {
            return err(
                span,
                "private runtime size intrinsic requires signature () -> s64 without context",
            );
        }
        let global = match self.runtime_storage_global {
            Some(global) => global,
            None => {
                let (_, size) = self.runtime_storage_settings();
                let global = self.program.add_global(ir::Global {
                    name: "__jaic_runtime_configuration.temporary_storage_size".into(),
                    size: 8,
                    align: 8,
                    init: size.to_le_bytes().to_vec(),
                    relocs: Vec::new(),
                    read_only: true,
                    export: None,
                });
                self.runtime_storage_global = Some(global);
                global
            }
        };
        let address = f.b.global_addr(global);
        Ok(Operand::Value {
            ty: TypeId::S64,
            val: f.b.load(ir::Ty::I64, address),
        })
    }

    /// Compile-time inspection or an unused declaration must not opt an older
    /// runtime into a configuration protocol its target code never reads.
    fn target_reads_runtime_storage(&self, target: ir::GlobalId) -> bool {
        let mut functions: Vec<usize> = self
            .program
            .funcs
            .iter()
            .enumerate()
            .filter_map(|(i, f)| {
                f.as_ref()
                    .filter(|f| matches!(f.linkage, ir::Linkage::Export(_)))
                    .map(|_| i)
            })
            .collect();
        let mut globals: Vec<ir::GlobalId> = self
            .program
            .globals
            .iter()
            .enumerate()
            .filter_map(|(i, g)| g.export.as_ref().map(|_| ir::GlobalId(i as u32)))
            .collect();
        let mut seen_functions = HashSet::default();
        let mut seen_globals = HashSet::default();
        while !functions.is_empty() || !globals.is_empty() {
            while let Some(global) = globals.pop() {
                if global == target {
                    return true;
                }
                if !seen_globals.insert(global) {
                    continue;
                }
                for relocation in &self.program.globals[global.0 as usize].relocs {
                    match relocation.target {
                        ir::RelocTarget::Func(func) => functions.push(func.0 as usize),
                        ir::RelocTarget::Global(global) => globals.push(global),
                        ir::RelocTarget::Foreign(_) => {}
                    }
                }
            }
            while let Some(index) = functions.pop() {
                if !seen_functions.insert(index) {
                    continue;
                }
                let Some(function) = &self.program.funcs[index] else {
                    continue;
                };
                for instruction in function.blocks.iter().flat_map(|block| &block.insts) {
                    match instruction {
                        ir::Inst::Call(call) => {
                            if let ir::Callee::Func(func) = call.callee {
                                functions.push(func.0 as usize);
                            }
                        }
                        ir::Inst::FuncAddr {
                            func, ..
                        } => functions.push(func.0 as usize),
                        ir::Inst::GlobalAddr {
                            global, ..
                        } => globals.push(*global),
                        _ => {}
                    }
                }
            }
        }
        false
    }

    /// Refresh private runtime data after workspace activity, immediately before
    /// target execution or emission. This Rust driver API adds no Jai export.
    pub fn finalize_runtime_configuration(&mut self) -> Result<()> {
        let (has_runtime, size) = self.runtime_storage_settings();
        if has_runtime && !(0..=i32::MAX as i64).contains(&size) {
            return err(
                Span::NONE,
                "temporary_storage_size must be nonnegative and fit s32",
            );
        }
        if has_runtime
            && self.options.runtime_support
            && self.options.runtime_initialization
            && size != DEFAULT_TEMPORARY_STORAGE_SIZE
            && !self
                .runtime_storage_global
                .is_some_and(|global| self.target_reads_runtime_storage(global))
        {
            return err(
                Span::NONE,
                "nondefault temporary_storage_size requires the target runtime to read private compiler runtime configuration",
            );
        }
        if let Some(global) = self.runtime_storage_global {
            self.program.globals[global.0 as usize].init = size.to_le_bytes().to_vec();
            self.interp
                .refresh_global(&self.program, global)
                .map_err(|trap| Box::new(Diagnostic::error(Span::NONE, trap.into_message())))?;
        }
        // Native target startup reads the private data; interpreter entry
        // context allocation uses the same finalized workspace setting.
        self.options.temporary_storage_size = size;
        Ok(())
    }
}
