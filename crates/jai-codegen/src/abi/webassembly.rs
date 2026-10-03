//! The canonical Wasm C ABI unwraps one unpadded element; other composites use memory.
use super::*;
impl<'ctx> Classifier<'ctx, '_, '_> {
    pub(super) fn webassembly_aggregate(
        &mut self,
        ty: TypeId,
        storage: BasicTypeEnum<'ctx>,
        layout: &Layout,
        result: bool,
    ) -> Result<Value<'ctx>, Error> {
        if let Some(element) = self.single_element_record(ty)? {
            let carrier = self.lowerer.basic(element)?;
            return Ok(Value::Coerce {
                pieces: vec![Piece {
                    ty: carrier,
                    offset: 0,
                }],
                carrier: Some(carrier),
            });
        }
        Ok(Value::Indirect {
            storage,
            alignment: layout.alignment,
            by_value: !result,
        })
    }
    // Source field multiplicity matters: a union with two identical fields is
    // not a single-element record even when ordinary leaf traversal deduplicates it.
    fn single_element_record(&mut self, root: TypeId) -> Result<Option<TypeId>, Error> {
        if !matches!(
            self.types.kind(root)?,
            TypeKind::Record(_) | TypeKind::Any(_)
        ) {
            return Ok(None);
        }
        let root_size = self.layout(root)?.size;
        let mut selected = root;
        for _ in 0..MAX_CLASSIFICATION_NODES {
            match self.types.kind(selected)? {
                TypeKind::Distinct(id) => selected = self.types.distinct(*id)?.representation,
                TypeKind::Record(_) | TypeKind::Any(_) => {
                    let fields = self
                        .types
                        .record_storage_definition(selected)?
                        .fields
                        .clone();
                    let mut nonempty = None;
                    for field in fields {
                        if self.layout(field)?.size == 0 {
                            continue;
                        }
                        if nonempty.replace(field).is_some() {
                            return Ok(None);
                        }
                    }
                    let Some(field) = nonempty else {
                        return Ok(None);
                    };
                    selected = field;
                }
                TypeKind::FixedArray {
                    element,
                    count: 1,
                } => selected = *element,
                TypeKind::Bool
                | TypeKind::Integer(_)
                | TypeKind::Float(_)
                | TypeKind::Pointer(_)
                | TypeKind::Procedure(_)
                | TypeKind::Enum(_)
                | TypeKind::Type => {
                    return Ok((self.layout(selected)?.size == root_size).then_some(selected));
                }
                _ => return Ok(None),
            }
        }
        Err(Error::ClassificationLimit {
            ty: root,
            limit: MAX_CLASSIFICATION_NODES,
        })
    }
}
