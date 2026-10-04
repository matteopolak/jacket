//! Concrete source body lowering shared by named and anonymous callable identities.
use super::*;

pub(super) struct SourceProcedureDefinition<'a> {
    pub signature: &'a Signature,
    pub source: &'a syntax::SourceProcedureSyntax,
    pub name: Option<Symbol>,
}

impl Resolver<'_> {
    pub(super) fn lower_source_procedure_body(
        &mut self,
        definition: SourceProcedureDefinition<'_>,
    ) -> Result<Procedure, Diagnostic> {
        let signature = definition.signature;
        let source = definition.source;
        let mut scopes = self.scopes.clone();
        scopes.push(HashMap::new());
        let mut local_scopes = self.local_scopes.clone();
        local_scopes.body_owner = Some(signature.id);
        let isolated_cache = crate::compile_time::Cache::default();
        let mut no_effects = jai_vm::NoEffects;
        let isolated_effects = crate::compile_time::SharedEffects::new(&mut no_effects);
        let child_context = self.compile_time.map(|context| {
            let (file, source) = self
                .graph_scope
                .map(|scope| {
                    let (file, _, source) = scope.code_origin();
                    (file, source)
                })
                .unwrap_or((context.file, context.source));
            if context.effect_mode == crate::compile_time::EffectsMode::Compiler
                && self
                    .graph_scope
                    .is_some_and(|scope| scope.is_isolated_procedure(signature.id))
            {
                context.isolated_for_source(
                    signature.id,
                    file,
                    source,
                    &isolated_cache,
                    &isolated_effects,
                )
            } else {
                context.for_source(signature.id, file, source)
            }
        });
        self.meta.storage_alignments.clear_procedure(signature.id);
        let debug_policy = self.debug.policy().nested(source.debug);
        let procedure = (|| {
            let mut child = Resolver {
                conditional_subjects: Vec::new(),
                debug: crate::debug_capture::Capture::new(self.debug.source()),
                checks: self.checks.overridden(source.checks),
                context: self.context,
                context_available: source.convention != CallingConvention::C
                    && source.context == ContextMode::Implicit,
                meta: &mut *self.meta,
                graph_scope: self.graph_scope,
                compile_time: child_context.as_ref(),
                target_layout: self.target_layout,
                procedure: signature.id,
                expression_owner: Some(signature.id),
                types: &mut *self.types,
                places: &mut *self.places,
                signatures: self.signatures,
                symbols: self.symbols,
                scopes,
                local_scopes,
                globals: self.globals,
                locals: vec![],
                span: source.span,
                results: &signature.results,
                loops: vec![],
                next_loop: 0,
                cleanups: vec![],
                active_push: None,
                next_push: 0,
                deferred_scopes: vec![],
                cleanup_context: None,
            };
            child.debug.enter_policy(debug_policy);
            let mut parameters = Vec::new();
            for parameter in &signature.parameters {
                if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                    child.bind_discarded_parameter(parameter.name, parameter.ty)?;
                    continue;
                }
                let ordinal = parameters.len();
                let local = child.declare_typed(parameter.name, parameter.ty)?;
                if let Some(source_parameter) = source
                    .parameters
                    .iter()
                    .find(|source| source.name == parameter.name)
                {
                    child.bind_callback_parameter(
                        local.place(),
                        source_parameter,
                        parameter.default.as_ref(),
                    )?;
                    child.debug_parameter(local, parameter.name, source_parameter.span, ordinal)?;
                }
                parameters.push(local);
            }
            for parameter in source.parameters.iter().filter(|parameter| parameter.using) {
                let storage = child.storage(parameter.name)?;
                child.using_record(storage)?;
            }
            let body = child.named_result_body(&source.results, &source.body)?;
            if !signature.results.is_empty() && body.flow != Flow::Terminates {
                return Err(Diagnostic::new(
                    source.span,
                    "value-returning nested procedure may reach its end",
                ));
            }
            child.remember_procedure_notes(signature.id, &source.notes)?;
            child.publish_source_debug(definition.name, source.span)?;
            Ok(Procedure {
                id: signature.id,
                signature: signature.ty,
                parameters,
                locals: child.locals,
                cleanups: child.cleanups,
                body,
            })
        })();
        if let (Some(parent), Some(child)) = (self.compile_time, child_context.as_ref()) {
            parent.merge_pending_from(child);
        }
        let procedure = procedure?;
        Ok(procedure)
    }
}
