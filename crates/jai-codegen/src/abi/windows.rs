//! Microsoft x64 passes only power-of-two aggregates of at most eight bytes directly.
use super::*;
impl<'ctx> Classifier<'ctx, '_, '_> {
    pub(super) fn windows_x86_64_aggregate(
        &self,
        storage: BasicTypeEnum<'ctx>,
        layout: &Layout,
        result: bool,
    ) -> Result<Value<'ctx>, Error> {
        if layout.size <= 8 && layout.size.is_power_of_two() {
            let carrier = integer_width(
                self.context,
                u32::try_from(layout.size * 8).map_err(|_| Error::InvalidCarrier)?,
            )?;
            Ok(Value::Coerce {
                pieces: vec![Piece {
                    ty: carrier,
                    offset: 0,
                }],
                carrier: Some(carrier),
            })
        } else {
            Ok(Value::Indirect {
                storage,
                // Microsoft requires caller-created aggregate temporaries to
                // be at least 16-byte aligned; sret retains source alignment.
                alignment: if result {
                    layout.alignment
                } else {
                    layout.alignment.max(16)
                },
                by_value: false,
            })
        }
    }
}
