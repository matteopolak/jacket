//! Procedure headers retain nominal template origins and introducing arguments.
use super::*;
use crate::overloads::{
    CountPattern, NominalArgumentKind, NominalArgumentPattern, TypePattern, TypeRestrictionPattern,
};
use syntax::{ExpressionKind as E, TypeSyntax as T};

pub(crate) struct FormalPatternRequest<'a> {
    pub(crate) file: FileInstanceId,
    pub(crate) syntax: &'a T,
    pub(crate) span: Span,
    pub(crate) substitution: Option<&'a Substitution>,
}

/// Macro formals match a real template origin without instantiating absent arguments.
pub(crate) fn formal_pattern(
    graph: &ModuleGraph,
    request: FormalPatternRequest<'_>,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<TypePattern, LocatedDiagnostic> {
    let mut variables = HashSet::new();
    collect(request.syntax, &mut variables);
    let scope = PatternScope {
        variables: &variables,
        substitution: request.substitution,
        bare_templates: true,
        owner_span: request.span,
    };
    let mut resolver = TypeResolver {
        graph,
        types,
        nominals,
        records,
        evaluate,
        scalar_pending: None,
        aliases: HashSet::new(),
        lexical: None,
        lexical_active: false,
        nominal_context: NominalAnnotationContext::Forbidden,
    };
    let mut patterns = HashMap::new();
    resolver
        .header_applications(request.file, request.syntax, &scope, &mut patterns)
        .map_err(|error| error.into_diagnostic(graph))?;
    resolver
        .header_pattern(
            request.file,
            request.syntax,
            request.span,
            &scope,
            &patterns,
        )
        .map_err(|error| error.into_diagnostic(graph))
}

struct PatternScope<'a> {
    variables: &'a HashSet<jai_source::Symbol>,
    substitution: Option<&'a Substitution>,
    bare_templates: bool,
    owner_span: Span,
}

struct HeaderSyntax<'a> {
    parameters: &'a [syntax::Parameter],
    results: &'a [syntax::ProcedureResult],
    span: Span,
}

pub(crate) fn procedure_patterns(
    graph: &ModuleGraph,
    file: FileInstanceId,
    procedure: &syntax::Procedure,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<HashMap<(usize, usize), TypePattern>, LocatedDiagnostic> {
    header_patterns(
        graph,
        file,
        HeaderSyntax {
            parameters: &procedure.parameters,
            results: &procedure.results,
            span: procedure.span,
        },
        types,
        nominals,
        records,
        evaluate,
    )
}

/// Lexical callable headers supply real imported template origins while the
/// pure template builder resolves their remaining annotations in that scope.
#[allow(clippy::too_many_arguments)]
pub(crate) fn procedure_application_patterns(
    graph: &ModuleGraph,
    file: FileInstanceId,
    procedure: &syntax::Procedure,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
    lexical: &LexicalTypeArguments,
) -> Result<HashMap<(usize, usize), TypePattern>, LocatedDiagnostic> {
    let annotations = procedure
        .parameters
        .iter()
        .filter_map(|parameter| match &parameter.binding {
            syntax::ParameterBinding::RequiredType(ty)
            | syntax::ParameterBinding::DefaultedType { ty: Some(ty), .. } => Some(ty),
            _ => None,
        })
        .chain(
            procedure
                .results
                .iter()
                .filter_map(|result| match &result.binding {
                    syntax::ResultBinding::Typed { ty, .. } => Some(ty),
                    _ => None,
                }),
        )
        .collect::<Vec<_>>();
    let mut variables = HashSet::new();
    for parameter in &procedure.parameters {
        if parameter.baking != syntax::ParameterBaking::None {
            variables.insert(parameter.name);
        }
    }
    for annotation in &annotations {
        collect(annotation, &mut variables);
    }
    let mut resolver = TypeResolver {
        graph,
        types,
        nominals,
        records,
        evaluate,
        scalar_pending: None,
        aliases: HashSet::new(),
        lexical: Some(lexical),
        lexical_active: true,
        nominal_context: NominalAnnotationContext::Forbidden,
    };
    let scope = PatternScope {
        variables: &variables,
        substitution: None,
        bare_templates: false,
        owner_span: procedure.span,
    };
    let mut patterns = HashMap::new();
    for annotation in annotations {
        resolver
            .header_applications(file, annotation, &scope, &mut patterns)
            .map_err(|error| error.into_diagnostic(graph))?;
    }
    Ok(patterns)
}
pub(crate) fn prototype_patterns(
    graph: &ModuleGraph,
    file: FileInstanceId,
    prototype: &syntax::ProcedurePrototype,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<HashMap<(usize, usize), TypePattern>, LocatedDiagnostic> {
    header_patterns(
        graph,
        file,
        HeaderSyntax {
            parameters: &prototype.parameters,
            results: &prototype.results,
            span: prototype.span,
        },
        types,
        nominals,
        records,
        evaluate,
    )
}
fn header_patterns(
    graph: &ModuleGraph,
    file: FileInstanceId,
    header: HeaderSyntax<'_>,
    types: &mut TypeRegistry,
    nominals: &Nominals<'_>,
    records: &mut RecordSpecializations,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ScalarConstant, LocatedDiagnostic>,
) -> Result<HashMap<(usize, usize), TypePattern>, LocatedDiagnostic> {
    let HeaderSyntax {
        parameters,
        results,
        span,
    } = header;
    let mut variables = HashSet::new();
    let annotations = parameters
        .iter()
        .filter_map(|parameter| match &parameter.binding {
            syntax::ParameterBinding::RequiredType(ty)
            | syntax::ParameterBinding::DefaultedType { ty: Some(ty), .. } => {
                Some((ty, parameter.span))
            }
            _ => None,
        })
        .chain(results.iter().filter_map(|result| match &result.binding {
            syntax::ResultBinding::Typed { ty, .. } => Some((ty, result.span)),
            _ => None,
        }))
        .collect::<Vec<_>>();
    for parameter in parameters {
        if parameter.baking != syntax::ParameterBaking::None {
            variables.insert(parameter.name);
        }
    }
    for (ty, _) in &annotations {
        collect(ty, &mut variables);
    }
    let mut resolver = TypeResolver {
        graph,
        types,
        nominals,
        records,
        evaluate,
        scalar_pending: None,
        aliases: HashSet::new(),
        lexical: None,
        lexical_active: false,
        nominal_context: NominalAnnotationContext::Forbidden,
    };
    let mut patterns = HashMap::new();
    let scope = PatternScope {
        variables: &variables,
        substitution: None,
        bare_templates: false,
        owner_span: span,
    };
    for (ty, span) in annotations {
        resolver
            .header_applications(file, ty, &scope, &mut patterns)
            .map_err(|error| error.into_diagnostic(graph))?;
        if resolver.has_bound_module_parameter(file, ty) || contains_restriction(ty) {
            let pattern = resolver
                .header_pattern(file, ty, span, &scope, &patterns)
                .map_err(|error| error.into_diagnostic(graph))?;
            patterns.insert((span.start, span.end), pattern);
        }
    }
    Ok(patterns)
}

use std::collections::HashMap;
fn collect(ty: &T, names: &mut HashSet<jai_source::Symbol>) {
    match ty {
        T::Variable(name) => {
            names.insert(*name);
        }
        T::Restricted {
            variable,
            restriction,
            ..
        } => {
            names.insert(*variable);
            match restriction {
                syntax::TypeRestrictionSyntax::Nominal(ty)
                | syntax::TypeRestrictionSyntax::Interface(ty) => collect(ty, names),
            }
        }
        T::Pointer(inner) | T::Slice(inner) | T::DynamicArray(inner) => collect(inner, names),
        T::FixedArray { count, element } => {
            collect(element, names);
            collect_expression(count, names);
        }
        T::Application(application) => {
            for argument in &application.arguments {
                collect_expression(&argument.value, names);
            }
        }
        T::Procedure(procedure) => {
            for parameter in procedure.parameters.iter().chain(&procedure.results) {
                collect(&parameter.ty, names);
            }
        }
        _ => {}
    }
}
fn contains_restriction(ty: &T) -> bool {
    match ty {
        T::Restricted { .. } => true,
        T::Pointer(ty) | T::Slice(ty) | T::DynamicArray(ty) => contains_restriction(ty),
        T::FixedArray { element, .. } => contains_restriction(element),
        T::Procedure(procedure) => procedure
            .parameters
            .iter()
            .chain(&procedure.results)
            .any(|parameter| contains_restriction(&parameter.ty)),
        T::Application(application) => application.arguments.iter().any(|argument| {
            type_expression(&argument.value).is_some_and(|ty| contains_restriction(&ty))
        }),
        _ => false,
    }
}
fn collect_expression(expression: &syntax::Expression, names: &mut HashSet<jai_source::Symbol>) {
    match &expression.kind {
        E::CompileVariable(name) => {
            names.insert(*name);
        }
        E::Type(ty) => collect(ty, names),
        E::AddressOf(inner) | E::Unary(_, inner) => collect_expression(inner, names),
        E::Call(_, arguments) | E::QualifiedCall(_, arguments) => {
            for argument in arguments {
                collect_expression(&argument.value, names);
            }
        }
        _ => {}
    }
}

impl<F> TypeResolver<'_, '_, F>
where
    F: FnMut(FileInstanceId, &syntax::Expression) -> Result<ScalarConstant, LocatedDiagnostic>,
{
    fn module_parameter(&self, file: FileInstanceId, name: jai_source::Symbol) -> bool {
        matches!(
            self.graph.lookup(file, &path(name)),
            Ok(jai_modules::Binding::Parameter(_))
        )
    }
    fn has_bound_module_parameter(&self, file: FileInstanceId, ty: &T) -> bool {
        match ty {
            T::Variable(name) => self.module_parameter(file, *name),
            T::Restricted {
                variable,
                restriction,
                ..
            } => {
                self.module_parameter(file, *variable)
                    || match restriction {
                        syntax::TypeRestrictionSyntax::Nominal(ty)
                        | syntax::TypeRestrictionSyntax::Interface(ty) => {
                            self.has_bound_module_parameter(file, ty)
                        }
                    }
            }
            T::Named(path) if path.members.is_empty() => self.module_parameter(file, path.root),
            T::Pointer(inner) | T::Slice(inner) | T::DynamicArray(inner) => {
                self.has_bound_module_parameter(file, inner)
            }
            T::FixedArray { count, element } => {
                self.has_bound_module_parameter(file, element)
                    || matches!(&count.kind, E::CompileVariable(name) if self.module_parameter(file, *name))
            }
            T::Procedure(procedure) => procedure
                .parameters
                .iter()
                .chain(&procedure.results)
                .any(|parameter| self.has_bound_module_parameter(file, &parameter.ty)),
            T::Application(application) => application.arguments.iter().any(|argument| {
                type_expression(&argument.value)
                    .is_some_and(|ty| self.has_bound_module_parameter(file, &ty))
            }),
            _ => false,
        }
    }
    fn header_applications(
        &mut self,
        file: FileInstanceId,
        ty: &T,
        scope: &PatternScope<'_>,
        patterns: &mut HashMap<(usize, usize), TypePattern>,
    ) -> TypeResult<()> {
        match ty {
            T::Restricted { restriction, .. } => match restriction {
                syntax::TypeRestrictionSyntax::Nominal(ty)
                | syntax::TypeRestrictionSyntax::Interface(ty) => {
                    self.header_applications(file, ty, scope, patterns)?;
                }
            },
            T::Application(application) => {
                for argument in &application.arguments {
                    if let Some(inner) = type_expression(&argument.value) {
                        self.header_applications(file, &inner, scope, patterns)?;
                    }
                }
                let pattern = self.application_pattern(file, application, scope, patterns)?;
                patterns.insert((application.span.start, application.span.end), pattern);
            }
            T::Pointer(inner) | T::Slice(inner) | T::DynamicArray(inner) => {
                self.header_applications(file, inner, scope, patterns)?
            }
            T::FixedArray { element, .. } => {
                self.header_applications(file, element, scope, patterns)?
            }
            T::Procedure(procedure) => {
                for parameter in procedure.parameters.iter().chain(&procedure.results) {
                    self.header_applications(file, &parameter.ty, scope, patterns)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
    fn header_pattern(
        &mut self,
        file: FileInstanceId,
        ty: &T,
        span: Span,
        scope: &PatternScope<'_>,
        patterns: &HashMap<(usize, usize), TypePattern>,
    ) -> TypeResult<TypePattern> {
        Ok(match ty {
            T::Restricted {
                variable,
                restriction,
                span,
            } => {
                let ty =
                    self.header_pattern(file, &T::Variable(*variable), *span, scope, patterns)?;
                let restriction = match restriction {
                    syntax::TypeRestrictionSyntax::Nominal(ty) => {
                        let nominal_scope = PatternScope {
                            bare_templates: true,
                            ..*scope
                        };
                        TypeRestrictionPattern::Nominal(Box::new(self.header_pattern(
                            file,
                            ty,
                            *span,
                            &nominal_scope,
                            patterns,
                        )?))
                    }
                    syntax::TypeRestrictionSyntax::Interface(ty) => {
                        TypeRestrictionPattern::Interface(self.resolve(
                            file,
                            ty,
                            scope.substitution,
                            *span,
                        )?)
                    }
                };
                TypePattern::Restricted {
                    ty: Box::new(ty),
                    restriction,
                }
            }
            T::Variable(name)
                if scope
                    .substitution
                    .and_then(|scope| scope.ty(*name))
                    .is_some()
                    || self.module_parameter(file, *name) =>
            {
                TypePattern::Concrete(self.resolve(file, ty, scope.substitution, span)?)
            }
            T::Variable(name) => TypePattern::Infer(*name),
            T::Named(path) if path.members.is_empty() && self.module_parameter(file, path.root) => {
                TypePattern::Concrete(self.resolve(file, ty, scope.substitution, span)?)
            }
            T::Named(path)
                if path.members.is_empty()
                    && scope
                        .substitution
                        .and_then(|scope| scope.ty(path.root))
                        .is_some() =>
            {
                TypePattern::Concrete(self.resolve(file, ty, scope.substitution, span)?)
            }
            T::Named(path) if path.members.is_empty() && scope.variables.contains(&path.root) => {
                TypePattern::Variable(path.root)
            }
            T::Named(path) if scope.bare_templates => {
                let declaration = template_origin(self.graph, file, path, span).ok();
                match declaration.filter(|id| matches!(&self.graph.declaration(*id).unwrap().syntax().kind, syntax::FileDeclarationKind::Record(record) if !record.parameters.is_empty())) {
                    Some(declaration) => TypePattern::NominalApplication { declaration, arguments: vec![] },
                    None => TypePattern::Concrete(self.resolve(file, ty, scope.substitution, span)?),
                }
            }
            T::Pointer(inner) => TypePattern::Pointer(Box::new(
                self.header_pattern(file, inner, span, scope, patterns)?,
            )),
            T::Slice(inner) => TypePattern::Slice(Box::new(
                self.header_pattern(file, inner, span, scope, patterns)?,
            )),
            T::DynamicArray(inner) => TypePattern::DynamicArray(Box::new(
                self.header_pattern(file, inner, span, scope, patterns)?,
            )),
            T::FixedArray { count, element } => {
                let count = match &count.kind {
                    E::CompileVariable(name)
                        if scope
                            .substitution
                            .and_then(|scope| scope.constant(*name))
                            .is_none()
                            && !self.module_parameter(file, *name) =>
                    {
                        CountPattern::Infer(*name)
                    }
                    E::Name(name)
                        if scope.variables.contains(name)
                            && scope
                                .substitution
                                .and_then(|scope| scope.constant(*name))
                                .is_none()
                            && !self.module_parameter(file, *name) =>
                    {
                        CountPattern::Variable(*name)
                    }
                    _ => {
                        let value = self.scalar(file, count, scope.substitution)?;
                        let integer = match value {
                            ScalarConstant::Literal(value) => Some(value),
                            ScalarConstant::Int(value) => Some(value.value()),
                            _ => None,
                        };
                        CountPattern::Exact(
                            integer
                                .and_then(|value| u64::try_from(value).ok())
                                .ok_or_else(|| {
                                    failure(
                                        self.graph,
                                        file,
                                        Diagnostic::new(
                                            count.span,
                                            "array count requires a nonnegative integer constant",
                                        ),
                                    )
                                })?,
                        )
                    }
                };
                TypePattern::FixedArray {
                    element: Box::new(self.header_pattern(file, element, span, scope, patterns)?),
                    count,
                }
            }
            T::Application(application) => patterns
                .get(&(application.span.start, application.span.end))
                .cloned()
                .ok_or_else(|| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            application.span,
                            "record application pattern was not prepared",
                        ),
                    )
                })?,
            T::Procedure(procedure) => diagnostic_bridge(self.graph, |diagnostic| {
                crate::overloads::procedure_pattern(procedure, span, |ty, span| {
                    self.header_pattern(file, ty, span, scope, patterns)
                        .map_err(&mut *diagnostic)
                        .map_err(|error| Diagnostic::at_source(error.location, error.message))
                })
                .map_err(|error| located(self.graph, file, error))
            })?,
            _ => TypePattern::Concrete(self.resolve(file, ty, scope.substitution, span)?),
        })
    }
    fn application_pattern(
        &mut self,
        caller_file: FileInstanceId,
        application: &syntax::TypeApplicationSyntax,
        pattern_scope: &PatternScope<'_>,
        patterns: &HashMap<(usize, usize), TypePattern>,
    ) -> TypeResult<TypePattern> {
        let T::Named(base) = application.base.as_ref() else {
            return Err(failure(
                self.graph,
                caller_file,
                Diagnostic::new(
                    application.span,
                    "record application pattern requires a named template",
                ),
            ));
        };
        let declaration_id = match self
            .lexical
            .filter(|_| self.lexical_active)
            .and_then(|lexical| {
                lexical
                    .templates
                    .get(&(application.span.start, application.span.end))
            })
            .copied()
        {
            Some(declaration) => declaration,
            None => template_origin(self.graph, caller_file, base, application.span)
                .map_err(|error| failure(self.graph, caller_file, error))?,
        };
        let declaration = self.graph.declaration(declaration_id).unwrap();
        let syntax::FileDeclarationKind::Record(record) = &declaration.syntax().kind else {
            unreachable!("template origin denotes a record");
        };
        let bound =
            binder::bind_arguments(&record.parameters, &application.arguments, application.span)
                .map_err(|error| failure(self.graph, caller_file, error))?;
        let mut scope = Substitution::default();
        let mut arguments = Vec::new();
        let mut unbound_types = HashSet::new();
        for bound in bound {
            if bound.defaulted {
                match self.parameter_type(declaration.file(), bound.parameter, &scope) {
                    Ok(expected) => match self.baked(
                        declaration.file(),
                        bound.expression,
                        expected,
                        Some(&scope),
                    ) {
                        Ok(value) => {
                            scope.bind_constant(bound.parameter.name, value);
                        }
                        Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                        Err(TypeFailure::Diagnostic(_)) => {}
                    },
                    Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                    Err(TypeFailure::Diagnostic(_)) => {}
                }
                arguments.push(NominalArgumentPattern {
                    name: bound.parameter.name,
                    kind: NominalArgumentKind::Default,
                });
                continue;
            }
            let expected = match self.parameter_type(declaration.file(), bound.parameter, &scope) {
                Ok(ty) => Some(ty),
                Err(pending @ TypeFailure::Pending(_)) => return Err(pending),
                Err(TypeFailure::Diagnostic(_))
                    if matches!(&bound.expression.kind, E::CompileVariable(_) | E::Name(_))
                        && matches!(&bound.parameter.binding, syntax::RecordParameterBinding::Typed {ty, ..} if depends_on(ty, &unbound_types)) =>
                {
                    None
                }
                Err(error) => return Err(error),
            };
            let file = if bound.defaulted {
                declaration.file()
            } else {
                caller_file
            };
            let kind = if expected == Some(self.types.meta_type()) {
                let ty = type_expression(bound.expression).ok_or_else(|| {
                    failure(
                        self.graph,
                        file,
                        Diagnostic::new(
                            bound.expression.span,
                            "record template argument requires a type pattern",
                        ),
                    )
                })?;
                let pattern =
                    self.header_pattern(file, &ty, bound.expression.span, pattern_scope, patterns)?;
                if let TypePattern::Concrete(ty) = pattern {
                    scope.bind_constant(bound.parameter.name, BakedValue::Type(ty));
                } else {
                    unbound_types.insert(bound.parameter.name);
                }
                NominalArgumentKind::Type(Box::new(pattern))
            } else {
                match &bound.expression.kind {
                    E::CompileVariable(name)
                        if pattern_scope
                            .substitution
                            .and_then(|scope| scope.constant(*name))
                            .is_none()
                            && !self.module_parameter(file, *name) =>
                    {
                        NominalArgumentKind::InferValue(*name)
                    }
                    E::Name(name)
                        if pattern_scope.variables.contains(name)
                            && pattern_scope
                                .substitution
                                .and_then(|scope| scope.constant(*name))
                                .is_none()
                            && !self.module_parameter(file, *name) =>
                    {
                        NominalArgumentKind::ValueVariable(*name)
                    }
                    _ => {
                        let evaluate = &mut self.evaluate;
                        let mut defaults = super::super::defaults::Defaults::with_evaluator(
                            self.graph,
                            self.types,
                            self.nominals,
                            &mut **evaluate,
                        )
                        .with_specializations(self.records)
                        .with_substitution(if bound.defaulted {
                            Some(scope.clone())
                        } else {
                            pattern_scope.substitution.cloned()
                        });
                        let expected = expected.ok_or_else(|| failure(self.graph, file, Diagnostic::new(bound.expression.span,
                            "concrete baked argument requires its preceding type parameter to be resolved")))?;
                        let value = defaults.expression(file, bound.expression, expected)?;
                        let value = BakedValue::runtime(value, self.types).map_err(|error| {
                            failure(
                                self.graph,
                                file,
                                Diagnostic::new(bound.expression.span, error.to_string()),
                            )
                        })?;
                        scope.bind_constant(bound.parameter.name, value.clone());
                        NominalArgumentKind::Value(value)
                    }
                }
            };
            arguments.push(NominalArgumentPattern {
                name: bound.parameter.name,
                kind,
            });
        }
        if record.modify.is_some() {
            if arguments
                .iter()
                .all(|argument| bound_nominal_argument(&argument.kind))
            {
                return self
                    .application(caller_file, application, pattern_scope.substitution)
                    .map(TypePattern::Concrete);
            }
            if arguments
                .iter()
                .any(|argument| matches!(argument.kind, NominalArgumentKind::Default))
            {
                return Err(failure(
                    self.graph,
                    caller_file,
                    Diagnostic::new(
                        pattern_scope.owner_span,
                        "an inferred modified-record pattern requires a checked default recipe",
                    ),
                ));
            }
        }
        Ok(TypePattern::NominalApplication {
            declaration: declaration_id,
            arguments,
        })
    }
}

fn bound_nominal_argument(argument: &NominalArgumentKind) -> bool {
    match argument {
        NominalArgumentKind::Type(pattern) => bound_pattern(pattern),
        NominalArgumentKind::Value(_) | NominalArgumentKind::Default => true,
        NominalArgumentKind::InferValue(_) | NominalArgumentKind::ValueVariable(_) => false,
    }
}

fn bound_pattern(pattern: &TypePattern) -> bool {
    match pattern {
        TypePattern::Concrete(_) => true,
        TypePattern::Infer(_) | TypePattern::Variable(_) => false,
        TypePattern::Restricted { ty, restriction } => {
            bound_pattern(ty)
                && match restriction {
                    TypeRestrictionPattern::Nominal(pattern) => bound_pattern(pattern),
                    TypeRestrictionPattern::Interface(_) => true,
                }
        }
        TypePattern::Pointer(inner)
        | TypePattern::Slice(inner)
        | TypePattern::DynamicArray(inner) => bound_pattern(inner),
        TypePattern::FixedArray { element, count } => {
            bound_pattern(element) && matches!(count, CountPattern::Exact(_))
        }
        TypePattern::Procedure(procedure) => procedure
            .parameters
            .iter()
            .chain(&procedure.results)
            .all(bound_pattern),
        TypePattern::NominalApplication { arguments, .. } => arguments
            .iter()
            .all(|argument| bound_nominal_argument(&argument.kind)),
    }
}

fn depends_on(ty: &T, names: &HashSet<jai_source::Symbol>) -> bool {
    match ty {
        T::Named(path) => names.contains(&path.root),
        T::Variable(name) => names.contains(name),
        T::Restricted {
            variable,
            restriction,
            ..
        } => {
            names.contains(variable)
                || match restriction {
                    syntax::TypeRestrictionSyntax::Nominal(ty)
                    | syntax::TypeRestrictionSyntax::Interface(ty) => depends_on(ty, names),
                }
        }
        T::Pointer(inner) | T::Slice(inner) | T::DynamicArray(inner) => depends_on(inner, names),
        T::FixedArray { element, .. } => depends_on(element, names),
        T::Procedure(procedure) => procedure
            .parameters
            .iter()
            .chain(&procedure.results)
            .any(|parameter| depends_on(&parameter.ty, names)),
        _ => false,
    }
}
