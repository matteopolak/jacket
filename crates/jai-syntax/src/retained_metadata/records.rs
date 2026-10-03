use super::*;

impl<A> Visitor<'_, A> {
    pub(super) fn source_procedure<E>(
        &mut self,
        source: &SourceProcedureSyntax,
        depth: usize,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.callable(&source.header.callable, depth + 1)?;
        if let Some(compiler) = &source.header.compiler
            && let Some(tag) = &compiler.tag
        {
            self.text(tag)?;
        }
        self.modify(&source.header.modify, depth + 1)?;
        self.sequence(&source.body, depth, Self::statement)
    }

    fn callable<E>(&mut self, source: &CallableHeaderSyntax, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        if let Some(deprecation) = &source.deprecation
            && let Some(message) = &deprecation.message
        {
            self.allocation::<u8, E>(message.capacity())?;
        }
        self.sequence(&source.notes, depth, Self::note)?;
        self.sequence(&source.parameters, depth, |this, parameter, depth| {
            this.node(depth)?;
            match &parameter.binding {
                ParameterBinding::Required(_) => Ok(()),
                ParameterBinding::RequiredType(ty) => this.ty(ty, depth + 1),
                ParameterBinding::Defaulted {
                    expression, ..
                } => this.expression(expression, depth + 1),
                ParameterBinding::DefaultedType {
                    ty,
                    expression,
                } => {
                    if let Some(ty) = ty {
                        this.ty(ty, depth + 1)?;
                    }
                    this.expression(expression, depth + 1)
                }
            }
        })?;
        self.sequence(&source.results, depth, |this, result, depth| {
            this.node(depth)?;
            match &result.binding {
                ResultBinding::Typed {
                    ty,
                    default,
                } => {
                    this.ty(ty, depth + 1)?;
                    if let Some(default) = default {
                        this.expression(default, depth + 1)?;
                    }
                    Ok(())
                }
                ResultBinding::InferredDefault(value) => this.expression(value, depth + 1),
            }
        })
    }

    pub(super) fn prototype<E>(&mut self, source: &ProcedurePrototype, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.callable(&source.header, depth + 1)?;
        match &source.binding {
            PrototypeBinding::EntryPoint => Ok(()),
            PrototypeBinding::Intrinsic {
                tag,
            }
            | PrototypeBinding::Compiler(CompilerProcedure {
                tag,
            }) => {
                if let Some(tag) = tag {
                    self.text(tag)?;
                }
                Ok(())
            }
            PrototypeBinding::Foreign(source) => {
                if let Some(library) = &source.library {
                    self.path(library, depth + 1)?;
                }
                if let Some(symbol) = &source.symbol {
                    self.text(symbol)?;
                }
                Ok(())
            }
        }
    }

    fn note<E>(&mut self, source: &NoteSyntax, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.sequence(&source.arguments, depth, |this, argument, depth| {
            this.node(depth)?;
            match &argument.value {
                NoteValue::Word(_) => Ok(()),
                NoteValue::Expression(value) => this.expression(value, depth + 1),
            }
        })
    }

    fn modify<E>(&mut self, source: &Option<ModifyDirective>, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        if let Some(source) = source {
            self.node(depth)?;
            self.sequence(&source.body, depth, Self::statement)?;
        }
        Ok(())
    }

    fn record_body<E>(
        &mut self,
        parameters: &Vec<RecordParameter>,
        members: &Vec<RecordMember>,
        attributes: &Vec<RecordAttribute>,
        notes: &Vec<NoteSyntax>,
        modify: &Option<ModifyDirective>,
        depth: usize,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.sequence(parameters, depth, |this, parameter, depth| {
            this.node(depth)?;
            match &parameter.binding {
                RecordParameterBinding::Typed {
                    ty,
                    default,
                } => {
                    this.ty(ty, depth + 1)?;
                    if let Some(value) = default {
                        this.expression(value, depth + 1)?;
                    }
                    Ok(())
                }
                RecordParameterBinding::InferredDefault(value) => this.expression(value, depth + 1),
            }
        })?;
        self.sequence(members, depth, Self::record_member)?;
        self.sequence(attributes, depth, |this, attribute, depth| {
            this.node(depth)?;
            match attribute {
                RecordAttribute::Alignment(value) => this.expression(value, depth + 1),
                RecordAttribute::NoPadding
                | RecordAttribute::TypeInfoNone
                | RecordAttribute::Reflection(_) => Ok(()),
            }
        })?;
        self.sequence(notes, depth, Self::note)?;
        self.modify(modify, depth + 1)
    }

    pub(super) fn inline_record<E>(&mut self, source: &RecordTypeSyntax, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.record_body(
            &source.parameters,
            &source.members,
            &source.attributes,
            &source.notes,
            &source.modify,
            depth,
        )
    }

    pub(super) fn record<E>(&mut self, source: &RecordDeclaration, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.record_body(
            &source.parameters,
            &source.members,
            &source.attributes,
            &source.notes,
            &source.modify,
            depth,
        )
    }

    fn enum_body<E>(
        &mut self,
        representation: &Option<TypeSyntax>,
        members: &Vec<EnumMember>,
        depth: usize,
    ) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        if let Some(ty) = representation {
            self.ty(ty, depth + 1)?;
        }
        self.sequence(members, depth, |this, member, depth| {
            this.node(depth)?;
            if let Some(value) = &member.initializer {
                this.expression(value, depth + 1)?;
            }
            Ok(())
        })
    }

    pub(super) fn inline_enum<E>(&mut self, source: &EnumTypeSyntax, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.enum_body(&source.representation, &source.members, depth)
    }

    pub(super) fn enumeration<E>(&mut self, source: &EnumDeclaration, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        self.enum_body(&source.representation, &source.members, depth)
    }

    pub(super) fn field<E>(&mut self, source: &FieldDeclaration, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match &source.binding {
            FieldBinding::Explicit {
                ty,
                initializer,
            } => {
                self.ty(ty, depth + 1)?;
                if let Some(value) = initializer {
                    self.expression(value, depth + 1)?;
                }
            }
            FieldBinding::Inferred(value) => self.expression(value, depth + 1)?,
        }
        self.sequence(&source.attributes, depth, |this, attribute, depth| {
            this.node(depth)?;
            match attribute {
                FieldAttribute::Alignment(value) => this.expression(value, depth + 1),
                FieldAttribute::Placement(FieldPlacementSyntax::Overlay {
                    target, ..
                }) => this.place(target, depth + 1),
            }
        })?;
        self.sequence(&source.notes, depth, Self::note)
    }

    pub(super) fn record_member<E>(&mut self, source: &RecordMember, depth: usize) -> Result<E>
    where
        A: FnMut(usize, usize) -> std::result::Result<(), E>,
    {
        self.node(depth)?;
        match source {
            RecordMember::Placement(source) => self.place(&source.target, depth + 1),
            RecordMember::AnonymousRecord(record) => {
                self.boxed(record.as_ref(), depth, Self::inline_record)
            }
            RecordMember::DefaultOverride {
                target,
                value,
                ..
            } => {
                self.place(target, depth + 1)?;
                self.expression(value, depth + 1)
            }
            RecordMember::Assert {
                condition,
                message,
                ..
            } => self.assertion(condition, message, depth),
            RecordMember::CompileTimeCases {
                cases, ..
            } => self.source_cases(cases, depth, Self::record_member),
            RecordMember::Conditional {
                condition,
                then_members,
                else_members,
                ..
            } => {
                self.expression(condition, depth + 1)?;
                self.sequence(then_members, depth, Self::record_member)?;
                self.sequence(else_members, depth, Self::record_member)
            }
            RecordMember::Field(field) => self.field(field, depth + 1),
            RecordMember::Constant(constant) => self.constant(constant, depth + 1),
            RecordMember::TypeAlias(alias) => self.ty(&alias.ty, depth + 1),
            RecordMember::Procedure(source) => {
                self.boxed(source.as_ref(), depth, |this, source, depth| {
                    this.source_procedure(&source.source, depth)
                })
            }
            RecordMember::ProcedurePrototype(source) => self.prototype(source, depth + 1),
            RecordMember::Record(source) => self.boxed(source.as_ref(), depth, Self::record),
            RecordMember::Enum(source) => self.enumeration(source, depth + 1),
            RecordMember::Insert(source) => self.insert(source, depth + 1),
        }
    }
}
