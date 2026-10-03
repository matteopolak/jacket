//! Constant declarations retain their original partial producer's source policy.
use super::*;
use crate::local_declarations::LocalDeclarationId;
use jai_ir::{ConstantKind, ConstantValue};
use jai_source::{DeclarationId, SourceSpan};
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ConstantOrigin {
    Graph(DeclarationId),
    Local(LocalDeclarationId),
}

#[derive(Clone)]
struct RetainedConstant {
    source: SourceSpan,
    value: ConstantValue,
    contract: Option<ValueContract>,
    revision: (jai_ir::ProcedureId, Option<usize>, Option<usize>),
}

#[derive(Default)]
pub(crate) struct BakedConstantContracts {
    declarations: HashMap<ConstantOrigin, RetainedConstant>,
    refreshing: HashSet<LocalDeclarationId>,
}

impl Resolver<'_> {
    pub(crate) fn baked_source_policy_revision(
        &self,
        owner: ProcedureId,
    ) -> (ProcedureId, Option<usize>, Option<usize>) {
        (
            owner,
            self.graph_scope
                .and_then(|scope| scope.baked_constant_contract_revision(owner)),
            self.meta.local_declarations.callback_body_revision(owner),
        )
    }

    pub(crate) fn baked_source_policy_failure(&self, owner: ProcedureId) -> Option<Diagnostic> {
        self.meta
            .local_declarations
            .callback_body_failure(owner)
            .cloned()
            .or_else(|| {
                self.graph_scope
                    .and_then(|scope| scope.baked_constant_contract_failure(owner))
            })
    }

    pub(crate) fn refresh_local_baked_constant(
        &mut self,
        id: LocalDeclarationId,
        source: &syntax::ConstantDeclaration,
    ) -> Result<(), Diagnostic> {
        let Some(previous) = self
            .meta
            .baked_constant_contracts
            .declarations
            .get(&ConstantOrigin::Local(id))
            .cloned()
        else {
            return Ok(());
        };
        if let Some(failure) = self.baked_source_policy_failure(previous.revision.0) {
            return Err(failure);
        }
        if previous.revision == self.baked_source_policy_revision(previous.revision.0) {
            return Ok(());
        }
        if !self.meta.baked_constant_contracts.refreshing.insert(id) {
            return Err(Diagnostic::at_source(
                previous.source,
                "cyclic partial constant policy refresh",
            ));
        }
        let path = syntax::NamePath {
            root: source.name,
            members: vec![],
        };
        let result = self.with_local_constant_source(
            &path,
            source.span,
            |definition, original_id, original| {
                if original_id != id {
                    return Err(Diagnostic::new(
                        original.span,
                        "partial constant policy refresh lost its original declaration",
                    ));
                }
                let checked = match &original.ty {
                    Some(annotation) => {
                        let expected = definition.lexical_annotation(annotation, original.span)?;
                        let expression =
                            definition.expr_expected(&original.initializer, expected)?;
                        definition.coerce_value(expression, expected, original.initializer.span)?
                    }
                    None => definition
                        .expr(&original.initializer)?
                        .value(original.initializer.span)?,
                };
                let value =
                    definition.literal_constant(checked.clone(), original.initializer.span)?;
                if value != previous.value {
                    return Err(Diagnostic::at_source(
                        previous.source,
                        "partial constant value changed during its original body recheck",
                    ));
                }
                definition.capture_baked_constant_declaration(original, &checked, &value)?;
                Ok(Some(()))
            },
        );
        self.meta.baked_constant_contracts.refreshing.remove(&id);
        result?.ok_or_else(|| {
            Diagnostic::at_source(
                previous.source,
                "partial constant policy refresh has no original source environment",
            )
        })
    }

    /// This extracts only the actual closed producer shape emitted for a partial
    /// callable. It neither evaluates effects nor discovers a callback policy.
    pub(crate) fn closed_baked_constant(
        &self,
        checked: &ValueExpr,
        span: Span,
    ) -> Result<Option<ConstantValue>, Diagnostic> {
        let ValueExpr::Bind {
            bindings,
            body,
            ty,
        } = checked
        else {
            return Ok(None);
        };
        let [
            (
                binding,
                ValueExpr::ProcedureValue {
                    procedure,
                    ty: producer_ty,
                },
            ),
        ] = bindings.as_slice()
        else {
            return Ok(None);
        };
        let ValueExpr::Bound {
            binding: selected,
            ty: body_ty,
        } = body.as_ref()
        else {
            return Ok(None);
        };
        let owner = self
            .expression_owner
            .map(|owner| self.compile_time.map_or(owner, |context| context.owner));
        if owner != Some(binding.procedure())
            || selected != binding
            || producer_ty != ty
            || body_ty != ty
        {
            return Err(Diagnostic::new(
                span,
                "closed partial constant differs from its actual producer binding",
            ));
        }
        let signature = self
            .contract_procedure_signature(*procedure)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "closed partial constant has no checked procedure signature",
                )
            })?;
        let contract = self
            .meta
            .callbacks
            .expression_producers
            .get(binding)
            .ok_or_else(|| {
                Diagnostic::new(
                    span,
                    "closed partial constant has no original checked producer",
                )
            })?;
        if signature.ty != *ty
            || !matches!(contract, Some(ValueContract {
            ty: captured_ty,
            kind: ContractKind::Callable { source: Some(captured), .. },
        }) if captured_ty == ty && captured == procedure)
        {
            return Err(Diagnostic::new(
                span,
                "closed partial constant lost its original producer proof",
            ));
        }
        Ok(Some(ConstantValue {
            ty: *ty,
            kind: ConstantKind::Procedure(*procedure),
        }))
    }

    pub(crate) fn bind_baked_source_constant(
        &mut self,
        source: &syntax::ConstantDeclaration,
    ) -> Result<Option<()>, Diagnostic> {
        let callable_cast = match &source.initializer.kind {
            syntax::ExpressionKind::TypeCast {
                ty, ..
            } => {
                let ty = self.lexical_annotation(ty, source.initializer.span)?;
                matches!(self.types.kind(ty), Ok(jai_types::TypeKind::Procedure(_)))
            }
            _ => false,
        };
        if !callable_cast
            && !matches!(
                source.initializer.kind,
                syntax::ExpressionKind::BakeArguments(_)
            )
        {
            return Ok(None);
        }
        let expression = match &source.ty {
            Some(annotation) => {
                let expected = self.lexical_annotation(annotation, source.span)?;
                let checked = self.expr_expected(&source.initializer, expected)?;
                self.coerce_value(checked, expected, source.initializer.span)?
            }
            None => self
                .expr(&source.initializer)?
                .value(source.initializer.span)?,
        };
        let constant = self.literal_constant(expression.clone(), source.initializer.span)?;
        self.capture_baked_constant_declaration(source, &expression, &constant)?;
        let id = self.meta.intern_constant(constant);
        self.bind_name(source.name, Binding::TypedConstant(id))?;
        Ok(Some(()))
    }

    pub(crate) fn capture_baked_constant_declaration(
        &mut self,
        source: &syntax::ConstantDeclaration,
        producer: &ValueExpr,
        value: &ConstantValue,
    ) -> Result<(), Diagnostic> {
        if value.ty != producer.type_id(self.types) {
            return Err(Diagnostic::new(
                source.span,
                "partial constant differs from its checked producer type",
            ));
        }
        let contract = match &source.ty {
            Some(annotation) => {
                self.annotation_value_contract(value.ty, annotation, source.span)?
            }
            None => {
                self.callback_expression_contract(&source.initializer, producer, source.span)?
            }
        };
        if contract.is_none() && !self.callback_contract_type(value.ty, 0) {
            return Ok(());
        }
        let local = self.local_baked_constant_origin(source);
        let origin = match local {
            Some(id) => {
                self.remember_local_constant_source(source.name);
                ConstantOrigin::Local(id)
            }
            None => {
                let scope = self.graph_scope.ok_or_else(|| {
                    Diagnostic::new(
                        source.span,
                        "partial constant has no original defining scope",
                    )
                })?;
                let id = scope.baked_constant_declaration(
                    &syntax::NamePath {
                        root: source.name,
                        members: vec![],
                    },
                    source.span,
                )?;
                ConstantOrigin::Graph(id)
            }
        };
        let location = self.ast_source_location(source.initializer.span)?;
        let owner = self
            .expression_owner
            .map(|owner| self.compile_time.map_or(owner, |context| context.owner))
            .ok_or_else(|| {
                Diagnostic::at_source(
                    location,
                    "partial constant has no actual source policy owner",
                )
            })?;
        let revision = self.baked_source_policy_revision(owner);
        if let Some(previous) = self
            .meta
            .baked_constant_contracts
            .declarations
            .get_mut(&origin)
        {
            if previous.value != *value
                || previous.source != location
                || previous.revision.0 != owner
            {
                return Err(Diagnostic::at_source(
                    location,
                    "partial constant changed within its original declaration environment",
                ));
            }
            let same = match (&previous.contract, &contract) {
                (Some(previous), Some(current)) => previous.same_binding_contract(current),
                (None, None) => true,
                _ => false,
            };
            if !same {
                // Refresh this declaration's exact newly checked policy. Equal
                // constants from other declarations do not participate.
                previous.contract = contract;
            }
            previous.revision = revision;
            return Ok(());
        }
        self.meta.baked_constant_contracts.declarations.insert(
            origin,
            RetainedConstant {
                source: location,
                value: value.clone(),
                contract,
                revision,
            },
        );
        Ok(())
    }

    pub(crate) fn checked_baked_constant_binding_contract(
        &mut self,
        path: &syntax::NamePath,
        span: Span,
    ) -> Result<Option<Option<ValueContract>>, Diagnostic> {
        let local = self.with_local_constant_source(path, span, |definition, id, original| {
            definition.refresh_local_baked_constant(id, original)?;
            Ok(definition
                .meta
                .baked_constant_contracts
                .declarations
                .get(&ConstantOrigin::Local(id))
                .cloned())
        })?;
        let retained = match local {
            Some(retained) => Some(retained),
            None => {
                let Some(scope) = self.graph_scope else {
                    return Ok(None);
                };
                let id = match self.lexical_graph_binding_ready(path, span)? {
                    Some(jai_modules::Binding::Declaration(id)) => Some(id),
                    Some(_) => None,
                    None if self.local_name_present(path.root) => None,
                    None => scope.baked_constant_declaration(path, span).ok(),
                };
                id.and_then(|id| {
                    self.meta
                        .baked_constant_contracts
                        .declarations
                        .get(&ConstantOrigin::Graph(id))
                        .cloned()
                })
            }
        };
        let Some(retained) = retained else {
            return Ok(None);
        };
        let binding = self.lookup_path(path, span)?;
        let actual = match binding {
            Binding::TypedConstant(id) => self.meta.constant(id).cloned(),
            Binding::Procedure {
                procedure,
                ty,
            } => Some(ConstantValue {
                ty,
                kind: ConstantKind::Procedure(procedure),
            }),
            _ => None,
        };
        if actual.as_ref() != Some(&retained.value) {
            return Err(Diagnostic::at_source(
                retained.source,
                "partial constant policy has another checked declaration value",
            ));
        }
        Ok(Some(retained.contract))
    }
}
