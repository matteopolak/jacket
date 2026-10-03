//! Debug traversal follows validated executable-IR paths, never AST counts.
use super::{Error, LineTables, Scope, VariableKind, VariableRecord};
use crate::{Generator, Slot};
use inkwell::{builder::Builder, context::Context};
use jai_ir::{
    BlockPath, BlockRoot, DebugBranch, DebugPathStep, DebugSources, LocalDeclaration, LocalId,
    ProcedureId, StatementPath,
};
use jai_types::Types;
use std::{
    collections::{HashMap, HashSet},
    num::NonZeroU32,
};
#[cfg(test)]
mod tests;

pub(crate) struct State<'ctx, 'program, 'tables> {
    tables: &'tables LineTables<'ctx, 'tables>,
    sources: &'program DebugSources,
    procedure: ProcedureId,
    function_scope: Scope<'ctx>,
    block: BlockPath,
    statement: Option<StatementPath>,
    scope: Scope<'ctx>,
    scopes: HashMap<BlockPath, Scope<'ctx>>,
    variables: HashMap<LocalId, Option<VariableRecord<'ctx>>>,
    declarations: HashMap<StatementPath, Vec<LocalId>>,
    parameters: Vec<LocalId>,
}

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(crate) fn debug_restore_location(
        &self,
        location: Option<inkwell::debug_info::DILocation<'ctx>>,
    ) {
        match location {
            Some(location) => self.builder.set_current_debug_location(location),
            None => self.builder.unset_current_debug_location(),
        }
    }
    fn debug_block(
        &mut self,
        path: Option<BlockPath>,
        block: &jai_ir::Block,
    ) -> Result<(), crate::Error> {
        let previous = self.builder.get_current_debug_location();
        let frame = match (&mut self.debug, path) {
            (Some(debug), Some(path)) => Some(debug.enter(path).map_err(crate::Error::Debug)?),
            _ => None,
        };
        let result = self.block(block);
        if let (Some(debug), Some(frame)) = (&mut self.debug, frame) {
            debug.leave(frame);
        }
        self.debug_restore_location(previous);
        result
    }
    pub(crate) fn debug_child_block(
        &mut self,
        branch: DebugBranch,
        block: &jai_ir::Block,
    ) -> Result<(), crate::Error> {
        let path = self
            .debug
            .as_ref()
            .map(|debug| debug.child(branch))
            .transpose()
            .map_err(crate::Error::Debug)?;
        self.debug_block(path, block)
    }
    pub(crate) fn debug_cleanup_block(
        &mut self,
        id: jai_ir::CleanupId,
        block: &jai_ir::Block,
    ) -> Result<(), crate::Error> {
        let path = self.debug.as_ref().map(|debug| debug.cleanup(id));
        self.debug_block(path, block)
    }
}
pub(crate) struct Frame<'ctx> {
    block: BlockPath,
    statement: Option<StatementPath>,
    scope: Scope<'ctx>,
}
impl<'ctx, 'program, 'tables> State<'ctx, 'program, 'tables> {
    pub(crate) fn artificial_call_location(&self, builder: &Builder<'ctx>) -> Result<(), Error> {
        self.tables
            .session
            .set_artificial_location(builder, self.function_scope.metadata)
            .map_err(super::bridge_error)
    }
    pub(crate) fn new(
        tables: &'tables LineTables<'ctx, 'tables>,
        sources: &'program DebugSources,
        procedure: ProcedureId,
        function_scope: Scope<'ctx>,
    ) -> Result<Self, Error> {
        let block = BlockPath::procedure(procedure);
        let scope = match sources.block(&block) {
            Some(location) => tables.lexical_scope(function_scope, location)?,
            None => function_scope,
        };
        let mut declarations: HashMap<StatementPath, Vec<LocalId>> = HashMap::new();
        let mut parameters = Vec::new();
        for (id, source) in sources
            .locals()
            .filter(|(id, _)| id.procedure() == procedure)
        {
            match &source.declaration {
                LocalDeclaration::Parameter(_) => parameters.push(id),
                LocalDeclaration::Statement(statement) => {
                    declarations.entry(statement.clone()).or_default().push(id)
                }
            }
        }
        parameters.sort_by_key(|id| {
            match &sources
                .local(*id)
                .expect("retained local source")
                .declaration
            {
                LocalDeclaration::Parameter(ordinal) => *ordinal,
                LocalDeclaration::Statement(_) => {
                    unreachable!("parameter list contains only parameters")
                }
            }
        });
        for locals in declarations.values_mut() {
            locals.sort_by_key(|id| id.index());
        }
        let mut scopes = HashMap::new();
        scopes.insert(block.clone(), scope);
        Ok(Self {
            tables,
            sources,
            procedure,
            function_scope,
            block,
            statement: None,
            scope,
            scopes,
            variables: HashMap::new(),
            declarations,
            parameters,
        })
    }
    pub(crate) fn parameters(
        &mut self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        types: &Types,
        slots: &[Slot<'ctx>],
    ) -> Result<(), Error> {
        for id in self.parameters.clone() {
            self.declare(context, builder, types, slots, id)?;
        }
        Ok(())
    }
    fn declare(
        &mut self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        types: &Types,
        slots: &[Slot<'ctx>],
        id: LocalId,
    ) -> Result<(), Error> {
        let source = self
            .sources
            .local(id)
            .ok_or_else(|| Error::Metadata("missing checked local source".into()))?;
        let slot = slots
            .get(id.index())
            .ok_or_else(|| Error::Metadata("missing checked local storage".into()))?;
        if !self.variables.contains_key(&id) {
            let (scope, kind) = match &source.declaration {
                LocalDeclaration::Parameter(ordinal) => {
                    let ordinal = ordinal
                        .checked_add(1)
                        .and_then(|ordinal| u32::try_from(ordinal).ok())
                        .and_then(NonZeroU32::new)
                        .ok_or_else(|| {
                            Error::Metadata("debug parameter ordinal overflows".into())
                        })?;
                    (self.function_scope, VariableKind::Parameter(ordinal))
                }
                LocalDeclaration::Statement(_) => {
                    (self.scope_for(&source.scope)?, VariableKind::Automatic)
                }
            };
            let variable = self.tables.create_variable_with_sources(
                context,
                scope,
                &source.location,
                &source.name,
                kind,
                slot.ty,
                types,
                slot.alignment,
                Some(self.sources),
            )?;
            self.variables.insert(id, variable);
        }
        if let Some(variable) = self.variables[&id] {
            self.tables.emit_variable(builder, slot.pointer, variable)?;
        }
        Ok(())
    }
    pub(crate) fn before_statement(
        &mut self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        types: &Types,
        slots: &[Slot<'ctx>],
        index: usize,
    ) -> Result<(), Error> {
        let path = StatementPath::in_block(&self.block, index);
        self.location(context, builder, &path)?;
        self.statement = Some(path.clone());
        if let Some(locals) = self.declarations.get(&path).cloned() {
            for local in locals {
                self.declare(context, builder, types, slots, local)?;
            }
        }
        Ok(())
    }
    fn location(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
        path: &StatementPath,
    ) -> Result<(), Error> {
        match self.sources.statement(path) {
            Some(source) => self
                .tables
                .statement_location(context, builder, self.scope, source),
            None => {
                builder.unset_current_debug_location();
                Ok(())
            }
        }
    }
    pub(crate) fn case_subject(
        &self,
        context: &'ctx Context,
        builder: &Builder<'ctx>,
    ) -> Result<(), Error> {
        let path = self
            .statement
            .as_ref()
            .ok_or_else(|| Error::Metadata("missing debug case path".into()))?
            .child(DebugBranch::CaseSubject);
        self.location(context, builder, &path)
    }
    pub(crate) fn child(&self, branch: DebugBranch) -> Result<BlockPath, Error> {
        let statement = self
            .statement
            .as_ref()
            .ok_or_else(|| Error::Metadata("missing debug parent statement".into()))?;
        Ok(statement.block_child(branch))
    }
    pub(crate) fn cleanup(&self, id: jai_ir::CleanupId) -> BlockPath {
        BlockPath::cleanup(self.procedure, id)
    }
    fn scope_for(&mut self, path: &BlockPath) -> Result<Scope<'ctx>, Error> {
        let mut chain = vec![path.clone()];
        let mut roots = HashSet::new();
        let mut scope = self.function_scope;
        loop {
            let current = chain.last().expect("scope path chain");
            if let Some(cached) = self.scopes.get(&BlockPath::new(current.root)) {
                scope = *cached;
                break;
            }
            if !roots.insert(current.root) {
                return Err(Error::Metadata("cyclic debug cleanup scope".into()));
            }
            let BlockRoot::Cleanup {
                procedure,
                cleanup,
            } = current.root
            else {
                break;
            };
            let Some(parent) = self.sources.cleanup_parent(procedure, cleanup) else {
                break;
            };
            chain.push(parent.clone());
        }
        while let Some(path) = chain.pop() {
            scope = self.scope_in_path(&path, scope)?;
        }
        Ok(scope)
    }
    fn scope_in_path(
        &mut self,
        path: &BlockPath,
        mut scope: Scope<'ctx>,
    ) -> Result<Scope<'ctx>, Error> {
        let mut prefix = BlockPath::new(path.root);
        for end in
            std::iter::once(0).chain(path.steps.iter().enumerate().filter_map(|(index, step)| {
                matches!(step, DebugPathStep::Child(branch) if *branch != DebugBranch::CaseSubject)
                    .then_some(index + 1)
            }))
        {
            prefix.steps = path.steps[..end].into();
            scope = if let Some(scope) = self.scopes.get(&prefix) {
                *scope
            } else {
                let scope = match self.sources.block(&prefix) {
                    Some(source) => self.tables.lexical_scope(scope, source)?,
                    None => scope,
                };
                self.scopes.insert(prefix.clone(), scope);
                scope
            };
        }
        Ok(scope)
    }
    pub(crate) fn enter(&mut self, block: BlockPath) -> Result<Frame<'ctx>, Error> {
        let scope = self.scope_for(&block)?;
        Ok(Frame {
            block: std::mem::replace(&mut self.block, block),
            statement: self.statement.take(),
            scope: std::mem::replace(&mut self.scope, scope),
        })
    }
    pub(crate) fn leave(&mut self, frame: Frame<'ctx>) {
        self.block = frame.block;
        self.statement = frame.statement;
        self.scope = frame.scope;
    }
}
