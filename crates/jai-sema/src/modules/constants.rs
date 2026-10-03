//! Resolve scalar constant dependencies by declaration identity.
use super::*;
mod preparation;
pub(crate) use preparation::ScalarPreparation;
use syntax::{Expression, ExpressionKind};
#[derive(Clone)]
enum ConstantState {
    Visiting,
    Ready(ConstantValue),
}
enum Work {
    Enter(DeclarationId, SourceSpan),
    Finish(DeclarationId),
}
#[derive(Clone, Copy)]
enum PrimitiveAnnotation {
    Scalar(ScalarType),
    Float(jai_types::FloatType),
}
pub(super) struct Constants<'a> {
    graph: &'a ModuleGraph,
    states: HashMap<DeclarationId, ConstantState>,
    enum_members: HashMap<(DeclarationId, Symbol), IntegerValue>,
    nominal_types: HashMap<DeclarationId, TypeId>,
    annotations: HashMap<DeclarationId, TypeId>,
    primitive_annotations: HashMap<DeclarationId, PrimitiveAnnotation>,
    checked_typed_values: HashMap<DeclarationId, TypeId>,
}
impl<'a> Constants<'a> {
    pub(super) fn new(graph: &'a ModuleGraph) -> Self {
        Self {
            graph,
            states: HashMap::new(),
            enum_members: HashMap::new(),
            nominal_types: HashMap::new(),
            annotations: HashMap::new(),
            primitive_annotations: HashMap::new(),
            checked_typed_values: HashMap::new(),
        }
    }
    pub(super) fn register_nominal_type(&mut self, declaration: DeclarationId, ty: TypeId) {
        self.nominal_types.insert(declaration, ty);
    }
    /// Primitive aliases need no value evaluation, so their canonical facts can
    /// precede array counts and other value-dependent type declarations.
    pub(super) fn register_startup_annotations(
        &mut self,
        types: &mut TypeRegistry,
    ) -> Result<(), LocatedDiagnostic> {
        for declaration in self.graph.declarations() {
            let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                continue;
            };
            let Some(annotation) = &constant.ty else {
                continue;
            };
            let Some(primitive) = self.primitive_annotation(declaration.file(), annotation) else {
                continue;
            };
            let ty = match primitive {
                PrimitiveAnnotation::Scalar(scalar) => types.scalar(scalar),
                PrimitiveAnnotation::Float(float) => types.float(float),
            };
            self.register_annotation(declaration.id(), ty, types)
                .map_err(|error| {
                    located(
                        self.graph,
                        declaration.file(),
                        Diagnostic::new(constant.span, error.to_string()),
                    )
                })?;
        }
        Ok(())
    }
    fn primitive_annotation(
        &self,
        mut file: FileInstanceId,
        annotation: &syntax::TypeSyntax,
    ) -> Option<PrimitiveAnnotation> {
        let mut syntax = annotation.clone();
        let mut visiting = std::collections::HashSet::new();
        loop {
            match syntax {
                syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(scalar)) => {
                    return Some(PrimitiveAnnotation::Scalar(scalar));
                }
                syntax::TypeSyntax::Builtin(syntax::BuiltinType::Float(float)) => {
                    return Some(PrimitiveAnnotation::Float(float));
                }
                syntax::TypeSyntax::Named(ref path) => match self.graph.lookup(file, path).ok()? {
                    jai_modules::Binding::Declaration(id) => {
                        if !visiting.insert(id) {
                            return None;
                        }
                        let declaration = self.graph.declaration(id)?;
                        syntax = match &declaration.syntax().kind {
                            FileDeclarationKind::TypeAlias(alias) => alias.ty.clone(),
                            FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                                super::aggregates::parameterized::type_expression(
                                    &constant.initializer,
                                )?
                            }
                            _ => return None,
                        };
                        file = declaration.file();
                    }
                    _ => return None,
                },
                // Nominal wrappers and aggregate types require their real typed phase.
                _ => return None,
            }
        }
    }
    /// Bound module types already have canonical registry identities before
    /// value-dependent aliases are defined. Publish those annotation facts
    /// without evaluating an initializer or resolving an unfinished alias.
    pub(super) fn register_module_parameter_annotations(
        &mut self,
        nominals: &super::aggregates::Nominals<'_>,
        types: &TypeRegistry,
    ) -> Result<(), LocatedDiagnostic> {
        for declaration in self.graph.declarations() {
            let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                continue;
            };
            let Some(annotation) = &constant.ty else {
                continue;
            };
            let Some(ty) =
                self.module_parameter_annotation(declaration.file(), annotation, nominals)
            else {
                continue;
            };
            self.register_annotation(declaration.id(), ty, types)
                .map_err(|error| {
                    located(
                        self.graph,
                        declaration.file(),
                        Diagnostic::new(constant.span, error.to_string()),
                    )
                })?;
        }
        Ok(())
    }
    fn module_parameter_annotation(
        &self,
        mut file: FileInstanceId,
        annotation: &syntax::TypeSyntax,
        nominals: &super::aggregates::Nominals<'_>,
    ) -> Option<TypeId> {
        let mut syntax = annotation.clone();
        let mut visiting = std::collections::HashSet::new();
        loop {
            let path = match &syntax {
                syntax::TypeSyntax::Named(path) => path.clone(),
                syntax::TypeSyntax::Variable(name) => NamePath {
                    root: *name,
                    members: Vec::new(),
                },
                _ => return None,
            };
            match self.graph.lookup(file, &path).ok()? {
                jai_modules::Binding::Parameter(id) => {
                    return nominals.module_parameter_types.get(&id).copied();
                }
                jai_modules::Binding::Declaration(id) => {
                    if !visiting.insert(id) {
                        return None;
                    }
                    let declaration = self.graph.declaration(id)?;
                    syntax = match &declaration.syntax().kind {
                        FileDeclarationKind::TypeAlias(alias) => alias.ty.clone(),
                        FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                            super::aggregates::parameterized::type_expression(
                                &constant.initializer,
                            )?
                        }
                        _ => return None,
                    };
                    file = declaration.file();
                }
                _ => return None,
            }
        }
    }
    pub(super) fn nominal_type(&self, declaration: DeclarationId) -> Option<TypeId> {
        self.nominal_types.get(&declaration).copied()
    }
    pub(super) fn annotation(&self, declaration: DeclarationId) -> Option<TypeId> {
        self.annotations.get(&declaration).copied()
    }
    pub(super) fn has_typed_annotation(&self, declaration: DeclarationId) -> bool {
        self.annotations.contains_key(&declaration)
            && !self.primitive_annotations.contains_key(&declaration)
    }
    pub(super) fn register_annotation(
        &mut self,
        declaration: DeclarationId,
        ty: TypeId,
        types: &TypeRegistry,
    ) -> Result<(), jai_types::TypeError> {
        self.annotations.insert(declaration, ty);
        self.register_nominal_type(declaration, ty);
        let primitive = match types.kind(ty)? {
            jai_types::TypeKind::Integer(integer) => {
                Some(PrimitiveAnnotation::Scalar(ScalarType::Int(*integer)))
            }
            jai_types::TypeKind::Bool => Some(PrimitiveAnnotation::Scalar(ScalarType::Bool)),
            jai_types::TypeKind::Float(float) => Some(PrimitiveAnnotation::Float(*float)),
            _ => None,
        };
        if let Some(primitive) = primitive {
            self.primitive_annotations.insert(declaration, primitive);
        }
        Ok(())
    }
    pub(super) fn register_enums(&mut self, nominals: &super::aggregates::Nominals<'_>) {
        for (declaration, ty) in &nominals.declarations {
            if let Some(enumeration) = nominals.enums.get(ty) {
                for (name, value) in &enumeration.members {
                    self.enum_members.insert((*declaration, *name), *value);
                }
            }
        }
    }
    fn external_value(
        &self,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
    ) -> Result<Option<ConstantValue>, LocatedDiagnostic> {
        if path.members.is_empty()
            && let Some(value) = self.graph.insertion_capture_value(file, path.root)
        {
            return match value {
                jai_modules::SourceCaptureValue::Scalar(value) => Ok(Some(value.clone())),
                _ => Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "non-scalar source capture requires typed constant resolution",
                    ),
                )),
            };
        }
        if path.members.is_empty()
            && self.graph.lookup(file, path).is_err()
            && let Some(value) = self.target_constant(file, path.root, span)?
        {
            return Ok(Some(value));
        }
        if let Ok(jai_modules::Binding::Parameter(id)) = self.graph.lookup(file, path) {
            return match &self.graph.parameter(id).unwrap().value {
                jai_modules::ParameterValue::Scalar(value) => Ok(Some(value.clone())),
                jai_modules::ParameterValue::String(_)
                | jai_modules::ParameterValue::Enumeration(_)
                | jai_modules::ParameterValue::ContextualMember(_)
                | jai_modules::ParameterValue::Type(_) => Err(located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "non-scalar module parameter requires typed constant resolution",
                    ),
                )),
            };
        }
        if let Some((&name, parents)) = path.members.split_last() {
            let parent = NamePath {
                root: path.root,
                members: parents.to_vec(),
            };
            if let Ok(jai_modules::Binding::Declaration(id)) = self.graph.lookup(file, &parent)
                && let Some(value) = self.enum_members.get(&(id, name))
            {
                return Ok(Some(ConstantValue::Int(*value)));
            }
        }
        Ok(None)
    }
    fn target_constant(
        &self,
        file: FileInstanceId,
        name: Symbol,
        span: Span,
    ) -> Result<Option<ConstantValue>, LocatedDiagnostic> {
        let Some(target) = self.graph.target() else {
            return Ok(None);
        };
        let (enum_name, member_name) = match self.graph.symbols().name(name) {
            "OS" | "BUILD_OS" => ("Operating_System_Tag", target.operating_system.source_tag()),
            "CPU" | "BUILD_CPU" => ("CPU_Tag", target.architecture.source_tag()),
            _ => return Ok(None),
        };
        let member_name = member_name.ok_or_else(|| {
            located(
                self.graph,
                file,
                Diagnostic::new(span, "selected target has no source-defined OS or CPU tag"),
            )
        })?;
        let enum_name = self.graph.symbols().find(enum_name).ok_or_else(|| {
            located(
                self.graph,
                file,
                Diagnostic::new(span, "target tag enum must be supplied by source"),
            )
        })?;
        let enum_file = self
            .graph
            .prelude()
            .map(|module| self.graph.module(module).unwrap().entry())
            .unwrap_or(file);
        let jai_modules::Binding::Declaration(declaration) = self
            .graph
            .lookup(
                enum_file,
                &NamePath {
                    root: enum_name,
                    members: Vec::new(),
                },
            )
            .map_err(|_| {
                located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "target tag enum is absent from the designated source scope",
                    ),
                )
            })?
        else {
            return Err(located(
                self.graph,
                file,
                Diagnostic::new(
                    span,
                    "target tag schema must denote a source enum declaration",
                ),
            ));
        };
        let member = self.graph.symbols().find(member_name).ok_or_else(|| {
            located(
                self.graph,
                file,
                Diagnostic::new(span, "selected target tag is absent from the source enum"),
            )
        })?;
        let value = self
            .enum_members
            .get(&(declaration, member))
            .copied()
            .ok_or_else(|| {
                located(
                    self.graph,
                    file,
                    Diagnostic::new(
                        span,
                        "selected target tag is absent from the resolved source enum",
                    ),
                )
            })?;
        Ok(Some(ConstantValue::Int(value)))
    }
    pub(super) fn lookup_value(
        &mut self,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
    ) -> Result<ConstantValue, Diagnostic> {
        if let Some(value) = self
            .external_value(file, path, span)
            .map_err(|error| Diagnostic::at_source(error.location, error.message))?
        {
            return Ok(value);
        }
        let id = declaration_id(self.graph, file, path, span)?;
        self.value(
            id,
            SourceSpan {
                source: self.graph.file(file).unwrap().source(),
                span,
            },
        )
        .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }
    pub(super) fn ready(
        &self,
        id: DeclarationId,
        site: SourceSpan,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        match self.states.get(&id) {
            Some(ConstantState::Ready(value)) => Ok(value.clone()),
            Some(ConstantState::Visiting) => Err(LocatedDiagnostic {
                location: site,
                message: "cyclic constant dependencies".into(),
            }),
            None => Err(LocatedDiagnostic {
                location: site,
                message: "declaration cannot supply a compile-time constant".into(),
            }),
        }
    }
    pub(super) fn value(
        &mut self,
        id: DeclarationId,
        site: SourceSpan,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        let mut work = vec![Work::Enter(id, site)];
        while let Some(step) = work.pop() {
            match step {
                Work::Enter(id, site) => {
                    match self.states.get(&id) {
                        Some(ConstantState::Ready(_)) => continue,
                        Some(ConstantState::Visiting) => {
                            self.ready(id, site)?;
                            unreachable!();
                        }
                        None => {}
                    }
                    let declaration = self
                        .graph
                        .declaration(id)
                        .expect("resolved declaration exists");
                    let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                        return self.ready(id, site);
                    };
                    self.states.insert(id, ConstantState::Visiting);
                    work.push(Work::Finish(id));
                    let file = declaration.file();
                    let source = declaration.location().source;
                    let mut expressions = vec![&constant.initializer];
                    let mut dependencies = Vec::new();
                    while let Some(expression) = expressions.pop() {
                        let name = match &expression.kind {
                            ExpressionKind::Name(name) => Some(path(*name)),
                            ExpressionKind::QualifiedName(path) => Some(path.clone()),
                            ExpressionKind::Unary(_, inner) | ExpressionKind::Cast(_, _, inner) => {
                                expressions.push(inner);
                                None
                            }
                            ExpressionKind::Binary(_, left, right) => {
                                expressions.push(right);
                                expressions.push(left);
                                None
                            }
                            ExpressionKind::Conditional(conditional) => {
                                if let Some(otherwise) = &conditional.else_value {
                                    expressions.push(otherwise);
                                }
                                expressions.push(&conditional.then_value);
                                expressions.push(&conditional.condition);
                                None
                            }
                            _ => None,
                        };
                        if let Some(path) = name {
                            if self.external_value(file, &path, expression.span)?.is_some() {
                                continue;
                            }
                            let dependency =
                                declaration_id(self.graph, file, &path, expression.span)
                                    .map_err(|error| located(self.graph, file, error))?;
                            dependencies.push(Work::Enter(
                                dependency,
                                SourceSpan {
                                    source,
                                    span: expression.span,
                                },
                            ));
                        }
                    }
                    work.extend(dependencies.into_iter().rev());
                }
                Work::Finish(id) => {
                    let declaration = self.graph.declaration(id).unwrap();
                    let FileDeclarationKind::Constant(constant) = &declaration.syntax().kind else {
                        unreachable!()
                    };
                    let target = self.primitive_annotations.get(&id).copied().or(
                        match constant.ty.as_ref() {
                            Some(syntax::TypeSyntax::Builtin(syntax::BuiltinType::Scalar(ty))) => {
                                Some(PrimitiveAnnotation::Scalar(*ty))
                            }
                            Some(syntax::TypeSyntax::Builtin(syntax::BuiltinType::Float(ty))) => {
                                Some(PrimitiveAnnotation::Float(*ty))
                            }
                            _ => None,
                        },
                    );
                    let value = match target {
                        Some(PrimitiveAnnotation::Float(float)) => ConstantValue::Float(
                            jai_eval::floats::evaluate_float_paths(
                                &constant.initializer,
                                float,
                                |path, span| self.ready_scalar_path(declaration.file(), path, span),
                            )
                            .map_err(|error| located(self.graph, declaration.file(), error))?,
                        ),
                        Some(PrimitiveAnnotation::Scalar(ty)) => self
                            .evaluate(declaration.file(), &constant.initializer)?
                            .coerce(ty, constant.span)
                            .map_err(|error| located(self.graph, declaration.file(), error))?,
                        None if constant.ty.is_none() => {
                            self.evaluate(declaration.file(), &constant.initializer)?
                        }
                        None => {
                            return Err(located(
                                self.graph,
                                declaration.file(),
                                Diagnostic::new(
                                    constant.span,
                                    "typed constant annotation requires canonical semantic resolution",
                                ),
                            ));
                        }
                    };
                    self.states.insert(id, ConstantState::Ready(value));
                }
            }
        }
        self.ready(id, site)
    }
    pub(super) fn evaluate_lazy(
        &mut self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        jai_eval::evaluate_paths_with_overflow_check(
            expression,
            jai_types::CheckMode::Enabled,
            |path, span| self.lookup_value(file, path, span),
        )
        .map(|value| value.with_fallback_source(self.graph.file(file).unwrap().source()))
        .map_err(|error| located(self.graph, file, error))
    }
    pub(super) fn evaluate(
        &self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic> {
        jai_eval::evaluate_paths_with_overflow_check(
            expression,
            jai_types::CheckMode::Enabled,
            |path, span| self.ready_scalar_path(file, path, span),
        )
        .map(|value| value.with_fallback_source(self.graph.file(file).unwrap().source()))
        .map_err(|error| located(self.graph, file, error))
    }
    fn ready_scalar_path(
        &self,
        file: FileInstanceId,
        path: &NamePath,
        span: Span,
    ) -> Result<ConstantValue, Diagnostic> {
        if let Some(value) = self
            .external_value(file, path, span)
            .map_err(|error| Diagnostic::at_source(error.location, error.message))?
        {
            return Ok(value);
        }
        let id = declaration_id(self.graph, file, path, span)?;
        self.ready(
            id,
            SourceSpan {
                source: self.graph.file(file).unwrap().source(),
                span,
            },
        )
        .map_err(|error| Diagnostic::at_source(error.location, error.message))
    }
}
