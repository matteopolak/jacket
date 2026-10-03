//! Admit borrowed syntax before the compiler Code domain retains any AST clone.
use jai_source::{Diagnostic, Span};
use jai_syntax as syntax;

const MAX_NODES: usize = 65_536;
const MAX_DEPTH: usize = 128;
const MAX_BYTES: usize = 1_048_576;

enum Node<'a> {
    Code(&'a syntax::CodeBody),
    Expression(&'a syntax::Expression),
    Statement(&'a syntax::Statement),
    Type(&'a syntax::TypeSyntax),
    Place(&'a syntax::PlaceSyntax),
    Declaration(&'a syntax::Declaration),
    Constant(&'a syntax::ConstantDeclaration),
    Procedure(&'a syntax::Procedure),
    SourceProcedure(&'a syntax::SourceProcedureSyntax),
    Prototype(&'a syntax::ProcedurePrototype),
    Parameter(&'a syntax::Parameter),
    Result(&'a syntax::ProcedureResult),
    Record(&'a syntax::RecordDeclaration),
    RecordType(&'a syntax::RecordTypeSyntax),
    Member(&'a syntax::RecordMember),
    Field(&'a syntax::FieldDeclaration),
    Enum(Option<&'a syntax::TypeSyntax>, &'a [syntax::EnumMember]),
    Insert(&'a syntax::InsertDirective),
    Note(&'a syntax::NoteSyntax),
    Selection(&'a syntax::UsingSelection),
}

struct Budget<'a> {
    pending: Vec<(Node<'a>, usize)>,
    nodes: usize,
    bytes: usize,
    span: Span,
}
impl<'a> Budget<'a> {
    fn fail(&self, dimension: &str) -> Diagnostic {
        Diagnostic::new(
            self.span,
            format!("compiler quotation exceeds its {dimension} limit"),
        )
    }
    fn charge(&mut self, count: usize) -> Result<(), Diagnostic> {
        self.nodes = self
            .nodes
            .checked_add(count)
            .filter(|n| *n <= MAX_NODES)
            .ok_or_else(|| self.fail("syntax node"))?;
        Ok(())
    }
    fn bytes(&mut self, count: usize) -> Result<(), Diagnostic> {
        self.bytes = self
            .bytes
            .checked_add(count)
            .filter(|n| *n <= MAX_BYTES)
            .ok_or_else(|| self.fail("payload byte"))?;
        Ok(())
    }
    fn push(&mut self, node: Node<'a>, depth: usize) -> Result<(), Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(self.fail("syntax depth"));
        }
        self.charge(1)?;
        self.pending.push((node, depth));
        Ok(())
    }
    fn statements(
        &mut self,
        values: &'a [syntax::Statement],
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if values.len() > MAX_NODES.saturating_sub(self.nodes) {
            return Err(self.fail("syntax node"));
        }
        for value in values {
            self.push(Node::Statement(value), depth)?;
        }
        Ok(())
    }
    fn expressions(
        &mut self,
        values: &'a [syntax::Expression],
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if values.len() > MAX_NODES.saturating_sub(self.nodes) {
            return Err(self.fail("syntax node"));
        }
        for value in values {
            self.push(Node::Expression(value), depth)?;
        }
        Ok(())
    }
    fn optional_expression(
        &mut self,
        value: Option<&'a syntax::Expression>,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if let Some(value) = value {
            self.push(Node::Expression(value), depth)?;
        }
        Ok(())
    }
    fn optional_type(
        &mut self,
        value: Option<&'a syntax::TypeSyntax>,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if let Some(value) = value {
            self.push(Node::Type(value), depth)?;
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        args: &'a [syntax::CallArgument],
        depth: usize,
    ) -> Result<(), Diagnostic> {
        self.charge(args.len())?;
        for arg in args {
            self.push(Node::Expression(&arg.value), depth)?;
        }
        Ok(())
    }
    fn path(&mut self, path: &syntax::NamePath) -> Result<(), Diagnostic> {
        self.charge(path.members.len())
    }
    fn notes(&mut self, notes: &'a [syntax::NoteSyntax], depth: usize) -> Result<(), Diagnostic> {
        for note in notes {
            self.push(Node::Note(note), depth)?;
        }
        Ok(())
    }
    fn deprecation(&mut self, value: Option<&syntax::Deprecation>) -> Result<(), Diagnostic> {
        if let Some(value) = value {
            self.charge(1)?;
            if let Some(bytes) = &value.message {
                self.bytes(bytes.len())?;
            }
        }
        Ok(())
    }
    fn parameters(
        &mut self,
        values: &'a [syntax::Parameter],
        depth: usize,
    ) -> Result<(), Diagnostic> {
        for value in values {
            self.push(Node::Parameter(value), depth)?;
        }
        Ok(())
    }
    fn results(
        &mut self,
        values: &'a [syntax::ProcedureResult],
        depth: usize,
    ) -> Result<(), Diagnostic> {
        for value in values {
            self.push(Node::Result(value), depth)?;
        }
        Ok(())
    }
    fn members(
        &mut self,
        values: &'a [syntax::RecordMember],
        depth: usize,
    ) -> Result<(), Diagnostic> {
        for value in values {
            self.push(Node::Member(value), depth)?;
        }
        Ok(())
    }
    fn record_header(
        &mut self,
        parameters: &'a [syntax::RecordParameter],
        attributes: &'a [syntax::RecordAttribute],
        notes: &'a [syntax::NoteSyntax],
        modify: Option<&'a syntax::ModifyDirective>,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        self.charge(parameters.len())?;
        for parameter in parameters {
            match &parameter.binding {
                syntax::RecordParameterBinding::Typed {
                    ty,
                    default,
                } => {
                    self.push(Node::Type(ty), depth)?;
                    self.optional_expression(default.as_ref(), depth)?;
                }
                syntax::RecordParameterBinding::InferredDefault(value) => {
                    self.push(Node::Expression(value), depth)?
                }
            }
        }
        self.charge(attributes.len())?;
        for attribute in attributes {
            if let syntax::RecordAttribute::Alignment(value) = attribute {
                self.push(Node::Expression(value), depth)?;
            }
        }
        self.notes(notes, depth)?;
        if let Some(modify) = modify {
            self.statements(&modify.body, depth)?;
        }
        Ok(())
    }
    fn compile_time(
        &mut self,
        body: &'a syntax::CompileTimeBody,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        match body {
            syntax::CompileTimeBody::Expression(value) => {
                self.push(Node::Expression(value), depth)?
            }
            syntax::CompileTimeBody::Block(values) => self.statements(values, depth)?,
            syntax::CompileTimeBody::Procedure {
                result,
                body,
            } => {
                self.push(Node::Type(result), depth)?;
                self.statements(body, depth)?;
            }
        }
        Ok(())
    }
    fn import(
        &mut self,
        args: &'a syntax::ImportArguments,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        for group in [&args.instance, &args.program].into_iter().flatten() {
            self.charge(group.len())?;
            for arg in group {
                match &arg.value {
                    syntax::ModuleArgumentValue::Expression(value) => {
                        self.push(Node::Expression(value), depth)?
                    }
                    syntax::ModuleArgumentValue::String(value) => self.bytes(value.len())?,
                }
            }
        }
        Ok(())
    }
    fn source_procedure(
        &mut self,
        value: &'a syntax::SourceProcedureSyntax,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        self.deprecation(value.deprecation.as_ref())?;
        self.notes(&value.notes, depth)?;
        self.parameters(&value.parameters, depth)?;
        self.results(&value.results, depth)?;
        self.statements(&value.body, depth)?;
        if let Some(compiler) = &value.compiler {
            if let Some(tag) = &compiler.tag {
                self.bytes(tag.len())?;
            }
        }
        if let Some(modify) = &value.modify {
            self.statements(&modify.body, depth)?;
        }
        Ok(())
    }
    fn step(&mut self, node: Node<'a>, depth: usize) -> Result<(), Diagnostic> {
        use syntax::{ExpressionKind as E, StatementKind as S, TypeSyntax as T};
        let d = depth + 1;
        match node {
            Node::Code(body) => match body {
                syntax::CodeBody::Null => {}
                syntax::CodeBody::Expression(value) => self.push(Node::Expression(value), d)?,
                syntax::CodeBody::Block(values) => self.statements(values, d)?,
                syntax::CodeBody::Statement(value) => self.push(Node::Statement(value), d)?,
            },
            Node::Expression(expression) => match &expression.kind {
                E::String(value) => self.bytes(value.len())?,
                E::HereString(value) => {
                    self.bytes(value.bytes.len())?;
                    self.charge(value.modifiers.len())?;
                }
                E::Float(syntax::FloatLiteral::Decimal(value)) => {
                    self.bytes(value.spelling().len())?
                }
                E::Integer(_)
                | E::Float(_)
                | E::Character(_)
                | E::Null
                | E::CompileTimePredicate
                | E::CallerLocation
                | E::SourceLocation
                | E::SourceFile
                | E::SourceFilepath
                | E::SourceLine
                | E::Uninitialized
                | E::Bool(_)
                | E::Name(_)
                | E::Context
                | E::InferredMember(_)
                | E::CompileVariable(_) => {}
                E::Type(ty) => self.push(Node::Type(ty), d)?,
                E::CompileTime(run) => self.compile_time(&run.body, d)?,
                E::ShortLambda(lambda) => {
                    self.charge(lambda.parameters.len())?;
                    for parameter in &lambda.parameters {
                        self.optional_type(parameter.ty.as_ref(), d)?;
                    }
                    match &lambda.body.kind {
                        syntax::ShortLambdaBodyKind::Expression(value) => {
                            self.push(Node::Expression(value), d)?
                        }
                        syntax::ShortLambdaBodyKind::Block(values) => self.statements(values, d)?,
                    }
                }
                E::AnonymousProcedure(source) => self.push(Node::SourceProcedure(source), d)?,
                E::BakeArguments(value) => {
                    self.push(Node::Expression(&value.callee), d)?;
                    self.arguments(&value.arguments, d)?;
                }
                E::Code(body) => self.push(Node::Code(body), d)?,
                E::Insert(insert) => self.push(Node::Insert(insert), d)?,
                E::QualifiedName(path) => self.path(path)?,
                E::Call(_, args) => self.arguments(args, d)?,
                E::QualifiedCall(path, args) => {
                    self.path(path)?;
                    self.arguments(args, d)?;
                }
                E::IndirectCall {
                    callee,
                    args,
                } => {
                    self.push(Node::Expression(callee), d)?;
                    self.arguments(args, d)?;
                }
                E::ContextCall {
                    callee,
                    args,
                    overrides,
                } => {
                    self.push(Node::Expression(callee), d)?;
                    self.arguments(args, d)?;
                    self.arguments(overrides, d)?;
                }
                E::CallHint {
                    call, ..
                }
                | E::AddressOf(call)
                | E::Dereference(call)
                | E::Unary(_, call)
                | E::Cast(_, _, call)
                | E::InferredCast {
                    value: call, ..
                }
                | E::TypeQuery {
                    value: call, ..
                }
                | E::Member {
                    base: call, ..
                } => self.push(Node::Expression(call), d)?,
                E::Index {
                    base,
                    index,
                }
                | E::Binary(_, base, index) => {
                    self.push(Node::Expression(base), d)?;
                    self.push(Node::Expression(index), d)?;
                }
                E::TypeCast {
                    ty,
                    value,
                    ..
                } => {
                    self.push(Node::Type(ty), d)?;
                    self.push(Node::Expression(value), d)?;
                }
                E::ArrayLiteral(value) => {
                    self.optional_type(value.element_type.as_ref(), d)?;
                    self.expressions(&value.elements, d)?;
                }
                E::StructLiteral(value) => {
                    if let Some(ty) = &value.ty {
                        self.path(ty)?;
                    }
                    self.charge(value.fields.len())?;
                    for field in &value.fields {
                        self.push(Node::Expression(&field.value), d)?;
                    }
                }
                E::PositionalStructLiteral(value) => {
                    if let Some(ty) = &value.ty {
                        self.path(ty)?;
                    }
                    self.expressions(&value.values, d)?;
                }
                E::Conditional(value) => {
                    self.push(Node::Expression(&value.condition), d)?;
                    self.push(Node::Expression(&value.then_value), d)?;
                    self.optional_expression(value.else_value.as_deref(), d)?;
                }
            },
            Node::Type(ty) => match ty {
                T::This | T::Builtin(_) | T::Variable(_) => {}
                T::Named(path) => self.path(path)?,
                T::TypeOf(value) => self.push(Node::Expression(value), d)?,
                T::Restricted {
                    restriction, ..
                } => {
                    let (syntax::TypeRestrictionSyntax::Nominal(ty)
                    | syntax::TypeRestrictionSyntax::Interface(ty)) = restriction;
                    self.push(Node::Type(ty), d)?;
                }
                T::InlineRecord(value) => self.push(Node::RecordType(value), d)?,
                T::InlineEnum(value) => {
                    self.push(Node::Enum(value.representation.as_ref(), &value.members), d)?
                }
                T::Variant {
                    base, ..
                }
                | T::Pointer(base)
                | T::Slice(base)
                | T::DynamicArray(base) => self.push(Node::Type(base), d)?,
                T::FixedArray {
                    count,
                    element,
                } => {
                    self.push(Node::Expression(count), d)?;
                    self.push(Node::Type(element), d)?;
                }
                T::Procedure(value) => {
                    self.charge(value.parameters.len())?;
                    self.charge(value.results.len())?;
                    for parameter in value.parameters.iter().chain(&value.results) {
                        self.push(Node::Type(&parameter.ty), d)?;
                        if let Some(default) = &parameter.default {
                            self.push(Node::Expression(default), d)?;
                        }
                    }
                }
                T::Application(value) => {
                    self.push(Node::Type(&value.base), d)?;
                    self.arguments(&value.arguments, d)?;
                }
            },
            Node::Place(place) => match &place.kind {
                syntax::PlaceKind::Name(_) => {}
                syntax::PlaceKind::Qualified(path) => self.path(path)?,
                syntax::PlaceKind::Insert(value) => self.push(Node::Insert(value), d)?,
                syntax::PlaceKind::Member {
                    base, ..
                }
                | syntax::PlaceKind::Dereference(base) => self.push(Node::Expression(base), d)?,
                syntax::PlaceKind::Index {
                    base,
                    index,
                } => {
                    self.push(Node::Expression(base), d)?;
                    self.push(Node::Expression(index), d)?;
                }
            },
            Node::Declaration(value) => {
                self.charge(value.attributes().len())?;
                for syntax::DeclarationAttribute::Alignment(expr) in value.attributes() {
                    self.push(Node::Expression(expr), d)?;
                }
                match value {
                    syntax::Declaration::GroupMember {
                        group, ..
                    } => {
                        self.charge(group.names().len())?;
                        self.push(Node::Declaration(group.source()), d)?;
                        self.charge(group.extra_initializers().len())?;
                        for initializer in group.extra_initializers() {
                            self.push(Node::Expression(initializer), d)?;
                        }
                    }
                    syntax::Declaration::External {
                        ty,
                        binding,
                        ..
                    } => {
                        self.push(Node::Type(ty), d)?;
                        if let syntax::ExternalDataSource::Library(path) = &binding.source {
                            self.path(path)?;
                        }
                        if let Some(symbol) = &binding.symbol {
                            self.bytes(symbol.len())?;
                        }
                    }
                    syntax::Declaration::Inferred {
                        initializer, ..
                    } => self.push(Node::Expression(initializer), d)?,
                    syntax::Declaration::Explicit {
                        initializer, ..
                    } => self.optional_expression(initializer.as_ref(), d)?,
                    syntax::Declaration::UnresolvedExplicit {
                        ty,
                        initializer,
                        ..
                    } => {
                        self.push(Node::Type(ty), d)?;
                        self.optional_expression(initializer.as_ref(), d)?;
                    }
                }
            }
            Node::Constant(value) => {
                self.optional_type(value.ty.as_ref(), d)?;
                self.push(Node::Expression(&value.initializer), d)?;
            }
            Node::Parameter(value) => match &value.binding {
                syntax::ParameterBinding::Required(_) => {}
                syntax::ParameterBinding::Defaulted {
                    expression, ..
                } => self.push(Node::Expression(expression), d)?,
                syntax::ParameterBinding::RequiredType(ty) => self.push(Node::Type(ty), d)?,
                syntax::ParameterBinding::DefaultedType {
                    ty,
                    expression,
                } => {
                    self.optional_type(ty.as_ref(), d)?;
                    self.push(Node::Expression(expression), d)?;
                }
            },
            Node::Result(value) => match &value.binding {
                syntax::ResultBinding::Typed {
                    ty,
                    default,
                } => {
                    self.push(Node::Type(ty), d)?;
                    self.optional_expression(default.as_ref(), d)?;
                }
                syntax::ResultBinding::InferredDefault(value) => {
                    self.push(Node::Expression(value), d)?
                }
            },
            Node::Procedure(value) => self.source_procedure(&value.source, d)?,
            Node::SourceProcedure(value) => self.source_procedure(value, d)?,
            Node::Prototype(value) => {
                self.deprecation(value.deprecation.as_ref())?;
                self.notes(&value.notes, d)?;
                self.parameters(&value.parameters, d)?;
                self.results(&value.results, d)?;
                match &value.binding {
                    syntax::PrototypeBinding::EntryPoint => {}
                    syntax::PrototypeBinding::Intrinsic {
                        tag,
                    } => {
                        if let Some(tag) = tag {
                            self.bytes(tag.len())?;
                        }
                    }
                    syntax::PrototypeBinding::Compiler(compiler) => {
                        if let Some(tag) = &compiler.tag {
                            self.bytes(tag.len())?;
                        }
                    }
                    syntax::PrototypeBinding::Foreign(foreign) => {
                        if let Some(path) = &foreign.library {
                            self.path(path)?;
                        }
                        if let Some(symbol) = &foreign.symbol {
                            self.bytes(symbol.len())?;
                        }
                    }
                }
            }
            Node::Record(value) => {
                self.record_header(
                    &value.parameters,
                    &value.attributes,
                    &value.notes,
                    value.modify.as_ref(),
                    d,
                )?;
                self.members(&value.members, d)?;
            }
            Node::RecordType(value) => {
                self.record_header(
                    &value.parameters,
                    &value.attributes,
                    &value.notes,
                    value.modify.as_ref(),
                    d,
                )?;
                self.members(&value.members, d)?;
            }
            Node::Field(value) => {
                self.charge(value.attributes.len())?;
                for attribute in &value.attributes {
                    match attribute {
                        syntax::FieldAttribute::Alignment(expr) => {
                            self.push(Node::Expression(expr), d)?
                        }
                        syntax::FieldAttribute::Placement(
                            syntax::FieldPlacementSyntax::Overlay {
                                target, ..
                            },
                        ) => self.push(Node::Place(target), d)?,
                    }
                }
                self.notes(&value.notes, d)?;
                match &value.binding {
                    syntax::FieldBinding::Explicit {
                        ty,
                        initializer,
                    } => {
                        self.push(Node::Type(ty), d)?;
                        self.optional_expression(initializer.as_ref(), d)?;
                    }
                    syntax::FieldBinding::Inferred(value) => {
                        self.push(Node::Expression(value), d)?
                    }
                }
            }
            Node::Enum(representation, members) => {
                self.optional_type(representation, d)?;
                self.charge(members.len())?;
                for member in members {
                    self.optional_expression(member.initializer.as_ref(), d)?;
                }
            }
            Node::Member(value) => match value {
                syntax::RecordMember::Placement(placement) => {
                    self.push(Node::Place(&placement.target), d)?
                }
                syntax::RecordMember::AnonymousRecord(value) => {
                    self.push(Node::RecordType(value), d)?
                }
                syntax::RecordMember::DefaultOverride {
                    target,
                    value,
                    ..
                } => {
                    self.push(Node::Place(target), d)?;
                    self.push(Node::Expression(value), d)?;
                }
                syntax::RecordMember::Assert {
                    condition,
                    message,
                    ..
                } => {
                    self.push(Node::Expression(condition), d)?;
                    self.optional_expression(message.as_ref(), d)?;
                }
                syntax::RecordMember::Conditional {
                    condition,
                    then_members,
                    else_members,
                    ..
                } => {
                    self.push(Node::Expression(condition), d)?;
                    self.members(then_members, d)?;
                    self.members(else_members, d)?;
                }
                syntax::RecordMember::CompileTimeCases {
                    cases, ..
                } => {
                    self.push(Node::Expression(&cases.value), d)?;
                    self.charge(cases.arms.len())?;
                    for arm in &cases.arms {
                        self.push(Node::Expression(&arm.label), d)?;
                        self.members(&arm.body, d)?;
                    }
                    if let Some(default) = &cases.default {
                        self.members(&default.body, d)?;
                    }
                }
                syntax::RecordMember::Field(value) => self.push(Node::Field(value), d)?,
                syntax::RecordMember::Constant(value) => self.push(Node::Constant(value), d)?,
                syntax::RecordMember::TypeAlias(value) => self.push(Node::Type(&value.ty), d)?,
                syntax::RecordMember::Procedure(value) => self.push(Node::Procedure(value), d)?,
                syntax::RecordMember::ProcedurePrototype(value) => {
                    self.push(Node::Prototype(value), d)?
                }
                syntax::RecordMember::Record(value) => self.push(Node::Record(value), d)?,
                syntax::RecordMember::Enum(value) => {
                    self.push(Node::Enum(value.representation.as_ref(), &value.members), d)?
                }
                syntax::RecordMember::Insert(value) => self.push(Node::Insert(value), d)?,
            },
            Node::Insert(value) => {
                self.push(Node::Expression(&value.value), d)?;
                self.charge(value.replacements.len())?;
                for replacement in &value.replacements {
                    match &replacement.body {
                        syntax::LoopControlReplacementBody::Code(body) => {
                            self.push(Node::Code(body), d)?
                        }
                        syntax::LoopControlReplacementBody::Assert {
                            condition, ..
                        } => self.push(Node::Expression(condition), d)?,
                    }
                }
            }
            Node::Note(value) => {
                self.charge(value.arguments.len())?;
                for arg in &value.arguments {
                    if let syntax::NoteValue::Expression(value) = &arg.value {
                        self.push(Node::Expression(value), d)?;
                    }
                }
            }
            Node::Selection(value) => match value {
                syntax::UsingSelection::All => {}
                syntax::UsingSelection::Map(value) => self.push(Node::Expression(value), d)?,
                syntax::UsingSelection::Only(names) | syntax::UsingSelection::Except(names) => {
                    match names {
                        syntax::UsingNames::Names(names) => self.charge(names.len())?,
                        syntax::UsingNames::Expression(value) => {
                            self.push(Node::Expression(value), d)?
                        }
                    }
                }
            },
            Node::Statement(statement) => match &statement.kind {
                // The statement node was admitted when enqueued; Empty owns no children.
                S::Empty => {}
                S::InstructionBytes(value) => self.bytes(value.bytes.len())?,
                S::Simd(value) => {
                    self.charge(value.features.len())?;
                    self.charge(value.statements.len())?;
                    for statement in &value.statements {
                        if let syntax::SimdStatement::Instruction(value) = statement {
                            self.charge(value.operands.len())?;
                            for operand in &value.operands {
                                if let syntax::SimdOperandKind::Memory(value) = &operand.kind {
                                    self.push(Node::Expression(value), d)?;
                                }
                            }
                        }
                    }
                }
                S::Import(value) => {
                    self.bytes(value.target.len())?;
                    self.import(&value.arguments, d)?;
                }
                S::Using(value) => {
                    self.push(Node::Expression(&value.target), d)?;
                    self.push(Node::Selection(&value.selection), d)?;
                }
                S::UsingDeclaration {
                    declaration,
                    selection,
                    ..
                } => {
                    self.push(Node::Statement(declaration), d)?;
                    self.push(Node::Selection(selection), d)?;
                }
                S::CallerExport(value) => self.push(Node::Statement(value), d)?,
                S::CompileTimeAssert {
                    condition,
                    message,
                } => {
                    self.push(Node::Expression(condition), d)?;
                    self.optional_expression(message.as_ref(), d)?;
                }
                S::CompileTimeIf {
                    condition,
                    then_body,
                    else_body,
                }
                | S::If(condition, then_body, else_body) => {
                    self.push(Node::Expression(condition), d)?;
                    self.statements(then_body, d)?;
                    self.statements(else_body, d)?;
                }
                S::CheckScope {
                    body, ..
                }
                | S::Block(body)
                | S::Defer(body) => self.statements(body, d)?,
                S::ContextField(value) => match value {
                    syntax::ContextFieldDeclaration::Field(value) => {
                        self.push(Node::Field(value), d)?
                    }
                    syntax::ContextFieldDeclaration::Variable(value) => {
                        self.push(Node::Declaration(&value.declaration), d)?
                    }
                    syntax::ContextFieldDeclaration::Constant(value) => {
                        self.push(Node::Constant(value), d)?
                    }
                },
                S::Library(value) => self.bytes(value.target.len())?,
                S::Procedure(value) => self.push(Node::Procedure(value), d)?,
                S::ProcedurePrototype(value) => self.push(Node::Prototype(value), d)?,
                S::Record(value) => self.push(Node::Record(value), d)?,
                S::Enum(value) => {
                    self.push(Node::Enum(value.representation.as_ref(), &value.members), d)?
                }
                S::TypeAlias(value) => self.push(Node::Type(&value.ty), d)?,
                S::Insert(value) => self.push(Node::Insert(value), d)?,
                S::Declare(value) => self.push(Node::Declaration(value), d)?,
                S::Constant(value) => self.push(Node::Constant(value), d)?,
                S::ConstantResults(value) => {
                    self.charge(value.names.len())?;
                    self.push(Node::Expression(&value.initializer), d)?;
                }
                S::Assign(_, value) | S::Update(_, _, value) | S::Expression(value) => {
                    self.push(Node::Expression(value), d)?
                }
                S::AssignPlace {
                    target,
                    value,
                }
                | S::UpdatePlace {
                    target,
                    value,
                    ..
                } => {
                    self.push(Node::Place(target), d)?;
                    self.push(Node::Expression(value), d)?;
                }
                S::DeclareResults {
                    names,
                    ty,
                    values,
                } => {
                    self.charge(names.len())?;
                    self.optional_type(ty.as_ref(), d)?;
                    self.expressions(values, d)?;
                }
                S::MixedResults {
                    bindings,
                    ty,
                    values,
                } => {
                    self.charge(bindings.len())?;
                    for binding in bindings {
                        if let syntax::ResultTargetBinding::Existing(place) = binding {
                            self.push(Node::Place(place), d)?;
                        }
                    }
                    self.optional_type(ty.as_ref(), d)?;
                    self.expressions(values, d)?;
                }
                S::AssignResults {
                    targets,
                    values,
                    ..
                } => {
                    for target in targets {
                        self.push(Node::Place(target), d)?;
                    }
                    self.expressions(values, d)?;
                }
                S::Return(value) => self.optional_expression(value.as_ref(), d)?,
                S::ReturnValues(values) => {
                    self.charge(values.len())?;
                    for value in values {
                        self.push(Node::Expression(&value.value), d)?;
                    }
                }
                S::Cases(value) => {
                    self.push(Node::Expression(&value.value), d)?;
                    self.charge(value.arms.len())?;
                    for (condition, body, _) in &value.arms {
                        self.push(Node::Expression(condition), d)?;
                        self.statements(body, d)?;
                    }
                    if let Some(default) = &value.default {
                        self.statements(default, d)?;
                    }
                }
                S::CompileTimeCases(value) => {
                    self.push(Node::Expression(&value.value), d)?;
                    self.charge(value.arms.len())?;
                    for arm in &value.arms {
                        self.push(Node::Expression(&arm.label), d)?;
                        self.statements(&arm.body, d)?;
                    }
                    if let Some(default) = &value.default {
                        self.statements(&default.body, d)?;
                    }
                }
                S::While(condition, body) => {
                    match condition {
                        syntax::WhileCondition::Expression(value)
                        | syntax::WhileCondition::Binding {
                            initializer: value,
                            ..
                        } => self.push(Node::Expression(value), d)?,
                    }
                    self.statements(body, d)?;
                }
                S::Range(value) => {
                    self.push(Node::Expression(&value.start), d)?;
                    self.push(Node::Expression(&value.end), d)?;
                    self.optional_expression(value.reverse_control.as_ref(), d)?;
                    self.statements(&value.body, d)?;
                }
                S::ArrayLoop(value) => {
                    if let Some(expansion) = &value.expansion {
                        self.path(expansion)?;
                    }
                    self.push(Node::Expression(&value.sequence), d)?;
                    self.optional_expression(value.reverse_control.as_ref(), d)?;
                    self.optional_expression(value.pointer_control.as_ref(), d)?;
                    self.statements(&value.body, d)?;
                }
                S::PushContext {
                    value,
                    body,
                } => {
                    self.optional_expression(value.as_ref(), d)?;
                    self.statements(body, d)?;
                }
                S::PushContextDeferred {
                    value,
                } => {
                    self.optional_expression(value.as_ref(), d)?;
                }
                S::Jump {
                    ..
                } => {}
            },
        }
        Ok(())
    }
}

/// One semantic session charges every compiler-source retention against the
/// same cumulative allocation allowance, including temporary overlapping copies.
#[derive(Default)]
pub(crate) struct CompilerQuoteBudget {
    nodes: usize,
    bytes: usize,
}
impl CompilerQuoteBudget {
    pub(crate) fn admit(&mut self, body: &syntax::CodeBody, span: Span) -> Result<(), Diagnostic> {
        self.admit_node(Node::Code(body), span)
    }
    pub(crate) fn admit_procedure(
        &mut self,
        value: &syntax::Procedure,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.admit_node(Node::Procedure(value), span)
    }
    pub(crate) fn admit_record(
        &mut self,
        value: &syntax::RecordDeclaration,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.admit_node(Node::Record(value), span)
    }
    pub(crate) fn admit_enum(
        &mut self,
        value: &syntax::EnumDeclaration,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.admit_node(
            Node::Enum(value.representation.as_ref(), &value.members),
            span,
        )
    }
    pub(crate) fn admit_type(
        &mut self,
        value: &syntax::TypeSyntax,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.admit_node(Node::Type(value), span)
    }
    pub(crate) fn admit_constant(
        &mut self,
        value: &syntax::ConstantDeclaration,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.admit_node(Node::Constant(value), span)
    }
    pub(crate) fn admit_prototype(
        &mut self,
        value: &syntax::ProcedurePrototype,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.admit_node(Node::Prototype(value), span)
    }
    pub(crate) fn admit_declaration(
        &mut self,
        value: &syntax::Declaration,
        span: Span,
    ) -> Result<(), Diagnostic> {
        self.admit_node(Node::Declaration(value), span)
    }
    pub(crate) fn retain_metadata(
        &mut self,
        count: usize,
        bytes: usize,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let mut budget = Budget {
            pending: Vec::new(),
            nodes: self.nodes,
            bytes: self.bytes,
            span,
        };
        budget.charge(count)?;
        budget.bytes(bytes)?;
        self.nodes = budget.nodes;
        self.bytes = budget.bytes;
        Ok(())
    }
    pub(crate) fn admit_substitution(
        &mut self,
        value: &crate::polymorphism::Substitution,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if !value.callables.is_empty() {
            return Err(Diagnostic::new(
                span,
                "compiler Code capture requires a checked retained size receipt for callable substitutions",
            ));
        }
        self.retain_metadata(
            value.types.len().saturating_add(value.constants.len()),
            0,
            span,
        )?;
        for binding in &value.constants {
            match &binding.value {
                crate::polymorphism::BakedValue::String(value) => {
                    self.retain_metadata(1, value.len(), span)?
                }
                crate::polymorphism::BakedValue::Value(value) => {
                    self.admit_value(value, span, 0)?
                }
                _ => self.retain_metadata(1, 0, span)?,
            }
        }
        Ok(())
    }
    fn admit_value(
        &mut self,
        value: &jai_ir::ConstantValue,
        span: Span,
        depth: usize,
    ) -> Result<(), Diagnostic> {
        if depth >= MAX_DEPTH {
            return Err(Diagnostic::new(
                span,
                "compiler source constant exceeds its retention depth limit",
            ));
        }
        self.retain_metadata(1, 0, span)?;
        match &value.kind {
            jai_ir::ConstantKind::Array(values) | jai_ir::ConstantKind::Record(values) => {
                for value in values {
                    self.admit_value(value, span, depth + 1)?;
                }
            }
            jai_ir::ConstantKind::Distinct(value)
            | jai_ir::ConstantKind::Union {
                value, ..
            } => self.admit_value(value, span, depth + 1)?,
            jai_ir::ConstantKind::StringBytes(value) => {
                self.retain_metadata(0, value.len(), span)?
            }
            _ => {}
        }
        Ok(())
    }
    fn admit_node(&mut self, node: Node<'_>, span: Span) -> Result<(), Diagnostic> {
        let mut budget = Budget {
            pending: Vec::new(),
            nodes: self.nodes,
            bytes: self.bytes,
            span,
        };
        budget.push(node, 0)?;
        while let Some((node, depth)) = budget.pending.pop() {
            budget.step(node, depth)?;
        }
        self.nodes = budget.nodes;
        self.bytes = budget.bytes;
        Ok(())
    }
}
#[cfg(test)]
fn admit_compiler_quote(body: &syntax::CodeBody, span: Span) -> Result<(), Diagnostic> {
    CompilerQuoteBudget::default().admit(body, span)
}
#[cfg(test)]
fn admit_compiler_code_procedure(
    procedure: &syntax::Procedure,
    span: Span,
) -> Result<(), Diagnostic> {
    CompilerQuoteBudget::default().admit_node(Node::Procedure(procedure), span)
}

#[cfg(test)]
mod tests {
    use super::*;
    use syntax::{CodeBody, Expression, ExpressionKind};
    fn leaf() -> Expression {
        Expression {
            span: Span::new(0, 1),
            kind: ExpressionKind::Integer(1),
        }
    }
    #[test]
    fn borrowed_deep_quote_is_rejected_without_cloning_or_owned_drop() {
        let mut value = leaf();
        for _ in 0..20_000 {
            value = Expression {
                span: value.span,
                kind: ExpressionKind::AddressOf(Box::new(value)),
            };
        }
        let body = CodeBody::Expression(Box::new(value));
        let diagnostic = admit_compiler_quote(&body, Span::new(4, 8)).unwrap_err();
        assert!(diagnostic.message.contains("syntax depth"));
        let CodeBody::Expression(mut current) = body else {
            unreachable!()
        };
        while let ExpressionKind::AddressOf(inner) = current.kind {
            current = inner;
        }
    }
    #[test]
    fn wide_and_large_payload_quotes_are_rejected_before_retention() {
        let wide = CodeBody::Expression(Box::new(Expression {
            span: Span::new(0, 1),
            kind: ExpressionKind::ArrayLiteral(syntax::ArrayLiteral {
                element_type: None,
                elements: vec![leaf(); MAX_NODES],
            }),
        }));
        assert!(
            admit_compiler_quote(&wide, Span::new(0, 1))
                .unwrap_err()
                .message
                .contains("syntax node")
        );
        let large = CodeBody::Expression(Box::new(Expression {
            span: Span::new(0, 1),
            kind: ExpressionKind::String(vec![0; MAX_BYTES + 1]),
        }));
        assert!(
            admit_compiler_quote(&large, Span::new(0, 1))
                .unwrap_err()
                .message
                .contains("payload byte")
        );
    }
    #[test]
    fn procedure_nested_type_and_note_payloads_are_admitted() {
        let mut symbols = jai_source::Symbols::default();
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("quote-budget.jai".into(), "q :: #code { Thing :: struct { x: [4] int; step :: (a: int) -> int { return a + 1; } } Answer :: 42; }".into());
        let file = syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!("declaration")
        };
        let syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
            panic!("constant")
        };
        let ExpressionKind::Code(body) = &constant.initializer.kind else {
            panic!("quotation")
        };
        admit_compiler_quote(body, constant.span).unwrap();
        let mut cumulative = CompilerQuoteBudget::default();
        cumulative.admit(body, constant.span).unwrap();
        cumulative.admit(body, constant.span).unwrap();
    }
    #[test]
    fn anonymous_source_bodies_are_charged_before_quotation_retention() {
        let module = syntax::parse("main :: () { callback := () { return; }; }").unwrap();
        let syntax::StatementKind::Declare(syntax::Declaration::Inferred {
            initializer, ..
        }) = &module.procedures()[0].body[0].kind
        else {
            panic!("anonymous declaration")
        };
        let mut initializer = initializer.clone();
        let ExpressionKind::AnonymousProcedure(source) = &mut initializer.kind else {
            panic!("anonymous source")
        };
        source.body.push(syntax::Statement {
            span: source.span,
            kind: syntax::StatementKind::Expression(Expression {
                span: source.span,
                kind: ExpressionKind::String(vec![0; MAX_BYTES + 1]),
            }),
        });
        let quote = CodeBody::Expression(Box::new(initializer));
        let diagnostic = admit_compiler_quote(&quote, Span::new(4, 8)).unwrap_err();
        assert!(diagnostic.message.contains("payload byte"));
    }
    #[test]
    fn cumulative_quotation_and_named_procedure_metadata_are_bounded() {
        let body = CodeBody::Expression(Box::new(Expression {
            span: Span::new(0, 1),
            kind: ExpressionKind::String(vec![0; MAX_BYTES / 2 + 1]),
        }));
        let mut cumulative = CompilerQuoteBudget::default();
        cumulative.admit(&body, Span::new(0, 1)).unwrap();
        assert!(
            cumulative
                .admit(&body, Span::new(2, 3))
                .unwrap_err()
                .message
                .contains("payload byte")
        );
        let mut symbols = jai_source::Symbols::default();
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert(
            "code-procedure-budget.jai".into(),
            "produce :: () -> Code { return #code Answer :: 42; }".into(),
        );
        let file = syntax::parse_file(sources.get(id).unwrap(), &mut symbols).unwrap();
        let syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!("declaration")
        };
        let syntax::FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!("procedure")
        };
        admit_compiler_code_procedure(procedure, procedure.span).unwrap();
        let mut procedure = procedure.clone();
        let procedure_span = procedure.span;
        procedure.notes.push(syntax::NoteSyntax {
            name: symbols.intern("payload"),
            span: procedure_span,
            arguments: vec![syntax::NoteArgument {
                name: None,
                value: syntax::NoteValue::Expression(Expression {
                    span: procedure_span,
                    kind: ExpressionKind::String(vec![0; MAX_BYTES + 1]),
                }),
            }],
        });
        assert!(
            admit_compiler_code_procedure(&procedure, procedure.span)
                .unwrap_err()
                .message
                .contains("payload byte")
        );
    }

    #[test]
    fn deferred_context_source_value_is_charged_before_quote_retention() {
        let quote = CodeBody::Statement(Box::new(syntax::Statement {
            span: Span::new(0, 1),
            kind: syntax::StatementKind::PushContextDeferred {
                value: Some(Expression {
                    span: Span::new(0, 1),
                    kind: ExpressionKind::String(vec![0; MAX_BYTES + 1]),
                }),
            },
        }));
        let diagnostic = admit_compiler_quote(&quote, Span::new(4, 8)).unwrap_err();
        assert!(diagnostic.message.contains("payload byte"));
    }
}

#[cfg(test)]
mod baked_arguments_budget_tests {
    use super::*;

    #[test]
    fn baked_callee_and_bound_arguments_are_charged_before_quotation_retention() {
        let text = "PARTIAL::#bake_arguments worker(value=40);";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("baked-budget.jai".into(), text.into());
        let parsed = syntax::parse_file(
            sources.get(id).unwrap(),
            &mut jai_source::Symbols::default(),
        )
        .unwrap();
        let syntax::FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let syntax::FileDeclarationKind::Constant(constant) = &declaration.kind else {
            panic!()
        };
        for callee in [false, true] {
            let mut initializer = constant.initializer.clone();
            let syntax::ExpressionKind::BakeArguments(value) = &mut initializer.kind else {
                panic!()
            };
            let target = if callee {
                value.callee.as_mut()
            } else {
                &mut value.arguments[0].value
            };
            target.kind = syntax::ExpressionKind::String(vec![0; MAX_BYTES + 1]);
            let quote = syntax::CodeBody::Expression(Box::new(initializer));
            let error =
                admit_compiler_quote(&quote, jai_source::Span::new(0, text.len())).unwrap_err();
            assert!(error.message.contains("payload byte"));
        }
    }
}
