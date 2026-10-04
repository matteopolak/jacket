//! Source identity and binding for procedures whose result belongs to the compiler.
use super::*;
use jai_modules::FileInstanceId;
use jai_source::{DeclarationId, SourceSpan};
use jai_vm::{
    CompilerCodePlan, CompilerCodePlanBuilder, CompilerCodePlanLimits, CompilerControlId,
    CompilerReturnSiteId, CompilerRuntimeInput, CompilerRuntimeLeaf, CompilerSlotId,
};
use std::sync::Arc;
use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompilerCodeOccurrence(usize);

/// A compiler procedure is identified by actual source, never a native signature.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CompilerCodeSourceKey {
    Named {
        declaration: DeclarationId,
        file: FileInstanceId,
        location: SourceSpan,
    },
    Anonymous {
        file: FileInstanceId,
        location: SourceSpan,
        occurrence: CompilerCodeOccurrence,
    },
}
impl CompilerCodeSourceKey {
    pub(crate) fn file(self) -> FileInstanceId {
        match self {
            Self::Named {
                file, ..
            }
            | Self::Anonymous {
                file, ..
            } => file,
        }
    }
    pub(crate) fn location(self) -> SourceSpan {
        match self {
            Self::Named {
                location, ..
            }
            | Self::Anonymous {
                location, ..
            } => location,
        }
    }
}

#[derive(Clone)]
pub(crate) struct CompilerCodeTemplate {
    pub(crate) declaration: DeclarationId,
    pub(crate) file: FileInstanceId,
    pub(crate) location: SourceSpan,
    pub(crate) source: Arc<syntax::Procedure>,
}

impl CompilerCodeTemplate {
    pub(crate) fn key(&self) -> CompilerCodeSourceKey {
        CompilerCodeSourceKey::Named {
            declaration: self.declaration,
            file: self.file,
            location: self.location,
        }
    }
}

/// Retained anonymous occurrence allocation and named source templates are independent.
#[derive(Default)]
pub(crate) struct CompilerCodeRegistry {
    templates: HashMap<DeclarationId, CompilerCodeTemplate>,
    anonymous:
        HashMap<(FileInstanceId, jai_source::SourceId, usize, usize), CompilerCodeOccurrence>,
    source_retention: Rc<RefCell<crate::metaprogram::CompilerQuoteBudget>>,
}
impl CompilerCodeRegistry {
    pub(crate) fn source_retention(&self) -> Rc<RefCell<crate::metaprogram::CompilerQuoteBudget>> {
        self.source_retention.clone()
    }
    pub(crate) fn template(&self, id: DeclarationId) -> Option<&CompilerCodeTemplate> {
        self.templates.get(&id)
    }
    pub(crate) fn register(&mut self, template: CompilerCodeTemplate) -> Result<(), Diagnostic> {
        if let Some(previous) = self.templates.get(&template.declaration) {
            if previous.file != template.file || previous.location != template.location {
                return Err(Diagnostic::at_source(
                    template.location,
                    "compiler Code declaration identity changed defining source",
                ));
            }
            return Ok(());
        }
        self.templates.insert(template.declaration, template);
        Ok(())
    }
    pub(crate) fn anonymous_key(
        &mut self,
        file: FileInstanceId,
        location: SourceSpan,
    ) -> Result<CompilerCodeSourceKey, Diagnostic> {
        let identity = (
            file,
            location.source,
            location.span.start,
            location.span.end,
        );
        let next = self.anonymous.len();
        if next == usize::MAX {
            return Err(Diagnostic::at_source(
                location,
                "compiler Code occurrence identity exhausted",
            ));
        }
        let occurrence = *self
            .anonymous
            .entry(identity)
            .or_insert(CompilerCodeOccurrence(next));
        Ok(CompilerCodeSourceKey::Anonymous {
            file,
            location,
            occurrence,
        })
    }
}

// Reflection owns the compiler quote receipt. This source plan retains the exact
// source key and selected return templates beside VM-only control identities.
pub(crate) struct CompilerCodeSourcePlan {
    pub(crate) key: CompilerCodeSourceKey,
    pub(crate) plan: CompilerCodePlan,
    pub(crate) quotations: Vec<crate::metaprogram::CompilerQuoteTemplate>,
    pub(crate) lexical: crate::local_declarations::LocalScopes,
}
impl CompilerCodeSourcePlan {
    pub(crate) fn quotation(
        &self,
        site: CompilerReturnSiteId,
    ) -> Option<&crate::metaprogram::CompilerQuoteTemplate> {
        if !self.plan.owns_site(site) {
            return None;
        }
        self.quotations.get(site.index()).filter(|quote| {
            let source = quote.source();
            let defining = self.key.location();
            quote.site() == site
                && quote.plan() == self.plan.id()
                && source.file == self.key.file()
                && source.location.source == defining.source
                && source.location.span.start >= defining.span.start
                && source.location.span.end <= defining.span.end
        })
    }
}

pub(crate) struct CompilerCodeQuotation {
    pub(crate) site: CompilerReturnSiteId,
    pub(crate) body: syntax::CodeBody,
    pub(crate) source: crate::metaprogram::CompilerQuoteSource,
    pub(crate) frames: Vec<Vec<(Symbol, crate::metaprogram::CompilerQuoteBinding)>>,
}

struct Binder {
    builder: CompilerCodePlanBuilder,
    quotations: Vec<CompilerCodeQuotation>,
    scopes: Vec<HashMap<Symbol, (CompilerSlotId, TypeId)>>,
    nodes: usize,
    key: CompilerCodeSourceKey,
    quote_budget: Rc<RefCell<crate::metaprogram::CompilerQuoteBudget>>,
}

impl Resolver<'_> {
    pub(crate) fn bind_compiler_code(
        &mut self,
        key: CompilerCodeSourceKey,
        body: &[syntax::Statement],
        span: Span,
    ) -> Result<CompilerCodeSourcePlan, Diagnostic> {
        let location = key.location();
        if location.span.start > location.span.end
            || body.iter().any(|statement| {
                statement.span.start < location.span.start
                    || statement.span.end > location.span.end
                    || statement.span.start > statement.span.end
            })
        {
            return Err(Diagnostic::at_source(
                location,
                "compiler Code body is outside its genuine defining source range",
            ));
        }
        let mut binder = Binder {
            builder: CompilerCodePlanBuilder::new(CompilerCodePlanLimits::default())
                .map_err(|error| Diagnostic::new(span, error.to_string()))?,
            quotations: vec![],
            scopes: vec![],
            nodes: 0,
            key,
            quote_budget: self.meta.compiler_code.source_retention(),
        };
        let root = self.compiler_code_block(&mut binder, body, span, 0)?;
        let plan = binder
            .builder
            .finish(root)
            .map_err(|error| Diagnostic::new(span, error.to_string()))?;
        let quotations = binder
            .quotations
            .into_iter()
            .map(|quote| {
                crate::metaprogram::CompilerQuoteTemplate::checked_in_session(
                    &plan,
                    quote.site,
                    &quote.body,
                    quote.source,
                    quote.frames,
                    binder.quote_budget.clone(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.local_scopes
            .admit_compiler_clone(&mut binder.quote_budget.borrow_mut(), span)?;
        Ok(CompilerCodeSourcePlan {
            key,
            plan,
            quotations,
            lexical: self.local_scopes.clone(),
        })
    }
    fn compiler_code_error(&self, span: Span, error: jai_vm::Error) -> Diagnostic {
        Diagnostic::new(span, error.to_string())
    }
    fn compiler_code_block(
        &mut self,
        binder: &mut Binder,
        body: &[syntax::Statement],
        span: Span,
        depth: usize,
    ) -> Result<CompilerControlId, Diagnostic> {
        if depth >= 128 {
            return Err(Diagnostic::new(
                span,
                "compiler Code control nesting limit exceeded",
            ));
        }
        binder.scopes.push(HashMap::new());
        self.scopes.push(HashMap::new());
        let result = (|| {
            let mut children = vec![];
            for statement in body {
                binder.nodes = binder
                    .nodes
                    .checked_add(1)
                    .filter(|&n| n <= 4096)
                    .ok_or_else(|| {
                        Diagnostic::new(
                            statement.span,
                            "compiler Code source statement limit exceeded",
                        )
                    })?;
                children.push(self.compiler_code_statement(binder, statement, depth + 1)?);
            }
            let locals = binder
                .scopes
                .last()
                .unwrap()
                .values()
                .map(|&(slot, _)| slot)
                .collect();
            binder
                .builder
                .block_with_locals(children, locals)
                .map_err(|error| self.compiler_code_error(span, error))
        })();
        self.scopes.pop();
        binder.scopes.pop();
        result
    }
    fn compiler_code_leaf(
        &mut self,
        binder: &Binder,
        source: &syntax::Expression,
        expected: Option<TypeId>,
    ) -> Result<CompilerRuntimeLeaf, Diagnostic> {
        let (inputs, bindings) = self.compiler_code_inputs(binder, source)?;
        self.scopes.push(bindings);
        let expression = self.expr(source);
        self.scopes.pop();
        let expression = expression?;
        let leaf = match expression {
            Expr::Void(call) if expected.is_none() => CompilerRuntimeLeaf::call(call),
            expression => {
                let ty = match expected {
                    Some(ty) => ty,
                    None => self.expression_type(&expression, source.span)?,
                };
                if matches!(
                    self.types
                        .kind(ty)
                        .map_err(|error| Diagnostic::new(source.span, error.to_string()))?,
                    jai_types::TypeKind::Code
                ) {
                    return Err(Diagnostic::new(
                        source.span,
                        "compiler Code cannot enter a native effect expression",
                    ));
                }
                CompilerRuntimeLeaf::expression(self.coerce_value(expression, ty, source.span)?)
            }
        };
        Ok(leaf.with_inputs(inputs))
    }
    fn compiler_code_inputs(
        &mut self,
        binder: &Binder,
        source: &syntax::Expression,
    ) -> Result<(Vec<CompilerRuntimeInput>, HashMap<Symbol, Binding>), Diagnostic> {
        admit_native_source(source)?;
        // Compiler-local readers are installed as typed expression inputs by the
        // semantic Binding adapter; they never allocate native Places or locals.
        let mut effective = HashMap::new();
        for scope in &binder.scopes {
            for (&name, &(slot, ty)) in scope {
                effective.insert(name, (slot, ty));
            }
        }
        let mut effective: Vec<_> = effective.into_iter().collect();
        effective.sort_by_key(|(name, _)| self.symbols.name(*name));
        let mut inputs = vec![];
        let mut bindings = HashMap::new();
        for (name, (slot, ty)) in effective {
            let binding = self.allocate_expression_binding(source.span)?;
            bindings.insert(
                name,
                Binding::CompilerInput {
                    binding,
                    ty,
                },
            );
            inputs.push(CompilerRuntimeInput {
                binding,
                slot,
                ty,
            });
        }
        Ok((inputs, bindings))
    }
    fn compiler_code_effect(
        &mut self,
        binder: &Binder,
        source: &syntax::Expression,
    ) -> Result<CompilerRuntimeLeaf, Diagnostic> {
        let (inputs, bindings) = self.compiler_code_inputs(binder, source)?;
        self.scopes.push(bindings);
        let leaf = (|| {
            Ok(match self.discard_call_results(source)? {
                Some(
                    Statement::CallVoid(call)
                    | Statement::CallResults {
                        call, ..
                    },
                ) => CompilerRuntimeLeaf::call(call),
                Some(_) => {
                    return Err(Diagnostic::new(
                        source.span,
                        "compiler Code effect requires a checked direct native call or expression",
                    ));
                }
                None => {
                    let expression = self.expr(source)?;
                    let ty = self.expression_type(&expression, source.span)?;
                    CompilerRuntimeLeaf::expression(self.coerce_value(
                        expression,
                        ty,
                        source.span,
                    )?)
                }
            })
        })();
        self.scopes.pop();
        let leaf = leaf?;
        Ok(leaf.with_inputs(inputs))
    }
    fn compiler_code_statement(
        &mut self,
        binder: &mut Binder,
        statement: &syntax::Statement,
        depth: usize,
    ) -> Result<CompilerControlId, Diagnostic> {
        let span = statement.span;
        match &statement.kind {
            syntax::StatementKind::Expression(source) => {
                let leaf = self.compiler_code_effect(binder, source)?;
                binder
                    .builder
                    .evaluate(leaf)
                    .map_err(|error| self.compiler_code_error(span, error))
            }
            syntax::StatementKind::Assign(name, source) => {
                let (slot, ty) = binder
                    .scopes
                    .iter()
                    .rev()
                    .find_map(|scope| scope.get(name))
                    .copied()
                    .ok_or_else(|| {
                        Diagnostic::new(
                            span,
                            "compiler Code assignment requires its own declared local",
                        )
                    })?;
                let leaf = self.compiler_code_leaf(binder, source, Some(ty))?;
                binder
                    .builder
                    .assign(slot, leaf)
                    .map_err(|error| self.compiler_code_error(span, error))
            }
            syntax::StatementKind::Declare(source) => {
                if !source.attributes().is_empty() {
                    return Err(Diagnostic::new(
                        span,
                        "compiler Code locals do not support native storage attributes",
                    ));
                }
                let (name, initializer, explicit) = match source {
                    syntax::Declaration::GroupMember {
                        ..
                    } => {
                        return Err(Diagnostic::new(
                            span,
                            "file declaration group requires its source publication owner",
                        ));
                    }
                    syntax::Declaration::Inferred {
                        name,
                        initializer,
                        ..
                    } => (*name, Some(initializer), None),
                    syntax::Declaration::Explicit {
                        name,
                        initializer,
                        ty,
                        ..
                    } => (*name, initializer.as_ref(), Some(self.types.scalar(*ty))),
                    syntax::Declaration::UnresolvedExplicit {
                        name,
                        initializer,
                        ty,
                        ..
                    } => (
                        *name,
                        initializer.as_ref(),
                        Some(self.lexical_annotation(ty, span)?),
                    ),
                    syntax::Declaration::External {
                        ..
                    } => {
                        return Err(Diagnostic::new(
                            span,
                            "compiler Code cannot own external native storage",
                        ));
                    }
                };
                if self.symbols.name(name) == "_" {
                    return Err(Diagnostic::new(
                        span,
                        "compiler Code discard declarations require checked result destination binding",
                    ));
                }
                if binder.scopes.last().unwrap().contains_key(&name) {
                    return Err(Diagnostic::new(span, "duplicate compiler Code local"));
                }
                let initializer = initializer.ok_or_else(|| {
                    Diagnostic::new(span, "compiler Code local requires an initializer")
                })?;
                let leaf = self.compiler_code_leaf(binder, initializer, explicit)?;
                let ty = match leaf.kind() {
                    jai_vm::CompilerRuntimeLeafKind::Expression(value) => value.type_id(self.types),
                    _ => {
                        return Err(Diagnostic::new(
                            span,
                            "compiler Code local initializer requires one value",
                        ));
                    }
                };
                if self.compiler_slot_contains_callback(ty, span)? {
                    return Err(Diagnostic::new(
                        span,
                        "compiler Code callback-bearing local requires checked compiler-slot source contracts",
                    ));
                }
                let slot = binder
                    .builder
                    .slot(ty)
                    .map_err(|error| self.compiler_code_error(span, error))?;
                binder.scopes.last_mut().unwrap().insert(name, (slot, ty));
                binder
                    .builder
                    .assign(slot, leaf)
                    .map_err(|error| self.compiler_code_error(span, error))
            }
            syntax::StatementKind::If(condition, yes, no) => {
                let bool_ty = self.types.scalar(ScalarType::Bool);
                let condition = self.compiler_code_leaf(binder, condition, Some(bool_ty))?;
                let yes = self.compiler_code_block(binder, yes, span, depth)?;
                let no = self.compiler_code_block(binder, no, span, depth)?;
                binder
                    .builder
                    .branch_leaf(condition, yes, no)
                    .map_err(|error| self.compiler_code_error(span, error))
            }
            syntax::StatementKind::Block(body) => {
                self.compiler_code_block(binder, body, span, depth)
            }
            syntax::StatementKind::Return(Some(source)) => {
                if source.span.start < binder.key.location().span.start
                    || source.span.end > binder.key.location().span.end
                    || source.span.start > source.span.end
                {
                    return Err(Diagnostic::new(
                        source.span,
                        "compiler Code return quotation changed its defining source range",
                    ));
                }
                let syntax::ExpressionKind::Code(body) = &source.kind else {
                    return Err(Diagnostic::new(
                        source.span,
                        "compiler Code return requires a retained quotation in this implementation",
                    ));
                };
                binder.quote_budget.borrow_mut().admit(body, source.span)?;
                let mut locals = HashMap::new();
                for scope in &binder.scopes {
                    for (&name, &(slot, ty)) in scope {
                        locals.insert(name, (slot, ty));
                    }
                }
                let mut locals: Vec<_> = locals
                    .into_iter()
                    .map(|(name, (slot, ty))| (name, slot, ty))
                    .collect();
                locals.sort_by_key(|(name, _, _)| self.symbols.name(*name));
                let site = binder
                    .builder
                    .reserve_return_site_with_captures(
                        locals.iter().map(|&(_, slot, _)| slot).collect(),
                    )
                    .map_err(|error| self.compiler_code_error(span, error))?;
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(
                        source.span,
                        "compiler Code quotation requires its genuine defining graph",
                    )
                })?;
                let (file, _, scope_source) = scope.code_origin();
                let source_id = self.debug.source().unwrap_or(scope_source);
                if source_id != binder.key.location().source || file != binder.key.file() {
                    return Err(Diagnostic::new(
                        source.span,
                        "compiler Code quotation changed its retained source owner",
                    ));
                }
                let mut effective = self.local_scopes.insertion_capture_bindings(
                    &self.scopes,
                    scope,
                    self.symbols,
                    source.span,
                )?;
                let mut frame = vec![];
                for (name, slot, ty) in &locals {
                    effective.remove(name);
                    frame.push((
                        *name,
                        crate::metaprogram::CompilerQuoteBinding::Native {
                            slot: *slot,
                            ty: *ty,
                        },
                    ));
                }
                for (name, binding) in effective {
                    if matches!(binding, Binding::CompilerInput { .. }) {
                        return Err(Diagnostic::new(
                            source.span,
                            "compiler quotation contains an unowned compiler input",
                        ));
                    }
                    let binding = match binding {
                        Binding::Storage(storage) => {
                            crate::metaprogram::CompilerQuoteBinding::Lexical(storage)
                        }
                        binding => crate::metaprogram::CompilerQuoteBinding::Static(binding),
                    };
                    frame.push((name, binding));
                }
                frame.sort_by_key(|(name, _)| self.symbols.name(*name));
                binder.quote_budget.borrow_mut().retain_metadata(
                    frame
                        .len()
                        .saturating_mul(16)
                        .saturating_add(self.debug.caller_origins().len()),
                    0,
                    source.span,
                )?;
                binder.quotations.push(CompilerCodeQuotation {
                    site,
                    body: body.clone(),
                    source: crate::metaprogram::CompilerQuoteSource {
                        file,
                        source_file: self.capture_source_file(source_id, source.span)?,
                        location: SourceSpan {
                            source: source_id,
                            span: source.span,
                        },
                        checks: self.checks,
                        debug: self.debug.policy(),
                        origins: self.debug.caller_origins().to_vec(),
                    },
                    frames: vec![frame],
                });
                binder
                    .builder
                    .return_code(site)
                    .map_err(|error| self.compiler_code_error(span, error))
            }
            _ => Err(Diagnostic::new(
                span,
                "compiler Code statement requires supported compiler control or a checked native effect expression",
            )),
        }
    }
}

// Admission precedes resolver binding, so explicit nested runs cannot publish
// scalar effects before the outer compiler Code transaction exists.
fn admit_native_source(source: &syntax::Expression) -> Result<(), Diagnostic> {
    use syntax::ExpressionKind as E;
    let mut pending = vec![source];
    let mut nodes = 0usize;
    while let Some(source) = pending.pop() {
        nodes = nodes.checked_add(1).filter(|&n| n <= 4096).ok_or_else(|| {
            Diagnostic::new(
                source.span,
                "compiler Code native expression admission limit exceeded",
            )
        })?;
        match &source.kind {
            E::Integer(_)
            | E::Float(_)
            | E::String(_)
            | E::HereString(_)
            | E::Character(_)
            | E::Null
            | E::Bool(_)
            | E::Name(_)
            | E::QualifiedName(_)
            | E::CompileTimePredicate
            | E::Context
            | E::SourceLocation
            | E::SourceFile
            | E::SourceFilepath
            | E::SourceLine
            | E::InferredMember(_) => {}
            E::Call(_, arguments) | E::QualifiedCall(_, arguments) => {
                pending.extend(arguments.iter().map(|argument| &argument.value))
            }
            E::IndirectCall {
                callee,
                args,
            } => {
                pending.push(callee);
                pending.extend(args.iter().map(|argument| &argument.value));
            }
            E::ContextCall {
                callee,
                args,
                overrides,
            } => {
                pending.push(callee);
                pending.extend(args.iter().chain(overrides).map(|argument| &argument.value));
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
            | E::Member {
                base: call, ..
            } => pending.push(call),
            E::Index {
                base,
                index,
            }
            | E::Binary(_, base, index) => {
                pending.push(base);
                pending.push(index);
            }
            E::Conditional(condition) => {
                pending.push(&condition.condition);
                if let Some(value) = condition.explicit_then() {
                    pending.push(value);
                }
                if let Some(no) = &condition.else_value {
                    pending.push(no);
                }
            }
            E::CompileTime(_) => {
                return Err(Diagnostic::new(
                    source.span,
                    "nested #run cannot execute while binding a compiler Code plan",
                ));
            }
            _ => {
                return Err(Diagnostic::new(
                    source.span,
                    "compiler Code native leaf requires an admitted native expression",
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_leaf_admission_rejects_nested_run_at_its_actual_source_span() {
        let module = syntax::parse("main::(){effect(1+#run nested());}").unwrap();
        let syntax::StatementKind::Expression(source) = &module.procedures()[0].body[0].kind else {
            panic!()
        };
        let error = admit_native_source(source).unwrap_err();
        assert_eq!(
            error.span.text("main::(){effect(1+#run nested());}"),
            "#run nested()"
        );
        assert!(error.message.contains("binding a compiler Code plan"));
    }
    #[test]
    fn native_leaf_admission_keeps_typed_call_arguments_and_branch_arithmetic() {
        let module = syntax::parse("main::(){effect(1+2,true&&false);}").unwrap();
        let syntax::StatementKind::Expression(source) = &module.procedures()[0].body[0].kind else {
            panic!()
        };
        admit_native_source(source).unwrap();
    }
}
