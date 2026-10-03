//! Admit owned host strings as a typed virtual slice, without native addresses.
use super::*;
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub fn allocate_string_slice(
        &mut self,
        ty: TypeId,
        arguments: &[String],
    ) -> std::result::Result<Value, Error> {
        if self.continuation.is_some() {
            return Err(Error::InvalidIr(
                "host arguments cannot change during an active continuation",
            ));
        }
        let action = (|| -> Result<Value> {
            let TypeKind::Slice(element) = self.provider.types().kind(ty)? else {
                return Err(Error::InvalidIr("host string arguments require []string").into());
            };
            let element = *element;
            if !matches!(self.provider.types().kind(element)?, TypeKind::String) {
                return Err(Error::InvalidIr("host string arguments require []string").into());
            }
            let cells = arguments
                .iter()
                .try_fold(arguments.len(), |cells, argument| {
                    cells
                        .checked_add(argument.len())
                        .filter(|cells| *cells <= self.limits.value_cells)
                        .ok_or(Error::Limit(LimitKind::ValueCells))
                })?;
            self.charge_work(cells)?;
            if arguments.is_empty() {
                return Ok(Value::Slice {
                    ty,
                    pointer: Pointer::null(element),
                    count: 0,
                });
            }
            self.prepare_layout(element)?;
            let layout = self
                .memory
                .prepared_layout(self.provider.types(), element)?;
            let bytes = self.sequence_pack_bytes(arguments.len(), layout.size)?;
            self.charge_sequence_allocation(arguments.len(), bytes, layout.alignment)?;
            self.charge_sequence_work(bytes)?;
            let mut snapshots = Vec::new();
            let mut metadata = 0usize;
            for argument in arguments {
                let value =
                    self.normalize_storage_value(Value::String(argument.as_bytes().to_vec()), 0)?;
                let snapshot = crate::ByteImage::encode(
                    self.provider.types(),
                    self.memory.target(),
                    element,
                    &value,
                    self.limits.value_cells,
                )?;
                let snapshot_cells = snapshot.metadata_cells();
                metadata = metadata
                    .checked_add(snapshot_cells)
                    .filter(|cells| *cells <= self.limits.value_cells)
                    .ok_or(Error::Limit(LimitKind::ValueCells))?;
                self.charge_work(snapshot_cells)?;
                snapshots.push(snapshot);
            }
            self.charge_work(metadata)?;
            let image = crate::ByteImage::concatenate(
                &snapshots,
                self.memory.target(),
                self.limits.value_cells,
            )?;
            self.charge_work(self.memory.retokenize_work_cost())?;
            let root = self.memory.allocate_sequence_buffer(
                self.provider.types(),
                element,
                arguments.len(),
            )?;
            self.memory
                .install_sequence_buffer(self.provider.types(), &root, image)?;
            // The allocation itself is raw byte backing with legacy String type.
            // Give even element zero an explicit typed byte projection; a root
            // string load otherwise reads the entire backing as source text.
            let bytes = self.memory.cast_pointer(
                self.provider.types(),
                &root,
                self.provider
                    .types()
                    .scalar(jai_types::ScalarType::Int(jai_types::IntegerType::U8)),
                CastMode::Unchecked,
            )?;
            let pointer = self.memory.cast_pointer(
                self.provider.types(),
                &bytes,
                element,
                CastMode::Unchecked,
            )?;
            Ok(Value::Slice {
                ty,
                pointer,
                count: i64::try_from(arguments.len()).map_err(|_| Error::CheckedCast)?,
            })
        })();
        action.map_err(|halt| match halt {
            Halt::Failed(error) => error,
            Halt::Pending(Dependency::Type(ty)) => Error::Type(TypeError::Incomplete(ty)),
            Halt::Pending(_) => Error::InvalidIr("host arguments require ready types"),
        })
    }
}
