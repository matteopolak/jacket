//! The nominal descriptor-pointer relation used by first-class runtime Type values.
use crate::{TypeError, TypeId, TypeKind, TypeView};

/// A runtime Type cell holds one pointer to the designated Type_Info header.
/// This proof contains language identities; it contains no address or ID encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RuntimeTypeSchema {
    ty: TypeId,
    header: TypeId,
    descriptor: TypeId,
}
impl RuntimeTypeSchema {
    pub fn from_view(types: &dyn TypeView) -> Result<Self, TypeError> {
        let ty = types.meta_type();
        let header = types
            .runtime_type_header()
            .ok_or(TypeError::Incomplete(ty))?;
        types.record_definition(header)?;
        let descriptor = types
            .lookup(&TypeKind::Pointer(header))
            .ok_or(TypeError::Incomplete(ty))?;
        Ok(Self {
            ty,
            header,
            descriptor,
        })
    }
    pub fn ty(self) -> TypeId {
        self.ty
    }
    pub fn header_type(self) -> TypeId {
        self.header
    }
    pub fn descriptor_type(self) -> TypeId {
        self.descriptor
    }
    pub fn validate(self, types: &dyn TypeView) -> Result<(), TypeError> {
        types.kind(self.ty)?;
        let expected = Self::from_view(types)?;
        if expected != self {
            return Err(TypeError::RuntimeTypeHeaderConflict {
                expected: expected.header,
                actual: self.header,
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntegerType, LayoutEngine, LayoutPolicy, RecordKind, ScalarLayout, ScalarType};

    #[test]
    fn header_binding_is_nominal_transactional_and_survives_freeze() {
        let mut types = crate::TypeRegistry::new();
        let meta = types.meta_type();
        assert_eq!(
            RuntimeTypeSchema::from_view(&types),
            Err(TypeError::Incomplete(meta))
        );
        let header = types.reserve_record(RecordKind::Struct);
        assert_eq!(
            types.bind_runtime_type_header(header),
            Err(TypeError::Incomplete(header))
        );
        assert_eq!(types.runtime_type_header(), None);
        assert_eq!(types.lookup(&TypeKind::Pointer(header)), None);
        types
            .define_record(
                header,
                [
                    types.scalar(ScalarType::Int(IntegerType::U32)),
                    types.scalar(ScalarType::Int(IntegerType::S64)),
                ],
            )
            .unwrap();
        types.bind_runtime_type_header(header).unwrap();
        types.bind_runtime_type_header(header).unwrap();
        let proof = RuntimeTypeSchema::from_view(&types).unwrap();
        assert_eq!(proof.ty(), meta);
        assert_eq!(proof.header_type(), header);
        assert_eq!(
            types.kind(proof.descriptor_type()).unwrap(),
            &TypeKind::Pointer(header)
        );
        let imitation = types.reserve_record(RecordKind::Struct);
        types
            .define_record(
                imitation,
                types.record_definition(header).unwrap().fields.to_vec(),
            )
            .unwrap();
        assert_eq!(
            types.bind_runtime_type_header(imitation),
            Err(TypeError::RuntimeTypeHeaderConflict {
                expected: header,
                actual: imitation
            })
        );
        let types = types.freeze().unwrap();
        assert_eq!(RuntimeTypeSchema::from_view(&types).unwrap(), proof);
        proof.validate(&types).unwrap();
        assert!(matches!(
            proof.validate(&crate::TypeRegistry::new()),
            Err(TypeError::ForeignType(_))
        ));
    }

    #[test]
    fn invalid_headers_cannot_create_a_relation() {
        let mut types = crate::TypeRegistry::new();
        let foreign = crate::TypeRegistry::new();
        assert!(matches!(
            types.bind_runtime_type_header(foreign.meta_type()),
            Err(TypeError::ForeignType(_))
        ));
        let boolean = types.scalar(ScalarType::Bool);
        assert_eq!(
            types.bind_runtime_type_header(boolean),
            Err(TypeError::WrongKind(boolean))
        );
        let union = types.reserve_record(RecordKind::Union);
        types.define_record(union, [boolean]).unwrap();
        assert_eq!(
            types.bind_runtime_type_header(union),
            Err(TypeError::WrongKind(union))
        );
        let any = types.reserve_any();
        assert_eq!(
            types.bind_runtime_type_header(any),
            Err(TypeError::WrongKind(any))
        );
        assert_eq!(types.runtime_type_header(), None);
    }

    #[test]
    fn type_cells_use_the_selected_pointer_width_without_a_host_address() {
        let mut types = crate::TypeRegistry::new();
        let meta = types.meta_type();
        let holder = types.reserve_record(RecordKind::Struct);
        let array = types.fixed_array(meta, 2).unwrap();
        types.define_record(holder, [meta, array]).unwrap();
        let ilp32 = LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                ScalarLayout::new(1, 1),
                ScalarLayout::new(2, 2),
                ScalarLayout::new(4, 4),
                ScalarLayout::new(8, 4),
            ],
            [ScalarLayout::new(4, 4), ScalarLayout::new(8, 4)],
            ScalarLayout::new(1, 1),
        )
        .unwrap();
        let types = types.freeze().unwrap();
        for policy in [ilp32, LayoutPolicy::lp64()] {
            let mut layouts = LayoutEngine::new(&types, policy);
            assert_eq!(layouts.layout(meta).unwrap().size, policy.pointer().size);
            assert_eq!(
                layouts.layout(meta).unwrap().alignment,
                policy.pointer().alignment
            );
            assert_eq!(
                layouts.layout(array).unwrap().array_stride,
                Some(policy.pointer().size)
            );
            assert_eq!(
                layouts.layout(holder).unwrap().field_offsets.as_ref(),
                &[0, policy.pointer().size]
            );
            assert_eq!(
                layouts.layout(holder).unwrap().size,
                3 * policy.pointer().size
            );
        }
    }
}
