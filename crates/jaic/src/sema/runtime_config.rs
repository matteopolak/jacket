//! Target runtime options are private data, finalized after workspace settlement.
use super::lower::{FnCtx, Operand};
use super::scope::Resolved;
use super::*;
use crate::types::StructId;


/// Where a compiler constant came from, kept outside reflected values/types.
#[derive(Default)]
pub(super) struct RuntimeStorageDependencies {
    pub captures: Vec<Vec<i64>>,
    pub entities: HashMap<scope::EntityId, Vec<i64>>,
    pub layouts: HashMap<StructId, Vec<i64>>,
    pub functions: HashMap<ir::FuncId, (Vec<i64>, Span)>,
}

#[derive(Default)]
struct RuntimeStorageUsage {
    dynamic: bool,
    constants: Vec<(i64, Span)>,
}

fn merge_values(target: &mut Vec<i64>, values: &[i64]) {
    for &value in values {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

impl Compiler {
    pub(super) fn begin_runtime_storage_dependencies(&mut self) {
        self.runtime_storage_dependencies.captures.push(Vec::new());
    }

    pub(super) fn end_runtime_storage_dependencies(&mut self) -> Result<Vec<i64>> {
        self.runtime_storage_dependencies
            .captures
            .pop()
            .ok_or_else(|| {
                Box::new(Diagnostic::error(
                    Span::NONE,
                    "unbalanced runtime configuration dependency capture",
                ))
            })
    }

    pub(super) fn note_runtime_storage_dependencies(&mut self, values: &[i64]) {
        if let Some(capture) = self.runtime_storage_dependencies.captures.last_mut() {
            merge_values(capture, values);
        }
    }

    pub(super) fn finish_runtime_storage_entity(&mut self, id: scope::EntityId, values: &[i64]) {
        let known = self
            .runtime_storage_dependencies
            .entities
            .entry(id)
            .or_default();
        merge_values(known, values);
        let values = known.clone();
        self.note_runtime_storage_dependencies(&values);
    }

    pub(super) fn finish_runtime_storage_layout(&mut self, id: StructId, values: &[i64]) {
        let known = self
            .runtime_storage_dependencies
            .layouts
            .entry(id)
            .or_default();
        merge_values(known, values);
        let values = known.clone();
        self.note_runtime_storage_dependencies(&values);
    }

    pub(super) fn use_runtime_storage_entity(&self, f: &mut FnCtx, id: scope::EntityId) {
        if let Some(values) = self.runtime_storage_dependencies.entities.get(&id) {
            merge_values(&mut f.runtime_storage_constants, values);
        }
    }

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
    fn target_runtime_storage_usage(&mut self) -> Result<RuntimeStorageUsage> {
        let mut usage = RuntimeStorageUsage::default();
        // Keep declaration provenance separate from structurally interned array
        // types: an unrelated same-sized buffer must not inherit an opt-in.
        let mut global_constants: HashMap<ir::GlobalId, Vec<(i64, Span)>> = HashMap::default();
        for (index, entity) in self.entities.iter().enumerate() {
            let scope::EntityState::Done(Resolved::Global {
                storage: ir::Storage::Data(global),
                ..
            }) = &entity.state
            else {
                continue;
            };
            if let Some(values) = self
                .runtime_storage_dependencies
                .entities
                .get(&scope::EntityId(index as u32))
            {
                for &value in values {
                    global_constants
                        .entry(*global)
                        .or_default()
                        .push((value, entity.span));
                }
            }
        }
        // Retained globals execute with their settled interpreter state, not
        // their declaration-time initializer. Use the same canonical snapshot
        // serializer as target emission; it also materializes pointed-to target
        // data when necessary. Do not bake/consume the retained-global registry:
        // later workspace activity may still change the effective target image.
        let retained: HashMap<_, _> = self.no_reset_globals.iter().copied().collect();
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
                if Some(global) == self.runtime_storage_global {
                    usage.dynamic = true;
                }
                if !seen_globals.insert(global) {
                    continue;
                }
                if let Some(values) = global_constants.get(&global) {
                    usage.constants.extend(values.iter().copied());
                }
                let relocations = match retained.get(&global).and_then(|ty| {
                    self.interp
                        .materialized_global(global)
                        .map(|address| (*ty, address))
                }) {
                    Some((ty, address)) => {
                        let Value::Bytes(image) = self.read_aggregate(address, ty, Span::NONE)?
                        else {
                            return err(
                                Span::NONE,
                                "retained runtime configuration dependency has no target image",
                            );
                        };
                        image.relocs.clone()
                    }
                    None => self.program.globals[global.0 as usize].relocs.clone(),
                };
                for relocation in relocations {
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
                if let Some((values, span)) = self
                    .runtime_storage_dependencies
                    .functions
                    .get(&ir::FuncId(index as u32))
                {
                    usage
                        .constants
                        .extend(values.iter().map(|&value| (value, *span)));
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
        Ok(usage)
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
        let target_runtime =
            has_runtime && self.options.runtime_support && self.options.runtime_initialization;
        let usage = if target_runtime {
            self.target_runtime_storage_usage()?
        } else {
            RuntimeStorageUsage::default()
        };
        if let Some(&(captured, at)) = usage
            .constants
            .iter()
            .find(|(captured, _)| *captured != size)
        {
            return Err(Box::new(Diagnostic::error(at, format!(
                "target TEMPORARY_STORAGE_SIZE was bound to {captured}, but the final workspace temporary_storage_size is {size}"
            )).with_help("set the workspace size before resolving target declarations that consume this compiler constant; static target storage cannot be resized by a late setting")));
        }
        if target_runtime
            && size != DEFAULT_TEMPORARY_STORAGE_SIZE
            && !usage.dynamic
            && usage.constants.is_empty()
        {
            let exports = self
                .program
                .funcs
                .iter()
                .flatten()
                .filter(|function| matches!(function.linkage, ir::Linkage::Export(_)))
                .count();
            return Err(Box::new(Diagnostic::error(
                Span::NONE,
                "nondefault temporary_storage_size requires the target runtime to consume TEMPORARY_STORAGE_SIZE or the private runtime configuration getter",
            ).with_note(Span::NONE, format!(
                "workspace {}; temporary storage {} bytes; system entry {}; initialization {}; exported target procedures {}; private configuration reader lowered {}",
                self.workspace, size, self.options.runtime_entry_point,
                self.options.runtime_initialization, exports,
                self.runtime_storage_global.is_some(),
            ))));
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
