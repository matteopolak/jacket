//! Select source branches after typed constant binding, before branch declarations.
use super::*;
use jai_source::SourceSpan;

pub(crate) fn has_contextual_member(source: &syntax::Expression) -> bool {
    let mut pending = vec![source];
    while let Some(expression) = pending.pop() {
        use syntax::ExpressionKind as E;
        match &expression.kind {
            E::InferredMember(_) => return true,
            E::Unary(_, value)
            | E::Cast(_, _, value)
            | E::TypeCast {
                value, ..
            } => pending.push(value),
            E::Binary(_, lhs, rhs) => pending.extend([lhs.as_ref(), rhs.as_ref()]),
            E::Conditional(value) => {
                pending.push(value.condition.as_ref());
                pending.extend(value.explicit_then());
                if let Some(value) = &value.else_value {
                    pending.push(value);
                }
            }
            _ => {}
        }
    }
    false
}

impl Resolver<'_> {
    pub(crate) fn select_compile_time_statements(
        &mut self,
        statements: &[syntax::Statement],
    ) -> Result<Vec<syntax::Statement>, Diagnostic> {
        let environment = self.local_import_environment();
        let result = self.select_compile_time_statements_in_environment(statements);
        self.restore_local_import_environment(environment);
        result
    }

    fn select_compile_time_statements_in_environment(
        &mut self,
        statements: &[syntax::Statement],
    ) -> Result<Vec<syntax::Statement>, Diagnostic> {
        let environment = self.local_import_environment();
        let mut selected = statements.to_vec();
        loop {
            self.restore_local_import_environment(environment.clone());
            self.refresh_local_import_environments(&selected)?;
            let mut progress = false;
            let mut pending_error = None;
            let mut index = 0;
            while index < selected.len() {
                if let syntax::StatementKind::Import(import) = &selected[index].kind {
                    self.bind_scoped_import(import)?;
                    index += 1;
                    continue;
                }
                if let syntax::StatementKind::Using(directive) = &selected[index].kind {
                    self.bind_checked_using_prefix(directive)?;
                    index += 1;
                    continue;
                }
                if let Some(directive) = selected[index].using_declaration_directive() {
                    self.bind_checked_using_prefix(&directive)?;
                    index += 1;
                    continue;
                }
                if let syntax::StatementKind::CompileTimeAssert {
                    condition,
                    message,
                } = &selected[index].kind
                {
                    match self.compile_time_assertion(
                        condition,
                        message.as_ref(),
                        selected[index].span,
                    ) {
                        Ok(()) => {
                            selected.remove(index);
                            progress = true;
                        }
                        Err(error) => {
                            pending_error.get_or_insert(error);
                            index += 1;
                        }
                    }
                    continue;
                }
                if let syntax::StatementKind::CompileTimeCases(cases) = &selected[index].kind {
                    let choice = match self.compile_time_case_selection(&cases.header()) {
                        Ok(choice) => choice,
                        Err(error) => {
                            pending_error.get_or_insert(error);
                            index += 1;
                            continue;
                        }
                    };
                    let body = cases.selected_body(choice).ok_or_else(|| {
                        Diagnostic::new(
                            cases.span,
                            "compile-time case selection has no valid fallthrough body",
                        )
                    })?;
                    self.register_local_declarations(&body)?;
                    selected.splice(index..index + 1, body);
                    progress = true;
                    continue;
                }
                let syntax::StatementKind::CompileTimeIf {
                    condition,
                    then_body,
                    else_body,
                } = &selected[index].kind
                else {
                    index += 1;
                    continue;
                };
                let choice = match self.compile_time_condition(condition) {
                    Ok(choice) => choice,
                    Err(error) => {
                        pending_error.get_or_insert(error);
                        index += 1;
                        continue;
                    }
                };
                let body = if choice {
                    then_body
                } else {
                    else_body
                };
                // Static-if braces do not introduce a scope. Register only active
                // declarations and retain their original source wrappers when
                // splicing them into the enclosing block's flow and cleanup list.
                self.register_local_declarations(body)?;
                let body = body.clone();
                selected.splice(index..index + 1, body);
                progress = true;
            }
            match pending_error {
                None => return Ok(selected),
                Some(error) if !progress => return Err(error),
                // A later selected declaration can satisfy an earlier guard.
                // Completed #run operands retain the existing readiness cache.
                Some(_) => {}
            }
        }
    }

    pub(crate) fn compile_time_assertion(
        &mut self,
        condition: &syntax::Expression,
        message: Option<&syntax::Expression>,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let prepared = self.prepare_condition_source(condition)?;
        self.reject_runtime_condition_names(&prepared)
            .map_err(assertion_condition_diagnostic)?;
        let bound = self
            .expr(&prepared)?
            .condition(condition.span, self.types)?;
        require_constant(
            &ValueExpr::Bool(bound),
            condition.span,
            &self.places.snapshot(),
            true,
        )
        .map_err(assertion_condition_diagnostic)?;
        let value = self
            .execute_compile_time(
                &syntax::CompileTimeRun {
                    flags: Default::default(),
                    body: syntax::CompileTimeBody::Expression(Box::new(prepared)),
                },
                condition.span,
                None,
            )?
            .ok_or_else(|| {
                Diagnostic::new(condition.span, "#assert condition produced no value")
            })?;
        let selected = match value.kind {
            ConstantKind::Bool(value) => value,
            ConstantKind::Int(value) | ConstantKind::Enum(value) => value.value() != 0,
            _ => {
                return Err(Diagnostic::new(
                    condition.span,
                    "#assert condition requires a compile-time boolean or integer",
                ));
            }
        };
        let message = match message {
            None => None,
            Some(message) => {
                let expression = self.expr(message)?;
                let value = expression.value(message.span)?;
                let value = self.evaluate_pure_constant(value, message.span)?;
                let ConstantKind::StringBytes(bytes) = value.kind else {
                    return Err(Diagnostic::new(
                        message.span,
                        "#assert message requires a compile-time string",
                    ));
                };
                Some(String::from_utf8_lossy(&bytes).into_owned())
            }
        };
        if selected {
            Ok(())
        } else {
            Err(Diagnostic::new(
                span,
                match message {
                    Some(message) => format!("compile-time assertion failed: {message}"),
                    None => "compile-time assertion failed".into(),
                },
            ))
        }
    }

    pub(crate) fn compile_time_condition(
        &mut self,
        source: &syntax::Expression,
    ) -> Result<bool, Diagnostic> {
        if let Some(scope) = self.graph_scope
            && let Some(selected) = scope.selected_condition(
                source.span,
                self.meta.source_specialization_keys.get(&self.procedure),
            )
        {
            return Ok(selected);
        }
        let prepared = self.prepare_condition_source(source)?;
        self.reject_runtime_condition_names(&prepared)?;
        let condition = self.expr(&prepared)?.condition(source.span, self.types)?;
        let value = self.evaluate_pure_constant(ValueExpr::Bool(condition), source.span)?;
        match value.kind {
            ConstantKind::Bool(value) => Ok(value),
            _ => Err(Diagnostic::new(
                source.span,
                "#if condition did not produce a compile-time boolean",
            )),
        }
    }

    pub(crate) fn prepare_condition_source(
        &mut self,
        source: &syntax::Expression,
    ) -> Result<syntax::Expression, Diagnostic> {
        let mut prepared = source.clone();
        let mut pending = vec![&mut prepared];
        while let Some(expression) = pending.pop() {
            use syntax::ExpressionKind as E;
            let count_root = match &expression.kind {
                E::QualifiedName(path)
                    if path.members.len() == 1 && self.symbols.name(path.members[0]) == "count" =>
                {
                    Some(path.root)
                }
                E::Member {
                    base,
                    member,
                } if self.symbols.name(*member) == "count" => {
                    if let E::Name(root) = base.kind {
                        Some(root)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(root) = count_root
                && self.is_runtime_local_name(root)
                && let Some(ty) = self.local_declared_type(root, expression.span)?
                && let Ok(jai_types::TypeKind::FixedArray {
                    count, ..
                }) = self.types.kind(ty)
            {
                // Fixed array length belongs to its type, not its storage.
                // This cannot execute or inspect the runtime initializer.
                expression.kind = E::Integer(i128::from(*count));
                continue;
            }
            match &mut expression.kind {
                E::Unary(_, value)
                | E::Cast(_, _, value)
                | E::TypeCast {
                    value, ..
                }
                | E::Member {
                    base: value, ..
                }
                | E::AddressOf(value)
                | E::Dereference(value) => pending.push(value),
                E::Binary(_, lhs, rhs)
                | E::Index {
                    base: lhs,
                    index: rhs,
                } => pending.extend([lhs.as_mut(), rhs.as_mut()]),
                E::Conditional(value) => {
                    pending.extend(value.expressions_mut());
                }
                _ => {}
            }
        }
        Ok(prepared)
    }

    pub(crate) fn evaluate_pure_constant(
        &self,
        value: ValueExpr,
        span: Span,
    ) -> Result<ConstantValue, Diagnostic> {
        let places = self.places.snapshot();
        let needs_globals = require_constant(&value, span, &places, false)?;
        let ty = value.type_id(self.types);
        let procedures = HashMap::new();
        let mut signatures: HashMap<_, _> = self
            .signatures
            .values()
            .map(|signature| (signature.id, signature.ty))
            .collect();
        if let Some(context) = self.compile_time {
            signatures.extend(context.signatures.iter().map(|(&id, &ty)| (id, ty)));
            signatures.extend(context.generics.borrow().signature_snapshot());
        }
        signatures.extend(self.meta.local_declarations.signature_snapshot());
        let globals = if needs_globals {
            let globals = self
                .compile_time
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "constant global addresses require a defining global scope",
                    )
                })?
                .globals;
            self.meta.external_globals.snapshot(globals)?
        } else {
            Vec::new()
        };
        let provider = compile_time::ReadyProcedures::new(
            self.types,
            &procedures,
            &signatures,
            &globals,
            &places,
        )
        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let location = SourceSpan {
            source: self
                .debug
                .source()
                .or_else(|| self.graph_scope.map(|scope| scope.source()))
                .ok_or_else(|| {
                    Diagnostic::new(
                        span,
                        "compile-time constant requires a defining source scope",
                    )
                })?,
            span,
        };
        let limits = self
            .compile_time
            .map(|context| context.limits)
            .unwrap_or_default();
        match compile_time::evaluate_for_target(
            &provider,
            jai_vm::NoEffects,
            &value,
            ty,
            location,
            limits,
            compile_time::EvaluationScope {
                owner: self
                    .expression_owner
                    .map(|owner| self.compile_time.map_or(owner, |context| context.owner)),
                target: self.compile_time.and_then(|context| context.target),
            },
        ) {
            compile_time::RunOutcome::Complete(Some(value)) => Ok(value),
            compile_time::RunOutcome::Failed(error) => {
                Err(Diagnostic::at_source(error.location, error.message))
            }
            _ => Err(Diagnostic::new(
                span,
                "constant expression did not produce a compile-time value",
            )),
        }
    }

    pub(crate) fn reject_runtime_condition_names(
        &self,
        source: &syntax::Expression,
    ) -> Result<(), Diagnostic> {
        let mut pending = vec![source];
        while let Some(expression) = pending.pop() {
            use syntax::ExpressionKind as E;
            match &expression.kind {
                E::Name(name) if self.is_runtime_local_name(*name) => {
                    return Err(nonconstant(source.span));
                }
                E::QualifiedName(path) if self.is_runtime_local_name(path.root) => {
                    return Err(nonconstant(source.span));
                }
                E::Unary(_, value)
                | E::Cast(_, _, value)
                | E::TypeCast {
                    value, ..
                }
                | E::Member {
                    base: value, ..
                }
                | E::AddressOf(value)
                | E::Dereference(value) => pending.push(value),
                E::Binary(_, lhs, rhs)
                | E::Index {
                    base: lhs,
                    index: rhs,
                } => pending.extend([lhs.as_ref(), rhs.as_ref()]),
                E::Conditional(value) => {
                    pending.push(value.condition.as_ref());
                    pending.extend(value.explicit_then());
                    if let Some(value) = &value.else_value {
                        pending.push(value);
                    }
                }
                // Type queries inspect the type rather than reading storage;
                // explicit #run owns and validates its own execution context.
                _ => {}
            }
        }
        Ok(())
    }
}

/// Check all bound operands even where VM evaluation short-circuits. Explicit
/// #run has already materialized its result using the normal readiness provider.
fn require_constant(
    value: &ValueExpr,
    span: Span,
    places: &Places,
    allow_calls: bool,
) -> Result<bool, Diagnostic> {
    enum Node<'a> {
        Bool(&'a BoolExpr),
        Int(&'a IntExpr),
        Float(&'a FloatExpr),
        Value(&'a ValueExpr),
        Place(Place),
    }
    let mut pending = vec![Node::Value(value)];
    let mut needs_globals = false;
    while let Some(node) = pending.pop() {
        match node {
            Node::Bool(value) => match value {
                BoolExpr::Constant(_) | BoolExpr::CompileTime => {}
                BoolExpr::Value(value) | BoolExpr::FromPointer(value) => {
                    pending.push(Node::Value(value))
                }
                BoolExpr::FromInt(value) => pending.push(Node::Int(value)),
                BoolExpr::Not(value) => pending.push(Node::Bool(value)),
                BoolExpr::CompareInts(_, left, right) => {
                    pending.extend([Node::Int(left), Node::Int(right)]);
                }
                BoolExpr::CompareFloats(_, left, right) => {
                    pending.extend([Node::Float(left), Node::Float(right)]);
                }
                BoolExpr::ComparePointers(_, left, right)
                | BoolExpr::CompareStrings(_, left, right) => {
                    pending.extend([Node::Value(left), Node::Value(right)]);
                }
                BoolExpr::CompareBools(_, left, right)
                | BoolExpr::And(left, right)
                | BoolExpr::Or(left, right) => {
                    pending.extend([Node::Bool(left), Node::Bool(right)]);
                }
                BoolExpr::Conditional(value) => {
                    pending.extend([
                        Node::Bool(&value.condition),
                        Node::Bool(&value.then_value),
                        Node::Bool(&value.else_value),
                    ]);
                }
                BoolExpr::Call(call) if allow_calls => {
                    pending.extend(call.arguments.iter().map(|(_, value)| Node::Value(value)))
                }
                BoolExpr::Load(_) | BoolExpr::Call(_) => return Err(nonconstant(span)),
            },
            Node::Int(value) => match value.kind() {
                IntExprKind::Constant(_) | IntExprKind::InvalidCheckedCast => {}
                IntExprKind::Value(value) | IntExprKind::EnumValue(value) => {
                    pending.push(Node::Value(value))
                }
                IntExprKind::FromBool(value) => pending.push(Node::Bool(value)),
                IntExprKind::FromFloat(_, value) => pending.push(Node::Float(value)),
                IntExprKind::Negate(value)
                | IntExprKind::Complement(value)
                | IntExprKind::Cast(_, value) => pending.push(Node::Int(value)),
                IntExprKind::Binary(_, left, right) => {
                    pending.extend([Node::Int(left), Node::Int(right)]);
                }
                IntExprKind::Conditional(value) => {
                    pending.extend([
                        Node::Bool(&value.condition),
                        Node::Int(&value.then_value),
                        Node::Int(&value.else_value),
                    ]);
                }
                IntExprKind::Call(call) if allow_calls => {
                    pending.extend(call.arguments.iter().map(|(_, value)| Node::Value(value)))
                }
                IntExprKind::Load(_)
                | IntExprKind::Call(_)
                | IntExprKind::FromPointer {
                    ..
                }
                | IntExprKind::PointerDifference {
                    ..
                } => return Err(nonconstant(span)),
            },
            Node::Float(value) => match value.kind() {
                FloatExprKind::Constant(_) => {}
                FloatExprKind::Value(value) => pending.push(Node::Value(value)),
                FloatExprKind::FromInt(value) => pending.push(Node::Int(value)),
                FloatExprKind::Negate(value) | FloatExprKind::Cast(value) => {
                    pending.push(Node::Float(value))
                }
                FloatExprKind::Binary(_, left, right) => {
                    pending.extend([Node::Float(left), Node::Float(right)]);
                }
                FloatExprKind::Conditional(value) => {
                    pending.extend([
                        Node::Bool(&value.condition),
                        Node::Float(&value.then_value),
                        Node::Float(&value.else_value),
                    ]);
                }
                FloatExprKind::Call(call) if allow_calls => {
                    pending.extend(call.arguments.iter().map(|(_, value)| Node::Value(value)))
                }
                FloatExprKind::Load(_) | FloatExprKind::Call(_) => return Err(nonconstant(span)),
            },
            Node::Value(value) => match value {
                ValueExpr::StorageBitcast {
                    source, ..
                } => match source {
                    jai_ir::StorageBitcastSource::Place(place) => pending.push(Node::Place(*place)),
                    jai_ir::StorageBitcastSource::Value(value) => pending.push(Node::Value(value)),
                },
                ValueExpr::Int(value)
                | ValueExpr::EnumFromInt {
                    value, ..
                } => pending.push(Node::Int(value)),
                ValueExpr::Bool(value) => pending.push(Node::Bool(value)),
                ValueExpr::Float(value) => pending.push(Node::Float(value)),
                ValueExpr::Bind {
                    bindings,
                    body,
                    ..
                } => {
                    pending.push(Node::Value(body));
                    pending.extend(bindings.iter().rev().map(|(_, value)| Node::Value(value)));
                }
                // The owned IR proof checks each reference's owner, scope, and
                // type before VM evaluation; every producer is checked above.
                ValueExpr::Bound {
                    ..
                } => {}
                ValueExpr::Zero(_)
                | ValueExpr::NativePointer(_)
                | ValueExpr::RuntimeType(_)
                | ValueExpr::StringBytes {
                    ..
                }
                | ValueExpr::Enum {
                    ..
                }
                | ValueExpr::StaticAddress {
                    ..
                }
                | ValueExpr::ProcedureValue {
                    ..
                } => {}
                ValueExpr::Conditional {
                    expression, ..
                } => {
                    pending.extend([
                        Node::Bool(&expression.condition),
                        Node::Value(&expression.then_value),
                        Node::Value(&expression.else_value),
                    ]);
                }
                ValueExpr::Array {
                    elements, ..
                }
                | ValueExpr::Record {
                    fields: elements, ..
                } => pending.extend(elements.iter().map(Node::Value)),
                ValueExpr::OrderedRecord {
                    initializers, ..
                } => {
                    pending.extend(initializers.iter().map(|(_, value)| Node::Value(value)));
                }
                ValueExpr::RecordBuild {
                    initializers, ..
                } => pending.extend(initializers.iter().map(|(_, value)| Node::Value(value))),
                ValueExpr::SequenceBuild {
                    initializers, ..
                } => pending.extend(initializers.iter().map(|(_, value)| Node::Value(value))),
                ValueExpr::Union {
                    value, ..
                }
                | ValueExpr::TypeDescriptor {
                    value, ..
                }
                | ValueExpr::Distinct {
                    value, ..
                }
                | ValueExpr::UnwrapDistinct {
                    value, ..
                }
                | ValueExpr::PointerCast {
                    value, ..
                }
                | ValueExpr::AddressOfValue {
                    value, ..
                } => pending.push(Node::Value(value)),
                ValueExpr::ArrayView {
                    array: value, ..
                }
                | ValueExpr::SequenceView {
                    sequence: value, ..
                }
                | ValueExpr::SequenceField {
                    base: value, ..
                }
                | ValueExpr::Field {
                    base: value, ..
                } => pending.push(Node::Value(value)),
                ValueExpr::Index {
                    base,
                    index,
                    ..
                } => {
                    pending.extend([Node::Value(base), Node::Int(index)]);
                }
                ValueExpr::PointerOffset {
                    pointer,
                    offset,
                    ..
                } => {
                    pending.extend([Node::Value(pointer), Node::Int(offset)]);
                }
                ValueExpr::PointerOffsetLeft {
                    pointer,
                    offset,
                    ..
                } => {
                    pending.extend([Node::Value(pointer), Node::Int(offset)]);
                }
                ValueExpr::PointerFromInteger {
                    value, ..
                } => pending.push(Node::Int(value)),
                ValueExpr::SequenceConcat {
                    parts, ..
                } => {
                    pending.extend(parts.iter().map(|part| match part {
                        jai_ir::SequencePackPart::Element(value)
                        | jai_ir::SequencePackPart::Spread(value) => Node::Value(value),
                    }));
                }
                ValueExpr::AddressOf {
                    place, ..
                } => pending.push(Node::Place(*place)),
                ValueExpr::Call {
                    call, ..
                } if allow_calls => {
                    pending.extend(call.arguments.iter().map(|(_, value)| Node::Value(value)))
                }
                ValueExpr::IndirectCall {
                    callee,
                    arguments,
                    ..
                } if allow_calls => {
                    pending.push(Node::Value(callee));
                    pending.extend(arguments.iter().map(|(_, value)| Node::Value(value)));
                }
                ValueExpr::Load(_)
                | ValueExpr::Context {
                    ..
                }
                | ValueExpr::ArrayToSlice {
                    ..
                }
                | ValueExpr::Call {
                    ..
                }
                | ValueExpr::IndirectCall {
                    ..
                } => return Err(nonconstant(span)),
            },
            Node::Place(place) => match place.kind() {
                PlaceKind::Global(_) => needs_globals = true,
                PlaceKind::Field(id) => {
                    let projection = places
                        .projection(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    pending.push(Node::Place(projection.base));
                }
                PlaceKind::Index(id) => {
                    let projection = places
                        .index(id)
                        .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                    pending.extend([Node::Place(projection.base), Node::Int(&projection.index)]);
                }
                PlaceKind::Context(_)
                | PlaceKind::Local(_)
                | PlaceKind::Dereference(_)
                | PlaceKind::SequenceField(_) => return Err(nonconstant(span)),
            },
        }
    }
    Ok(needs_globals)
}

fn nonconstant(span: Span) -> Diagnostic {
    Diagnostic::new(
        span,
        "#if condition requires compile-time values; runtime storage and procedure calls require explicit #run",
    )
}

fn assertion_condition_diagnostic(mut diagnostic: Diagnostic) -> Diagnostic {
    if diagnostic.message.starts_with("#if") {
        diagnostic.message = "#assert condition requires compile-time values; runtime storage cannot supply assertion operands".into();
    }
    diagnostic
}
