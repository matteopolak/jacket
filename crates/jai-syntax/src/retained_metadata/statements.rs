use super::*;

impl<A> Visitor<'_, A> {
    pub(super) fn assertion<E>(
        &mut self,
        condition: &Expression,
        message: &Option<Expression>,
        depth: usize,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.expression(condition, depth + 1)?;
        if let Some(message) = message {
            self.expression(message, depth + 1)?;
        }
        Ok(())
    }

    pub(super) fn source_cases<T, E>(
        &mut self,
        source: &CompileTimeCases<T>,
        depth: usize,
        mut visit: impl FnMut(&mut Self, &T, usize) -> Result<E>,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.expression(&source.value, depth + 1)?;
        self.allocation::<CompileTimeCaseArm<T>, E>(source.arms.capacity())?;
        for arm in &source.arms {
            self.node(depth + 1)?;
            self.expression(&arm.label, depth + 2)?;
            self.sequence(&arm.body, depth + 1, &mut visit)?;
        }
        if let Some(default) = &source.default {
            self.sequence(&default.body, depth + 1, visit)?;
        }
        Ok(())
    }

    pub(super) fn constant<E>(&mut self, source: &ConstantDeclaration, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        if let Some(ty) = &source.ty {
            self.ty(ty, depth + 1)?;
        }
        self.expression(&source.initializer, depth + 1)
    }

    pub(super) fn declaration<E>(&mut self, source: &Declaration, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        let attributes = match source {
            Declaration::GroupMember {
                group, ..
            } => {
                self.admit(1, group.retained_owner_byte_bound())?;
                self.allocation::<(Symbol, Span), E>(group.name_capacity())?;
                self.declaration(group.source(), depth + 1)?;
                return self.sequence(group.extra_initializers(), depth + 1, Self::expression);
            }
            Declaration::External {
                ty,
                binding,
                attributes,
                ..
            } => {
                self.ty(ty, depth + 1)?;
                match &binding.source {
                    ExternalDataSource::Program => {}
                    ExternalDataSource::Library(path) => self.path(path, depth + 1)?,
                }
                if let Some(symbol) = &binding.symbol {
                    self.text(symbol)?;
                }
                attributes
            }
            Declaration::Inferred {
                initializer,
                attributes,
                ..
            } => {
                self.expression(initializer, depth + 1)?;
                attributes
            }
            Declaration::Explicit {
                initializer,
                attributes,
                ..
            } => {
                if let Some(initializer) = initializer {
                    self.expression(initializer, depth + 1)?;
                }
                attributes
            }
            Declaration::UnresolvedExplicit {
                ty,
                initializer,
                attributes,
                ..
            } => {
                self.ty(ty, depth + 1)?;
                if let Some(initializer) = initializer {
                    self.expression(initializer, depth + 1)?;
                }
                attributes
            }
        };
        self.sequence(attributes, depth, |this, attribute, depth| {
            this.node(depth)?;
            match attribute {
                DeclarationAttribute::Alignment(value) => this.expression(value, depth + 1),
            }
        })
    }

    pub(super) fn place<E>(&mut self, source: &PlaceSyntax, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match &source.kind {
            PlaceKind::Name(_) => Ok(()),
            PlaceKind::Qualified(path) => self.path(path, depth + 1),
            PlaceKind::Insert(source) => self.boxed(source.as_ref(), depth, Self::insert),
            PlaceKind::Member {
                base, ..
            }
            | PlaceKind::Dereference(base) => self.boxed(base.as_ref(), depth, Self::expression),
            PlaceKind::Index {
                base,
                index,
            } => {
                self.boxed(base.as_ref(), depth, Self::expression)?;
                self.boxed(index.as_ref(), depth, Self::expression)
            }
        }
    }

    pub(super) fn selection<E>(&mut self, selection: &UsingSelection, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match selection {
            UsingSelection::All => Ok(()),
            UsingSelection::Map(expression) => {
                self.boxed(expression.as_ref(), depth, Self::expression)
            }
            UsingSelection::Only(names) | UsingSelection::Except(names) => match names {
                UsingNames::Names(names) => self.allocation::<UsingName, E>(names.capacity()),
                UsingNames::Expression(expression) => {
                    self.boxed(expression.as_ref(), depth, Self::expression)
                }
            },
        }
    }

    pub(super) fn import_arguments<E>(
        &mut self,
        source: &ImportArguments,
        depth: usize,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        for arguments in [&source.instance, &source.program].into_iter().flatten() {
            self.sequence(arguments, depth, |this, argument, depth| {
                this.node(depth)?;
                match &argument.value {
                    ModuleArgumentValue::Expression(value) => this.expression(value, depth + 1),
                    ModuleArgumentValue::String(value) => this.text(value),
                }
            })?;
        }
        Ok(())
    }

    pub(super) fn context_field<E>(
        &mut self,
        source: &ContextFieldDeclaration,
        depth: usize,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match source {
            ContextFieldDeclaration::Field(source) => self.field(source, depth + 1),
            ContextFieldDeclaration::Variable(source) => {
                self.declaration(&source.declaration, depth + 1)
            }
            ContextFieldDeclaration::Constant(source) => self.constant(source, depth + 1),
        }
    }

    pub(super) fn statement<E>(&mut self, source: &Statement, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match &source.kind {
            StatementKind::Empty => Ok(()),
            StatementKind::Jump {
                ..
            } => Ok(()),
            StatementKind::InstructionBytes(source) => self.allocation::<u8, E>(source.bytes.len()),
            StatementKind::Simd(source) => {
                self.allocation::<SimdFeatureRequirement, E>(source.features.capacity())?;
                self.sequence(&source.statements, depth, |this, statement, depth| {
                    this.node(depth)?;
                    match statement {
                        SimdStatement::Instruction(instruction) => {
                            this.sequence(&instruction.operands, depth, |this, operand, depth| {
                                this.node(depth)?;
                                match &operand.kind {
                                    SimdOperandKind::Register {
                                        ..
                                    } => Ok(()),
                                    SimdOperandKind::Memory(value) => {
                                        this.expression(value, depth + 1)
                                    }
                                }
                            })
                        }
                        SimdStatement::Unsupported {
                            ..
                        }
                        | SimdStatement::DebugTrap {
                            ..
                        }
                        | SimdStatement::Interrupt {
                            ..
                        }
                        | SimdStatement::RegisterDeclaration {
                            ..
                        } => Ok(()),
                    }
                })
            }
            StatementKind::Import(source) => {
                self.text(&source.target)?;
                self.import_arguments(&source.arguments, depth + 1)
            }
            StatementKind::Using(source) => {
                self.expression(&source.target, depth + 1)?;
                self.selection(&source.selection, depth + 1)
            }
            StatementKind::UsingDeclaration {
                declaration,
                selection,
                ..
            } => {
                self.boxed(declaration.as_ref(), depth, Self::statement)?;
                self.selection(selection, depth + 1)
            }
            StatementKind::CallerExport(statement) => {
                self.boxed(statement.as_ref(), depth, Self::statement)
            }
            StatementKind::CompileTimeAssert {
                condition,
                message,
            } => self.assertion(condition, message, depth),
            StatementKind::CompileTimeIf {
                condition,
                then_body,
                else_body,
            }
            | StatementKind::If(condition, then_body, else_body) => {
                self.expression(condition, depth + 1)?;
                self.sequence(then_body, depth, Self::statement)?;
                self.sequence(else_body, depth, Self::statement)
            }
            StatementKind::CheckScope {
                body, ..
            }
            | StatementKind::Block(body)
            | StatementKind::Defer(body) => self.sequence(body, depth, Self::statement),
            StatementKind::ContextField(field) => self.context_field(field, depth + 1),
            StatementKind::Library(source) => self.text(&source.target),
            StatementKind::Procedure(source) => {
                self.boxed(source.as_ref(), depth, |this, source, depth| {
                    this.source_procedure(&source.source, depth)
                })
            }
            StatementKind::ProcedurePrototype(source) => self.prototype(source, depth + 1),
            StatementKind::Record(source) => self.record(source, depth + 1),
            StatementKind::Enum(source) => self.enumeration(source, depth + 1),
            StatementKind::TypeAlias(source) => self.ty(&source.ty, depth + 1),
            StatementKind::Insert(source) => self.insert(source, depth + 1),
            StatementKind::Declare(source) => self.declaration(source, depth + 1),
            StatementKind::Constant(source) => self.constant(source, depth + 1),
            StatementKind::ConstantResults(source) => {
                self.allocation::<(Symbol, Span), E>(source.names.capacity())?;
                self.expression(&source.initializer, depth + 1)
            }
            StatementKind::Assign(_, value)
            | StatementKind::Update(_, _, value)
            | StatementKind::Expression(value) => self.expression(value, depth + 1),
            StatementKind::AssignPlace {
                target,
                value,
            }
            | StatementKind::UpdatePlace {
                target,
                value,
                ..
            } => {
                self.place(target, depth + 1)?;
                self.expression(value, depth + 1)
            }
            StatementKind::DeclareResults {
                names,
                ty,
                values,
            } => {
                self.allocation::<Symbol, E>(names.capacity())?;
                if let Some(ty) = ty {
                    self.ty(ty, depth + 1)?;
                }
                self.sequence(values, depth, Self::expression)
            }
            StatementKind::MixedResults {
                bindings,
                ty,
                values,
            } => {
                self.sequence(bindings, depth, |this, binding, depth| {
                    this.node(depth)?;
                    match binding {
                        ResultTargetBinding::New {
                            ..
                        } => Ok(()),
                        ResultTargetBinding::Existing(place) => this.place(place, depth + 1),
                    }
                })?;
                if let Some(ty) = ty {
                    self.ty(ty, depth + 1)?;
                }
                self.sequence(values, depth, Self::expression)
            }
            StatementKind::AssignResults {
                targets,
                values,
                ..
            } => {
                self.sequence(targets, depth, Self::place)?;
                self.sequence(values, depth, Self::expression)
            }
            StatementKind::Return(value)
            | StatementKind::PushContextDeferred {
                value,
            } => {
                if let Some(value) = value {
                    self.expression(value, depth + 1)?;
                }
                Ok(())
            }
            StatementKind::ReturnValues(values) => {
                self.sequence(values, depth, |this, value, depth| {
                    this.node(depth)?;
                    this.expression(&value.value, depth + 1)
                })
            }
            StatementKind::Cases(source) => {
                self.expression(&source.value, depth + 1)?;
                self.sequence(&source.arms, depth, |this, (condition, body, _), depth| {
                    this.node(depth)?;
                    this.expression(condition, depth + 1)?;
                    this.sequence(body, depth, Self::statement)
                })?;
                if let Some(body) = &source.default {
                    self.sequence(body, depth, Self::statement)?;
                }
                Ok(())
            }
            StatementKind::CompileTimeCases(source) => {
                self.source_cases(source, depth, Self::statement)
            }
            StatementKind::While(condition, body) => {
                match condition {
                    WhileCondition::Expression(value)
                    | WhileCondition::Binding {
                        initializer: value,
                        ..
                    } => self.expression(value, depth + 1)?,
                }
                self.sequence(body, depth, Self::statement)
            }
            StatementKind::Range(source) => {
                self.expression(&source.start, depth + 1)?;
                self.expression(&source.end, depth + 1)?;
                if let Some(control) = &source.reverse_control {
                    self.expression(control, depth + 1)?;
                }
                self.sequence(&source.body, depth, Self::statement)
            }
            StatementKind::ArrayLoop(source) => {
                if let Some(expansion) = &source.expansion {
                    self.path(expansion, depth + 1)?;
                }
                self.expression(&source.sequence, depth + 1)?;
                for control in [&source.reverse_control, &source.pointer_control]
                    .into_iter()
                    .flatten()
                {
                    self.expression(control, depth + 1)?;
                }
                self.sequence(&source.body, depth, Self::statement)
            }
            StatementKind::PushContext {
                value,
                body,
            } => {
                if let Some(value) = value {
                    self.expression(value, depth + 1)?;
                }
                self.sequence(body, depth, Self::statement)
            }
        }
    }
}
