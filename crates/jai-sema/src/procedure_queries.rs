//! Source identity queries use checked definitions without running their operand.
use crate::{Binding, Diagnostic, Expr, Resolver, Span, syntax};
use jai_ir::{ConstantKind, ProcedureId, ValueExpr};
use jai_source::Symbol;
use jai_types::TypeId;

#[derive(Clone, Copy)]
struct ProcedureFacts {
    procedure: ProcedureId,
    ty: TypeId,
    name: Option<Symbol>,
}

enum ThisFacts {
    Record(TypeId),
    Procedure(ProcedureFacts),
}

impl Resolver<'_> {
    fn checked_procedure_facts(
        &self,
        procedure: ProcedureId,
        ty: Option<TypeId>,
        span: Span,
    ) -> Result<ProcedureFacts, Diagnostic> {
        let source = self
            .meta
            .local_declarations
            .procedure_source_identity(procedure)
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.procedure_source_identity(procedure))
            });
        let (actual, name) = if let Some(source) = source {
            (source.ty, source.name)
        } else if let Some((name, signature)) = self
            .signatures
            .iter()
            .find(|(_, signature)| signature.id == procedure)
        {
            (signature.ty, Some(*name))
        } else {
            return Err(Diagnostic::new(
                span,
                "procedure query requires retained checked definition facts",
            ));
        };
        if ty.is_some_and(|ty| ty != actual) {
            return Err(Diagnostic::new(
                span,
                "procedure value differs from its checked source signature",
            ));
        }
        self.types
            .procedure_definition(actual)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        Ok(ProcedureFacts {
            procedure,
            ty: actual,
            name,
        })
    }

    fn current_procedure_facts(&self, span: Span) -> Result<ProcedureFacts, Diagnostic> {
        if self.local_definition_queries_forbidden() {
            return Err(Diagnostic::new(
                span,
                "#this is not allowed in procedure headers or record parameter lists",
            ));
        }
        if self.expression_owner != Some(self.procedure) {
            return Err(Diagnostic::new(
                span,
                "procedure query requires an actual enclosing procedure definition",
            ));
        }
        if matches!(
            self.local_definition_owner(),
            Some(crate::local_declarations::LexicalScopeOwner::Record(_))
        ) {
            return Err(Diagnostic::new(
                span,
                "procedure query requires an enclosing procedure, but #this denotes a record",
            ));
        }
        self.checked_procedure_facts(self.procedure, None, span)
    }

    fn checked_this_facts(&self, span: Span) -> Result<ThisFacts, Diagnostic> {
        if self.local_definition_queries_forbidden() {
            return Err(Diagnostic::new(
                span,
                "#this is not allowed in procedure headers or record parameter lists",
            ));
        }
        if let Some(crate::local_declarations::LexicalScopeOwner::Record(ty)) =
            self.local_definition_owner()
        {
            self.types
                .kind(ty)
                .map_err(|error| Diagnostic::new(span, error.to_string()))?;
            return Ok(ThisFacts::Record(ty));
        }
        self.current_procedure_facts(span).map(ThisFacts::Procedure)
    }

    pub(crate) fn this_expression(&mut self, span: Span) -> Result<Expr, Diagnostic> {
        match self.checked_this_facts(span)? {
            ThisFacts::Record(ty) => Ok(Expr::Type(ty)),
            ThisFacts::Procedure(facts) => self.typed_value(
                ValueExpr::ProcedureValue {
                    procedure: facts.procedure,
                    ty: facts.ty,
                },
                facts.ty,
                span,
            ),
        }
    }

    pub(crate) fn describe_this(
        &self,
        span: Span,
    ) -> Result<crate::overloads::ArgumentInfo, Diagnostic> {
        use crate::polymorphism::BakedValue;
        if self.expression_owner.is_none() {
            if self.local_definition_queries_forbidden() {
                return Err(Diagnostic::new(
                    span,
                    "#this is not allowed in procedure headers or record parameter lists",
                ));
            }
            if let Some(preview) = self.meta.short_lambda_preview {
                self.types
                    .procedure_definition(preview.ty)
                    .map_err(|error| Diagnostic::new(span, error.to_string()))?;
                return Ok(crate::overloads::ArgumentInfo::typed(preview.ty));
            }
        }
        Ok(match self.checked_this_facts(span)? {
            ThisFacts::Record(ty) => crate::overloads::ArgumentInfo::constant(
                BakedValue::Type(ty),
                self.types.meta_type(),
            ),
            ThisFacts::Procedure(facts) => crate::overloads::ArgumentInfo::constant(
                BakedValue::Value(jai_ir::ConstantValue {
                    ty: facts.ty,
                    kind: ConstantKind::Procedure(facts.procedure),
                }),
                facts.ty,
            ),
        })
    }

    fn known_procedure_operand(
        &self,
        value: &syntax::Expression,
    ) -> Result<ProcedureFacts, Diagnostic> {
        let path = match &value.kind {
            syntax::ExpressionKind::This => return self.current_procedure_facts(value.span),
            syntax::ExpressionKind::Name(name) => syntax::NamePath {
                root: *name,
                members: Vec::new(),
            },
            syntax::ExpressionKind::QualifiedName(path) => path.clone(),
            _ => {
                return Err(Diagnostic::new(
                    value.span,
                    "#procedure_name requires a statically known procedure operand",
                ));
            }
        };
        match self.lookup_path(&path, value.span) {
            Ok(Binding::Procedure {
                procedure,
                ty,
            }) => self.checked_procedure_facts(procedure, Some(ty), value.span),
            Ok(Binding::TypedConstant(id)) => {
                let constant = self.meta.constant(id).ok_or_else(|| {
                    Diagnostic::new(
                        value.span,
                        "procedure constant belongs to another semantic context",
                    )
                })?;
                if let ConstantKind::Procedure(procedure) = &constant.kind {
                    self.checked_procedure_facts(*procedure, Some(constant.ty), value.span)
                } else {
                    Err(Diagnostic::new(
                        value.span,
                        "#procedure_name requires a statically known procedure operand",
                    ))
                }
            }
            Ok(Binding::Imported(binding)) => {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(
                        value.span,
                        "imported procedure query requires its defining graph",
                    )
                })?;
                let declarations = scope.imported_callable(binding, value.span)?;
                let [declaration] = declarations.as_slice() else {
                    return Err(Diagnostic::new(
                        value.span,
                        "#procedure_name requires one statically known procedure",
                    ));
                };
                let signature = scope.concrete_signature(*declaration).ok_or_else(|| {
                    Diagnostic::new(
                        value.span,
                        "#procedure_name requires a checked concrete procedure signature",
                    )
                })?;
                self.checked_procedure_facts(signature.id, Some(signature.ty), value.span)
            }
            Err(error) => {
                if self.local_name_present(path.root) {
                    return Err(error);
                }
                if let Some(scope) = self.graph_scope
                    && let Ok(signature) = scope.signature(&path, value.span)
                {
                    return self.checked_procedure_facts(
                        signature.id,
                        Some(signature.ty),
                        value.span,
                    );
                }
                Err(error)
            }
            Ok(_) => Err(Diagnostic::new(
                value.span,
                "#procedure_name requires a statically known procedure operand",
            )),
        }
    }

    pub(crate) fn checked_procedure_name(
        &self,
        value: Option<&syntax::Expression>,
        span: Span,
    ) -> Result<Box<[u8]>, Diagnostic> {
        let facts = match value {
            Some(value) => self.known_procedure_operand(value)?,
            None => self.current_procedure_facts(span)?,
        };
        let name = facts.name.ok_or_else(|| {
            Diagnostic::new(span, "anonymous procedure has no declared source name")
        })?;
        Ok(self.symbols.name(name).as_bytes().into())
    }

    pub(crate) fn procedure_name_expression(
        &mut self,
        value: Option<&syntax::Expression>,
        span: Span,
    ) -> Result<Expr, Diagnostic> {
        let name = self.checked_procedure_name(value, span)?;
        self.string_literal(&name, span)
    }
}
