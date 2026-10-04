//! Source-bound type identities are materialized by the existing semantic registry.
use super::*;
mod retained_metadata;
use jai_syntax::{BuiltinType, Expression, ExpressionKind, TypeSyntax};
use jai_types::{CallingConvention, ContextMode, FloatType, ScalarType};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModuleBuiltin {
    Scalar(ScalarType),
    Float(FloatType),
    String,
    Void,
    Type,
    Context,
    Any,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ModuleType {
    Builtin(ModuleBuiltin),
    Declaration(DeclarationId),
    /// Actual template identity and normalized values in formal declaration order.
    Application {
        template: DeclarationId,
        arguments: Vec<ModuleBoundArgument>,
    },
    Pointer(Box<ModuleType>),
    Slice(Box<ModuleType>),
    DynamicArray(Box<ModuleType>),
    FixedArray {
        element: Box<ModuleType>,
        count: u64,
    },
    Procedure(ModuleProcedureType),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ModuleProcedureType {
    pub parameters: Vec<ModuleType>,
    pub results: Vec<ModuleType>,
    pub convention: CallingConvention,
    pub return_abi: jai_types::ForeignReturnAbi,
    pub context: ContextMode,
    pub variadic: ModuleVariadic,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ModuleVariadic {
    None,
    C { fixed_parameters: usize },
    Jai { parameter: usize },
}

impl From<BuiltinType> for ModuleBuiltin {
    fn from(value: BuiltinType) -> Self {
        match value {
            BuiltinType::Scalar(ty) => Self::Scalar(ty),
            BuiltinType::Float(ty) => Self::Float(ty),
            BuiltinType::String => Self::String,
            BuiltinType::Void => Self::Void,
            BuiltinType::Type => Self::Type,
            BuiltinType::Context => Self::Context,
            BuiltinType::Any => Self::Any,
        }
    }
}
impl Builder<'_> {
    pub(super) fn bind_module_type(
        &self,
        file: FileInstanceId,
        syntax: &TypeSyntax,
        span: jai_source::Span,
        active: &mut Vec<DeclarationId>,
    ) -> Result<ModuleType, GraphError> {
        let location = SourceSpan {
            source: self.graph.files[file.index()].source,
            span,
        };
        let pending = |message| {
            let diagnostic = self.graph.diagnostic(location, message);
            let rendered = diagnostic.render(&self.graph.sources);
            GraphError::Pending {
                diagnostic,
                rendered,
            }
        };
        let root = match syntax {
            TypeSyntax::Variable(name) => Some(*name),
            TypeSyntax::Named(path) if path.members.is_empty() => Some(path.root),
            _ => None,
        };
        if let Some(name) = root
            && let Some(argument) = self.specialization_argument(file, span, name)
        {
            return match argument {
                ModuleBoundArgument::Type(ty) => Ok(ty.clone()),
                _ => Err(self.located(location, "specialization parameter does not denote a type")),
            };
        }
        if let Some(name) = root
            && let Some(value) = self.graph.insertion_capture_value(file, name)
        {
            return match value {
                SourceCaptureValue::Type(value) => Ok(value.clone()),
                _ => Err(self.located(location, "insertion capture does not denote a type")),
            };
        }
        Ok(match syntax {
            TypeSyntax::Builtin(ty) => ModuleType::Builtin((*ty).into()),
            TypeSyntax::Variable(name) => {
                let binding = self
                    .graph
                    .lookup(
                        file,
                        &NamePath {
                            root: *name,
                            members: vec![],
                        },
                    )
                    .map_err(|_| pending("module type variable is not bound yet"))?;
                let Binding::Parameter(id) = binding else {
                    return Err(self.located(
                        location,
                        "module type variable must refer to a type parameter",
                    ));
                };
                let ParameterValue::Type(value) = &self.graph.parameters[id.index()].value else {
                    return Err(self.located(location, "module parameter does not denote a type"));
                };
                value.clone()
            }
            TypeSyntax::Named(path) => {
                let binding = self
                    .graph
                    .lookup(file, path)
                    .map_err(|_| pending("module type declaration is not available yet"))?;
                match binding {
                    Binding::Parameter(id) => match &self.graph.parameters[id.index()].value {
                        ParameterValue::Type(value) => value.clone(),
                        _ => {
                            return Err(
                                self.located(location, "module parameter does not denote a type")
                            );
                        }
                    },
                    Binding::Declaration(id) => {
                        let declaration = &self.graph.declarations[id.index()];
                        if active.contains(&id) {
                            return Err(self.located(
                                declaration.location(),
                                "cyclic module type alias dependency",
                            ));
                        }
                        match &declaration.syntax.kind {
                            FileDeclarationKind::Record(_) | FileDeclarationKind::Enum(_) => {
                                ModuleType::Declaration(id)
                            }
                            FileDeclarationKind::TypeAlias(alias)
                                if matches!(alias.ty, TypeSyntax::Variant { .. }) =>
                            {
                                ModuleType::Declaration(id)
                            }
                            FileDeclarationKind::TypeAlias(alias) => {
                                active.push(id);
                                let value = self.bind_module_type(
                                    declaration.file,
                                    &alias.ty,
                                    alias.span,
                                    active,
                                );
                                active.pop();
                                value?
                            }
                            FileDeclarationKind::Constant(constant) if constant.ty.is_none() => {
                                active.push(id);
                                let value = self.module_type_expression(
                                    declaration.file,
                                    &constant.initializer,
                                    active,
                                );
                                active.pop();
                                value?.ok_or_else(|| {
                                    self.located(
                                        declaration.location(),
                                        "module argument declaration does not denote a type",
                                    )
                                })?
                            }
                            _ => {
                                return Err(self.located(
                                    location,
                                    "module argument declaration does not denote a type",
                                ));
                            }
                        }
                    }
                    _ => {
                        return Err(self.located(
                            location,
                            "module namespace or procedure does not denote a type",
                        ));
                    }
                }
            }
            TypeSyntax::Pointer(inner) => {
                ModuleType::Pointer(Box::new(self.bind_module_type(file, inner, span, active)?))
            }
            TypeSyntax::Slice(inner) => {
                ModuleType::Slice(Box::new(self.bind_module_type(file, inner, span, active)?))
            }
            TypeSyntax::DynamicArray(inner) => ModuleType::DynamicArray(Box::new(
                self.bind_module_type(file, inner, span, active)?,
            )),
            TypeSyntax::FixedArray {
                count,
                element,
            } => {
                let value = self.constant_expression(file, count, &mut vec![])?;
                let count = match value {
                    jai_eval::Value::Literal(value) => u64::try_from(value).ok(),
                    jai_eval::Value::Int(value) => u64::try_from(value.value()).ok(),
                    _ => None,
                }
                .ok_or_else(|| {
                    self.located(
                        location,
                        "module fixed array count must be a nonnegative integer",
                    )
                })?;
                ModuleType::FixedArray {
                    element: Box::new(self.bind_module_type(file, element, span, active)?),
                    count,
                }
            }
            TypeSyntax::Procedure(procedure) => {
                let mut parameters = Vec::new();
                let mut variadic = ModuleVariadic::None;
                for (index, parameter) in procedure.parameters.iter().enumerate() {
                    if parameter.variadic && procedure.convention == CallingConvention::C {
                        variadic = ModuleVariadic::C {
                            fixed_parameters: index,
                        };
                        continue;
                    }
                    let ty = &parameter.ty;
                    parameters.push(self.bind_module_type(file, ty, parameter.span, active)?);
                    if parameter.variadic {
                        variadic = ModuleVariadic::Jai {
                            parameter: index,
                        };
                    }
                }
                let mut results = Vec::new();
                for result in &procedure.results {
                    results.push(self.bind_module_type(file, &result.ty, result.span, active)?);
                }
                normalize_module_results(&mut results);
                ModuleType::Procedure(ModuleProcedureType {
                    parameters,
                    results,
                    return_abi: procedure.return_abi,
                    convention: procedure.convention,
                    context: procedure.context,
                    variadic,
                })
            }
            TypeSyntax::This => {
                return Err(self.located(
                    location,
                    "#this type requires an enclosing record field annotation",
                ));
            }
            TypeSyntax::Restricted {
                ..
            } => {
                let ParameterResponse::Type(ty) = self.semantic_parameter(
                    file,
                    location,
                    ParameterTask::ResolveType {
                        syntax: syntax.clone(),
                    },
                    "module type restriction requires semantic constraint proof",
                )?
                else {
                    unreachable!("type response shape was checked on submission")
                };
                ty
            }
            TypeSyntax::TypeOf(_) => {
                let ParameterResponse::Type(ty) = self.semantic_parameter(
                    file,
                    location,
                    ParameterTask::ResolveType {
                        syntax: syntax.clone(),
                    },
                    "module type query requires semantic expression type resolution",
                )?
                else {
                    unreachable!("type response shape was checked on submission")
                };
                ty
            }
            TypeSyntax::InlineRecord(_)
            | TypeSyntax::InlineEnum(_)
            | TypeSyntax::Variant {
                ..
            }
            | TypeSyntax::Application(_) => {
                let ParameterResponse::Type(ty) = self.semantic_parameter(
                    file,
                    location,
                    ParameterTask::ResolveType {
                        syntax: syntax.clone(),
                    },
                    "module type value requires semantic nominal specialization",
                )?
                else {
                    unreachable!("type response shape was checked on submission")
                };
                ty
            }
        })
    }
    pub(super) fn module_type_expression(
        &self,
        file: FileInstanceId,
        expression: &Expression,
        active: &mut Vec<DeclarationId>,
    ) -> Result<Option<ModuleType>, GraphError> {
        let syntax = match &expression.kind {
            ExpressionKind::Type(ty) => Some(ty.clone()),
            ExpressionKind::TypeQuery {
                query: jai_syntax::TypeQueryKind::TypeOf,
                value,
            } => Some(TypeSyntax::TypeOf(value.clone())),
            ExpressionKind::CompileVariable(name) => Some(TypeSyntax::Variable(*name)),
            ExpressionKind::Name(root) => Some(
                BuiltinType::from_spelling(self.graph.symbols.name(*root))
                    .map(TypeSyntax::Builtin)
                    .unwrap_or_else(|| {
                        TypeSyntax::Named(NamePath {
                            root: *root,
                            members: vec![],
                        })
                    }),
            ),
            ExpressionKind::QualifiedName(path) => Some(TypeSyntax::Named(path.clone())),
            ExpressionKind::Call(root, arguments) => self.module_type_application(
                file,
                NamePath {
                    root: *root,
                    members: vec![],
                },
                arguments,
                expression.span,
            ),
            ExpressionKind::QualifiedCall(path, arguments) => {
                self.module_type_application(file, path.clone(), arguments, expression.span)
            }
            ExpressionKind::AddressOf(inner) => {
                return self
                    .module_type_expression(file, inner, active)
                    .map(|value| value.map(|value| ModuleType::Pointer(Box::new(value))));
            }
            _ => None,
        };
        let Some(syntax) = syntax else {
            return Ok(None);
        };
        // A name may also be an ordinary value; only declarations known to denote types enter this path.
        if let TypeSyntax::Named(path) = &syntax {
            if path.members.is_empty()
                && let Some(value) = self.graph.insertion_capture_value(file, path.root)
            {
                return match value {
                    SourceCaptureValue::Type(value) => Ok(Some(value.clone())),
                    _ => Ok(None),
                };
            }
            match self.graph.lookup(file, path) {
                Ok(Binding::Declaration(id))
                    if matches!(
                        self.graph.declarations[id.index()].syntax.kind,
                        FileDeclarationKind::Record(_)
                            | FileDeclarationKind::Enum(_)
                            | FileDeclarationKind::TypeAlias(_)
                    ) => {}
                Ok(Binding::Parameter(id))
                    if matches!(
                        self.graph.parameters[id.index()].value,
                        ParameterValue::Type(_)
                    ) => {}
                Ok(Binding::Declaration(id)) => {
                    if let FileDeclarationKind::Constant(constant) =
                        &self.graph.declarations[id.index()].syntax.kind
                        && constant.ty.is_none()
                    {
                        if active.contains(&id) {
                            return Err(self.located(
                                self.graph.declarations[id.index()].location(),
                                "cyclic module type alias dependency",
                            ));
                        }
                        active.push(id);
                        let value = self.module_type_expression(
                            self.graph.declarations[id.index()].file,
                            &constant.initializer,
                            active,
                        );
                        active.pop();
                        return value;
                    }
                    return Ok(None);
                }
                _ => return Ok(None),
            }
        }
        self.bind_module_type(file, &syntax, expression.span, active)
            .map(Some)
    }
    fn module_type_application(
        &self,
        file: FileInstanceId,
        path: NamePath,
        arguments: &[jai_syntax::CallArgument],
        span: jai_source::Span,
    ) -> Option<TypeSyntax> {
        let Ok(Binding::Declaration(id)) = self.graph.lookup(file, &path) else {
            return None;
        };
        if !matches!(&self.graph.declarations[id.index()].syntax.kind, FileDeclarationKind::Record(record) if !record.parameters.is_empty())
        {
            return None;
        }
        Some(TypeSyntax::Application(jai_syntax::TypeApplicationSyntax {
            base: Box::new(TypeSyntax::Named(path)),
            arguments: arguments.to_vec(),
            span,
        }))
    }
}

impl Builder<'_> {
    pub(super) fn validate_module_interface(
        &self,
        file: FileInstanceId,
        actual: &ModuleType,
        constraint: &ModuleType,
        location: SourceSpan,
    ) -> Result<(), GraphError> {
        match self.validate_module_interface_pure(file, actual, constraint, location) {
            Err(GraphError::Pending {
                diagnostic, ..
            }) => {
                let response = self.semantic_parameter(
                    file,
                    location,
                    ParameterTask::CheckInterface {
                        actual: actual.clone(),
                        required: constraint.clone(),
                    },
                    &diagnostic.message,
                )?;
                debug_assert!(matches!(response, ParameterResponse::InterfaceSatisfied));
                Ok(())
            }
            result => result,
        }
    }
    fn validate_module_interface_pure(
        &self,
        _file: FileInstanceId,
        actual: &ModuleType,
        constraint: &ModuleType,
        location: SourceSpan,
    ) -> Result<(), GraphError> {
        let (ModuleType::Declaration(actual), ModuleType::Declaration(constraint)) =
            (actual, constraint)
        else {
            if matches!(actual, ModuleType::Application { .. })
                || matches!(constraint, ModuleType::Application { .. })
            {
                return Err(self.interface_pending(
                    location,
                    "generic module interfaces require semantic specialization",
                ));
            }
            return Err(self.located(
                location,
                "interface-constrained module argument requires a named record type",
            ));
        };
        let actual = &self.graph.declarations[actual.index()];
        let constraint = &self.graph.declarations[constraint.index()];
        let (
            FileDeclarationKind::Record(actual_record),
            FileDeclarationKind::Record(required_record),
        ) = (&actual.syntax.kind, &constraint.syntax.kind)
        else {
            return Err(self.located(
                location,
                "module interface and replacement must be record types",
            ));
        };
        if actual_record.modify.is_some() || required_record.modify.is_some() {
            return Err(self.interface_pending(
                location,
                "modified module interface types require semantic constraint resolution",
            ));
        }
        if !actual_record.parameters.is_empty() || !required_record.parameters.is_empty() {
            return Err(self.interface_pending(
                location,
                "generic module interfaces require semantic specialization",
            ));
        }
        for required in &required_record.members {
            let (name, required_type) =
                self.interface_member_type(constraint.file, required, location)?;
            let mut candidates = Vec::new();
            for member in &actual_record.members {
                if interface_member_name(member) == Some(name) {
                    candidates.push(self.interface_member_type(actual.file, member, location)?.1);
                }
            }
            if candidates.is_empty() {
                if actual_record.fields().any(|field| field.using) {
                    return Err(self.interface_pending(
                        location,
                        "interface members supplied through using fields require semantic member resolution",
                    ));
                }
                return Err(self.located(
                    location,
                    format!(
                        "module type does not satisfy interface: missing member '{}'",
                        self.graph.symbols.name(name)
                    ),
                ));
            }
            if !candidates.contains(&required_type) {
                return Err(self.located(
                    location,
                    format!(
                        "module type does not satisfy interface: incompatible member '{}'",
                        self.graph.symbols.name(name)
                    ),
                ));
            }
        }
        Ok(())
    }
    fn interface_member_type(
        &self,
        file: FileInstanceId,
        member: &jai_syntax::RecordMember,
        location: SourceSpan,
    ) -> Result<(Symbol, ModuleType), GraphError> {
        use jai_syntax::{FieldBinding, RecordMember};
        match member {
            RecordMember::Procedure(procedure) => self.interface_signature(
                file,
                procedure.name,
                &procedure.parameters,
                &procedure.results,
                (
                    procedure.convention,
                    procedure.return_abi,
                    procedure.context,
                ),
                location,
            ),
            RecordMember::ProcedurePrototype(procedure) => self.interface_signature(
                file,
                procedure.name,
                &procedure.parameters,
                &procedure.results,
                (
                    procedure.convention,
                    procedure.return_abi,
                    procedure.context,
                ),
                location,
            ),
            RecordMember::Field(field) => {
                let FieldBinding::Explicit {
                    ty, ..
                } = &field.binding
                else {
                    return Err(self.interface_pending(
                        location,
                        "inferred interface fields require semantic type inference",
                    ));
                };
                self.bind_module_type(file, ty, field.span, &mut vec![])
                    .map(|ty| (field.name, ty))
            }
            _ => Err(self.interface_pending(
                location,
                "interface member requires semantic constraint resolution",
            )),
        }
    }
    fn interface_pending(&self, location: SourceSpan, message: &str) -> GraphError {
        let diagnostic = self.graph.diagnostic(location, message);
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Pending {
            diagnostic,
            rendered,
        }
    }
    fn interface_signature(
        &self,
        file: FileInstanceId,
        name: Symbol,
        parameters: &[jai_syntax::Parameter],
        results: &[jai_syntax::ProcedureResult],
        abi: (CallingConvention, jai_types::ForeignReturnAbi, ContextMode),
        location: SourceSpan,
    ) -> Result<(Symbol, ModuleType), GraphError> {
        use jai_syntax::{ParameterBinding as P, ResultBinding as R};
        let (convention, return_abi, context) = abi;
        let mut bound_parameters = Vec::new();
        let mut variadic = ModuleVariadic::None;
        for (index, parameter) in parameters.iter().enumerate() {
            if parameter.baking != jai_syntax::ParameterBaking::None {
                return Err(self.interface_pending(
                    location,
                    "generic interface methods require semantic specialization",
                ));
            }
            if parameter.variadic && convention == CallingConvention::C {
                variadic = ModuleVariadic::C {
                    fixed_parameters: index,
                };
                continue;
            }
            let ty = match &parameter.binding {
                P::Required(ty)
                | P::Defaulted {
                    ty: Some(ty), ..
                } => TypeSyntax::Builtin(BuiltinType::Scalar(*ty)),
                P::RequiredType(ty)
                | P::DefaultedType {
                    ty: Some(ty), ..
                } => ty.clone(),
                _ => {
                    return Err(self.interface_pending(
                        location,
                        "inferred interface parameters require semantic type inference",
                    ));
                }
            };
            bound_parameters.push(self.bind_module_type(file, &ty, parameter.span, &mut vec![])?);
            if parameter.variadic {
                variadic = ModuleVariadic::Jai {
                    parameter: index,
                };
            }
        }
        let mut bound_results = Vec::new();
        for result in results {
            let R::Typed {
                ty, ..
            } = &result.binding
            else {
                return Err(self.interface_pending(
                    location,
                    "inferred interface results require semantic type inference",
                ));
            };
            bound_results.push(self.bind_module_type(file, ty, result.span, &mut vec![])?);
        }
        normalize_module_results(&mut bound_results);
        Ok((
            name,
            ModuleType::Procedure(ModuleProcedureType {
                parameters: bound_parameters,
                results: bound_results,
                return_abi,
                convention,
                context,
                variadic,
            }),
        ))
    }
}
fn interface_member_name(member: &jai_syntax::RecordMember) -> Option<Symbol> {
    use jai_syntax::RecordMember;
    match member {
        RecordMember::Field(field) => Some(field.name),
        RecordMember::Procedure(procedure) => Some(procedure.name),
        RecordMember::ProcedurePrototype(procedure) => Some(procedure.name),
        _ => None,
    }
}

impl Builder<'_> {
    pub(super) fn same_module_source(&self, left: ModuleId, right: ModuleId) -> bool {
        if left == right {
            return true;
        }
        // Entry is published after parameter binding; its file is already the first member.
        let entry = |module: ModuleId| {
            let module = &self.graph.modules[module.index()];
            module.entry.or_else(|| module.files.first().copied())
        };
        let (Some(left), Some(right)) = (entry(left), entry(right)) else {
            return false;
        };
        self.graph
            .sources
            .get(self.graph.files[left.index()].source)
            .map(|source| source.path())
            == self
                .graph
                .sources
                .get(self.graph.files[right.index()].source)
                .map(|source| source.path())
    }
    pub(super) fn rebind_program_type(&self, file: FileInstanceId, ty: ModuleType) -> ModuleType {
        let module = self.graph.files[file.index()].module;
        match ty {
            ModuleType::Declaration(id) => {
                let origin =
                    self.graph.files[self.graph.declarations[id.index()].file.index()].module;
                if !self.same_module_source(origin, module) {
                    return ModuleType::Declaration(id);
                }
                let source = self.graph.declarations[id.index()].location();
                let current = self
                    .graph
                    .declarations
                    .iter()
                    .find(|declaration| {
                        self.graph.files[declaration.file.index()].module == module
                            && declaration.location() == source
                    })
                    .map(|declaration| declaration.id);
                ModuleType::Declaration(current.unwrap_or(id))
            }
            ModuleType::Application {
                template,
                arguments,
            } => {
                let ModuleType::Declaration(template) =
                    self.rebind_program_type(file, ModuleType::Declaration(template))
                else {
                    unreachable!()
                };
                ModuleType::Application {
                    template,
                    arguments: arguments
                        .into_iter()
                        .map(|argument| match argument {
                            ModuleBoundArgument::Type(ty) => {
                                ModuleBoundArgument::Type(self.rebind_program_type(file, ty))
                            }
                            ModuleBoundArgument::Enumeration(value) => {
                                let ModuleType::Declaration(declaration) = self
                                    .rebind_program_type(
                                        file,
                                        ModuleType::Declaration(value.declaration),
                                    )
                                else {
                                    unreachable!()
                                };
                                ModuleBoundArgument::Enumeration(EnumParameter {
                                    declaration,
                                    ..value
                                })
                            }
                            value => value,
                        })
                        .collect(),
                }
            }
            ModuleType::Pointer(inner) => {
                ModuleType::Pointer(Box::new(self.rebind_program_type(file, *inner)))
            }
            ModuleType::Slice(inner) => {
                ModuleType::Slice(Box::new(self.rebind_program_type(file, *inner)))
            }
            ModuleType::DynamicArray(inner) => {
                ModuleType::DynamicArray(Box::new(self.rebind_program_type(file, *inner)))
            }
            ModuleType::FixedArray {
                element,
                count,
            } => ModuleType::FixedArray {
                element: Box::new(self.rebind_program_type(file, *element)),
                count,
            },
            ModuleType::Procedure(mut procedure) => {
                procedure.parameters = procedure
                    .parameters
                    .into_iter()
                    .map(|ty| self.rebind_program_type(file, ty))
                    .collect();
                procedure.results = procedure
                    .results
                    .into_iter()
                    .map(|ty| self.rebind_program_type(file, ty))
                    .collect();
                ModuleType::Procedure(procedure)
            }
            builtin => builtin,
        }
    }
    pub(super) fn coerce_module_value(
        &self,
        file: FileInstanceId,
        ty: &ModuleType,
        value: ParameterValue,
        location: SourceSpan,
        program_wide: bool,
    ) -> Result<ParameterValue, GraphError> {
        match self.coerce_module_value_pure(file, ty, value.clone(), location, program_wide) {
            Err(GraphError::Pending {
                diagnostic, ..
            }) => {
                let ParameterResponse::Value(value) = self.semantic_parameter(
                    file,
                    location,
                    ParameterTask::CoerceValue {
                        expected: ty.clone(),
                        value,
                        program_wide,
                    },
                    &diagnostic.message,
                )?
                else {
                    unreachable!("value response shape was checked on submission")
                };
                Ok(value)
            }
            result => result,
        }
    }
    fn coerce_module_value_pure(
        &self,
        file: FileInstanceId,
        ty: &ModuleType,
        value: ParameterValue,
        location: SourceSpan,
        program_wide: bool,
    ) -> Result<ParameterValue, GraphError> {
        match (ty, value) {
            (ModuleType::Builtin(ModuleBuiltin::Scalar(ty)), ParameterValue::Scalar(value)) => {
                value
                    .coerce(*ty, location.span)
                    .map(ParameterValue::Scalar)
                    .map_err(|error| self.parameter_diagnostic(location, error))
            }
            (ModuleType::Builtin(ModuleBuiltin::Float(ty)), ParameterValue::Scalar(value)) => {
                crate::params::coerce_float(value, *ty, location.span)
                    .map(ParameterValue::Scalar)
                    .map_err(|error| self.parameter_diagnostic(location, error))
            }
            (ModuleType::Builtin(ModuleBuiltin::String), value @ ParameterValue::String(_))
            | (ModuleType::Builtin(ModuleBuiltin::Type), value @ ParameterValue::Type(_)) => {
                Ok(value)
            }
            (ModuleType::Declaration(id), value)
                if matches!(
                    self.graph.declarations[id.index()].syntax.kind,
                    FileDeclarationKind::Enum(_)
                ) =>
            {
                self.coerce_enum(file, *id, value, location, program_wide)
            }
            (
                ModuleType::Declaration(_)
                | ModuleType::Application {
                    ..
                }
                | ModuleType::Pointer(_)
                | ModuleType::Slice(_)
                | ModuleType::DynamicArray(_)
                | ModuleType::FixedArray {
                    ..
                }
                | ModuleType::Procedure(_),
                _,
            ) => Err(self.interface_pending(
                location,
                "module value requires semantic nominal or aggregate resolution",
            )),
            _ => Err(self.located(location, "module argument type differs from parameter type")),
        }
    }
}

fn normalize_module_results(results: &mut Vec<ModuleType>) {
    if matches!(
        results.as_slice(),
        [ModuleType::Builtin(ModuleBuiltin::Void)]
    ) {
        results.clear();
    }
}
