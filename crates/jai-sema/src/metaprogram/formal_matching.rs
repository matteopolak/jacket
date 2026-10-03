//! Collection expansion formals use real origins and immutable inferred bindings.
use super::*;
use crate::overloads::{
    Argument, ArgumentInfo, Candidate, CandidateVariadic, ConversionRank, Match, Parameter,
    TypePattern,
};
use crate::polymorphism::{Substitution, TypeBinding};
use jai_types::{TypeId, TypeKind};

pub(crate) struct ExpandedSourceMatch {
    pub expected: TypeId,
    pub matched: Match<MacroId>,
}

pub(crate) enum ExpandedSourceError {
    Definition(Diagnostic),
    Mismatch(Diagnostic),
}
impl From<Diagnostic> for ExpandedSourceError {
    fn from(error: Diagnostic) -> Self {
        Self::Definition(error)
    }
}

impl Resolver<'_> {
    pub(crate) fn match_expanded_source_ranked(
        &mut self,
        target: &ExpandedTarget,
        formal: &syntax::TypeSyntax,
        formal_span: Span,
        actual: TypeId,
        caller_span: Span,
    ) -> Result<ExpandedSourceMatch, ExpandedSourceError> {
        let scope = self
            .graph_scope
            .ok_or_else(|| {
                Diagnostic::new(
                    caller_span,
                    "collection expansion requires its defining module scope",
                )
            })?
            .code_file(target.file);
        let mut definition = target
            .capture
            .as_ref()
            .and_then(|capture| capture.substitution.clone())
            .unwrap_or_default();
        if let Some(capture) = &target.capture {
            // Captured static type declarations preserve lexical shadowing.
            for frame in &capture.frames {
                for (&name, binding) in frame {
                    if let Binding::Type(ty) = binding {
                        if let Some(previous) = definition
                            .types
                            .iter_mut()
                            .find(|binding| binding.name == name)
                        {
                            previous.ty = *ty;
                        } else {
                            definition.types.push(TypeBinding {
                                name,
                                ty: *ty,
                            });
                        }
                    }
                }
            }
        }
        let pattern = scope.expanded_formal_pattern(
            formal,
            self.types,
            &mut self.meta.record_specializations,
            Some(&definition),
            formal_span,
        )?;
        let parameter = target
            .procedure
            .parameters
            .iter()
            .find(|parameter| parameter.span == formal_span)
            .ok_or_else(|| {
                Diagnostic::new(
                    formal_span,
                    "collection formal is outside the actual expansion header",
                )
            })?;
        let candidate = Candidate {
            result_type_parameters: Vec::new(),
            declaration: target.id,
            parameters: vec![Parameter {
                evaluation: syntax::ParameterEvaluation::Evaluate,
                name: parameter.name,
                ty: pattern.clone(),
                default: None,
                baking: syntax::ParameterBaking::None,
            }],
            variadic: CandidateVariadic::None,
        };
        crate::overloads::validate_candidate(&candidate, formal_span)?;
        let mut possibilities = vec![actual];
        if matches!(pattern, TypePattern::Pointer(_))
            && !matches!(self.types.kind(actual), Ok(TypeKind::Pointer(_)))
        {
            possibilities.push(
                self.types
                    .pointer(actual)
                    .map_err(|error| Diagnostic::new(caller_span, error.to_string()))?,
            );
        } else if !matches!(pattern, TypePattern::Pointer(_))
            && let Ok(TypeKind::Pointer(pointee)) = self.types.kind(actual)
        {
            possibilities.push(*pointee);
        }
        let mut rejection = None;
        for source in possibilities {
            let arguments = [Argument {
                name: None,
                spread: false,
                info: ArgumentInfo::typed(source),
                span: caller_span,
            }];
            match crate::overloads::match_candidate_with_nominals(
                self.types,
                self,
                &candidate,
                &arguments,
                caller_span,
            ) {
                Ok(mut matched) => {
                    if source != actual {
                        for rank in &mut matched.conversions {
                            *rank = (*rank).max(ConversionRank::Widening);
                        }
                    }
                    let mut substitution = definition.clone();
                    for binding in std::mem::take(&mut matched.substitution.types) {
                        if let Some(previous) = substitution
                            .types
                            .iter_mut()
                            .find(|previous| previous.name == binding.name)
                        {
                            *previous = binding;
                        } else {
                            substitution.types.push(binding);
                        }
                    }
                    for binding in std::mem::take(&mut matched.substitution.constants) {
                        if let Some(previous) = substitution
                            .constants
                            .iter_mut()
                            .find(|previous| previous.name == binding.name)
                        {
                            *previous = binding;
                        } else {
                            substitution.constants.push(binding);
                        }
                    }
                    let expected = self.materialize_matched_expanded_type(
                        &pattern,
                        source,
                        &substitution,
                        caller_span,
                    )?;
                    matched.substitution = substitution;
                    return Ok(ExpandedSourceMatch {
                        expected,
                        matched,
                    });
                }
                Err(error) => {
                    rejection.get_or_insert(error);
                }
            }
        }
        Err(ExpandedSourceError::Mismatch(
            rejection.expect("at least the actual type is checked"),
        ))
    }

    fn materialize_matched_expanded_type(
        &mut self,
        pattern: &TypePattern,
        actual: TypeId,
        substitution: &Substitution,
        span: Span,
    ) -> Result<TypeId, Diagnostic> {
        Ok(match pattern {
            TypePattern::Concrete(ty) => *ty,
            TypePattern::Restricted {
                ty, ..
            } => self.materialize_matched_expanded_type(ty, actual, substitution, span)?,
            TypePattern::Infer(name) | TypePattern::Variable(name) => substitution
                .ty(*name)
                .ok_or_else(|| Diagnostic::new(span, "expansion type variable is not bound"))?,
            TypePattern::Pointer(inner) => {
                let TypeKind::Pointer(pointee) = self
                    .types
                    .kind(actual)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                else {
                    return Err(Diagnostic::new(
                        span,
                        "matched expansion source is not a pointer",
                    ));
                };
                let pointee =
                    self.materialize_matched_expanded_type(inner, *pointee, substitution, span)?;
                self.types
                    .pointer(pointee)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
            }
            TypePattern::Slice(inner) => {
                let element = match self
                    .types
                    .kind(actual)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
                {
                    TypeKind::Slice(element)
                    | TypeKind::DynamicArray(element)
                    | TypeKind::FixedArray {
                        element, ..
                    } => *element,
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "matched expansion source is not an array",
                        ));
                    }
                };
                let element =
                    self.materialize_matched_expanded_type(inner, element, substitution, span)?;
                self.types
                    .slice(element)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?
            }
            // Pure matching has already proved the exact nominal origin,
            // argument bindings, count, and callback signature. Retain that
            // concrete identity, including bare generic record formals.
            TypePattern::NominalApplication {
                ..
            }
            | TypePattern::FixedArray {
                ..
            }
            | TypePattern::DynamicArray(_)
            | TypePattern::Procedure(_) => actual,
        })
    }
}
