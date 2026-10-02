//! Registry-backed LLVM storage types and explicitly limited internal signatures.
use inkwell::{
    AddressSpace,
    context::Context,
    targets::TargetData,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType, IntType, StructType},
};
use jai_types::{
    CallingConvention, ContextMode, FloatType, IntegerType, Layout, LayoutEngine, LayoutError,
    LayoutPolicy, RecordKind, ScalarLayout, TypeError, TypeId, TypeKind, Types,
};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Debug)]
pub enum Error {
    Type(TypeError),
    Layout(LayoutError),
    NoStorage(TypeId),
    UnsupportedUnion(TypeId),
    ArrayTooLarge {
        ty: TypeId,
        count: u64,
    },
    NotProcedure(TypeId),
    UnsupportedSignature(TypeId),
    LayoutMismatch {
        ty: TypeId,
        expected: Layout,
        actual: Layout,
    },
}
impl From<TypeError> for Error {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<LayoutError> for Error {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => write!(f, "invalid LLVM type identity: {error}"),
            Self::Layout(error) => write!(f, "unsupported LLVM target layout: {error}"),
            Self::NoStorage(ty) => write!(f, "type {ty:?} has no LLVM storage representation"),
            Self::UnsupportedUnion(ty) => {
                write!(f, "union {ty:?} requires a storage and field-access ABI")
            }
            Self::ArrayTooLarge { ty, count } => {
                write!(f, "array {ty:?} length {count} exceeds the LLVM API limit")
            }
            Self::NotProcedure(ty) => write!(f, "type {ty:?} is not a procedure signature"),
            Self::UnsupportedSignature(ty) => write!(
                f,
                "procedure {ty:?} requires an unsupported calling convention, context, or result ABI"
            ),
            Self::LayoutMismatch {
                ty,
                expected,
                actual,
            } => write!(
                f,
                "LLVM layout for {ty:?} differs from registry policy: expected {expected:?}, actual {actual:?}"
            ),
        }
    }
}
impl std::error::Error for Error {}

/// A per-context cache. Semantic pointees and nominal identities remain in `Types`.
pub struct TypeLowerer<'ctx, 'types> {
    context: &'ctx Context,
    types: &'types Types,
    cache: HashMap<TypeId, BasicTypeEnum<'ctx>>,
    ready: HashSet<TypeId>,
}
impl<'ctx, 'types> TypeLowerer<'ctx, 'types> {
    pub fn new(context: &'ctx Context, types: &'types Types) -> Self {
        // Reserve every named record before any body is installed. LLVM pointers
        // are opaque; source-level recursive pointees stay in the registry.
        let cache = types
            .iter()
            .filter(|(_, kind)| matches!(kind, TypeKind::Record(_)))
            .map(|(id, _)| {
                (
                    id,
                    context
                        .opaque_struct_type(&format!("jai.type.{}", id.index()))
                        .into(),
                )
            })
            .collect();
        Self {
            context,
            types,
            cache,
            ready: HashSet::new(),
        }
    }

    pub fn basic(&mut self, root: TypeId) -> Result<BasicTypeEnum<'ctx>, Error> {
        self.types.kind(root)?; // Check provenance before consulting the cache.
        let mut pending = vec![(root, false)];
        while let Some((id, finish)) = pending.pop() {
            if self.ready.contains(&id) {
                continue;
            }
            let kind = self.types.kind(id)?;
            if !finish {
                let dependencies: Vec<TypeId> = match kind {
                    TypeKind::FixedArray { element, count } => {
                        if *count > u64::from(u32::MAX) {
                            return Err(Error::ArrayTooLarge {
                                ty: id,
                                count: *count,
                            });
                        }
                        vec![*element]
                    }
                    TypeKind::Record(record) => {
                        let definition = self.types.record(*record)?;
                        if definition.kind == RecordKind::Union {
                            return Err(Error::UnsupportedUnion(id));
                        }
                        definition.fields.to_vec()
                    }
                    _ => vec![],
                };
                if !dependencies.is_empty() {
                    pending.push((id, true));
                    pending.extend(dependencies.into_iter().rev().map(|id| (id, false)));
                    continue;
                }
            }
            let lowered: BasicTypeEnum<'ctx> = match kind {
                TypeKind::Void | TypeKind::Type => return Err(Error::NoStorage(id)),
                TypeKind::Bool => self.context.bool_type().into(),
                TypeKind::Integer(integer) => integer_type(self.context, *integer).into(),
                TypeKind::Float(FloatType::F32) => self.context.f32_type().into(),
                TypeKind::Float(FloatType::F64) => self.context.f64_type().into(),
                TypeKind::Pointer(_) | TypeKind::Procedure(_) => {
                    self.context.ptr_type(AddressSpace::default()).into()
                }
                TypeKind::Enum(enumeration) => integer_type(
                    self.context,
                    self.types.enumeration(*enumeration)?.representation,
                )
                .into(),
                TypeKind::FixedArray { element, count } => self.cache[element]
                    .array_type(u32::try_from(*count).map_err(|_| Error::ArrayTooLarge {
                        ty: id,
                        count: *count,
                    })?)
                    .into(),
                TypeKind::String | TypeKind::Slice(_) => self.descriptor().into(),
                TypeKind::DynamicArray(_) => {
                    let pointer = self.context.ptr_type(AddressSpace::default()).into();
                    let count = self.context.i64_type().into();
                    let allocator = self.context.struct_type(&[pointer, pointer], false).into();
                    self.context
                        .struct_type(&[count, pointer, count, allocator], false)
                        .into()
                }
                TypeKind::Record(record) => {
                    let definition = self.types.record(*record)?;
                    if definition.kind == RecordKind::Union {
                        return Err(Error::UnsupportedUnion(id));
                    }
                    let fields: Vec<_> = definition
                        .fields
                        .iter()
                        .map(|field| self.cache[field])
                        .collect();
                    let structure = self.cache[&id].into_struct_type();
                    structure.set_body(&fields, false);
                    structure.into()
                }
            };
            self.cache.insert(id, lowered);
            self.ready.insert(id);
        }
        Ok(self.cache[&root])
    }

    fn descriptor(&self) -> StructType<'ctx> {
        self.context.struct_type(
            &[
                self.context.i64_type().into(),
                self.context.ptr_type(AddressSpace::default()).into(),
            ],
            false,
        )
    }

    /// Internal LLVM signatures only. Foreign ABIs, implicit contexts, and
    /// multiple results require a separate ABI classifier before emission.
    pub fn function(&mut self, ty: TypeId) -> Result<FunctionType<'ctx>, Error> {
        let TypeKind::Procedure(id) = self.types.kind(ty)? else {
            return Err(Error::NotProcedure(ty));
        };
        let signature = self.types.procedure(*id)?;
        if signature.convention != CallingConvention::Jai
            || signature.context != ContextMode::None
            || signature.results.len() > 1
        {
            return Err(Error::UnsupportedSignature(ty));
        }
        let parameters = signature
            .parameters
            .iter()
            .map(|&ty| self.basic(ty).map(BasicMetadataTypeEnum::from))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(match signature.results.as_ref() {
            [] => self.context.void_type().fn_type(&parameters, false),
            [result] => self.basic(*result)?.fn_type(&parameters, false),
            _ => unreachable!(),
        })
    }

    /// Derive primitive policy from the actual LLVM TargetData, then compare
    /// storage size, alignment, field offsets, and array stride with the registry.
    pub fn verify_layout(&mut self, ty: TypeId, target: &TargetData) -> Result<Layout, Error> {
        let lowered = self.basic(ty)?;
        let policy = layout_policy(self.context, target)?;
        let expected = LayoutEngine::new(self.types, policy).layout(ty)?.clone();
        let fields = match lowered {
            BasicTypeEnum::StructType(structure) => (0..structure.count_fields())
                .map(|index| {
                    target
                        .offset_of_element(&structure, index)
                        .expect("validated LLVM field index")
                })
                .collect(),
            _ => vec![],
        };
        let array_stride = match lowered {
            BasicTypeEnum::ArrayType(array) => Some(target.get_abi_size(&array.get_element_type())),
            _ => None,
        };
        let actual = Layout {
            size: target.get_abi_size(&lowered),
            alignment: target.get_abi_alignment(&lowered),
            field_offsets: fields.into_boxed_slice(),
            array_stride,
        };
        if actual != expected {
            return Err(Error::LayoutMismatch {
                ty,
                expected,
                actual,
            });
        }
        Ok(actual)
    }
}

/// TargetData comes from the selected target machine, never Rust host layouts.
pub fn layout_policy(context: &Context, target: &TargetData) -> Result<LayoutPolicy, Error> {
    fn scalar(target: &TargetData, ty: BasicTypeEnum<'_>) -> ScalarLayout {
        ScalarLayout::new(target.get_abi_size(&ty), target.get_abi_alignment(&ty))
    }
    Ok(LayoutPolicy::new(
        scalar(target, context.ptr_type(AddressSpace::default()).into()),
        [
            IntegerType::U8,
            IntegerType::U16,
            IntegerType::U32,
            IntegerType::U64,
        ]
        .map(|integer| scalar(target, integer_type(context, integer).into())),
        [
            scalar(target, context.f32_type().into()),
            scalar(target, context.f64_type().into()),
        ],
        scalar(target, context.bool_type().into()),
    )?)
}

/// LLVM integer storage is unsignedness-neutral; checked semantics retain it in Types.
pub(crate) fn integer_type(context: &Context, ty: IntegerType) -> IntType<'_> {
    match ty {
        IntegerType::S8 | IntegerType::U8 => context.i8_type(),
        IntegerType::S16 | IntegerType::U16 => context.i16_type(),
        IntegerType::S32 | IntegerType::U32 => context.i32_type(),
        IntegerType::S64 | IntegerType::U64 => context.i64_type(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use inkwell::{
        OptimizationLevel,
        targets::{CodeModel, InitializationConfig, RelocMode, Target, TargetMachine},
    };
    use jai_types::{Integer, ProcedureType, ScalarType, TypeRegistry};

    fn native_data() -> TargetData {
        Target::initialize_native(&InitializationConfig::default()).unwrap();
        let triple = TargetMachine::get_default_triple();
        Target::from_triple(&triple)
            .unwrap()
            .create_target_machine(
                &triple,
                "generic",
                "",
                OptimizationLevel::None,
                RelocMode::Default,
                CodeModel::Default,
            )
            .unwrap()
            .get_target_data()
    }

    #[test]
    fn native_storage_matches_registry_and_produces_verified_modules() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let boolean = registry.scalar(ScalarType::Bool);
        let float = registry.float(FloatType::F32);
        let double = registry.float(FloatType::F64);
        let enumeration = registry.reserve_enum(IntegerType::U16);
        registry
            .define_enum(
                enumeration,
                [Integer::checked(IntegerType::U16, 7).unwrap()],
            )
            .unwrap();
        let a = registry.reserve_record(RecordKind::Struct);
        let b = registry.reserve_record(RecordKind::Struct);
        let pa = registry.pointer(a).unwrap();
        let pb = registry.pointer(b).unwrap();
        registry
            .define_record(a, [boolean, float, double, enumeration, pb])
            .unwrap();
        registry.define_record(b, [byte, pa]).unwrap();
        let same_fields_a = registry.reserve_record(RecordKind::Struct);
        let same_fields_b = registry.reserve_record(RecordKind::Struct);
        registry.define_record(same_fields_a, [byte]).unwrap();
        registry.define_record(same_fields_b, [byte]).unwrap();
        let array = registry.fixed_array(a, 3).unwrap();
        let empty_array = registry.fixed_array(a, 0).unwrap();
        let slice = registry.slice(a).unwrap();
        let dynamic = registry.dynamic_array(a).unwrap();
        let string = registry.string();
        let callable = registry
            .procedure(ProcedureType {
                parameters: Box::new([a, float, enumeration, slice]),
                results: Box::new([double]),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
            })
            .unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let target = native_data();
        let mut lowerer = TypeLowerer::new(&context, &types);
        let module = context.create_module("type-test");
        module.set_data_layout(&target.get_data_layout());
        for id in [
            byte,
            same_fields_a,
            same_fields_b,
            boolean,
            float,
            double,
            enumeration,
            pa,
            pb,
            a,
            b,
            array,
            empty_array,
            slice,
            dynamic,
            string,
            callable,
        ] {
            let layout = lowerer.verify_layout(id, &target).unwrap();
            let storage = lowerer.basic(id).unwrap();
            assert_eq!(storage, lowerer.basic(id).unwrap());
            assert_eq!(layout.size, target.get_abi_size(&storage));
            module
                .add_global(storage, None, &format!("test.{}", id.index()))
                .set_initializer(&storage.const_zero());
        }
        assert_ne!(lowerer.basic(a).unwrap(), lowerer.basic(b).unwrap());
        assert_ne!(
            lowerer.basic(same_fields_a).unwrap(),
            lowerer.basic(same_fields_b).unwrap()
        );
        assert_eq!(
            lowerer.basic(slice).unwrap(),
            lowerer.basic(string).unwrap()
        );
        assert_eq!(
            lowerer
                .basic(dynamic)
                .unwrap()
                .into_struct_type()
                .get_field_type_at_index(3)
                .unwrap()
                .into_struct_type()
                .count_fields(),
            2
        );
        let signature = lowerer.function(callable).unwrap();
        assert_eq!(
            signature.get_return_type().unwrap(),
            context.f64_type().into()
        );
        assert_eq!(signature.count_param_types(), 4);
        module.add_function("internal", signature, None);
        module.verify().unwrap();
    }

    #[test]
    fn explicit_32_bit_target_data_drives_descriptor_and_array_layout() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let double = registry.float(FloatType::F64);
        let slice = registry.slice(byte).unwrap();
        let dynamic = registry.dynamic_array(byte).unwrap();
        let record = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record(record, [byte, double, slice])
            .unwrap();
        let array = registry.fixed_array(record, 2).unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let target = TargetData::create("e-p:32:32-i64:32-f64:32");
        let mut lowerer = TypeLowerer::new(&context, &types);
        assert_eq!(
            layout_policy(&context, &target).unwrap().pointer(),
            ScalarLayout::new(4, 4)
        );
        assert_eq!(lowerer.verify_layout(slice, &target).unwrap().size, 12);
        assert_eq!(lowerer.verify_layout(dynamic, &target).unwrap().size, 28);
        assert_eq!(
            lowerer
                .verify_layout(record, &target)
                .unwrap()
                .field_offsets
                .as_ref(),
            &[0, 4, 12]
        );
        assert_eq!(
            lowerer.verify_layout(array, &target).unwrap().array_stride,
            Some(24)
        );
    }

    #[test]
    fn unsupported_abis_and_storage_fail_without_poisoning_the_cache() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let void = registry.void();
        let meta = registry.meta_type();
        let union = registry.reserve_record(RecordKind::Union);
        registry.define_record(union, [byte]).unwrap();
        let pointer_to_union = registry.pointer(union).unwrap();
        let huge = registry.fixed_array(byte, u64::from(u32::MAX) + 1).unwrap();
        let multi = registry
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([byte, byte]),
                convention: CallingConvention::Jai,
                context: ContextMode::None,
            })
            .unwrap();
        let foreign = registry
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                convention: CallingConvention::C,
                context: ContextMode::None,
            })
            .unwrap();
        let implicit = registry
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
            })
            .unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let mut lowerer = TypeLowerer::new(&context, &types);
        assert!(matches!(lowerer.basic(void), Err(Error::NoStorage(id)) if id == void));
        assert!(matches!(lowerer.basic(meta), Err(Error::NoStorage(id)) if id == meta));
        assert!(matches!(lowerer.basic(union), Err(Error::UnsupportedUnion(id)) if id == union));
        assert!(matches!(lowerer.basic(huge), Err(Error::ArrayTooLarge { ty, .. }) if ty == huge));
        for id in [multi, foreign, implicit] {
            assert!(
                matches!(lowerer.function(id), Err(Error::UnsupportedSignature(ty)) if ty == id)
            );
            assert!(lowerer.basic(id).unwrap().is_pointer_type());
        }
        assert!(lowerer.basic(pointer_to_union).unwrap().is_pointer_type());
        assert_eq!(
            lowerer.basic(byte).unwrap().into_int_type().get_bit_width(),
            8
        );
        assert!(matches!(lowerer.function(byte), Err(Error::NotProcedure(id)) if id == byte));
        let other = TypeRegistry::new();
        assert!(matches!(
            lowerer.basic(other.scalar(ScalarType::Bool)),
            Err(Error::Type(TypeError::ForeignType(_)))
        ));
    }

    #[test]
    fn nested_record_lowering_uses_an_iterative_dependency_walk() {
        let mut registry = TypeRegistry::new();
        let mut root = registry.scalar(ScalarType::Int(IntegerType::U64));
        for _ in 0..4096 {
            let record = registry.reserve_record(RecordKind::Struct);
            registry.define_record(record, [root]).unwrap();
            root = record;
        }
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let mut lowerer = TypeLowerer::new(&context, &types);
        assert!(lowerer.basic(root).unwrap().is_struct_type());
    }
}
