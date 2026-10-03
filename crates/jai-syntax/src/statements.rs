//! Statement parsing preserves declaration, place, and control-flow boundaries.
use super::*;

impl Parser<'_> {
    pub(super) fn data_declaration(
        &mut self,
        name: Symbol,
        span: Span,
    ) -> Result<Statement, Diagnostic> {
        let statement = if self.take(Punct::Constant) {
            StatementKind::Constant(ConstantDeclaration {
                name,
                span,
                ty: None,
                initializer: self.expression(0)?,
            })
        } else if self.take(Punct::Infer) {
            StatementKind::Declare(Declaration::Inferred {
                name,
                initializer: self.expression(0)?,
                attributes: Vec::new(),
            })
        } else {
            self.need(Punct::Colon)?;
            let ty = if self.allow_qualified {
                self.type_syntax()?
            } else {
                TypeSyntax::Builtin(BuiltinType::Scalar(self.scalar_type()?))
            };
            if self.take(Punct::Colon) {
                StatementKind::Constant(ConstantDeclaration {
                    name,
                    span,
                    ty: Some(ty),
                    initializer: self.expression(0)?,
                })
            } else {
                let attributes = self.declaration_attributes()?;
                if let Some(binding) = self.external_data_binding()? {
                    if self.is(Punct::Assign) {
                        return Err(
                            self.error("external data declaration cannot have an initializer")
                        );
                    }
                    StatementKind::Declare(Declaration::External {
                        name,
                        ty,
                        binding,
                        attributes,
                    })
                } else {
                    let initializer = if self.take(Punct::Assign) {
                        Some(self.initializer()?)
                    } else {
                        None
                    };
                    StatementKind::Declare(if let Some(ty) = ty.as_scalar() {
                        Declaration::Explicit {
                            name,
                            ty,
                            initializer,
                            attributes,
                        }
                    } else {
                        Declaration::UnresolvedExplicit {
                            name,
                            ty,
                            initializer,
                            attributes,
                        }
                    })
                }
            }
        };
        let self_terminated = match &statement {
            StatementKind::Declare(Declaration::UnresolvedExplicit {
                ty: TypeSyntax::InlineRecord(_),
                initializer: None,
                ..
            }) => true,
            StatementKind::Declare(Declaration::Inferred { initializer, .. })
            | StatementKind::Constant(ConstantDeclaration { initializer, .. }) => {
                initializer_terminates_declaration(initializer)
            }
            StatementKind::Declare(
                Declaration::Explicit {
                    initializer: Some(initializer),
                    ..
                }
                | Declaration::UnresolvedExplicit {
                    initializer: Some(initializer),
                    ..
                },
            ) => initializer_terminates_declaration(initializer),
            _ => false,
        };
        if !self.take(Punct::Semicolon) && !self_terminated {
            return Err(self.error("expected ';' after declaration"));
        }
        Ok(Statement::new(self.consumed_span(span.start), statement))
    }
    pub(super) fn block(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        self.need(Punct::OpenBrace)?;
        let mut out = Vec::new();
        while !self.take(Punct::CloseBrace) {
            if self.token().kind == Kind::Eof {
                return Err(self.error("unterminated block"));
            }
            out.push(self.statement()?);
        }
        Ok(out)
    }
    pub(super) fn body(&mut self) -> Result<Vec<Statement>, Diagnostic> {
        if self.is(Punct::OpenBrace) {
            self.block()
        } else {
            self.keyword(Keyword::Then);
            Ok(vec![self.statement()?])
        }
    }
    pub(super) fn named_prefix(&self, punctuation: Punct) -> bool {
        self.token().kind == Kind::Ident
            && self
                .tokens
                .get(self.at + 1)
                .is_some_and(|t| t.kind == Kind::Punctuation(punctuation))
    }
    pub(super) fn statement(&mut self) -> Result<Statement, Diagnostic> {
        let start = self.token().span.start;
        let kind = self.statement_kind()?;
        Ok(Statement::new(self.consumed_span(start), kind))
    }
    fn consumed_span(&self, start: usize) -> Span {
        Span::new(start, self.tokens[self.at - 1].span.end)
    }
    fn statement_kind(&mut self) -> Result<StatementKind, Diagnostic> {
        if self.token().kind == Kind::Directive(Directive::Bytes) {
            return Ok(StatementKind::InstructionBytes(self.instruction_bytes()?));
        }
        if self.token().kind == Kind::Directive(Directive::Asm) {
            return Ok(StatementKind::Simd(self.simd_block()?));
        }
        if self.import_prefix() {
            if !self.allow_qualified {
                return Err(self.error("local imports require module and lexical scope resolution"));
            }
            return Ok(StatementKind::Import(self.scoped_import_declaration()?));
        }
        if self.token().kind == Kind::Keyword(Keyword::Using) {
            return self.using_statement();
        }
        if self.is(Punct::Backtick) {
            return self.caller_export_statement();
        }
        if self.token().kind == Kind::Directive(Directive::Assert) {
            let (condition, message) = self.assertion_arguments()?;
            return Ok(StatementKind::CompileTimeAssert { condition, message });
        }
        if self.token().kind == Kind::Directive(Directive::If) {
            return self.statement_conditional();
        }
        if safety_checks::is_check_directive(self.token().kind) {
            return self.safety_check_scope();
        }
        if self.token().kind == Kind::Directive(Directive::AddContext) {
            if !self.allow_qualified {
                return Err(self.error("#add_context requires context schema resolution"));
            }
            return Ok(StatementKind::ContextField(
                self.context_field_declaration()?,
            ));
        }
        if self.token().kind == Kind::Directive(Directive::Insert) {
            if !self.allow_qualified {
                return Err(self.error("#insert requires source insertion resolution"));
            }
            let directive = self.insert_directive(0)?;
            self.insert_terminator(&directive)?;
            return Ok(StatementKind::Insert(directive));
        }
        if self.starts_procedure() && !self.type_alias_prefix() {
            return Ok(match self.procedure_declaration()? {
                FileDeclarationKind::Procedure(procedure) => {
                    StatementKind::Procedure(Box::new(procedure))
                }
                FileDeclarationKind::ProcedurePrototype(prototype) => {
                    StatementKind::ProcedurePrototype(prototype)
                }
                FileDeclarationKind::OperatorAlias(alias) => {
                    return Err(Diagnostic::new(
                        alias.span,
                        "operator aliases require a file source namespace",
                    ));
                }
                _ => unreachable!("procedure declaration produces defined body or prototype"),
            });
        }
        if self.allow_qualified {
            if self.starts_library() {
                return Ok(StatementKind::Library(self.library_declaration()?));
            }
            match self.nominal_prefix() {
                Some(Keyword::Struct | Keyword::Union) => {
                    return Ok(StatementKind::Record(self.record_declaration()?));
                }
                Some(Keyword::Enum | Keyword::EnumFlags) => {
                    return Ok(StatementKind::Enum(self.enum_declaration()?));
                }
                _ => {}
            }
            if self.type_alias_prefix() {
                return Ok(StatementKind::TypeAlias(self.type_alias_declaration()?));
            }
        }
        if self.is(Punct::OpenBrace) {
            return Ok(StatementKind::Block(self.block()?));
        }
        if self.keyword(Keyword::Defer) {
            return Ok(StatementKind::Defer(self.body()?));
        }
        if self.keyword(Keyword::PushContext) {
            if !self.allow_qualified {
                return Err(self.error("push_context requires context resolution"));
            }
            let value = if self.is(Punct::OpenBrace) {
                None
            } else {
                Some(self.expression(0)?)
            };
            return Ok(StatementKind::PushContext {
                value,
                body: self.block()?,
            });
        }
        if self.keyword(Keyword::Return) {
            if self.take(Punct::Semicolon) {
                return Ok(StatementKind::Return(None));
            }
            let mut values = Vec::new();
            loop {
                let name = if self.named_prefix(Punct::Assign) {
                    let name = self.name()?;
                    self.need(Punct::Assign)?;
                    Some(name)
                } else {
                    None
                };
                values.push(ReturnValue {
                    name,
                    value: self.expression(0)?,
                });
                if !self.take(Punct::Comma) {
                    break;
                }
                if !self.allow_qualified {
                    return Err(self.error("multiple returns require result binding resolution"));
                }
            }
            self.need(Punct::Semicolon)?;
            if values.len() == 1 && values[0].name.is_none() {
                return Ok(StatementKind::Return(Some(values.remove(0).value)));
            }
            if !self.allow_qualified {
                return Err(self.error("named returns require result binding resolution"));
            }
            return Ok(StatementKind::ReturnValues(values));
        }
        if self.keyword(Keyword::If) {
            let complete = self.token().kind == Kind::Directive(Directive::Complete);
            if complete {
                self.at += 1;
            }
            let cond = self.expression(0)?;
            if self.is(Punct::OpenBrace) {
                let operator = match self.tokens[self.at - 1].kind {
                    Kind::Punctuation(Punct::Equal) => Some(CaseOperator::Equal),
                    Kind::Punctuation(Punct::NotEqual) => Some(CaseOperator::NotEqual),
                    _ => None,
                };
                if let Some(operator) = operator {
                    return self.case_statement(cond, operator, complete);
                }
            }
            if complete {
                return Err(self.error("#complete requires if-case"));
            }
            let yes = self.body()?;
            let no = if self.keyword(Keyword::Else) {
                self.body()?
            } else {
                Vec::new()
            };
            return Ok(StatementKind::If(cond, yes, no));
        }
        if self.keyword(Keyword::While) {
            let exported_binding = self.is(Punct::Backtick)
                && self
                    .tokens
                    .get(self.at + 1)
                    .is_some_and(|token| token.kind == Kind::Ident)
                && self
                    .tokens
                    .get(self.at + 2)
                    .is_some_and(|token| token.kind == Kind::Punctuation(Punct::Infer));
            let condition = if exported_binding || self.named_prefix(Punct::Infer) {
                let export_start = if exported_binding {
                    if !self.allow_qualified {
                        return Err(self.error("caller exports require checked macro expansion"));
                    }
                    let start = self.token().span.start;
                    self.need(Punct::Backtick)?;
                    Some(start)
                } else {
                    None
                };
                let name = self.name()?;
                let export_span = export_start.map(|start| self.consumed_span(start));
                self.need(Punct::Infer)?;
                WhileCondition::Binding {
                    name,
                    export_span,
                    initializer: self.expression(0)?,
                }
            } else {
                WhileCondition::Expression(self.expression(0)?)
            };
            return Ok(StatementKind::While(condition, self.body()?));
        }
        if self.keyword(Keyword::For) {
            let mut direction = Direction::Forward;
            let mut by_pointer = false;
            let mut reverse_control = None;
            let mut pointer_control = None;
            let mut reverse_seen = false;
            let mut pointer_seen = false;
            loop {
                if self.is(Punct::Less) || self.is(Punct::LessEqual) {
                    if reverse_seen {
                        return Err(self.error("duplicate reverse iteration modifier"));
                    }
                    reverse_seen = true;
                    if self.take(Punct::LessEqual) {
                        reverse_control = Some(self.iteration_flag_expression()?);
                    } else {
                        self.need(Punct::Less)?;
                        direction = Direction::Reverse;
                    }
                } else if self.is(Punct::Mul) || self.is(Punct::MulAssign) {
                    if pointer_seen {
                        return Err(self.error("duplicate pointer iteration modifier"));
                    }
                    pointer_seen = true;
                    if self.take(Punct::MulAssign) {
                        pointer_control = Some(self.iteration_flag_expression()?);
                    } else {
                        self.need(Punct::Mul)?;
                        by_pointer = true;
                    }
                } else {
                    break;
                }
                self.take(Punct::Comma);
            }
            let expansion = if self.take(Punct::Colon) {
                Some(self.name_path()?)
            } else {
                None
            };
            let (iterator, index, iterator_export, index_export) = if self.is(Punct::Backtick)
                || (self.token().kind == Kind::Ident
                    && (self.named_prefix(Punct::Colon) || self.named_prefix(Punct::Comma)))
            {
                let iterator_export = self.take(Punct::Backtick);
                let name = self.name()?;
                let (index, index_export) = if self.take(Punct::Comma) {
                    let exported = self.take(Punct::Backtick);
                    (Some(self.name()?), exported)
                } else {
                    (None, false)
                };
                self.need(Punct::Colon)?;
                (name, index, iterator_export, index_export)
            } else {
                (self.symbols.intern("it"), None, false, false)
            };
            let start = self.expression(0)?;
            if !self.take(Punct::Range) {
                if !self.allow_qualified {
                    return Err(self.error("array iteration requires sequence type resolution"));
                }
                return Ok(StatementKind::ArrayLoop(ArrayLoop {
                    expansion,
                    iterator,
                    iterator_export,
                    index,
                    index_export,
                    sequence: start,
                    direction,
                    reverse_control,
                    by_pointer,
                    pointer_control,
                    body: self.body()?,
                }));
            }
            if expansion.is_some() {
                return Err(self.error("range loops do not accept custom expansion selectors"));
            }
            if by_pointer || pointer_control.is_some() || index.is_some() {
                return Err(self.error("range loops do not accept pointer or index bindings"));
            }
            let end = self.expression(0)?;
            return Ok(StatementKind::Range(RangeLoop {
                iterator,
                iterator_export,
                start,
                end,
                direction,
                reverse_control,
                body: self.body()?,
            }));
        }
        let jump = match self.token().kind {
            Kind::Keyword(Keyword::Break) => Some(JumpKind::Break),
            Kind::Keyword(Keyword::Continue) => Some(JumpKind::Continue),
            Kind::Keyword(Keyword::Remove) => Some(JumpKind::Remove),
            _ => None,
        };
        if let Some(kind) = jump {
            let span = self.token().span;
            self.at += 1;
            let target = if self.token().kind == Kind::Ident {
                LoopTarget::Named(self.name()?)
            } else if kind == JumpKind::Remove {
                return Err(Diagnostic::new(
                    span,
                    "remove requires an array iterator name",
                ));
            } else {
                LoopTarget::Innermost
            };
            self.need(Punct::Semicolon)?;
            return Ok(StatementKind::Jump { kind, target, span });
        }
        if (self.token().kind == Kind::Ident
            || (self.token().kind == Kind::Keyword(Keyword::Context)
                && matches!(
                    self.tokens[self.at + 1].kind,
                    Kind::Punctuation(Punct::Infer | Punct::Colon | Punct::Constant)
                )))
            && !(self.tokens[self.at + 1].kind == Kind::Punctuation(Punct::Assign)
                && self.tokens[self.at + 2].kind == Kind::Punctuation(Punct::Comma))
            && !(self.tokens[self.at + 1].kind == Kind::Punctuation(Punct::Colon)
                && matches!(
                    self.tokens[self.at + 2].kind,
                    Kind::Punctuation(Punct::Comma | Punct::Assign)
                ))
            && matches!(self.tokens[self.at + 1].kind, Kind::Punctuation(p)
                if matches!(p, Punct::Infer | Punct::Colon | Punct::Constant | Punct::Assign) || BinaryOp::compound(p).is_some())
        {
            let span = self.token().span;
            let name = self.name()?;
            if let Kind::Punctuation(p) = self.token().kind
                && let Some(op) = BinaryOp::compound(p)
            {
                self.at += 1;
                let value = self.expression(0)?;
                self.need(Punct::Semicolon)?;
                return Ok(StatementKind::Update(name, op, value));
            }
            if self.take(Punct::Assign) {
                let v = self.expression(0)?;
                self.need(Punct::Semicolon)?;
                return Ok(StatementKind::Assign(name, v));
            }
            return self
                .data_declaration(name, span)
                .map(|statement| statement.kind);
        }
        let expr = self.expression(0)?;
        if self.allow_qualified
            && (self.is(Punct::Assign)
                || self.is(Punct::Comma)
                || self.is(Punct::Colon)
                || matches!(self.token().kind, Kind::Punctuation(p) if BinaryOp::compound(p).is_some()))
        {
            return self.place_statement(expr);
        }
        if matches!(
            &expr.kind,
            ExpressionKind::CompileTime(CompileTimeRun {
                body: CompileTimeBody::Block(_) | CompileTimeBody::Procedure { .. },
                ..
            })
        ) {
            self.take(Punct::Semicolon);
        } else {
            self.need(Punct::Semicolon)?;
        }
        Ok(StatementKind::Expression(expr))
    }

    fn iteration_flag_expression(&mut self) -> Result<Expression, Diagnostic> {
        // A parenthesized modifier ends before a following <= modifier, as used
        // by current source containers. Its interior still parses normally.
        let minimum = if self.is(Punct::OpenParen) { 14 } else { 0 };
        self.expression(minimum)
    }
    fn expression_values(&mut self) -> Result<Vec<Expression>, Diagnostic> {
        let mut values = vec![self.expression(0)?];
        while self.take(Punct::Comma) {
            values.push(self.expression(0)?);
        }
        Ok(values)
    }
    fn place_statement(&mut self, first: Expression) -> Result<StatementKind, Diagnostic> {
        let mut targets = vec![];
        let mut new_markers = Vec::new();
        let mut target = first;
        loop {
            let existing = self.is(Punct::Assign)
                && matches!(
                    self.tokens[self.at + 1].kind,
                    Kind::Punctuation(Punct::Comma | Punct::Infer | Punct::Colon)
                );
            if existing {
                self.at += 1;
            }
            let new = self.is(Punct::Colon)
                && matches!(
                    self.tokens[self.at + 1].kind,
                    Kind::Punctuation(Punct::Comma | Punct::Assign)
                );
            if new {
                self.at += 1;
            }
            new_markers.push(new);
            targets.push((target, existing));
            if !self.take(Punct::Comma) {
                break;
            }
            target = self.expression(0)?;
        }
        if new_markers.iter().any(|marked| *marked) {
            let mut bindings = Vec::with_capacity(targets.len());
            for ((target, existing), new) in targets.into_iter().zip(new_markers) {
                if new {
                    if existing {
                        return Err(Diagnostic::new(
                            target.span,
                            "result destination cannot be both new and existing",
                        ));
                    }
                    let ExpressionKind::Name(name) = target.kind else {
                        return Err(Diagnostic::new(
                            target.span,
                            "declaration requires identifier bindings",
                        ));
                    };
                    bindings.push(crate::ResultTargetBinding::New {
                        name,
                        span: target.span,
                    });
                } else {
                    bindings.push(crate::ResultTargetBinding::Existing(PlaceSyntax::try_from(
                        target,
                    )?));
                }
            }
            self.need(Punct::Assign)?;
            let values = self.expression_values()?;
            self.need(Punct::Semicolon)?;
            return Ok(StatementKind::MixedResults {
                bindings,
                ty: None,
                values,
            });
        }
        if self.take(Punct::Constant) {
            let mut names = Vec::new();
            for (target, existing) in targets {
                if existing {
                    return Err(Diagnostic::new(
                        target.span,
                        "constant declarations require new bindings",
                    ));
                }
                let ExpressionKind::Name(name) = target.kind else {
                    return Err(Diagnostic::new(
                        target.span,
                        "constant declarations require identifier bindings",
                    ));
                };
                names.push((name, target.span));
            }
            let initializer = self.expression(0)?;
            self.need(Punct::Semicolon)?;
            let span = Span::new(names[0].1.start, self.tokens[self.at - 1].span.end);
            return Ok(StatementKind::ConstantResults(
                crate::ConstantResultsDeclaration {
                    names,
                    initializer,
                    span,
                },
            ));
        }
        if self.is(Punct::Infer) || self.is(Punct::Colon) {
            let mixed = targets.iter().any(|(_, existing)| *existing);
            let mut names = Vec::new();
            let mut bindings = Vec::new();
            for (target, existing) in targets {
                if existing {
                    bindings.push(crate::ResultTargetBinding::Existing(PlaceSyntax::try_from(
                        target,
                    )?));
                    continue;
                }
                let ExpressionKind::Name(name) = target.kind else {
                    return Err(Diagnostic::new(
                        target.span,
                        "declaration requires identifier bindings",
                    ));
                };
                names.push(name);
                bindings.push(crate::ResultTargetBinding::New {
                    name,
                    span: target.span,
                });
            }
            let (ty, values) = if self.take(Punct::Infer) {
                (None, self.expression_values()?)
            } else {
                self.need(Punct::Colon)?;
                let ty = self.type_syntax()?;
                let values = if self.take(Punct::Assign) {
                    self.expression_values()?
                } else {
                    Vec::new()
                };
                (Some(ty), values)
            };
            self.need(Punct::Semicolon)?;
            if mixed {
                return Ok(StatementKind::MixedResults {
                    bindings,
                    ty,
                    values,
                });
            }
            return Ok(StatementKind::DeclareResults { names, ty, values });
        }
        if targets.iter().any(|(_, existing)| *existing) {
            return Err(self.error("existing-result markers require a declaration"));
        }
        let operation = if self.take(Punct::Assign) {
            None
        } else {
            let Kind::Punctuation(punctuation) = self.token().kind else {
                return Err(self.error("expected assignment operator"));
            };
            let operation = BinaryOp::compound(punctuation)
                .ok_or_else(|| self.error("expected assignment operator"))?;
            self.at += 1;
            Some(operation)
        };
        let mut targets: Vec<PlaceSyntax> = targets
            .into_iter()
            .map(|(target, _)| PlaceSyntax::try_from(target))
            .collect::<Result<_, _>>()?;
        let mut values = self.expression_values()?;
        self.need(Punct::Semicolon)?;
        if targets.len() == 1 && values.len() == 1 {
            let target = targets.remove(0);
            let value = values.remove(0);
            return Ok(match operation {
                None => StatementKind::AssignPlace { target, value },
                Some(operation) => StatementKind::UpdatePlace {
                    target,
                    operation,
                    value,
                },
            });
        }
        Ok(StatementKind::AssignResults {
            targets,
            values,
            operation,
        })
    }
    fn case_statement(
        &mut self,
        value: Expression,
        operator: CaseOperator,
        complete: bool,
    ) -> Result<StatementKind, Diagnostic> {
        self.need(Punct::OpenBrace)?;
        let mut arms = Vec::new();
        let mut default = None;
        while !self.take(Punct::CloseBrace) {
            if !self.keyword(Keyword::Case) {
                return Err(self.error("expected case label"));
            }
            if default.is_some() {
                return Err(self.error("default case must be last"));
            }
            let label = if self.is(Punct::Semicolon) {
                None
            } else {
                Some(self.expression(0)?)
            };
            self.need(Punct::Semicolon)?;
            let mut body = Vec::new();
            let mut through = false;
            while !self.is(Punct::CloseBrace) && self.token().kind != Kind::Keyword(Keyword::Case) {
                if self.token().kind == Kind::Eof {
                    return Err(self.error("unterminated case block"));
                }
                if self.token().kind == Kind::Directive(Directive::Through) {
                    self.at += 1;
                    self.need(Punct::Semicolon)?;
                    through = true;
                    if !self.is(Punct::CloseBrace)
                        && self.token().kind != Kind::Keyword(Keyword::Case)
                    {
                        return Err(self.error("#through must be the last case statement"));
                    }
                    break;
                }
                body.push(self.statement()?);
            }
            if let Some(label) = label {
                arms.push((label, body, through));
            } else {
                if through {
                    return Err(self.error("default case cannot #through"));
                }
                default = Some(body);
            }
        }
        Ok(StatementKind::Cases(CaseStatement {
            value,
            operator,
            arms,
            default,
            complete,
        }))
    }
}

fn initializer_terminates_declaration(initializer: &Expression) -> bool {
    matches!(
        initializer.kind,
        ExpressionKind::HereString(_)
            | ExpressionKind::Type(TypeSyntax::InlineRecord(_) | TypeSyntax::InlineEnum(_))
            | ExpressionKind::CompileTime(CompileTimeRun {
                body: CompileTimeBody::Block(_) | CompileTimeBody::Procedure { .. },
                ..
            })
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_source::SourceMap;

    #[test]
    fn scalar_adapter_preserves_local_procedures_with_explicit_no_context() {
        let module = parse(
            "main::()->int{ add::(value:int)->int #no_context {return value+7;} return add(5); }",
        )
        .unwrap();
        let StatementKind::Procedure(local) = &module.procedures()[0].body[0].kind else {
            panic!("expected local procedure")
        };
        assert_eq!(local.context, jai_types::ContextMode::None);
        assert_eq!(
            local.scalar_return_type(),
            Some(ReturnType::Value(ScalarType::Int(IntegerType::S64)))
        );
        assert!(parse("main::(){ local::(value:Point)->Point{return value;} }").is_err());
    }

    #[test]
    fn sequence_loops_preserve_explicit_and_implicit_binding_modes() {
        let source = "main :: () { for values {} for value,index: values {} for < *value,index: values {} for i: 1..3 {} push_context {} push_context context {} }";
        let mut sources = SourceMap::default();
        let id = sources.insert("statements.jai".into(), source.into());
        let mut symbols = Symbols::default();
        let file = parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(main),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected procedure")
        };
        let StatementKind::ArrayLoop(implicit) = &main.body[0].kind else {
            panic!("expected sequence loop")
        };
        assert_eq!(symbols.name(implicit.iterator), "it");
        assert_eq!(implicit.index, None);
        let StatementKind::ArrayLoop(explicit) = &main.body[1].kind else {
            panic!("expected sequence loop")
        };
        assert_eq!(symbols.name(explicit.iterator), "value");
        assert_eq!(symbols.name(explicit.index.unwrap()), "index");
        let StatementKind::ArrayLoop(reverse) = &main.body[2].kind else {
            panic!("expected sequence loop")
        };
        assert_eq!(reverse.direction, Direction::Reverse);
        assert!(reverse.by_pointer);
        assert!(matches!(main.body[3].kind, StatementKind::Range(_)));
        assert!(matches!(
            main.body[4].kind,
            StatementKind::PushContext { value: None, .. }
        ));
        assert!(matches!(
            main.body[5].kind,
            StatementKind::PushContext { value: Some(_), .. }
        ));
    }

    #[test]
    fn sequence_binding_modes_are_not_silently_applied_to_integer_ranges() {
        for source in [
            "main :: () { for *i: 1..3 {} }",
            "main :: () { for i,index: 1..3 {} }",
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("range.jai".into(), source.into());
            let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
            assert_eq!(
                error.message,
                "range loops do not accept pointer or index bindings"
            );
        }
    }
}
