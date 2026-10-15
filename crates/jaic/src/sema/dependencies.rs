//! Explicit deferred-body dependencies, local to one owning compiler/workspace.
use super::*;

#[derive(Default)]
pub(super) struct Dependencies {
    /// Every active lowering records the placeholder reads beneath it. Nested
    /// generators therefore retain their transitive dependency in the caller.
    lowering: Vec<(ProcId, Vec<EntityId>)>,
    pub bodies: HashMap<ProcId, Vec<EntityId>>,
    pub controllers: Vec<ProcId>,
    pub running_runs: HashSet<usize>,
    pub finished_runs: HashSet<usize>,
    pub bindings: HashMap<EntityId, Vec<EntityId>>,
    pub source_scopes: HashMap<FileId, ScopeId>,
}

impl Compiler {
    pub(super) fn unresolved_body_placeholders(&self, procedure: ProcId) -> Vec<EntityId> {
        self.dependencies
            .bodies
            .get(&procedure)
            .into_iter()
            .flatten()
            .copied()
            .filter(|placeholder| {
                if self.dependencies.bindings.contains_key(placeholder) {
                    return false;
                }
                let entity = self.entity(*placeholder);
                !self
                    .scope(entity.scope)
                    .names
                    .get(&entity.name)
                    .is_some_and(|definitions| {
                        definitions.iter().any(|definition| {
                            !matches!(
                                self.entity(*definition).kind,
                                scope::EntityKind::Placeholder
                            )
                        })
                    })
            })
            .collect()
    }

    /// Run or resume a fully checked thunk without replaying an executed prefix.
    pub(super) fn call_with_dependencies(
        &mut self,
        function: ir::FuncId,
        args: &[u64],
        deferred: &mut Option<Box<Diagnostic>>,
    ) -> std::result::Result<Vec<u64>, crate::interp::Trap> {
        use crate::interp::CompileCall;
        let mut call = self.interp.call_for_compiler(&self.program, function, args);
        loop {
            match call {
                CompileCall::Complete(result) => return result,
                CompileCall::NeedsBody {
                    function,
                    callable,
                    optional,
                    continuation,
                } => {
                    let ready = match self.satisfy_body_dependency(function, callable, optional) {
                        Ok(ready) => ready,
                        Err(error) => {
                            self.interp
                                .cancel_compiler_call(&self.program, continuation);
                            *deferred = Some(error);
                            return Err(crate::interp::Trap::new("compile-time dependency failed"));
                        }
                    };
                    if callable && ready {
                        self.interp.mark_callable_ready(function);
                    }
                    call = self.interp.resume_for_compiler(&self.program, continuation);
                }
            }
        }
    }

    fn satisfy_body_dependency(
        &mut self,
        function: ir::FuncId,
        callable: bool,
        optional: bool,
    ) -> Result<bool> {
        let Some(index) = self.procs.iter().position(|procedure| {
            matches!(procedure.target, Some(procs::ProcTarget::Func(id)) if id == function)
        }) else {
            return err(Span::NONE, "compile-time dependency has no declared procedure");
        };
        let procedure = ProcId(index as u32);
        let info = self.proc(procedure);
        if info.body_state == procs::BodyState::Lowering {
            if optional {
                // An unrelated foreign call inside this body's generator does
                // not constitute an invocation of the callback being lowered.
                return Ok(false);
            }
            return err(
                info.span,
                format!("cyclic compile-time body dependency for `{}`", info.name),
            );
        }
        self.dependencies.controllers.push(procedure);
        let result = self.satisfy_body_dependency_inner(procedure, function, callable, optional);
        let popped = self.dependencies.controllers.pop();
        debug_assert_eq!(popped, Some(procedure));
        if result.is_ok() && self.reaches_type_table(function) {
            self.refresh_type_table();
        }
        result
    }

    fn satisfy_body_dependency_inner(
        &mut self,
        procedure: ProcId,
        function: ir::FuncId,
        callable: bool,
        optional: bool,
    ) -> Result<bool> {
        loop {
            if !callable && self.proc(procedure).body_state == procs::BodyState::Done {
                return Ok(true);
            }
            let traps = self.ct_traps;
            let executions = self.ct_executions;
            let lowered = if callable {
                self.dependencies.begin_body(procedure);
                let failure = self.lower_reachable_from(function);
                self.dependencies.end_body(procedure, failure.is_some());
                failure.map_or(Ok(()), Err)
            } else {
                self.lower_body(procedure)
            };
            let error = match lowered {
                Ok(()) => return Ok(true),
                Err(error) => error,
            };
            if self.ct_traps != traps {
                return Err(error);
            }
            let placeholders = self.unresolved_body_placeholders(procedure);
            if placeholders.is_empty() || self.ct_executions != executions {
                // A completed generator cannot be replayed merely because a
                // later stage of the body encounters another dependency.
                return Err(error);
            }
            if optional {
                // A retained deferred value must not stall an unrelated C call.
                // Direct callback arguments require readiness; opaque memory
                // receives a nonblocking readiness inspection instead.
                return Ok(false);
            }
            let sources = self.pull_workspace_sources()?;
            if sources {
                continue;
            }
            let completed = self.runs_done;
            self.run_top_level()?;
            if self.runs_done == completed {
                let info = self.proc(procedure);
                let mut diagnostic = Diagnostic::error(info.span, format!(
                    "compile-time dependency deadlock: body of `{}` awaits an unresolved #placeholder",
                    info.name
                )).with_note(error.span, error.message);
                for placeholder in placeholders {
                    let entity = self.entity(placeholder);
                    diagnostic = diagnostic.with_note(
                        entity.span,
                        format!(
                            "definition required for #placeholder `{}` in its declaring scope",
                            entity.name
                        ),
                    );
                }
                return Err(Box::new(diagnostic));
            }
        }
    }
}

impl Dependencies {
    pub fn read_placeholder(&mut self, entity: EntityId) {
        for (_, reads) in &mut self.lowering {
            if !reads.contains(&entity) {
                reads.push(entity);
            }
        }
    }

    pub fn begin_body(&mut self, procedure: ProcId) {
        self.lowering.push((procedure, Vec::new()));
    }

    pub fn end_body(&mut self, procedure: ProcId, failed: bool) {
        let (owner, reads) = self.lowering.pop().expect("paired body lowering");
        debug_assert_eq!(owner, procedure);
        if failed && !reads.is_empty() {
            self.bodies.insert(procedure, reads);
        } else {
            self.bodies.remove(&procedure);
        }
    }
}
