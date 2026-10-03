use super::*;
impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn static_address(
        &mut self,
        data: &std::sync::Arc<StaticData>,
        address: &StaticAddress,
        depth: usize,
    ) -> Result<Pointer> {
        self.step(depth)?;
        // Checked IR already validated this immutable graph against the provider.
        // Its sole append authority preserves the previously published Arc prefix.
        let initialized = self
            .static_publications
            .get(&data.identity())
            .copied()
            .unwrap_or(0);
        let suffix = &data.objects()[initialized.min(data.objects().len())..];
        if !suffix.is_empty() {
            // Reserve the whole suffix first, so cycles and forward references work.
            for object in suffix {
                self.step(depth + 1)?;
                if let Some(identity) = object.runtime_type_identity()
                    && identity.policy() != self.memory.target().policy
                {
                    return Err(Error::InvalidIr(
                        "runtime Type descriptor uses a different target layout",
                    )
                    .into());
                }
                if self.static_objects.contains_key(&object.id()) {
                    return Err(Error::InvalidIr("static publication cache is inconsistent").into());
                }
                self.prepare_layout(object.ty())?;
                let pointer = self
                    .memory
                    .allocate(self.provider.types(), object.ty(), None)?;
                self.static_objects.insert(object.id(), pointer);
            }
            for object in suffix {
                if let Some(identity) = object.runtime_type_identity() {
                    self.step(depth + 1)?;
                    let address = object.descriptor_header().ok_or(Error::InvalidIr(
                        "runtime Type binding has no descriptor header",
                    ))?;
                    let pointer = self.static_projection(data, address, depth + 1)?;
                    self.memory
                        .register_runtime_type(self.provider.types(), &pointer, identity)?;
                }
            }
            for object in suffix {
                self.step(depth + 1)?;
                let pointer = self
                    .static_objects
                    .get(&object.id())
                    .cloned()
                    .ok_or(Error::InvalidIr("missing static allocation"))?;
                let value = self.static_value(data, object.value(), depth + 1)?;
                self.store_pointer(&pointer, value, depth + 1)?;
                self.memory.freeze(&pointer)?;
            }
            self.static_publications
                .insert(data.identity(), data.objects().len());
            self.static_data
                .insert(data.identity(), std::sync::Arc::clone(data));
        }
        self.static_projection(data, address, depth + 1)
    }
    fn static_projection(
        &mut self,
        data: &StaticData,
        address: &StaticAddress,
        depth: usize,
    ) -> Result<Pointer> {
        self.step(depth)?;
        data.object(address.object())
            .map_err(|error| Error::IrValidation(error.to_string()))?;
        let mut pointer = self
            .static_objects
            .get(&address.object())
            .cloned()
            .ok_or(Error::InvalidIr("missing static address root"))?;
        for projection in address.path() {
            self.step(depth + 1)?;
            pointer = match projection {
                StaticProjection::Field(field) => {
                    self.prepare_field_layouts(&pointer, field.index())?;
                    self.memory
                        .field(self.provider.types(), &pointer, field.index())?
                }
                StaticProjection::ByteView(view) => {
                    if address.path().len() != 1 || view.object() != address.object() {
                        return Err(
                            Error::InvalidIr("static byte view requires its own root").into()
                        );
                    }
                    self.prepare_layout(view.backing_type())?;
                    self.memory
                        .static_byte_view(self.provider.types(), &pointer, view)?
                }
                StaticProjection::Index(index) => {
                    self.prepare_pointer_layouts(&pointer, true)?;
                    self.memory.index(
                        self.provider.types(),
                        &pointer,
                        usize::try_from(*index).map_err(|_| Error::CheckedCast)?,
                    )?
                }
            };
        }
        Ok(pointer)
    }
    fn static_value(
        &mut self,
        data: &StaticData,
        value: &StaticValue,
        depth: usize,
    ) -> Result<Value> {
        self.step(depth)?;
        let value = match &value.kind {
            StaticValueKind::Constant(value) => self.constant_value(value, depth + 1)?,
            StaticValueKind::Record(fields) => {
                let mut values = Vec::with_capacity(fields.len().min(self.limits.value_cells));
                let mut cells = 1;
                if fields.len() > self.limits.value_cells {
                    return Err(Error::Limit(LimitKind::ValueCells).into());
                }
                for field in fields {
                    let value = self.static_value(data, field, depth + 1)?;
                    self.push_value(&mut values, &mut cells, value)?;
                }
                Value::Record {
                    ty: value.ty,
                    fields: values,
                }
            }
            StaticValueKind::Array(elements) => {
                if elements.len() > self.limits.value_cells {
                    return Err(Error::Limit(LimitKind::ValueCells).into());
                }
                let mut values = Vec::with_capacity(elements.len());
                let mut cells = 1;
                for element in elements {
                    let value = self.static_value(data, element, depth + 1)?;
                    self.push_value(&mut values, &mut cells, value)?;
                }
                Value::Array {
                    ty: value.ty,
                    elements: values,
                }
            }
            StaticValueKind::Address(address) => {
                let pointer = self.static_projection(data, address, depth + 1)?;
                if matches!(self.provider.types().kind(value.ty)?, TypeKind::Type) {
                    let value = Value::Type {
                        descriptor: Some(pointer),
                    };
                    self.runtime_type_identity(&value)?;
                    value
                } else {
                    Value::Pointer(pointer)
                }
            }
            StaticValueKind::Slice {
                data: address,
                count,
            } => {
                let TypeKind::Slice(element) = self.provider.types().kind(value.ty)? else {
                    return Err(Error::InvalidIr("static slice type mismatch").into());
                };
                let pointer = if let Some(address) = address {
                    self.static_projection(data, address, depth + 1)?
                } else {
                    Pointer::null(*element)
                };
                Value::Slice {
                    ty: value.ty,
                    pointer,
                    count: i64::try_from(*count).map_err(|_| Error::CheckedCast)?,
                }
            }
        };
        value.cells(self.limits.value_cells)?;
        Ok(value)
    }
}
