//! Prepare real record modifier source before any specialization is reserved.
use super::*;
use crate::modifiers::{ModifierPlan, ModifierSlot};
use crate::polymorphism::integration::{ModifierSource, build_modifier_source};
use jai_ir::ProcedureId;
use jai_types::ContextMode;

pub(super) struct RecordModifierSource<'a> {
    pub(super) record: &'a syntax::RecordDeclaration,
    pub(super) initial: &'a Substitution,
    pub(super) context: ContextMode,
    pub(super) checks: syntax::SafetyChecks,
}

pub(super) fn build(
    source: RecordModifierSource<'_>,
    id: ProcedureId,
    types: &mut TypeRegistry,
) -> Result<(crate::Signature, syntax::Procedure, ModifierPlan), Diagnostic> {
    let record = source.record;
    let modifier = record
        .modify
        .as_ref()
        .ok_or_else(|| Diagnostic::new(record.span, "record has no specialization modifier"))?;
    let mut parameters = Vec::with_capacity(record.parameters.len());
    let mut slots = Vec::with_capacity(record.parameters.len());
    for parameter in &record.parameters {
        let value = source
            .initial
            .constant(parameter.name)
            .cloned()
            .or_else(|| source.initial.ty(parameter.name).map(BakedValue::Type))
            .ok_or_else(|| {
                Diagnostic::new(
                    parameter.span,
                    "record modifier requires a bound formal argument",
                )
            })?;
        let slot = match value {
            BakedValue::Type(_) => ModifierSlot::Type {
                name: parameter.name,
            },
            BakedValue::Value(value) => ModifierSlot::Baked {
                name: parameter.name,
                ty: value.ty,
            },
            BakedValue::Float(value) => ModifierSlot::Baked {
                name: parameter.name,
                ty: types.float(value.ty()),
            },
            BakedValue::String(_) => ModifierSlot::Baked {
                name: parameter.name,
                ty: types.string(),
            },
            BakedValue::Code(_) => {
                return Err(Diagnostic::new(
                    parameter.span,
                    "record #modify cannot store compiler-only Code bindings",
                ));
            }
        };
        let binding = match &parameter.binding {
            syntax::RecordParameterBinding::Typed { ty, .. } => {
                syntax::ParameterBinding::RequiredType(ty.clone())
            }
            syntax::RecordParameterBinding::InferredDefault(expression) => {
                // The checked auxiliary Signature supplies the bound slot's type;
                // this original syntax is retained rather than reverse-printing a TypeId.
                syntax::ParameterBinding::DefaultedType {
                    ty: None,
                    expression: expression.clone(),
                }
            }
        };
        parameters.push(syntax::Parameter {
            name: parameter.name,
            binding,
            using: false,
            baking: syntax::ParameterBaking::None,
            variadic: false,
            evaluation: syntax::ParameterEvaluation::Evaluate,
            span: parameter.span,
        });
        slots.push(slot);
    }
    build_modifier_source(
        ModifierSource {
            name: record.name,
            parameters: &parameters,
            modifier,
            slots,
            context: source.context,
            checks: source.checks,
        },
        id,
        types,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_record_modifier_source_uses_bound_formal_order_and_reserved_identity() {
        let mut sources = jai_source::SourceMap::default();
        let source = sources.insert(
            "record-modifier-source.jai".into(),
            "Buffer::struct(N:int=3) #modify {if N<8 N=8;return true;} {values:[N]int;}".into(),
        );
        let mut symbols = jai_source::Symbols::default();
        let file = syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
        let syntax::FileItem::Declaration(declaration) = &file.items()[0] else {
            panic!("record source")
        };
        let syntax::FileDeclarationKind::Record(record) = &declaration.kind else {
            panic!("record source")
        };
        let mut types = TypeRegistry::new();
        let ty = types.scalar(jai_types::ScalarType::Int(jai_types::IntegerType::S64));
        let mut initial = Substitution::default();
        initial.bind_constant(
            record.parameters[0].name,
            BakedValue::Value(jai_ir::ConstantValue {
                ty,
                kind: jai_ir::ConstantKind::Int(jai_types::Integer::wrapping(
                    jai_types::IntegerType::S64,
                    3,
                )),
            }),
        );
        let id = ProcedureId::new(19);
        let (signature, procedure, plan) = build(
            RecordModifierSource {
                record,
                initial: &initial,
                context: ContextMode::None,
                checks: syntax::SafetyChecks::default(),
            },
            id,
            &mut types,
        )
        .unwrap();
        assert_eq!(signature.id, id);
        assert_eq!(signature.parameters.len(), 1);
        assert_eq!(signature.parameters[0].ty, ty);
        assert!(signature.parameters[0].default.is_none());
        assert_eq!(signature.results.len(), 3);
        assert!(
            matches!(plan.slots(),[ModifierSlot::Baked {name,ty:actual}] if *name==record.parameters[0].name && *actual==ty)
        );
        assert_eq!(procedure.span, record.modify.as_ref().unwrap().span);
        assert!(procedure.modify.is_none());
        assert_eq!(
            procedure.execution,
            jai_types::ProcedureExecution::CompileTimeOnly
        );
        assert!(matches!(
            record.modify.as_ref().unwrap().body.last().unwrap().kind,
            syntax::StatementKind::Return(Some(_))
        ));
        assert!(
            matches!(procedure.body.last().unwrap().kind,syntax::StatementKind::ReturnValues(ref values) if values.len()==3)
        );
    }
}
