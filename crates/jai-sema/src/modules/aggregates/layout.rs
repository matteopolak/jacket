//! Convert record layout syntax into checked target-independent constraints.
use super::super::*;
use jai_types::RecordLayout;

pub(super) fn record_layout(
    graph: &ModuleGraph,
    file: FileInstanceId,
    record: &syntax::RecordDeclaration,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic>,
) -> Result<RecordLayout, LocatedDiagnostic> {
    record_body_layout(
        graph,
        file,
        record.span,
        &record.attributes,
        &record.members,
        evaluate,
    )
}
pub(super) fn record_body_layout(
    graph: &ModuleGraph,
    file: FileInstanceId,
    _span: Span,
    attributes: &[syntax::RecordAttribute],
    members: &[syntax::RecordMember],
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic>,
) -> Result<RecordLayout, LocatedDiagnostic> {
    let mut layout = RecordLayout::default();
    for attribute in attributes {
        match attribute {
            syntax::RecordAttribute::Alignment(expression) => {
                layout.minimum_alignment = Some(alignment(graph, file, expression, evaluate)?);
            }
            syntax::RecordAttribute::NoPadding => layout.packed = true,
            syntax::RecordAttribute::TypeInfoNone | syntax::RecordAttribute::Reflection(_) => {}
        }
    }
    let mut fields = Vec::new();
    for field in members.iter().filter_map(|member| match member {
        syntax::RecordMember::Field(field) => {
            Some(crate::local_declarations::FieldSourceRef::Named(field))
        }
        syntax::RecordMember::AnonymousRecord(record) => Some(
            crate::local_declarations::FieldSourceRef::AnonymousRecord(record),
        ),
        _ => None,
    }) {
        let mut field_alignment = None;
        for attribute in field.attributes() {
            match attribute {
                syntax::FieldAttribute::Alignment(expression) => {
                    field_alignment = Some(alignment(graph, file, expression, evaluate)?);
                }
                syntax::FieldAttribute::Placement(_) => {}
            }
        }
        fields.push(field_alignment);
    }
    if fields.iter().any(Option::is_some) {
        layout.field_alignments = fields.into_boxed_slice();
    }
    Ok(layout)
}

fn alignment(
    graph: &ModuleGraph,
    file: FileInstanceId,
    expression: &syntax::Expression,
    evaluate: &mut impl FnMut(
        FileInstanceId,
        &syntax::Expression,
    ) -> Result<ConstantValue, LocatedDiagnostic>,
) -> Result<u32, LocatedDiagnostic> {
    let value = crate::storage_alignment::integer_alignment(evaluate(file, expression)?);
    value.ok_or_else(|| {
        located(
            graph,
            file,
            Diagnostic::new(
                expression.span,
                "alignment requires a nonzero power-of-two integer constant representable as u32",
            ),
        )
    })
}
