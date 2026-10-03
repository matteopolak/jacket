//! AAPCS64 argument alignment excludes a record's own minimum-alignment attribute.
use super::*;

impl Classifier<'_, '_, '_> {
    pub(super) fn unadjusted_aggregate_alignment(&mut self, mut ty: TypeId) -> Result<u32, Error> {
        while let TypeKind::Distinct(id) = self.types.kind(ty)? {
            ty = self.types.distinct(*id)?.representation;
        }
        if !matches!(self.types.kind(ty)?, TypeKind::Record(_) | TypeKind::Any(_)) {
            return Ok(self.layout(ty)?.alignment);
        }
        let record = self.types.record_storage_definition(ty)?;
        let mut alignment = 1;
        for (index, &field) in record.fields.iter().enumerate() {
            let natural = self.layout(field)?.alignment;
            let selected = record
                .layout
                .field_alignments
                .get(index)
                .copied()
                .flatten()
                .unwrap_or(if record.layout.packed {
                    1
                } else {
                    natural
                });
            alignment = alignment.max(selected);
        }
        Ok(alignment)
    }

    pub(super) fn stack_alignment(
        &mut self,
        ty: TypeId,
        value: &Value<'_>,
    ) -> Result<Option<u32>, Error> {
        if !self.platform.is_aapcs64() {
            return Ok(None);
        }
        match value {
            Value::Coerce {
                carrier: Some(BasicTypeEnum::ArrayType(array)),
                ..
            } if array.get_element_type().is_float_type() => {
                Ok(Some(self.unadjusted_aggregate_alignment(ty)?.clamp(8, 16)))
            }
            _ => Ok(None),
        }
    }
}
