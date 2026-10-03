//! Source-nominal enum parameter binding, before the semantic type registry is built.
use super::*;
use jai_eval::Value;
use jai_source::Diagnostic;
use jai_syntax::{BinaryOp, BuiltinType, EnumKind, Expression, ExpressionKind, TypeSyntax};
use jai_types::{Integer, IntegerType, ScalarType};

impl Builder<'_> {
    fn pending_enum(&self, location: SourceSpan, message: impl Into<String>) -> GraphError {
        let diagnostic = self.graph.diagnostic(location, message);
        let rendered = diagnostic.render(&self.graph.sources);
        GraphError::Pending {
            diagnostic,
            rendered,
        }
    }
    pub(super) fn enum_type(
        &self,
        file: FileInstanceId,
        path: &NamePath,
        location: SourceSpan,
    ) -> Result<DeclarationId, GraphError> {
        match self.graph.lookup(file, path) {
            Ok(Binding::Declaration(id))
                if matches!(
                    self.graph.declarations[id.index()].syntax.kind,
                    FileDeclarationKind::Enum(_)
                ) =>
            {
                Ok(id)
            }
            _ => Err(self.pending_enum(
                location,
                "module parameter type requires semantic resolution",
            )),
        }
    }
    pub(super) fn coerce_enum(
        &self,
        _file: FileInstanceId,
        declaration: DeclarationId,
        value: ParameterValue,
        location: SourceSpan,
        shared_program_value: bool,
    ) -> Result<ParameterValue, GraphError> {
        match value {
            ParameterValue::Enumeration(value) if value.declaration == declaration => {
                Ok(ParameterValue::Enumeration(value))
            }
            ParameterValue::Enumeration(value)
                if shared_program_value
                    && self.graph.declarations[value.declaration.index()].location()
                        == self.graph.declarations[declaration.index()].location()
                    && self.same_module_source(
                        self.graph.files[self.graph.declarations[value.declaration.index()]
                            .file
                            .index()]
                        .module,
                        self.graph.files[self.graph.declarations[declaration.index()].file.index()]
                            .module,
                    ) =>
            {
                Ok(ParameterValue::Enumeration(EnumParameter {
                    declaration,
                    value: value.value,
                }))
            }
            ParameterValue::ContextualMember(name) => self
                .enum_member(declaration, name, location)
                .map(ParameterValue::Enumeration),
            _ => Err(self.located(
                location,
                "module argument has a different nominal enum type",
            )),
        }
    }
    pub(super) fn nominal_argument(
        &self,
        file: FileInstanceId,
        expression: &Expression,
        active: &mut Vec<DeclarationId>,
    ) -> Result<Option<ParameterValue>, GraphError> {
        let location = SourceSpan {
            source: self.graph.files[file.0].source,
            span: expression.span,
        };
        if matches!(
            expression.kind,
            ExpressionKind::Unary(jai_syntax::UnaryOp::Complement, _)
                | ExpressionKind::Binary(
                    BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor,
                    _,
                    _
                )
        ) {
            return self
                .enum_mask(file, expression, None, active, 0)
                .map(|value| value.map(ParameterValue::Enumeration));
        }
        let path = match &expression.kind {
            ExpressionKind::InferredMember(name) => {
                return Ok(Some(ParameterValue::ContextualMember(*name)));
            }
            ExpressionKind::String(bytes) => {
                return String::from_utf8(bytes.clone())
                    .map(ParameterValue::String)
                    .map(Some)
                    .map_err(|_| {
                        self.located(location, "module string parameter must contain UTF-8")
                    });
            }
            ExpressionKind::Name(root) => NamePath {
                root: *root,
                members: vec![],
            },
            ExpressionKind::QualifiedName(path) => path.clone(),
            _ => return Ok(None),
        };
        if path.members.is_empty()
            && let Some(value) = self.insertion_argument_value(file, path.root, expression.span)?
        {
            return Ok(Some(value));
        }
        if path.members.len() == 1
            && let Some(SourceCaptureValue::Type(ModuleType::Declaration(declaration))) =
                self.graph.insertion_capture_value(file, path.root)
        {
            if matches!(
                self.graph.declarations[declaration.index()].syntax.kind,
                FileDeclarationKind::Enum(_)
            ) {
                return self
                    .enum_member(*declaration, path.members[0], location)
                    .map(ParameterValue::Enumeration)
                    .map(Some);
            }
            return Err(self.pending_enum(
                location,
                "captured static record member requires semantic module argument resolution",
            ));
        }
        if let Some(value) = self.target_argument(file, &path, location)? {
            return Ok(Some(ParameterValue::Enumeration(value)));
        }
        if let Ok(binding) = self.graph.lookup(file, &path) {
            match binding {
                Binding::SourceMember {
                    declaration,
                    member,
                } => {
                    if matches!(
                        self.graph.declarations[declaration.index()].syntax.kind,
                        FileDeclarationKind::Enum(_)
                    ) {
                        return self
                            .enum_member(declaration, member, location)
                            .map(ParameterValue::Enumeration)
                            .map(Some);
                    }
                    return Err(self.pending_enum(
                        location,
                        "static record member module argument requires semantic resolution",
                    ));
                }
                Binding::Parameter(id) => {
                    return Ok(Some(self.graph.parameters[id.index()].value.clone()));
                }
                Binding::Declaration(id) => {
                    let declaration = &self.graph.declarations[id.index()];
                    if let FileDeclarationKind::Constant(constant) = &declaration.syntax.kind {
                        if active.contains(&id) {
                            return Err(self.located(
                                declaration.location(),
                                "cyclic module argument constant dependency",
                            ));
                        }
                        active.push(id);
                        let value =
                            self.nominal_argument(declaration.file, &constant.initializer, active);
                        active.pop();
                        return value.and_then(|value| match (constant.ty.as_ref(), value) {
                            (Some(annotation), Some(value)) => {
                                let ty = self.bind_module_type(
                                    declaration.file,
                                    annotation,
                                    constant.span,
                                    &mut vec![],
                                )?;
                                self.coerce_module_value(
                                    declaration.file,
                                    &ty,
                                    value,
                                    declaration.location(),
                                    false,
                                )
                                .map(Some)
                            }
                            (_, value) => Ok(value),
                        });
                    }
                }
                _ => {}
            }
        }
        if let Some((&name, parents)) = path.members.split_last() {
            let parent = NamePath {
                root: path.root,
                members: parents.to_vec(),
            };
            if let Ok(Binding::Declaration(id)) = self.graph.lookup(file, &parent)
                && matches!(
                    self.graph.declarations[id.index()].syntax.kind,
                    FileDeclarationKind::Enum(_)
                )
            {
                return self
                    .enum_member(id, name, location)
                    .map(ParameterValue::Enumeration)
                    .map(Some);
            }
        }
        Ok(None)
    }
    pub(super) fn enum_member(
        &self,
        declaration: DeclarationId,
        name: Symbol,
        location: SourceSpan,
    ) -> Result<EnumParameter, GraphError> {
        let declaration_data = &self.graph.declarations[declaration.index()];
        let FileDeclarationKind::Enum(enumeration) = &declaration_data.syntax.kind else {
            return Err(self.located(location, "contextual module argument requires an enum type"));
        };
        let representation = match &enumeration.representation {
            None => IntegerType::S64,
            Some(TypeSyntax::Builtin(BuiltinType::Scalar(ScalarType::Int(ty)))) => *ty,
            _ => {
                return Err(self.pending_enum(
                    declaration_data.location(),
                    "enum parameter representation requires semantic resolution",
                ));
            }
        };
        let mut previous = HashMap::new();
        let mut next = Some(if enumeration.kind == EnumKind::Flags {
            1i128
        } else {
            0i128
        });
        let mut found = None;
        for member in &enumeration.members {
            let number = if let Some(expression) = &member.initializer {
                let mut dependency = None;
                let result = jai_eval::evaluate_paths(expression, |path, span| {
                    if path.members.is_empty()
                        && let Some(value) = previous.get(&path.root)
                    {
                        return Ok(Value::Int(*value));
                    }
                    self.constant_path(declaration_data.file, path, &mut vec![declaration], span)
                        .map_err(|error| {
                            dependency = Some(error);
                            Diagnostic::new(span, "enum initializer dependency is pending")
                        })
                });
                if let Some(error) = dependency {
                    return Err(error);
                }
                let value = result.map_err(|d| {
                    self.located(
                        SourceSpan {
                            source: declaration_data.location().source,
                            span: d.span,
                        },
                        d.message,
                    )
                })?;
                match value {
                    Value::Literal(n) => n,
                    Value::Int(n) => n.value(),
                    _ => {
                        return Err(self
                            .located(location, "enum parameter initializer requires an integer"));
                    }
                }
            } else {
                next.ok_or_else(|| {
                    self.located(
                        location,
                        "enum automatic value overflow or unsupported flags progression",
                    )
                })?
            };
            let value = Integer::checked(representation, number).ok_or_else(|| {
                self.located(
                    location,
                    "enum parameter member is out of representation range",
                )
            })?;
            if previous.insert(member.name, value).is_some() {
                return Err(self.located(location, "duplicate enum parameter member"));
            }
            if member.name == name {
                found = Some(EnumParameter {
                    declaration,
                    value,
                });
            }
            next = match enumeration.kind {
                EnumKind::Values => number.checked_add(1),
                EnumKind::Flags if number > 0 && (number as u128).is_power_of_two() => {
                    number.checked_mul(2)
                }
                EnumKind::Flags => None,
            };
        }
        if let Some(value) = found {
            return Ok(value);
        }
        Err(self.located(
            location,
            format!(
                "unknown enum parameter member '{}'",
                self.graph.symbols.name(name)
            ),
        ))
    }
    pub(super) fn enum_tests(
        &self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<Expression, GraphError> {
        if let Some(Value::Bool(value)) = self.enum_comparison(file, expression)? {
            return Ok(Expression {
                kind: ExpressionKind::Bool(value),
                span: expression.span,
            });
        }
        let mut expanded = expression.clone();
        match &mut expanded.kind {
            ExpressionKind::Binary(_, lhs, rhs) => {
                **lhs = self.enum_tests(file, lhs)?;
                **rhs = self.enum_tests(file, rhs)?;
            }
            ExpressionKind::Unary(_, value) | ExpressionKind::Cast(_, _, value) => {
                **value = self.enum_tests(file, value)?;
            }
            ExpressionKind::TypeCast {
                value, ..
            } => {
                **value = self.enum_tests(file, value)?;
            }
            ExpressionKind::Conditional(value) => {
                *value.condition = self.enum_tests(file, &value.condition)?;
                *value.then_value = self.enum_tests(file, &value.then_value)?;
                if let Some(no) = value.else_value.as_mut() {
                    **no = self.enum_tests(file, no)?;
                }
            }
            _ => {}
        }
        Ok(expanded)
    }
    pub(super) fn enum_comparison(
        &self,
        file: FileInstanceId,
        expression: &Expression,
    ) -> Result<Option<Value>, GraphError> {
        let ExpressionKind::Binary(op, lhs, rhs) = &expression.kind else {
            return Ok(None);
        };
        if !matches!(
            op,
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::Less
                | BinaryOp::LessEqual
                | BinaryOp::Greater
                | BinaryOp::GreaterEqual
        ) {
            return Ok(None);
        }
        if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual) {
            let left = self.module_type_expression(file, lhs, &mut vec![])?;
            let right = self.module_type_expression(file, rhs, &mut vec![])?;
            if let (Some(left), Some(right)) = (left, right) {
                return Ok(Some(Value::Bool(if *op == BinaryOp::Equal {
                    left == right
                } else {
                    left != right
                })));
            }
        }
        let mut left = self.nominal_argument(file, lhs, &mut vec![])?;
        let mut right = self.nominal_argument(file, rhs, &mut vec![])?;
        if let Some(ParameterValue::Enumeration(value)) = &left
            && right.is_none()
        {
            right = self
                .enum_mask(file, rhs, Some(value.declaration), &mut vec![], 0)?
                .map(ParameterValue::Enumeration);
        }
        if let Some(ParameterValue::Enumeration(value)) = &right
            && left.is_none()
        {
            left = self
                .enum_mask(file, lhs, Some(value.declaration), &mut vec![], 0)?
                .map(ParameterValue::Enumeration);
        }
        let location = SourceSpan {
            source: self.graph.files[file.0].source,
            span: expression.span,
        };
        if let (Some(ParameterValue::Enumeration(value)), None) = (&left, &right) {
            if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
                && self.enum_is_flags(value.declaration)
                && matches!(
                    self.constant_expression(file, rhs, &mut vec![])?,
                    Value::Literal(0)
                )
            {
                return Ok(Some(Value::Bool(if *op == BinaryOp::Equal {
                    value.value.bits() == 0
                } else {
                    value.value.bits() != 0
                })));
            }
            return Err(self.located(
                location,
                "enum operation requires values of the same nominal type",
            ));
        }
        if let (None, Some(ParameterValue::Enumeration(value))) = (&left, &right) {
            if matches!(op, BinaryOp::Equal | BinaryOp::NotEqual)
                && self.enum_is_flags(value.declaration)
                && matches!(
                    self.constant_expression(file, lhs, &mut vec![])?,
                    Value::Literal(0)
                )
            {
                return Ok(Some(Value::Bool(if *op == BinaryOp::Equal {
                    value.value.bits() == 0
                } else {
                    value.value.bits() != 0
                })));
            }
            return Err(self.located(
                location,
                "enum operation requires values of the same nominal type",
            ));
        }
        let (left, right) = match (left, right) {
            (Some(ParameterValue::Enumeration(left)), Some(right)) => (
                left,
                self.coerce_enum(file, left.declaration, right, location, false)?,
            ),
            (Some(left), Some(ParameterValue::Enumeration(right))) => {
                let ParameterValue::Enumeration(left) =
                    self.coerce_enum(file, right.declaration, left, location, false)?
                else {
                    unreachable!()
                };
                (left, ParameterValue::Enumeration(right))
            }
            _ => return Ok(None),
        };
        let ParameterValue::Enumeration(right) = right else {
            unreachable!()
        };
        let left = left.value.value();
        let right = right.value.value();
        Ok(Some(Value::Bool(match op {
            BinaryOp::Equal => left == right,
            BinaryOp::NotEqual => left != right,
            BinaryOp::Less => left < right,
            BinaryOp::LessEqual => left <= right,
            BinaryOp::Greater => left > right,
            BinaryOp::GreaterEqual => left >= right,
            _ => unreachable!(),
        })))
    }
    fn target_argument(
        &self,
        file: FileInstanceId,
        path: &NamePath,
        location: SourceSpan,
    ) -> Result<Option<EnumParameter>, GraphError> {
        if !path.members.is_empty() {
            return Ok(None);
        }
        let Some(target) = self.graph.target.as_ref() else {
            return Ok(None);
        };
        let (enum_name, member) = match self.graph.symbols.name(path.root) {
            "OS" | "BUILD_OS" => ("Operating_System_Tag", target.operating_system.source_tag()),
            "CPU" | "BUILD_CPU" => ("CPU_Tag", target.architecture.source_tag()),
            _ => return Ok(None),
        };
        if let Ok(binding) = self.graph.lookup(file, path) {
            let Binding::Declaration(id) = binding else {
                return Ok(None);
            };
            let declaration = &self.graph.declarations[id.index()];
            if self.graph.prelude != Some(self.graph.files[declaration.file.index()].module)
                || !matches!(declaration.syntax.kind, FileDeclarationKind::Constant(_))
            {
                return Ok(None);
            }
        }
        let member = member.ok_or_else(|| {
            self.pending_enum(location, "target has no source-defined OS or CPU tag")
        })?;
        let enum_symbol = self.graph.symbols.find(enum_name).ok_or_else(|| {
            self.pending_enum(location, "target tag enum must be supplied by source")
        })?;
        let member_symbol = self.graph.symbols.find(member).ok_or_else(|| {
            self.pending_enum(location, "target tag member is absent from supplied source")
        })?;
        let defining_file = if let Some(prelude) = self.graph.prelude {
            if self.graph.files[file.index()].module == prelude {
                file
            } else {
                self.graph.modules[prelude.index()]
                    .discovered_entry()
                    .ok_or_else(|| {
                        self.pending_enum(location, "target tag source scope is not yet available")
                    })?
            }
        } else {
            file
        };
        let declaration = self.enum_type(
            defining_file,
            &NamePath {
                root: enum_symbol,
                members: vec![],
            },
            location,
        )?;
        self.enum_member(declaration, member_symbol, location)
            .map(Some)
    }
}
