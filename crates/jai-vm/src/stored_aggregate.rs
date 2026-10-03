//! Sealed aggregate values retain storage that their active semantic fields omit.
use crate::{ByteImage, Error, LimitKind, Value};
use jai_types::{LayoutEngine, RecordKind, TypeId, TypeKind, TypeView};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct StoredAggregate {
    ty: TypeId,
    semantic: Option<Box<Value>>,
    image: Arc<ByteImage>,
}
impl StoredAggregate {
    pub(crate) fn new(
        ty: TypeId,
        semantic: Value,
        image: ByteImage,
        limit: usize,
    ) -> Result<Self, Error> {
        Self::with_semantic(ty, Some(semantic), image, limit)
    }
    pub(crate) fn opaque(
        types: &dyn TypeView,
        ty: TypeId,
        image: ByteImage,
        layout: &jai_types::Layout,
        limit: usize,
    ) -> Result<Self, Error> {
        if usize::try_from(layout.size).ok() != Some(image.len()) {
            return Err(Error::InvalidIr(
                "aggregate snapshot has the wrong storage extent",
            ));
        }
        let value = Self::with_semantic(ty, None, image, limit)?;
        value.validate_storage(types)?;
        Ok(value)
    }
    fn with_semantic(
        ty: TypeId,
        semantic: Option<Value>,
        image: ByteImage,
        limit: usize,
    ) -> Result<Self, Error> {
        let storage = image
            .len()
            .checked_add(image.metadata_cells())
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        semantic
            .as_ref()
            .map_or(Ok(0), |value| value.cells(limit))?
            .checked_add(storage)
            .and_then(|n| n.checked_add(1))
            .filter(|n| *n <= limit)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(Self {
            ty,
            semantic: semantic.map(Box::new),
            image: Arc::new(image),
        })
    }
    pub fn ty(&self) -> TypeId {
        self.ty
    }
    /// Read-only semantic inspection. Typed copies must retain the complete carrier.
    pub fn decoded_semantic(&self) -> Option<&Value> {
        self.semantic.as_deref()
    }
    pub(crate) fn validate_storage(&self, types: &dyn TypeView) -> Result<(), Error> {
        // The private constructor binds the extent to a checked target layout.
        // Inspection validates the arena/ready root without rescanning its graph.
        match types.kind(self.ty)? {
            TypeKind::Record(_) | TypeKind::Any(_) => {
                types.record_storage_definition(self.ty)?;
            }
            TypeKind::FixedArray { .. } => {}
            TypeKind::Distinct(id) => {
                types.distinct(*id)?;
            }
            _ => return Err(Error::UnsupportedType(self.ty)),
        }
        Ok(())
    }
    pub fn image(&self) -> &ByteImage {
        &self.image
    }
    pub fn storage_cells(&self) -> usize {
        self.image.len().saturating_add(self.image.metadata_cells())
    }
    pub fn field(&self, types: &dyn TypeView, index: usize, limit: usize) -> Result<Value, Error> {
        let (offset, ty) = self.field_layout(types, index)?;
        self.read_child(types, offset, ty, limit)
    }
    pub fn index(&self, types: &dyn TypeView, index: usize, limit: usize) -> Result<Value, Error> {
        let element = self.element_type(types, index)?;
        let mut layouts = LayoutEngine::new(types, self.image.target().policy);
        let stride =
            usize::try_from(layouts.layout(element)?.size).map_err(|_| Error::CheckedCast)?;
        let offset = stride.checked_mul(index).ok_or(Error::CheckedCast)?;
        self.read_child(types, offset, element, limit)
    }
    fn read_child(
        &self,
        types: &dyn TypeView,
        offset: usize,
        ty: TypeId,
        limit: usize,
    ) -> Result<Value, Error> {
        if self.semantic.is_none() {
            self.image.read_partial_preserving_with_limit(
                types,
                self.image.target(),
                offset,
                ty,
                limit,
            )
        } else {
            self.image
                .read_preserving_with_limit(types, self.image.target(), offset, ty, limit)
        }
    }
    pub(crate) fn element_type(&self, types: &dyn TypeView, index: usize) -> Result<TypeId, Error> {
        let TypeKind::FixedArray { element, count } = *types.kind(self.ty)? else {
            return Err(Error::UnsupportedType(self.ty));
        };
        let length = usize::try_from(count).map_err(|_| Error::CheckedCast)?;
        if index >= length {
            return Err(Error::OutOfBounds { index, length });
        }
        Ok(element)
    }
    pub fn with_field(
        &self,
        types: &dyn TypeView,
        index: usize,
        value: &Value,
        limit: usize,
    ) -> Result<Value, Error> {
        let (offset, ty) = self.field_layout(types, index)?;
        let mut image = self.image.clone_with_limit(limit)?;
        image.write(types, image.target(), offset, ty, value)?;
        if types.record_storage_definition(self.ty)?.kind == RecordKind::Union {
            image.note_union_field(types, 0, self.ty, index)?;
        }
        image.read_preserving_with_limit(types, image.target(), 0, self.ty, limit)
    }
    pub fn representation(&self, types: &dyn TypeView, limit: usize) -> Result<Value, Error> {
        let TypeKind::Distinct(id) = types.kind(self.ty)? else {
            return Err(Error::UnsupportedType(self.ty));
        };
        self.image.read_preserving_with_limit(
            types,
            self.image.target(),
            0,
            types.distinct(*id)?.representation,
            limit,
        )
    }
    fn field_layout(&self, types: &dyn TypeView, index: usize) -> Result<(usize, TypeId), Error> {
        let fields = &types.record_storage_definition(self.ty)?.fields;
        let ty = *fields.get(index).ok_or(Error::OutOfBounds {
            index,
            length: fields.len(),
        })?;
        let mut layouts = LayoutEngine::new(types, self.image.target().policy);
        let offset = usize::try_from(layouts.layout(self.ty)?.field_offsets[index])
            .map_err(|_| Error::CheckedCast)?;
        Ok((offset, ty))
    }
    /// Publication is permitted only when semantic encoding loses no storage facts.
    pub fn publication_semantic(
        &self,
        types: &dyn TypeView,
        limit: usize,
    ) -> Result<&Value, Error> {
        if !self.image.fully_initialized() {
            return Err(Error::UnsupportedPointerOperation(
                "aggregate snapshot with initialization holes cannot be published",
            ));
        }
        let semantic = self
            .decoded_semantic()
            .ok_or(Error::UnsupportedPointerOperation(
                "storage-only aggregate snapshot requires an explicit storage publication receipt",
            ))?;
        let mut canonical =
            ByteImage::encode(types, self.image.target(), self.ty, semantic, limit)?;
        canonical.retokenize_handles_from(&self.image)?;
        if canonical != *self.image {
            return Err(Error::UnsupportedPointerOperation(
                "aggregate snapshot has inactive bytes or provenance that semantic publication would discard",
            ));
        }
        Ok(semantic)
    }
}
