//! Ordinary record operators use the same pure candidate matching as calls.
use super::*;
use crate::overloads::{Argument, ArgumentInfo, ArgumentType, ConversionRank, Match};
use jai_syntax::{Expression, ExpressionKind, OperatorDeclaration, OperatorKind};
use jai_types::TypeKind;
mod contracts;
mod index_updates;
pub(crate) use contracts::validate_signature;

struct SelectedOperator {
    origin: OperatorOrigin,
    kind: OperatorKind,
    arguments: Vec<syntax::CallArgument>,
    conversions: Vec<ConversionRank>,
    substitution: crate::polymorphism::Substitution,
}

enum OperatorOrigin {
    File(Match),
    Local(Signature),
    LocalGeneric(Match<crate::local_declarations::LocalDeclarationId>),
}

impl Resolver<'_> {
    pub(crate) fn overloaded_index_assignment(
        &mut self,
        target: &syntax::PlaceSyntax,
        value: &Expression,
    ) -> Option<Result<Statement, Diagnostic>> {
        let syntax::PlaceKind::Index {
            base,
            index,
        } = &target.kind
        else {
            return None;
        };
        let operand = self.mutation_operator_operand(base);
        Some((|| {
            let operand = operand?;
            let selection = self.try_select_operator(
                OperatorKind::IndexAssign,
                &[&operand, index, value],
                target.span,
            );
            let Some(selection) = selection else {
                return Ok(None);
            };
            let selected = selection?;
            let (signature, call) = self.bind_operator_selection(selected, target.span)?;
            self.validate_operator_signature(OperatorKind::IndexAssign, &signature, target.span)?;
            Ok(Some(Statement::CallVoid(call)))
        })())
        .and_then(optional_result)
    }

    pub(crate) fn overloaded_index_address(
        &mut self,
        source: &Expression,
        span: Span,
    ) -> Option<Result<Expr, Diagnostic>> {
        let ExpressionKind::Index {
            base,
            index,
        } = &source.kind
        else {
            return None;
        };
        let operand = self.mutation_operator_operand(base);
        Some((|| {
            let operand = operand?;
            self.overloaded_operator(OperatorKind::IndexAddress, &[&operand, index], span)
                .transpose()
        })())
        .and_then(optional_result)
    }

    pub(crate) fn overloaded_index_place(
        &mut self,
        base: &Expression,
        index: &Expression,
        span: Span,
    ) -> Option<Result<Place, Diagnostic>> {
        let source = Expression {
            span,
            kind: ExpressionKind::Index {
                base: Box::new(base.clone()),
                index: Box::new(index.clone()),
            },
        };
        self.overloaded_index_address(&source, span).map(|result| {
            let Expr::Pointer {
                value, ..
            } = result?
            else {
                return Err(Diagnostic::new(
                    span,
                    "index address operator must return a pointer",
                ));
            };
            self.places
                .dereference(value, self.types)
                .map_err(|error| Diagnostic::new(span, error.to_string()))
        })
    }

    fn mutation_operator_operand(&mut self, source: &Expression) -> Result<Expression, Diagnostic> {
        let info = self.describe_argument(source)?;
        if matches!(info.ty, ArgumentType::Known(ty) if matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_))))
        {
            return Ok(source.clone());
        }
        Ok(Expression {
            span: source.span,
            kind: ExpressionKind::AddressOf(Box::new(source.clone())),
        })
    }

    pub(crate) fn overloaded_compound_update(
        &mut self,
        target: &syntax::PlaceSyntax,
        place: Place,
        operation: BinaryOp,
        right: &Expression,
    ) -> Option<Result<Statement, Diagnostic>> {
        let call_span = Span::new(target.span.start, right.span.end);
        let target_expression = place_expression(target);
        let address_expression = Expression {
            span: target.span,
            kind: ExpressionKind::AddressOf(Box::new(target_expression.clone())),
        };
        let mutating = self.select_operator(
            OperatorKind::Compound(operation),
            &[&address_expression, right],
            right.span,
            true,
        );
        let (kind, selection, pointer_operand) = match mutating {
            Some(selection) => (OperatorKind::Compound(operation), selection, true),
            None => match self.select_operator(
                OperatorKind::Compound(operation),
                &[&target_expression, right],
                right.span,
                true,
            ) {
                Some(selection) => (OperatorKind::Compound(operation), selection, false),
                None => (
                    OperatorKind::Binary(operation),
                    self.try_select_operator(
                        OperatorKind::Binary(operation),
                        &[&target_expression, right],
                        right.span,
                    )?,
                    false,
                ),
            },
        };
        Some((|| {
            let selected = selection?;
            self.check_captured_operator_projection(&selected, right.span)?;
            let signature = match selected.origin {
                OperatorOrigin::File(matched) => {
                    self.materialize_declaration_match(matched, right.span)?
                }
                OperatorOrigin::Local(signature) => signature,
                OperatorOrigin::LocalGeneric(matched) => self
                    .materialize_local_operator_match_with_contracts(
                        matched,
                        &selected.arguments,
                        right.span,
                    )?,
            };
            self.validate_operator_signature(kind, &signature, right.span)?;
            let descriptor = self
                .types
                .procedure_definition(signature.ty)
                .map_err(|error| Diagnostic::new(right.span, error.to_string()))?;
            self.check_call_context(descriptor, right.span)?;
            let destinations = selected
                .arguments
                .iter()
                .map(|argument| {
                    signature
                        .parameters
                        .iter()
                        .position(|parameter| Some(parameter.name) == argument.name)
                        .ok_or_else(|| {
                            Diagnostic::new(
                                right.span,
                                "operator operand destination is unavailable",
                            )
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;
            let left = if pointer_operand {
                let ty = self
                    .types
                    .pointer(place.ty())
                    .map_err(|error| Diagnostic::new(target.span, error.to_string()))?;
                Expr::Pointer {
                    ty,
                    value: ValueExpr::AddressOf {
                        place,
                        ty,
                    },
                }
            } else {
                self.typed_value(ValueExpr::Load(place), place.ty(), target.span)?
            };
            let left =
                self.coerce_value(left, signature.parameters[destinations[0]].ty, target.span)?;
            let right_value =
                self.expr_expected(right, signature.parameters[destinations[1]].ty)?;
            let right_value = self.coerce_value(
                right_value,
                signature.parameters[destinations[1]].ty,
                right.span,
            )?;
            let mut arguments = vec![
                (ParameterId::new(destinations[0]), left),
                (ParameterId::new(destinations[1]), right_value),
            ];
            for (index, parameter) in signature.parameters.iter().enumerate() {
                if destinations.contains(&index) {
                    continue;
                }
                let default = parameter.default.as_ref().ok_or_else(|| {
                    Diagnostic::new(call_span, "operator trailing parameter requires a default")
                })?;
                arguments.push((
                    ParameterId::new(index),
                    self.materialize_parameter_default(default, parameter.ty, call_span)?,
                ));
            }
            self.warn_deprecated_procedure(signature.id, call_span)?;
            let call = Call::new(signature.id, arguments);
            if signature.results.is_empty() {
                return Ok(Statement::CallVoid(call));
            }
            let result = signature.results[0].ty;
            let result = self.typed_value(
                ValueExpr::Call {
                    ty: result,
                    call,
                },
                result,
                right.span,
            )?;
            Ok(Statement::Store(
                place,
                self.coerce_value(result, place.ty(), right.span)?,
            ))
        })())
    }

    fn bind_operator_selection(
        &mut self,
        selected: SelectedOperator,
        span: Span,
    ) -> Result<(Signature, Call), Diagnostic> {
        if matches!(
            selected.kind,
            OperatorKind::IndexAssign | OperatorKind::Compound(_)
        ) {
            self.check_captured_operator_projection(&selected, span)?;
        }
        match selected.origin {
            OperatorOrigin::File(matched) => {
                let signature = self.materialize_declaration_match(matched.clone(), span)?;
                self.validate_file_operator_match(selected.kind, &matched, &signature, span)?;
                self.bind_declaration_match(matched, &selected.arguments, span)
            }
            OperatorOrigin::Local(signature) => {
                let call = self.bind_signature_arguments(&signature, &selected.arguments, span)?;
                Ok((signature, call))
            }
            OperatorOrigin::LocalGeneric(matched) => {
                let signature = self.materialize_local_operator_match_with_contracts(
                    matched,
                    &selected.arguments,
                    span,
                )?;
                let call = self.bind_signature_arguments(&signature, &selected.arguments, span)?;
                Ok((signature, call))
            }
        }
    }

    pub(crate) fn describe_operator_expression(
        &mut self,
        expression: &Expression,
    ) -> Option<Result<ArgumentInfo, Diagnostic>> {
        let (kind, operands): (_, Vec<&Expression>) = match &expression.kind {
            ExpressionKind::Unary(operation, value) => {
                (OperatorKind::Unary(*operation), vec![value])
            }
            ExpressionKind::Binary(operation, left, right) => {
                (OperatorKind::Binary(*operation), vec![left, right])
            }
            ExpressionKind::Index {
                base,
                index,
            } => {
                let selection = self.select_operator(
                    OperatorKind::Index,
                    &[base, index],
                    expression.span,
                    true,
                );
                if let Some(selection) = selection {
                    return Some(selection.and_then(|selected| {
                        self.describe_selected_operator(selected, expression.span)
                    }));
                }
                let address = Expression {
                    span: expression.span,
                    kind: ExpressionKind::AddressOf(Box::new(expression.clone())),
                };
                return self.describe_operator_expression(&address).map(|result| {
                    let info = result?;
                    let ArgumentType::Known(ty) = info.ty else {
                        return Err(Diagnostic::new(
                            expression.span,
                            "index address result is not a typed pointer",
                        ));
                    };
                    let Ok(TypeKind::Pointer(pointee)) = self.types.kind(ty) else {
                        return Err(Diagnostic::new(
                            expression.span,
                            "index address operator must return a pointer",
                        ));
                    };
                    Ok(ArgumentInfo::typed(*pointee))
                });
            }
            ExpressionKind::AddressOf(source) => {
                let ExpressionKind::Index {
                    base,
                    index,
                } = &source.kind
                else {
                    return None;
                };
                let operand = match self.mutation_operator_operand(base) {
                    Ok(operand) => operand,
                    Err(error) => return Some(Err(error)),
                };
                return self
                    .try_select_operator(
                        OperatorKind::IndexAddress,
                        &[&operand, index],
                        expression.span,
                    )
                    .map(|selection| self.describe_selected_operator(selection?, expression.span));
            }
            _ => return None,
        };
        self.try_select_operator(kind, &operands, expression.span)
            .map(|selection| {
                let selected = selection?;
                self.describe_selected_operator(selected, expression.span)
            })
    }

    fn describe_selected_operator(
        &mut self,
        selected: SelectedOperator,
        span: Span,
    ) -> Result<ArgumentInfo, Diagnostic> {
        match selected.origin {
            OperatorOrigin::File(matched) => self.describe_declaration_match(matched, span),
            OperatorOrigin::LocalGeneric(matched) => {
                self.preview_local_operator_match(&matched, span)
            }
            OperatorOrigin::Local(signature) => match signature.results.as_slice() {
                [result] => Ok(ArgumentInfo::typed(result.ty)),
                _ => Err(Diagnostic::new(span, "operator requires one result")),
            },
        }
    }

    pub(crate) fn overloaded_operator(
        &mut self,
        kind: OperatorKind,
        operands: &[&Expression],
        span: Span,
    ) -> Option<Result<Expr, Diagnostic>> {
        self.try_select_operator(kind, operands, span)
            .map(|selection| {
                let selected = selection?;
                let effective_kind = selected.kind;
                let source_arguments = selected.arguments.clone();
                let (signature, call) = self.bind_operator_selection(selected, span)?;
                let [result] = signature.results.as_slice() else {
                    return Err(Diagnostic::new(span, "operator requires one result"));
                };
                let contract = self
                    .call_result_contracts_from_source(&call, &source_arguments, span)?
                    .into_iter()
                    .next()
                    .flatten();
                let producer = ValueExpr::Call {
                    ty: result.ty,
                    call,
                };
                let producer = if let Some(contract) = contract {
                    let binding = self.allocate_expression_binding(span)?;
                    self.capture_selected_call_result_contract(
                        binding,
                        &producer,
                        Some(contract),
                        span,
                    )?;
                    ValueExpr::Bind {
                        bindings: vec![(binding, producer)],
                        body: Box::new(ValueExpr::Bound {
                            binding,
                            ty: result.ty,
                        }),
                        ty: result.ty,
                    }
                } else {
                    producer
                };
                let value = self.typed_value(producer, result.ty, span)?;
                if kind == OperatorKind::Binary(BinaryOp::NotEqual)
                    && effective_kind == OperatorKind::Binary(BinaryOp::Equal)
                {
                    Ok(Expr::Bool(BoolExpr::Not(Box::new(
                        value.condition(span, self.types)?,
                    ))))
                } else {
                    Ok(value)
                }
            })
    }

    fn try_select_operator(
        &mut self,
        kind: OperatorKind,
        operands: &[&Expression],
        span: Span,
    ) -> Option<Result<SelectedOperator, Diagnostic>> {
        if kind == OperatorKind::Binary(BinaryOp::NotEqual) {
            return self
                .select_operator(kind, operands, span, true)
                .or_else(|| {
                    self.select_operator(
                        OperatorKind::Binary(BinaryOp::Equal),
                        operands,
                        span,
                        false,
                    )
                });
        }
        self.select_operator(kind, operands, span, false)
    }

    fn select_operator(
        &mut self,
        kind: OperatorKind,
        operands: &[&Expression],
        span: Span,
        missing_is_absent: bool,
    ) -> Option<Result<SelectedOperator, Diagnostic>> {
        let local_depth = self.local_operator_scope_depth(kind);
        let mut declarations = if local_depth.is_some() {
            vec![]
        } else {
            self.graph_scope
                .map_or_else(Vec::new, |scope| scope.operator_declarations(kind))
        };
        if let Some(scope) = self.graph_scope {
            declarations.extend(self.local_using_operator_declarations(kind));
            for module in self.local_operator_imports(kind) {
                declarations.extend(scope.exported_operator_declarations(module, kind));
            }
            declarations.sort_by_key(|id| id.index());
            declarations.dedup();
        }
        Some((|| {
            if declarations.is_empty() && local_depth.is_none() {
                return Ok(None);
            }
            let descriptions = operands
                .iter()
                .map(|operand| self.describe_argument(operand))
                .collect::<Result<Vec<_>, _>>()?;
            if !descriptions
                .iter()
                .any(|info| self.nominal_operator_operand(info))
            {
                return Ok(None);
            }
            let locals = self.local_operator_candidates(kind, span)?;
            if declarations.is_empty() && locals.is_empty() {
                return Ok(None);
            }
            let pointer_builtin = descriptions.iter().any(|info| {
                matches!(info.ty, ArgumentType::Known(ty) if matches!(self.types.kind(ty), Ok(TypeKind::Pointer(_))))
            }) && !descriptions.iter().any(|info| match info.ty {
                ArgumentType::Known(ty) => matches!(self.types.kind(ty), Ok(TypeKind::Record(_) | TypeKind::Distinct(_))),
                ArgumentType::RecordLiteral { .. } => true,
                _ => false,
            });
            let mut matches = vec![];
            let mut rejection = None;
            let mut pending = None;
            if let Some(scope) = self.graph_scope {
                for declaration in declarations {
                    let candidate = scope.candidate(declaration, self.types, span)?;
                    let operator = scope
                        .operator_metadata(declaration)
                        .expect("typed operator lookup retains metadata");
                    let parameters = &candidate.parameters;
                    for reversed in orientations(operator) {
                        let arguments = operator_arguments(
                            operands,
                            parameters.iter().map(|parameter| parameter.name),
                            reversed,
                        );
                        let described = describe_arguments(&arguments, &descriptions);
                        let matched = crate::overloads::match_candidate_with_nominals(
                            self.types, self, &candidate, &described, span,
                        )
                        .and_then(|matched| {
                            self.refine_source_declaration_match(
                                &candidate,
                                &arguments,
                                &described,
                                matched,
                                span,
                            )
                        });
                        match matched {
                            Ok(found) => matches.push(SelectedOperator {
                                origin: OperatorOrigin::File(found.clone()),
                                kind,
                                arguments,
                                conversions: found.conversions,
                                substitution: found.substitution,
                            }),
                            Err(error) => {
                                if self
                                    .compile_time
                                    .is_some_and(|context| !context.pending.borrow().is_empty())
                                {
                                    pending.get_or_insert_with(|| error.clone());
                                }
                                rejection.get_or_insert(error);
                            }
                        }
                    }
                }
            }
            for (candidate, signature, operator) in locals {
                for reversed in orientations(operator) {
                    let arguments = operator_arguments(
                        operands,
                        candidate.parameters.iter().map(|parameter| parameter.name),
                        reversed,
                    );
                    let described = describe_arguments(&arguments, &descriptions);
                    let matched = crate::overloads::match_candidate_with_nominals(
                        self.types, self, &candidate, &described, span,
                    ).and_then(|matched| {
                        if signature.is_some() { Ok(matched) }
                        else { self.refine_local_operator_match(&candidate, &arguments, &described, matched, span) }
                    });
                    match matched {
                        Ok(found) => matches.push(SelectedOperator {
                            origin: match &signature {
                                Some(signature) => OperatorOrigin::Local(signature.clone()),
                                None => OperatorOrigin::LocalGeneric(found.clone()),
                            },
                            kind,
                            arguments,
                            conversions: found.conversions,
                            substitution: found.substitution,
                        }),
                        Err(error) => {
                            if self.compile_time.is_some_and(|context| !context.pending.borrow().is_empty()) {
                                pending.get_or_insert_with(|| error.clone());
                            }
                            rejection.get_or_insert(error);
                        }
                    }
                }
            }
            if let Some(error) = pending {
                return Err(error);
            }
            // The same symmetric declaration with identical operand types has
            // one semantic candidate, even when both orientations are exact.
            let mut unique = vec![];
            for candidate in matches {
                if unique.iter().any(|previous: &SelectedOperator| {
                    same_origin(&previous.origin, &candidate.origin)
                        && previous.conversions == candidate.conversions
                        && previous.substitution == candidate.substitution
                }) {
                    continue;
                }
                unique.push(candidate);
            }
            let survivors = unique
                .iter()
                .enumerate()
                .filter_map(|(index, candidate)| {
                    (!unique.iter().enumerate().any(|(other, challenger)| {
                        other != index
                            && ranks_dominate(&challenger.conversions, &candidate.conversions)
                    }))
                    .then_some(index)
                })
                .collect::<Vec<_>>();
            match survivors.as_slice() {
                [winner] => Ok(Some(unique.swap_remove(*winner))),
                [] if missing_is_absent || pointer_builtin => Ok(None),
                [] => Err(Diagnostic::new(
                    span,
                    format!(
                        "no operator overload matches the operands{}",
                        rejection.map_or(String::new(), |error| format!(": {}", error.message))
                    ),
                )),
                _ => Err(Diagnostic::new(span, "ambiguous operator overload")),
            }
        })())
        .and_then(|result| match result {
            Ok(Some(selected)) => Some(Ok(selected)),
            Ok(None) => None,
            Err(error) => Some(Err(error)),
        })
    }

    fn nominal_operator_operand(&self, info: &ArgumentInfo) -> bool {
        match &info.ty {
            ArgumentType::Known(ty)
            | ArgumentType::RecordLiteral {
                ty: Some(ty), ..
            } => nominal_type(self.types, *ty),
            ArgumentType::ContextualCast {
                value, ..
            } => self.nominal_operator_operand(value),
            ArgumentType::RecordLiteral {
                ty: None, ..
            } => true,
            _ => false,
        }
    }

    pub(crate) fn validate_operator_signature(
        &self,
        kind: OperatorKind,
        signature: &Signature,
        span: Span,
    ) -> Result<(), Diagnostic> {
        validate_signature(self.types, kind, signature, span)
    }
}

fn optional_result<T>(result: Result<Option<T>, Diagnostic>) -> Option<Result<T, Diagnostic>> {
    match result {
        Ok(Some(value)) => Some(Ok(value)),
        Ok(None) => None,
        Err(error) => Some(Err(error)),
    }
}

fn nominal_type(types: &TypeRegistry, ty: TypeId) -> bool {
    match types.kind(ty) {
        Ok(TypeKind::Record(_) | TypeKind::Distinct(_)) => true,
        Ok(TypeKind::Pointer(pointee)) => matches!(
            types.kind(*pointee),
            Ok(TypeKind::Record(_) | TypeKind::Distinct(_))
        ),
        _ => false,
    }
}

fn orientations(operator: OperatorDeclaration) -> impl Iterator<Item = bool> {
    [false, true].into_iter().take(if operator.symmetric {
        2
    } else {
        1
    })
}
fn operator_arguments(
    operands: &[&Expression],
    parameters: impl Iterator<Item = Symbol>,
    reversed: bool,
) -> Vec<syntax::CallArgument> {
    let mut parameters = parameters.take(operands.len()).collect::<Vec<_>>();
    if reversed {
        parameters.reverse();
    }
    operands
        .iter()
        .zip(parameters)
        .map(|(operand, parameter)| syntax::CallArgument {
            name: Some(parameter),
            value: (*operand).clone(),
            spread: false,
        })
        .collect()
}
fn describe_arguments(
    arguments: &[syntax::CallArgument],
    descriptions: &[ArgumentInfo],
) -> Vec<Argument> {
    arguments
        .iter()
        .zip(descriptions)
        .map(|(argument, info)| Argument {
            name: argument.name,
            spread: false,
            info: info.clone(),
            span: argument.value.span,
        })
        .collect()
}
fn same_origin(left: &OperatorOrigin, right: &OperatorOrigin) -> bool {
    match (left, right) {
        (OperatorOrigin::File(left), OperatorOrigin::File(right)) => {
            left.declaration == right.declaration
        }
        (OperatorOrigin::Local(left), OperatorOrigin::Local(right)) => left.id == right.id,
        (OperatorOrigin::LocalGeneric(left), OperatorOrigin::LocalGeneric(right)) => {
            left.declaration == right.declaration
        }
        _ => false,
    }
}
fn ranks_dominate(left: &[ConversionRank], right: &[ConversionRank]) -> bool {
    left.iter().zip(right).all(|(left, right)| left <= right)
        && left.iter().zip(right).any(|(left, right)| left < right)
}

fn place_expression(place: &syntax::PlaceSyntax) -> Expression {
    Expression {
        span: place.span,
        kind: match &place.kind {
            syntax::PlaceKind::Insert(directive) => ExpressionKind::Insert(directive.clone()),
            syntax::PlaceKind::Name(name) => ExpressionKind::Name(*name),
            syntax::PlaceKind::Qualified(path) => ExpressionKind::QualifiedName(path.clone()),
            syntax::PlaceKind::Member {
                base,
                member,
            } => ExpressionKind::Member {
                base: base.clone(),
                member: *member,
            },
            syntax::PlaceKind::Index {
                base,
                index,
            } => ExpressionKind::Index {
                base: base.clone(),
                index: index.clone(),
            },
            syntax::PlaceKind::Dereference(pointer) => ExpressionKind::Dereference(pointer.clone()),
        },
    }
}
