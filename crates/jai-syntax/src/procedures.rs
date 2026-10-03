//! Procedure declarations and unresolved signature parsing.
use super::*;
use jai_types::{
    CallingConvention, ContextMode, DebugPolicy, ForeignReturnAbi, InlineHint, ProcedureExecution,
};

#[derive(Clone, Debug)]
pub struct Parameter {
    pub evaluation: ParameterEvaluation,
    pub name: Symbol,
    pub binding: ParameterBinding,
    pub using: bool,
    pub baking: ParameterBaking,
    pub variadic: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ParameterBinding {
    Required(ScalarType),
    Defaulted {
        ty: Option<ScalarType>,
        expression: Expression,
    },
    RequiredType(TypeSyntax),
    DefaultedType {
        ty: Option<TypeSyntax>,
        expression: Expression,
    },
}
#[derive(Clone, Debug)]
pub struct CallArgument {
    pub name: Option<Symbol>,
    pub value: Expression,
    pub spread: bool,
}
#[derive(Clone, Debug)]
pub struct Procedure {
    pub name: Symbol,
    pub operator: Option<OperatorDeclaration>,
    pub source: SourceProcedureSyntax,
}
impl std::ops::Deref for Procedure {
    type Target = SourceProcedureSyntax;
    fn deref(&self) -> &Self::Target {
        &self.source
    }
}
impl std::ops::DerefMut for Procedure {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.source
    }
}

#[derive(Clone, Debug)]
pub struct ModifyDirective {
    pub body: Vec<Statement>,
    pub span: Span,
}
pub(super) struct ProcedureModifiers {
    pub(super) deprecation: Option<Deprecation>,
    pub(super) convention: CallingConvention,
    pub(super) return_abi: ForeignReturnAbi,
    pub(super) context: ContextMode,
    pub(super) expands: bool,
    pub(super) checks: SafetyChecks,
    pub(super) symmetric: bool,
    pub(super) execution: ProcedureExecution,
    pub(super) debug: DebugPolicy,
}
impl Procedure {
    /// Only the legacy unnamed, non-defaulted scalar signature has this bridge.
    pub fn scalar_return_type(&self) -> Option<ReturnType> {
        match self.results.as_slice() {
            [] => Some(ReturnType::Void),
            [
                ProcedureResult {
                    name: None,
                    binding:
                        ResultBinding::Typed {
                            ty,
                            default: None,
                        },
                    usage: ResultUsage::Optional,
                    ..
                },
            ] => ty.as_scalar().map(ReturnType::Value),
            _ => None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct ProcedureResult {
    pub name: Option<Symbol>,
    pub binding: ResultBinding,
    pub usage: ResultUsage,
    pub span: Span,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResultUsage {
    #[default]
    Optional,
    Required,
}
#[derive(Clone, Debug)]
pub enum ResultBinding {
    Typed {
        ty: TypeSyntax,
        default: Option<Expression>,
    },
    InferredDefault(Expression),
}
#[derive(Clone, Debug)]
pub struct ReturnValue {
    pub name: Option<Symbol>,
    pub value: Expression,
}
#[derive(Clone, Debug)]
pub struct ProcedurePrototype {
    pub name: Symbol,
    pub header: CallableHeaderSyntax,
    pub binding: PrototypeBinding,
    pub span: Span,
}
impl std::ops::Deref for ProcedurePrototype {
    type Target = CallableHeaderSyntax;
    fn deref(&self) -> &Self::Target {
        &self.header
    }
}
impl std::ops::DerefMut for ProcedurePrototype {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.header
    }
}

#[derive(Clone, Debug)]
pub enum PrototypeBinding {
    EntryPoint,
    Intrinsic { tag: Option<String> },
    Foreign(ForeignProcedure),
    Compiler(CompilerProcedure),
}

#[derive(Clone, Debug)]
pub struct CompilerProcedure {
    pub tag: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ForeignProcedure {
    pub library: Option<NamePath>,
    pub symbol: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ProcedureTypeParameter {
    pub evaluation: ParameterEvaluation,
    pub usage: ResultUsage,
    pub variadic: bool,
    pub name: Option<Symbol>,
    pub ty: TypeSyntax,
    pub default: Option<Expression>,
    pub using: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct ProcedureTypeSyntax {
    pub parameters: Vec<ProcedureTypeParameter>,
    pub results: Vec<ProcedureTypeParameter>,
    pub convention: CallingConvention,
    pub return_abi: ForeignReturnAbi,
    pub context: ContextMode,
}

impl Parser<'_> {
    pub(super) fn starts_procedure(&self) -> bool {
        if self.token().kind == Kind::Keyword(Keyword::Operator) {
            return true;
        }
        if self.token().kind != Kind::Ident
            || self
                .tokens
                .get(self.at + 1)
                .is_none_or(|t| t.kind != Kind::Punctuation(Punct::Constant))
        {
            return false;
        }
        let mut header = self.at + 2;
        while self.tokens.get(header).is_some_and(|token| {
            matches!(
                token.kind,
                Kind::Keyword(Keyword::Inline | Keyword::NoInline)
            )
        }) {
            header += 1;
        }
        if self
            .tokens
            .get(header)
            .is_none_or(|token| token.kind != Kind::Punctuation(Punct::OpenParen))
        {
            return false;
        }
        let mut depth = 0;
        for (offset, token) in self.tokens[header..].iter().enumerate() {
            match token.kind {
                Kind::Punctuation(Punct::OpenParen) => depth += 1,
                Kind::Punctuation(Punct::CloseParen) => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(header + offset + 1).is_some_and(|next| {
                            matches!(
                                next.kind,
                                Kind::Punctuation(Punct::OpenBrace | Punct::Arrow)
                                    | Kind::Directive(
                                        Directive::NoContext
                                            | Directive::NoDebug
                                            | Directive::Deprecated
                                            | Directive::CompileTime
                                            | Directive::CCall
                                            | Directive::CppMethod
                                            | Directive::CppReturnTypeIsNonPod
                                            | Directive::Foreign
                                            | Directive::Compiler
                                            | Directive::Expand
                                            | Directive::Intrinsic
                                            | Directive::EntryPoint
                                            | Directive::Modify
                                            | Directive::NoArrayBoundsCheck
                                            | Directive::NoArithmeticOverflowCheck
                                    )
                            )
                        });
                    }
                }
                Kind::Eof => return false,
                _ => {}
            }
        }
        false
    }
    pub(super) fn procedure(&mut self) -> Result<Procedure, Diagnostic> {
        match self.procedure_declaration()? {
            FileDeclarationKind::Procedure(procedure) => Ok(procedure),
            FileDeclarationKind::OperatorAlias(alias) => Err(Diagnostic::new(
                alias.span,
                "operator aliases require a file source namespace",
            )),
            _ => Err(self.error("procedure prototypes require module resolution")),
        }
    }
    pub(super) fn procedure_declaration(&mut self) -> Result<FileDeclarationKind, Diagnostic> {
        let span = self.token().span;
        let operator_prefix = self.operator_prefix()?;
        let name = match operator_prefix {
            Some((_, label)) => label,
            None => self.name()?,
        };
        self.need(Punct::Constant)?;
        if let Some((kind, name)) = operator_prefix
            && self.token().kind == Kind::Ident
        {
            return self
                .operator_alias(kind, name, span.start)
                .map(FileDeclarationKind::OperatorAlias);
        }
        let (header, symmetric, modifier_start) =
            self.source_procedure_header(operator_prefix.is_some())?;
        let SourceProcedureHeader {
            callable:
                CallableHeaderSyntax {
                    mut deprecation,
                    notes: _,
                    parameters,
                    results,
                    mut convention,
                    return_abi,
                    mut context,
                },
            checks,
            inline_hint,
            execution,
            debug,
            compiler: _,
            expands,
            modify: _,
        } = header;
        if !debug.emits()
            && matches!(
                self.token().kind,
                Kind::Directive(Directive::Foreign | Directive::Intrinsic | Directive::EntryPoint)
            )
        {
            return Err(self.error("#no_debug requires a source procedure body"));
        }
        if execution == ProcedureExecution::CompileTimeOnly
            && matches!(
                self.token().kind,
                Kind::Directive(
                    Directive::Foreign
                        | Directive::Intrinsic
                        | Directive::EntryPoint
                        | Directive::Compiler
                )
            )
        {
            return Err(self.error("#compile_time requires a source procedure body"));
        }
        let operator = operator_prefix
            .map(|(kind, _)| self.finish_operator_declaration(kind, &parameters, symmetric, span))
            .transpose()?;
        if operator.is_some()
            && matches!(
                self.token().kind,
                Kind::Directive(
                    Directive::Foreign
                        | Directive::Intrinsic
                        | Directive::EntryPoint
                        | Directive::Compiler
                )
            )
        {
            return Err(self.error("operator declarations require a source body"));
        }
        let saw_no_context = self.tokens[modifier_start..self.at]
            .iter()
            .any(|token| token.kind == Kind::Directive(Directive::NoContext));
        if self.token().kind == Kind::Directive(Directive::EntryPoint) {
            self.at += 1;
            self.require_inline_body(inline_hint)?;
            if expands || checks.has_overrides() {
                return Err(self
                    .error("#entry_point aliases cannot have expansion or body check modifiers"));
            }
            self.deprecation_suffix(&mut deprecation)?;
            self.need(Punct::Semicolon)?;
            return Ok(FileDeclarationKind::ProcedurePrototype(
                ProcedurePrototype {
                    name,
                    header: CallableHeaderSyntax {
                        deprecation,
                        notes: self.procedure_notes()?,
                        parameters,
                        results,
                        convention,
                        return_abi,
                        context,
                    },
                    binding: PrototypeBinding::EntryPoint,
                    span,
                },
            ));
        }
        if self.token().kind == Kind::Directive(Directive::Foreign) {
            self.require_inline_body(inline_hint)?;
            if checks.has_overrides() {
                return Err(self.error("check directives require a source body"));
            }
            if expands {
                return Err(self.error("foreign declarations cannot be expanded"));
            }
            self.at += 1;
            let library = if self.token().kind == Kind::Ident {
                Some(self.name_path()?)
            } else {
                None
            };
            let symbol = if self.token().kind == Kind::String {
                Some(self.module_string()?)
            } else {
                None
            };
            let source_contract = convention == CallingConvention::Jai
                && return_abi == ForeignReturnAbi::Natural
                && library.is_none()
                && symbol.is_none()
                && crate::source_contracts::packed_defaults(&parameters);
            if !source_contract {
                if convention == CallingConvention::Jai {
                    convention = CallingConvention::C;
                }
                context = ContextMode::None;
                self.validate_c_variadic(&parameters)?;
            }
            self.deprecation_suffix(&mut deprecation)?;
            self.need(Punct::Semicolon)?;
            return Ok(FileDeclarationKind::ProcedurePrototype(
                ProcedurePrototype {
                    name,
                    header: CallableHeaderSyntax {
                        deprecation,
                        notes: self.procedure_notes()?,
                        parameters,
                        results,
                        convention,
                        return_abi,
                        context,
                    },
                    binding: PrototypeBinding::Foreign(ForeignProcedure {
                        library,
                        symbol,
                    }),
                    span,
                },
            ));
        }
        if self.token().kind == Kind::Directive(Directive::Intrinsic) {
            self.require_inline_body(inline_hint)?;
            if checks.has_overrides() {
                return Err(self.error("check directives require a source body"));
            }
            if !self.allow_qualified {
                return Err(self.error("intrinsic prototypes require intrinsic resolution"));
            }
            if expands {
                return Err(self.error("intrinsic prototypes cannot be expanded"));
            }
            self.at += 1;
            let tag = if self.token().kind == Kind::String {
                Some(self.module_string()?)
            } else {
                None
            };
            self.deprecation_suffix(&mut deprecation)?;
            if self.is(Punct::OpenBrace) {
                return Err(self.error("intrinsic prototypes cannot have a source body"));
            }
            self.need(Punct::Semicolon)?;
            return Ok(FileDeclarationKind::ProcedurePrototype(
                ProcedurePrototype {
                    name,
                    header: CallableHeaderSyntax {
                        deprecation,
                        notes: self.procedure_notes()?,
                        parameters,
                        results,
                        convention,
                        return_abi,
                        context: ContextMode::None,
                    },
                    binding: PrototypeBinding::Intrinsic {
                        tag,
                    },
                    span,
                },
            ));
        }
        let compiler = if self.token().kind == Kind::Directive(Directive::Compiler) {
            self.at += 1;
            let tag = if self.token().kind == Kind::String {
                Some(self.module_string()?)
            } else {
                None
            };
            self.deprecation_suffix(&mut deprecation)?;
            if self.token().kind == Kind::Directive(Directive::NoContext) {
                if saw_no_context {
                    return Err(self.error("duplicate #no_context"));
                }
                self.at += 1;
                context = ContextMode::None;
                if self.token().kind == Kind::Directive(Directive::NoContext) {
                    return Err(self.error("duplicate #no_context"));
                }
            }
            self.deprecation_suffix(&mut deprecation)?;
            let compiler = CompilerProcedure {
                tag,
            };
            if self.take(Punct::Semicolon) {
                if !debug.emits() {
                    return Err(self.error("#no_debug requires a source procedure body"));
                }
                self.require_inline_body(inline_hint)?;
                if checks.has_overrides() {
                    return Err(self.error("check directives require a source body"));
                }
                if expands {
                    return Err(self.error("compiler prototypes cannot be expanded"));
                }
                return Ok(FileDeclarationKind::ProcedurePrototype(
                    ProcedurePrototype {
                        name,
                        header: CallableHeaderSyntax {
                            deprecation,
                            notes: self.procedure_notes()?,
                            parameters,
                            results,
                            convention,
                            return_abi,
                            context,
                        },
                        binding: PrototypeBinding::Compiler(compiler),
                        span,
                    },
                ));
            }
            Some(compiler)
        } else {
            None
        };
        if convention == CallingConvention::C {
            self.validate_c_variadic(&parameters)?;
        }
        let modify = self.modify_directive()?;
        let body = self.block()?;
        self.take(Punct::Semicolon);
        Ok(FileDeclarationKind::Procedure(Procedure {
            name,
            operator,
            source: SourceProcedureSyntax {
                header: SourceProcedureHeader {
                    callable: CallableHeaderSyntax {
                        deprecation,
                        notes: self.procedure_notes()?,
                        parameters,
                        results,
                        convention,
                        return_abi,
                        context,
                    },
                    checks,
                    inline_hint,
                    execution,
                    debug,
                    compiler,
                    expands,
                    modify,
                },
                body,
                span,
            },
        }))
    }

    pub(super) fn procedure_inline_hint(&mut self) -> Result<InlineHint, Diagnostic> {
        let hint = match self.token().kind {
            Kind::Keyword(Keyword::Inline) => InlineHint::Always,
            Kind::Keyword(Keyword::NoInline) => InlineHint::Never,
            _ => return Ok(InlineHint::Automatic),
        };
        self.at += 1;
        if matches!(
            self.token().kind,
            Kind::Keyword(Keyword::Inline | Keyword::NoInline)
        ) {
            return Err(self.error("duplicate or conflicting procedure inlining modifiers"));
        }
        Ok(hint)
    }

    fn require_inline_body(&self, hint: InlineHint) -> Result<(), Diagnostic> {
        if hint != InlineHint::Automatic {
            return Err(self.error("procedure inlining modifiers require a source body"));
        }
        Ok(())
    }

    pub(super) fn modify_directive(&mut self) -> Result<Option<ModifyDirective>, Diagnostic> {
        if self.token().kind != Kind::Directive(Directive::Modify) {
            return Ok(None);
        }
        if !self.allow_qualified {
            return Err(self.error("#modify requires specialization resolution"));
        }
        let start = self.token().span.start;
        self.at += 1;
        let body = self.block()?;
        Ok(Some(ModifyDirective {
            body,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        }))
    }

    pub(super) fn validate_c_variadic(&self, parameters: &[Parameter]) -> Result<(), Diagnostic> {
        if parameters
            .iter()
            .position(|parameter| parameter.variadic)
            .is_some_and(|index| index + 1 != parameters.len())
        {
            return Err(self.error("C variadic parameter must be last"));
        }
        Ok(())
    }

    pub(super) fn signature_type(&mut self) -> Result<TypeSyntax, Diagnostic> {
        if self.allow_qualified {
            self.parameter_type_syntax()
        } else {
            Ok(TypeSyntax::Builtin(BuiltinType::Scalar(
                self.scalar_type()?,
            )))
        }
    }
    fn procedure_result(&mut self) -> Result<ProcedureResult, Diagnostic> {
        let start = self.token().span.start;
        let (name, binding) = if self.named_prefix(Punct::Infer) {
            let name = self.name()?;
            self.need(Punct::Infer)?;
            (
                Some(name),
                ResultBinding::InferredDefault(self.expression(0)?),
            )
        } else {
            let name = if self.named_prefix(Punct::Colon) {
                let name = self.name()?;
                self.need(Punct::Colon)?;
                Some(name)
            } else {
                None
            };
            let ty = self.signature_type()?;
            let default = if self.take(Punct::Assign) {
                Some(self.expression(0)?)
            } else {
                None
            };
            (
                name,
                ResultBinding::Typed {
                    ty,
                    default,
                },
            )
        };
        if !self.allow_qualified
            && (name.is_some()
                || matches!(
                    binding,
                    ResultBinding::Typed {
                        default: Some(_),
                        ..
                    } | ResultBinding::InferredDefault(_)
                ))
        {
            return Err(self.error("named and defaulted results require result binding resolution"));
        }
        let usage = if self.token().kind == Kind::Directive(Directive::Must) {
            if !self.allow_qualified {
                return Err(self.error("#must requires result obligation resolution"));
            }
            self.at += 1;
            if self.token().kind == Kind::Directive(Directive::Must) {
                return Err(self.error("duplicate #must result obligation"));
            }
            ResultUsage::Required
        } else {
            ResultUsage::Optional
        };
        Ok(ProcedureResult {
            name,
            binding,
            usage,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    pub(super) fn procedure_results(&mut self) -> Result<Vec<ProcedureResult>, Diagnostic> {
        if !self.take(Punct::Arrow) {
            return Ok(Vec::new());
        }
        let parenthesized = self.take(Punct::OpenParen);
        let mut results = Vec::new();
        if parenthesized && self.take(Punct::CloseParen) {
            return Ok(results);
        }
        loop {
            results.push(self.procedure_result()?);
            if parenthesized && self.take(Punct::CloseParen) {
                break;
            }
            if !self.take(Punct::Comma) {
                if parenthesized {
                    self.need(Punct::CloseParen)?;
                }
                break;
            }
            if !self.allow_qualified {
                return Err(self.error("multiple results require result binding resolution"));
            }
            if parenthesized && self.take(Punct::CloseParen) {
                break;
            }
        }
        Ok(results)
    }
    pub(super) fn procedure_modifiers(
        &mut self,
        operator: bool,
    ) -> Result<ProcedureModifiers, Diagnostic> {
        let mut deprecation = None;
        let mut convention = CallingConvention::Jai;
        let mut return_abi = ForeignReturnAbi::Natural;
        let mut context = ContextMode::Implicit;
        let mut saw_no_context = false;
        let mut expands = false;
        let mut checks = SafetyChecks::default();
        let mut symmetric = false;
        let mut execution = ProcedureExecution::RuntimeAndCompileTime;
        let mut debug = DebugPolicy::Emit;
        loop {
            if self.token().kind == Kind::Directive(Directive::Deprecated) {
                self.deprecation_suffix(&mut deprecation)?;
                continue;
            }
            if safety_checks::is_check_directive(self.token().kind) {
                checks = self.safety_checks(checks)?;
                continue;
            }
            match self.token().kind {
                Kind::Directive(Directive::NoDebug) => {
                    if !debug.emits() {
                        return Err(self.error("duplicate #no_debug procedure attribute"));
                    }
                    debug = DebugPolicy::Suppress;
                }
                Kind::Directive(Directive::CompileTime) => {
                    if execution == ProcedureExecution::CompileTimeOnly {
                        return Err(self.error("duplicate #compile_time procedure attribute"));
                    }
                    execution = ProcedureExecution::CompileTimeOnly;
                }
                Kind::Directive(Directive::Symmetric) => {
                    if !operator {
                        return Err(self.error("#symmetric requires an operator declaration"));
                    }
                    if symmetric {
                        return Err(self.error("duplicate #symmetric"));
                    }
                    symmetric = true;
                }
                Kind::Directive(Directive::Expand) => {
                    if expands {
                        return Err(self.error("duplicate #expand"));
                    }
                    expands = true;
                }
                Kind::Directive(Directive::NoContext) => {
                    if saw_no_context {
                        return Err(self.error("duplicate #no_context"));
                    }
                    saw_no_context = true;
                    context = ContextMode::None;
                }
                Kind::Directive(Directive::CCall) => {
                    if convention != CallingConvention::Jai {
                        return Err(self.error("duplicate #c_call"));
                    }
                    convention = CallingConvention::C;
                    context = ContextMode::None;
                }
                Kind::Directive(Directive::CppReturnTypeIsNonPod) => {
                    if return_abi != ForeignReturnAbi::Natural {
                        return Err(self.error("duplicate #cpp_return_type_is_non_pod"));
                    }
                    return_abi = ForeignReturnAbi::CppNonPod;
                }
                Kind::Directive(Directive::CppMethod) => {
                    if convention != CallingConvention::Jai {
                        return Err(self.error("conflicting procedure calling conventions"));
                    }
                    convention = CallingConvention::CppMethod;
                    context = ContextMode::None;
                }
                _ => break,
            }
            if !self.allow_qualified
                && !matches!(
                    self.token().kind,
                    Kind::Directive(Directive::NoContext | Directive::NoDebug)
                )
            {
                return Err(self.error("procedure modifiers require procedure type resolution"));
            }
            self.at += 1;
        }
        Ok(ProcedureModifiers {
            deprecation,
            convention,
            return_abi,
            context,
            expands,
            checks,
            symmetric,
            execution,
            debug,
        })
    }
    fn procedure_type_parameter(
        &mut self,
        result: bool,
    ) -> Result<ProcedureTypeParameter, Diagnostic> {
        let start = self.token().span.start;
        let evaluation = self.parameter_evaluation(result)?;
        let using = self.keyword(Keyword::Using);
        let name = if self.named_prefix(Punct::Colon) {
            let name = self.name()?;
            self.need(Punct::Colon)?;
            Some(name)
        } else {
            None
        };
        if using && name.is_none() {
            return Err(self.error("using procedure parameters require a name"));
        }
        let variadic = self.take(Punct::Range);
        let ty = self.parameter_type_syntax()?;
        let default = if !result && self.take(Punct::Assign) {
            if variadic {
                return Err(self.error("variadic callback parameters cannot have defaults"));
            }
            Some(self.expression(0)?)
        } else {
            None
        };
        let usage = if self.token().kind == Kind::Directive(Directive::Must) {
            if !result {
                return Err(self.error("#must applies only to procedure results"));
            }
            self.at += 1;
            if self.token().kind == Kind::Directive(Directive::Must) {
                return Err(self.error("duplicate #must"));
            }
            ResultUsage::Required
        } else {
            ResultUsage::Optional
        };
        Ok(ProcedureTypeParameter {
            evaluation,
            usage,
            variadic,
            name,
            ty,
            default,
            using,
            span: Span::new(start, self.tokens[self.at - 1].span.end),
        })
    }
    fn procedure_type_parameters(
        &mut self,
        result: bool,
    ) -> Result<Vec<ProcedureTypeParameter>, Diagnostic> {
        self.need(Punct::OpenParen)?;
        let mut parameters = Vec::new();
        if !self.take(Punct::CloseParen) {
            loop {
                parameters.push(self.procedure_type_parameter(result)?);
                if self.take(Punct::CloseParen) {
                    break;
                }
                self.need(Punct::Comma)?;
            }
        }
        Ok(parameters)
    }
    pub(super) fn procedure_type_syntax(
        &mut self,
        result_list: bool,
    ) -> Result<ProcedureTypeSyntax, Diagnostic> {
        let parameters = self.procedure_type_parameters(false)?;
        let mut results = Vec::new();
        if self.take(Punct::Arrow) {
            if self.is(Punct::OpenParen) {
                results = self.procedure_type_parameters(true)?;
            } else {
                loop {
                    results.push(self.procedure_type_parameter(true)?);
                    if !result_list || !self.take(Punct::Comma) {
                        break;
                    }
                }
            }
        }
        let ProcedureModifiers {
            deprecation,
            convention,
            return_abi,
            context,
            expands,
            checks,
            execution,
            debug,
            ..
        } = self.procedure_modifiers(false)?;
        if let Some(deprecation) = deprecation {
            return Err(Diagnostic::new(
                deprecation.span,
                "#deprecated annotates a declaration, not a procedure type",
            ));
        }
        if !debug.emits() {
            return Err(
                self.error("#no_debug is a source body policy, not a procedure type ABI modifier")
            );
        }
        if execution == ProcedureExecution::CompileTimeOnly {
            return Err(self.error(
                "#compile_time is a source body contract, not a procedure type ABI modifier",
            ));
        }
        if checks.has_overrides() {
            return Err(self.error("check directives require a source body"));
        }
        if expands {
            return Err(self.error("procedure types cannot be #expand declarations"));
        }
        if parameters
            .iter()
            .filter(|parameter| parameter.variadic)
            .count()
            > 1
        {
            return Err(self.error("a procedure type can have only one variadic parameter"));
        }
        if convention == CallingConvention::C
            && parameters
                .iter()
                .position(|parameter| parameter.variadic)
                .is_some_and(|index| index + 1 != parameters.len())
        {
            return Err(self.error("C variadic parameter must be last"));
        }
        if results.iter().any(|result| result.variadic) {
            return Err(self.error("procedure results cannot be variadic"));
        }
        Ok(ProcedureTypeSyntax {
            parameters,
            results,
            convention,
            return_abi,
            context,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlining_prefixes_preserve_policy_on_named_and_nested_source_definitions() {
        let (parsed, _) = file(
            "automatic::(){} forced::inline(x:int)->int{return x;} blocked::no_inline(){} main::(){ nested::inline(){} }",
        );
        assert_eq!(procedure(&parsed, 0).inline_hint, InlineHint::Automatic);
        assert_eq!(procedure(&parsed, 1).inline_hint, InlineHint::Always);
        assert_eq!(procedure(&parsed, 2).inline_hint, InlineHint::Never);
        let StatementKind::Procedure(nested) = &procedure(&parsed, 3).body[0].kind else {
            panic!("expected nested source procedure")
        };
        assert_eq!(nested.inline_hint, InlineHint::Always);
    }

    #[test]
    fn inlining_conflicts_and_bodyless_prototypes_are_diagnosed() {
        for (text, message) in [
            (
                "f::inline no_inline(){}",
                "duplicate or conflicting procedure inlining modifiers",
            ),
            (
                "f::inline inline(){}",
                "duplicate or conflicting procedure inlining modifiers",
            ),
            (
                "f::no_inline no_inline(){}",
                "duplicate or conflicting procedure inlining modifiers",
            ),
            (
                "f::inline() #foreign libc;",
                "procedure inlining modifiers require a source body",
            ),
            (
                "f::no_inline() #intrinsic;",
                "procedure inlining modifiers require a source body",
            ),
            (
                "f::inline() #compiler;",
                "procedure inlining modifiers require a source body",
            ),
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("bad-hint.jai".into(), text.into());
            assert_eq!(
                parse_file(sources.get(id).unwrap(), &mut Symbols::default())
                    .unwrap_err()
                    .message,
                message,
                "{text}"
            );
        }
    }

    #[test]
    fn intrinsic_prototypes_preserve_tags_and_cannot_be_source_definitions() {
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("intrinsics.jai".into(), "memcpy::(dest:*void,source:*void,count:int) #intrinsic; main::(){ llvm_trap::() #intrinsic \"llvm.debugtrap\"; }".into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::ProcedurePrototype(memcpy),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected prototype")
        };
        assert_eq!(memcpy.context, ContextMode::None);
        assert!(matches!(
            &memcpy.binding,
            PrototypeBinding::Intrinsic {
                tag: None
            }
        ));
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(main),
            ..
        }) = &file.items()[1]
        else {
            panic!("expected procedure")
        };
        assert!(
            matches!(&main.body[0].kind, StatementKind::ProcedurePrototype(ProcedurePrototype { binding: PrototypeBinding::Intrinsic { tag: Some(tag) }, .. }) if tag=="llvm.debugtrap")
        );
        for (text, message) in [
            (
                "f::() #intrinsic {}",
                "intrinsic prototypes cannot have a source body",
            ),
            (
                "f::() #expand #intrinsic;",
                "intrinsic prototypes cannot be expanded",
            ),
        ] {
            let id = sources.insert("bad-intrinsic.jai".into(), text.into());
            let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
            assert_eq!(error.message, message);
        }
    }

    #[test]
    fn modifier_body_is_separate_from_the_specialized_procedure_body() {
        let source = "select :: (value: $T) -> $R #modify { R = T; return true, \"accepted\"; } { return value; }";
        let mut sources = jai_source::SourceMap::default();
        let id = sources.insert("modify.jai".into(), source.into());
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::Procedure(procedure),
            ..
        }) = &file.items()[0]
        else {
            panic!("expected procedure")
        };
        let modifier = procedure.modify.as_ref().unwrap();
        assert_eq!(
            modifier.span.text(source),
            "#modify { R = T; return true, \"accepted\"; }"
        );
        assert!(matches!(
            modifier.body.as_slice(),
            [
                Statement {
                    kind: StatementKind::Assign(_, _),
                    ..
                },
                Statement {
                    kind: StatementKind::ReturnValues(_),
                    ..
                }
            ]
        ));
        assert!(matches!(
            procedure.body.as_slice(),
            [Statement {
                kind: StatementKind::Return(Some(_)),
                ..
            }]
        ));
        assert!(parse("main :: () #modify { return true; } {}").is_err());
    }
    use jai_source::SourceMap;

    fn file(source: &str) -> (ParsedFile, Symbols) {
        let mut sources = SourceMap::default();
        let id = sources.insert("procedures.jai".into(), source.into());
        let mut symbols = Symbols::default();
        (
            parse_file(sources.get(id).unwrap(), &mut symbols).unwrap(),
            symbols,
        )
    }
    fn procedure(file: &ParsedFile, index: usize) -> &Procedure {
        let FileItem::Declaration(declaration) = &file.items()[index] else {
            panic!()
        };
        let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
            panic!()
        };
        procedure
    }

    #[test]
    fn cpp_method_preserves_contextless_receiver_abi() {
        let (file, _) = file(
            "score :: (this:*void,delta:s32)->s32 #cpp_method {return delta;} Table :: struct { score:(this:*void,delta:s32)->s32 #cpp_method; }",
        );
        let callback = procedure(&file, 0);
        assert_eq!(callback.convention, CallingConvention::CppMethod);
        assert_eq!(callback.context, ContextMode::None);
        assert_eq!(callback.parameters.len(), 2);
    }

    #[test]
    fn result_obligations_preserve_each_required_result_and_intrinsic_boundary() {
        let (parsed, _) = file(
            "producer :: () -> int #must, bool { return 1, true; } memcmp :: (a: *void, b: *void, count: s64) -> s16 #must #intrinsic;",
        );
        let producer = procedure(&parsed, 0);
        assert_eq!(producer.results[0].usage, ResultUsage::Required);
        assert_eq!(producer.results[1].usage, ResultUsage::Optional);
        assert_eq!(producer.scalar_return_type(), None);
        let FileItem::Declaration(FileDeclaration {
            kind: FileDeclarationKind::ProcedurePrototype(prototype),
            ..
        }) = &parsed.items()[1]
        else {
            panic!("expected prototype")
        };
        assert_eq!(prototype.results[0].usage, ResultUsage::Required);
        assert!(matches!(
            prototype.binding,
            PrototypeBinding::Intrinsic {
                tag: None
            }
        ));
    }

    #[test]
    fn duplicate_result_obligations_are_located_and_legacy_cannot_erase_them() {
        let text = "test :: () -> int #must #must { return 1; }";
        let mut sources = SourceMap::default();
        let id = sources.insert("required.jai".into(), text.into());
        let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.message, "duplicate #must result obligation");
        assert_eq!(error.location.span.text(text), "#must");
        assert_eq!(
            parse("test :: () -> int #must { return 1; }")
                .unwrap_err()
                .message,
            "#must requires result obligation resolution"
        );
    }

    #[test]
    fn aggregate_signature_and_member_write_form_the_pipeline_fixture() {
        let (file, symbols) = file(
            "Point :: struct { x: int; } change :: (p: Point) -> Point { p.x += 2; return p; } main :: () -> int { p: Point = .{x=40}; return change(p).x; }",
        );
        let change = procedure(&file, 1);
        assert!(
            matches!(&change.parameters[0].binding, ParameterBinding::RequiredType(TypeSyntax::Named(path)) if symbols.name(path.root) == "Point")
        );
        assert!(
            matches!(&change.results[0].binding, ResultBinding::Typed {ty:TypeSyntax::Named(path), default:None} if symbols.name(path.root) == "Point")
        );
        assert_eq!(change.scalar_return_type(), None);
        assert!(
            matches!(&change.body[0].kind, StatementKind::UpdatePlace {target: PlaceSyntax {kind: PlaceKind::Qualified(path), ..},operation:BinaryOp::Add,..} if symbols.name(path.root) == "p" && symbols.name(path.members[0]) == "x")
        );
        assert_eq!(
            procedure(&file, 2).scalar_return_type(),
            Some(ReturnType::Value(ScalarType::Int(IntegerType::S64)))
        );
    }

    #[test]
    fn result_lists_defaults_and_named_returns_preserve_binding_order() {
        let (file, symbols) = file(
            "split :: (value: int) -> (first: int = 1, second := 2, flag: bool) { return flag=true, second=value, first=3; } main :: () { a, b, flag := split(40); a, b = 3, 4; a, b += split(0); }",
        );
        let split = procedure(&file, 0);
        assert_eq!(split.results.len(), 3);
        assert_eq!(symbols.name(split.results[0].name.unwrap()), "first");
        assert!(matches!(
            split.results[0].binding,
            ResultBinding::Typed {
                default: Some(_),
                ..
            }
        ));
        assert!(matches!(
            split.results[1].binding,
            ResultBinding::InferredDefault(_)
        ));
        assert_eq!(split.scalar_return_type(), None);
        let StatementKind::ReturnValues(values) = &split.body[0].kind else {
            panic!()
        };
        assert_eq!(
            values
                .iter()
                .map(|value| symbols.name(value.name.unwrap()))
                .collect::<Vec<_>>(),
            ["flag", "second", "first"]
        );
        let main = procedure(&file, 1);
        assert!(
            matches!(&main.body[0].kind, StatementKind::DeclareResults {names,ty:None,values} if names.len()==3 && values.len()==1)
        );
        assert!(
            matches!(&main.body[1].kind, StatementKind::AssignResults {targets,values,operation:None} if targets.len()==2 && values.len()==2)
        );
        assert!(matches!(
            &main.body[2].kind,
            StatementKind::AssignResults {
                operation: Some(BinaryOp::Add),
                ..
            }
        ));
    }

    #[test]
    fn callback_result_comma_does_not_consume_following_outer_parameters() {
        let (file, symbols) = file(
            "apply :: (f: (x: int) -> int, value: int, count: int) -> int { return f(value); } pair :: () -> int, bool { return 42, true; }",
        );
        let apply = procedure(&file, 0);
        assert_eq!(apply.parameters.len(), 3);
        let ParameterBinding::RequiredType(TypeSyntax::Procedure(signature)) =
            &apply.parameters[0].binding
        else {
            panic!()
        };
        assert_eq!(signature.results.len(), 1);
        assert_eq!(symbols.name(apply.parameters[1].name), "value");
        assert_eq!(procedure(&file, 1).results.len(), 2);
        assert!(matches!(
            procedure(&file, 1).body[0].kind,
            StatementKind::ReturnValues(_)
        ));
    }

    #[test]
    fn generic_introductions_and_baked_parameters_are_tagged() {
        let (file, symbols) = file(
            "identity :: (value: $T, other: T, $Count: int) -> T { return value; } raw :: () #no_context { } array_shape :: (values: [$N] $Element) { }",
        );
        let identity = procedure(&file, 0);
        assert!(
            matches!(&identity.parameters[0].binding, ParameterBinding::RequiredType(TypeSyntax::Variable(name)) if symbols.name(*name)=="T")
        );
        assert!(
            matches!(&identity.parameters[1].binding, ParameterBinding::RequiredType(TypeSyntax::Named(path)) if symbols.name(path.root)=="T")
        );
        assert_eq!(identity.parameters[2].baking, ParameterBaking::Required);
        assert_eq!(symbols.name(identity.parameters[2].name), "Count");
        assert_eq!(procedure(&file, 1).context, ContextMode::None);
        assert!(
            matches!(&procedure(&file,2).parameters[0].binding,ParameterBinding::RequiredType(TypeSyntax::FixedArray {count,element}) if matches!(&count.kind,ExpressionKind::CompileVariable(name) if symbols.name(*name)=="N") && matches!(element.as_ref(),TypeSyntax::Variable(name) if symbols.name(*name)=="Element"))
        );
    }

    #[test]
    fn compiler_prototype_context_modifiers_work_in_both_source_orders() {
        let (parsed, _) = file(
            "compile_time_debug_break :: () #compiler #no_context; before :: () #no_context #compiler; tagged :: () #compiler \"debug_break\" #no_context;",
        );
        for item in parsed.items() {
            let FileItem::Declaration(FileDeclaration {
                kind: FileDeclarationKind::ProcedurePrototype(prototype),
                ..
            }) = item
            else {
                panic!("expected compiler prototype")
            };
            assert_eq!(prototype.context, ContextMode::None);
            assert!(matches!(prototype.binding, PrototypeBinding::Compiler(_)));
        }
        let text = "invalid :: () #no_context #compiler #no_context;";
        let mut sources = SourceMap::default();
        let id = sources.insert("duplicate-context.jai".into(), text.into());
        let error = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap_err();
        assert_eq!(error.message, "duplicate #no_context");
        assert_eq!(error.location.span.text(text), "#no_context");
    }

    #[test]
    fn foreign_and_compiler_prototypes_are_bodyless_declarations() {
        let (parsed, symbols) = file(
            "c_free :: (memory: *void) #foreign libc \"free\"; run_command :: (args: ..string) -> int #compiler \"run_command\";",
        );
        let FileItem::Declaration(declaration) = &parsed.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.kind else {
            panic!()
        };
        let PrototypeBinding::Foreign(binding) = &prototype.binding else {
            panic!()
        };
        assert_eq!(symbols.name(binding.library.as_ref().unwrap().root), "libc");
        assert_eq!(binding.symbol.as_deref(), Some("free"));
        assert_eq!(prototype.convention, CallingConvention::C);
        assert_eq!(prototype.context, ContextMode::None);
        let FileItem::Declaration(declaration) = &parsed.items()[1] else {
            panic!()
        };
        let FileDeclarationKind::ProcedurePrototype(prototype) = &declaration.kind else {
            panic!()
        };
        assert!(prototype.parameters[0].variadic);
        assert!(
            matches!(&prototype.binding, PrototypeBinding::Compiler(CompilerProcedure {tag:Some(tag)}) if tag=="run_command")
        );
        assert_eq!(prototype.context, ContextMode::Implicit);
    }

    #[test]
    fn compiler_annotation_preserves_its_runtime_fallback_body() {
        let (parsed, _) = file("workspace :: () -> int #compiler \"workspace\" { return 0; }");
        let value = procedure(&parsed, 0);
        assert_eq!(
            value.compiler.as_ref().unwrap().tag.as_deref(),
            Some("workspace")
        );
        assert!(matches!(value.body[0].kind, StatementKind::Return(_)));
    }

    #[test]
    fn expanded_procedures_preserve_the_modifier_without_becoming_prototypes() {
        let (parsed, _) = file("macro :: (value:int)->int #expand #no_context {return value;}");
        assert!(procedure(&parsed, 0).expands);
        assert_eq!(procedure(&parsed, 0).context, ContextMode::None);
    }

    #[test]
    fn typed_variadic_and_c_callbacks_preserve_abi() {
        let (parsed, _) = file(
            "panic :: (values: ..int, status: int = 1) {} callback :: (x:int) -> int #c_call { return x; } Handler :: #type (values: ..int) -> int #c_call;",
        );
        assert!(procedure(&parsed, 0).parameters[0].variadic);
        assert_eq!(procedure(&parsed, 1).context, ContextMode::None);
        let FileItem::Declaration(declaration) = &parsed.items()[2] else {
            panic!()
        };
        let FileDeclarationKind::TypeAlias(alias) = &declaration.kind else {
            panic!()
        };
        let TypeSyntax::Procedure(signature) = &alias.ty else {
            panic!()
        };
        assert!(signature.parameters[0].variadic);
        assert_eq!(signature.context, ContextMode::None);
    }

    #[test]
    fn callback_result_usage_is_preserved_separately_from_parameter_annotations() {
        let mut sources = SourceMap::default();
        let id = sources.insert(
            "callback.jai".into(),
            "Callback::#type(value:int)->(int #must, int);".into(),
        );
        let file = parse_file(sources.get(id).unwrap(), &mut Symbols::default()).unwrap();
        let FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!()
        };
        let FileDeclarationKind::TypeAlias(alias) = &declaration.kind else {
            panic!()
        };
        let TypeSyntax::Procedure(signature) = &alias.ty else {
            panic!()
        };
        assert_eq!(signature.parameters[0].usage, ResultUsage::Optional);
        assert_eq!(signature.results[0].usage, ResultUsage::Required);
        assert_eq!(signature.results[1].usage, ResultUsage::Optional);
        for source in [
            "Callback::#type(int #must)->int;",
            "Callback::#type()->int #must #must;",
        ] {
            let id = sources.insert("bad-callback.jai".into(), source.into());
            assert!(parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err());
        }
    }

    #[test]
    fn malformed_results_and_baked_names_are_rejected() {
        for source in [
            "f :: () -> (x:) {}",
            "f :: () -> (x:=) {}",
            "f :: () -> int, {}",
            "f :: ($:int) {}",
            "f :: () #no_context #no_context {}",
            "f :: () { return x=; }",
            "f :: (args: ..int=1) {}",
            "f :: (a: ..int, b: ..int) {}",
            "f :: () #foreign libc {}",
            "f :: (args: ..int, x:int) #c_call {}",
        ] {
            let mut sources = SourceMap::default();
            let id = sources.insert("bad.jai".into(), source.into());
            assert!(
                parse_file(sources.get(id).unwrap(), &mut Symbols::default()).is_err(),
                "{source}"
            );
        }
        assert!(parse("f :: () -> int, bool { return 1, true; }").is_err());
        assert!(parse("f :: (x: Point) -> Point {return x;}").is_err());
    }
}
