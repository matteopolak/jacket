//! Checked erased formal types belong to the original annotation environment.
use super::*;
use crate::polymorphism::Substitution;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ProcedureAnnotationOrigin {
    Graph(jai_modules::FileInstanceId),
    Local {
        source: Option<jai_source::SourceId>,
        procedure: ProcedureId,
        scopes: Vec<crate::local_declarations::LexicalScopeId>,
    },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ProcedureAnnotationKey {
    origin: ProcedureAnnotationOrigin,
    node: Vec<(usize, usize)>,
    substitution: Substitution,
    target: Option<jai_types::LayoutPolicy>,
}

pub(crate) struct AnnotationPublication<'a> {
    pub(crate) file: jai_modules::FileInstanceId,
    pub(crate) source: &'a syntax::ProcedureTypeSyntax,
    pub(crate) substitution: Option<&'a Substitution>,
    pub(crate) parameters: Vec<TypeId>,
    pub(crate) defaults: Vec<Option<ParameterDefault>>,
    pub(crate) ty: TypeId,
    pub(crate) span: Span,
}
impl ProcedureAnnotationKey {
    pub(crate) fn new(
        origin: ProcedureAnnotationOrigin,
        source: &syntax::ProcedureTypeSyntax,
        substitution: Option<&Substitution>,
        target: Option<jai_types::LayoutPolicy>,
    ) -> Self {
        Self {
            origin,
            node: source
                .parameters
                .iter()
                .chain(&source.results)
                .map(|parameter| (parameter.span.start, parameter.span.end))
                .collect(),
            substitution: substitution.cloned().unwrap_or_default(),
            target,
        }
    }
}

/// Its constructor proves the source policy projects to the canonical ABI.
#[derive(Clone)]
pub(crate) struct CheckedSourceProcedureType {
    ty: TypeId,
    parameters: Box<[TypeId]>,
    defaults: Box<[Option<ParameterDefault>]>,
    source_variadic: crate::overloads::CandidateVariadic,
}
impl CheckedSourceProcedureType {
    pub(crate) fn checked(
        ty: TypeId,
        source: &syntax::ProcedureTypeSyntax,
        parameters: Vec<TypeId>,
        defaults: Vec<Option<ParameterDefault>>,
        types: &TypeRegistry,
        span: Span,
    ) -> Result<Self, Diagnostic> {
        let runtime = types
            .procedure_definition(ty)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let source_parameters = source
            .parameters
            .iter()
            .filter(|parameter| !(parameter.variadic && source.convention == CallingConvention::C))
            .collect::<Vec<_>>();
        if source_parameters.len() != parameters.len() || defaults.len() != parameters.len() {
            return Err(Diagnostic::new(
                span,
                "checked callback source formal type count is inconsistent",
            ));
        }
        let evaluated = source_parameters
            .iter()
            .zip(&parameters)
            .filter(|(source, _)| source.evaluation == syntax::ParameterEvaluation::Evaluate)
            .map(|(_, ty)| *ty)
            .collect::<Vec<_>>();
        let source_variadic = source
            .parameters
            .iter()
            .position(|parameter| parameter.variadic)
            .map_or(crate::overloads::CandidateVariadic::None, |parameter| {
                if source.convention == CallingConvention::C {
                    crate::overloads::CandidateVariadic::C {
                        fixed_parameters: parameters.len(),
                    }
                } else {
                    crate::overloads::CandidateVariadic::Jai {
                        parameter,
                    }
                }
            });
        let variadic = match source_variadic {
            crate::overloads::CandidateVariadic::None => jai_types::Variadic::None,
            crate::overloads::CandidateVariadic::C {
                ..
            } => {
                if source.parameters.last().is_some_and(|parameter| {
                    parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                }) {
                    jai_types::Variadic::C {
                        fixed_parameters: evaluated.len(),
                    }
                } else {
                    jai_types::Variadic::None
                }
            }
            crate::overloads::CandidateVariadic::Jai {
                parameter,
            } => {
                let element = match types.kind(parameters[parameter]) {
                    Ok(jai_types::TypeKind::Slice(element)) => *element,
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "checked callback variadic formal is not a slice",
                        ));
                    }
                };
                if source.parameters[parameter].evaluation == syntax::ParameterEvaluation::Evaluate
                {
                    jai_types::Variadic::Jai {
                        parameter: source.parameters[..parameter]
                            .iter()
                            .filter(|parameter| {
                                parameter.evaluation == syntax::ParameterEvaluation::Evaluate
                            })
                            .count(),
                        element,
                    }
                } else {
                    jai_types::Variadic::None
                }
            }
        };
        if evaluated.as_slice() != runtime.parameters.as_ref()
            || variadic != runtime.variadic
            || source.convention != runtime.convention
            || source.return_abi != runtime.return_abi
            || source.context != runtime.context
        {
            return Err(Diagnostic::new(
                span,
                "checked callback source policy does not project to its canonical ABI",
            ));
        }
        Ok(Self {
            ty,
            parameters: parameters.into_boxed_slice(),
            defaults: defaults.into_boxed_slice(),
            source_variadic,
        })
    }
    pub(crate) fn ty(&self) -> TypeId {
        self.ty
    }
    pub(crate) fn parameters(&self) -> &[TypeId] {
        &self.parameters
    }
    pub(crate) fn defaults(&self) -> &[Option<ParameterDefault>] {
        &self.defaults
    }
}

#[derive(Default)]
pub(crate) struct SourceProcedureAnnotations {
    entries: HashMap<ProcedureAnnotationKey, Arc<CheckedSourceProcedureType>>,
}
impl SourceProcedureAnnotations {
    pub(crate) fn remember(
        &mut self,
        key: ProcedureAnnotationKey,
        proof: CheckedSourceProcedureType,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if let Some(previous) = self.entries.get(&key) {
            if previous.ty != proof.ty
                || previous.parameters != proof.parameters
                || previous.source_variadic != proof.source_variadic
            {
                return Err(Diagnostic::new(
                    span,
                    "callback annotation environment has conflicting checked source types",
                ));
            }
            return Ok(());
        }
        self.entries.insert(key, Arc::new(proof));
        Ok(())
    }
    pub(crate) fn get(
        &self,
        key: &ProcedureAnnotationKey,
    ) -> Option<Arc<CheckedSourceProcedureType>> {
        self.entries.get(key).cloned()
    }
}

impl Resolver<'_> {
    pub(crate) fn remember_local_procedure_annotation(
        &mut self,
        source: &syntax::ProcedureTypeSyntax,
        parameters: Vec<TypeId>,
        ty: TypeId,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !source.parameters.iter().any(|parameter| {
            parameter.evaluation == syntax::ParameterEvaluation::Discard
                || parameter.default.is_some()
        }) {
            return Ok(());
        }
        let defaults = source
            .parameters
            .iter()
            .filter(|parameter| !(parameter.variadic && source.convention == CallingConvention::C))
            .zip(&parameters)
            .map(|(parameter, ty)| {
                parameter
                    .default
                    .as_ref()
                    .map(|expression| {
                        if parameter.evaluation == syntax::ParameterEvaluation::Discard {
                            self.validate_discarded_expression(expression)?;
                            Ok(ParameterDefault::Discarded)
                        } else {
                            self.preview_source_parameter_default(expression, *ty)
                        }
                    })
                    .transpose()
            })
            .collect::<Result<Vec<_>, Diagnostic>>()?;
        let proof = CheckedSourceProcedureType::checked(
            ty, source, parameters, defaults, self.types, span,
        )?;
        let key = ProcedureAnnotationKey::new(
            ProcedureAnnotationOrigin::Local {
                source: self
                    .debug
                    .source()
                    .or_else(|| self.graph_scope.map(|scope| scope.source())),
                procedure: self.local_scopes.body_owner().unwrap_or(self.procedure),
                scopes: self.local_scopes.capture_identity(),
            },
            source,
            self.graph_scope.and_then(|scope| scope.substitution),
            self.target_layout,
        );
        self.meta
            .callbacks
            .source_annotations
            .remember(key, proof, span)
    }

    pub(crate) fn checked_callback_origin_parameter_types(
        &self,
        ty: TypeId,
        origin: &contracts::CallbackSyntaxOrigin,
        span: Span,
    ) -> Result<Vec<TypeId>, Diagnostic> {
        let proof = origin
            .proof
            .clone()
            .or_else(|| self.original_annotation_proof(origin))
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "discarded callback annotation is pending its checked source type proof",
                )
            })?;
        if proof.ty() != ty {
            return Err(Diagnostic::new(
                span,
                "callback annotation proof has a different canonical ABI",
            ));
        }
        Ok(proof.parameters().to_vec())
    }
    pub(crate) fn original_annotation_proof(
        &self,
        origin: &contracts::CallbackSyntaxOrigin,
    ) -> Option<Arc<CheckedSourceProcedureType>> {
        use contracts::ContractOwner;
        let environment = &origin.environment;
        let annotation_origin = match environment.owner {
            ContractOwner::File => ProcedureAnnotationOrigin::Graph(environment.file?),
            ContractOwner::Procedure(procedure) => ProcedureAnnotationOrigin::Local {
                source: environment.source,
                procedure,
                scopes: environment.lexical_scopes.clone(),
            },
        };
        let key = ProcedureAnnotationKey::new(
            annotation_origin,
            &origin.original,
            environment.substitution.as_ref(),
            environment.target,
        );
        match environment.owner {
            ContractOwner::File => self
                .graph_scope
                .and_then(|scope| scope.checked_procedure_annotation(&key)),
            _ => self.meta.callbacks.source_annotations.get(&key),
        }
    }
}
