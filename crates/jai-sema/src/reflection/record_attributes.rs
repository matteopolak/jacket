//! Apply real declaration attributes to the same nominal record policy.
use jai_types::{RecordReflectionFlag, RecordReflectionPolicy, TypeError, TypeId, TypeRegistry};

/// Source attributes are fixed facts of this checked record definition. They
/// change descriptor metadata, while its physical fields and layout stay intact.
pub(crate) fn apply_source_record_attributes(
    types: &mut TypeRegistry,
    record: TypeId,
    attributes: &[jai_syntax::RecordAttribute],
) -> Result<(), TypeError> {
    let policy =
        RecordReflectionPolicy::from_flags(attributes.iter().filter_map(
            |attribute| match attribute {
                jai_syntax::RecordAttribute::TypeInfoNone => Some(RecordReflectionFlag::NoTypeInfo),
                jai_syntax::RecordAttribute::Reflection(setting) => Some(setting.flag),
                _ => None,
            },
        ));
    types.add_record_reflection_flags(record, policy)?;
    Ok(())
}
