//! Registry-backed LLVM storage types and explicitly limited internal signatures.
use inkwell::{
    AddressSpace,
    context::Context,
    targets::TargetData,
    types::{BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType, IntType, StructType},
    values::{BasicValue, BasicValueEnum, StructValue},
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
    UnsupportedCustomLayout(TypeId),
    MissingRecordTarget(TypeId),
    RecordStorageTooLarge {
        ty: TypeId,
        size: u64,
        address_bits: u32,
    },
    UnsupportedPlacement(TypeId),
    UnsupportedAlignment {
        requested: u32,
        actual: u32,
    },
    InvalidCustomLayout,
    MissingDescriptorTarget(TypeId),
    DescriptorTargetMismatch {
        ty: TypeId,
        descriptor: Box<LayoutPolicy>,
        target: Box<LayoutPolicy>,
    },
    Union {
        ty: TypeId,
        error: crate::unions::Error,
    },
    ArrayTooLarge {
        ty: TypeId,
        count: u64,
    },
    ArrayStorageTooLarge {
        ty: TypeId,
        count: u64,
        stride: u64,
        address_bits: u32,
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
            Self::UnsupportedCustomLayout(ty) => write!(
                f,
                "record {ty:?} requires bound target data for custom layout lowering"
            ),
            Self::RecordStorageTooLarge {
                ty,
                size,
                address_bits,
            } => write!(
                f,
                "record {ty:?} storage of {size} bytes exceeds the selected {address_bits}-bit address extent"
            ),
            Self::MissingRecordTarget(ty) => {
                write!(f, "record {ty:?} requires explicitly selected target data")
            }
            Self::UnsupportedPlacement(ty) => write!(
                f,
                "record {ty:?} requires ordered placement initialization support"
            ),
            Self::UnsupportedAlignment {
                requested,
                actual,
            } => write!(
                f,
                "target represents requested alignment {requested} as {actual}"
            ),
            Self::InvalidCustomLayout => {
                f.write_str("custom record representation disagrees with the target layout")
            }
            Self::MissingDescriptorTarget(ty) => write!(
                f,
                "runtime type descriptor for {ty:?} requires explicitly selected target data"
            ),
            Self::DescriptorTargetMismatch {
                ty,
                descriptor,
                target,
            } => write!(
                f,
                "runtime type descriptor for {ty:?} uses layout {descriptor:?}, but LLVM selected {target:?}"
            ),
            Self::UnsupportedUnion(ty) => {
                write!(f, "union {ty:?} requires a storage and field-access ABI")
            }
            Self::Union {
                ty,
                error,
            } => write!(f, "union {ty:?}: {error}"),
            Self::ArrayTooLarge {
                ty,
                count,
            } => {
                write!(f, "array {ty:?} length {count} exceeds the LLVM API limit")
            }
            Self::ArrayStorageTooLarge {
                ty,
                count,
                stride,
                address_bits,
            } => write!(
                f,
                "array {ty:?} length {count} with element stride {stride} exceeds {address_bits}-bit target storage"
            ),
            Self::NotProcedure(ty) => write!(f, "type {ty:?} is not a procedure signature"),
            Self::UnsupportedSignature(ty) => write!(
                f,
                "procedure {ty:?} requires an unsupported calling convention or context ABI"
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
impl std::error::Error for Error {
}

/// A per-context cache. Semantic pointees and nominal identities remain in `Types`.
pub struct TypeLowerer<'ctx, 'types> {
    context: &'ctx Context,
    types: &'types Types,
    target: Option<&'types TargetData>,
    context_pointer: Option<TypeId>,
    cache: HashMap<TypeId, BasicTypeEnum<'ctx>>,
    ready: HashSet<TypeId>,
    layouts: Option<LayoutEngine<'types>>,
}
impl<'ctx, 'types> TypeLowerer<'ctx, 'types> {
    pub fn new(context: &'ctx Context, types: &'types Types) -> Self {
        // Reserve every named record before any body is installed. LLVM pointers
        // are opaque; source-level recursive pointees stay in the registry.
        let cache = types
            .iter()
            .filter(|(_, kind)| kind.record_storage_id().is_some())
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
            target: None,
            context_pointer: None,
            cache,
            ready: HashSet::new(),
            layouts: None,
        }
    }

    pub fn with_target(
        context: &'ctx Context,
        types: &'types Types,
        target: &'types TargetData,
    ) -> Self {
        let mut lowerer = Self::new(context, types);
        lowerer.target = Some(target);
        lowerer
    }

    /// Bind the checked per-library context schema before lowering signatures.
    pub fn set_context_pointer(&mut self, pointer: Option<TypeId>) -> Result<(), Error> {
        if let Some(pointer) = pointer
            && !matches!(self.types.kind(pointer)?, TypeKind::Pointer(_))
        {
            return Err(Error::UnsupportedSignature(pointer));
        }
        self.context_pointer = pointer;
        Ok(())
    }
    pub fn registry(&self) -> &'types Types {
        self.types
    }
    pub fn context(&self) -> &'ctx Context {
        self.context
    }
    pub fn target_data(&self) -> Option<&TargetData> {
        self.target
    }
    pub(crate) fn semantic_layout(&mut self, ty: TypeId) -> Result<Layout, Error> {
        if self.layouts.is_none() {
            let target = self.target.ok_or(Error::UnsupportedCustomLayout(ty))?;
            self.layouts = Some(LayoutEngine::new(
                self.types,
                layout_policy(self.context, target)?,
            ));
        }
        Ok(self
            .layouts
            .as_mut()
            .expect("layout engine initialized")
            .layout(ty)?
            .clone())
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
                    TypeKind::Distinct(distinct) => {
                        vec![self.types.distinct(*distinct)?.representation]
                    }
                    TypeKind::FixedArray {
                        element,
                        count,
                    } => {
                        if *count > u64::from(u32::MAX) {
                            return Err(Error::ArrayTooLarge {
                                ty: id,
                                count: *count,
                            });
                        }
                        vec![*element]
                    }
                    TypeKind::DynamicArray(_) => self
                        .types
                        .allocator_schema()
                        .map(|schema| vec![schema.ty()])
                        .unwrap_or_default(),
                    kind if kind.record_storage_id().is_some() => {
                        let definition = self.types.record_storage_definition(id)?;
                        if (definition.layout.packed
                            || definition.layout.minimum_alignment.is_some()
                            || !definition.layout.field_alignments.is_empty())
                            && self.target.is_none()
                        {
                            return Err(Error::UnsupportedCustomLayout(id));
                        }
                        if definition.kind == RecordKind::Union && self.target.is_none() {
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
                TypeKind::Void | TypeKind::Code => {
                    return Err(Error::NoStorage(id));
                }
                TypeKind::Bool => self.context.bool_type().into(),
                TypeKind::Integer(integer) => integer_type(self.context, *integer).into(),
                TypeKind::Float(FloatType::F32) => self.context.f32_type().into(),
                TypeKind::Float(FloatType::F64) => self.context.f64_type().into(),
                TypeKind::Type | TypeKind::Pointer(_) | TypeKind::Procedure(_) => {
                    self.context.ptr_type(AddressSpace::default()).into()
                }
                TypeKind::Enum(enumeration) => integer_type(
                    self.context,
                    self.types.enumeration(*enumeration)?.representation,
                )
                .into(),
                TypeKind::FixedArray {
                    element,
                    count,
                } => {
                    if let Some(target) = self.target {
                        let address_bits = self
                            .context
                            .ptr_sized_int_type(target, None)
                            .get_bit_width();
                        let stride = target.get_abi_size(&self.cache[element]);
                        let maximum = u64::MAX
                            .checked_shr(64u32.saturating_sub(address_bits))
                            .unwrap_or(0);
                        if address_bits > 64
                            || count
                                .checked_mul(stride)
                                .is_none_or(|bytes| bytes > maximum)
                        {
                            return Err(Error::ArrayStorageTooLarge {
                                ty: id,
                                count: *count,
                                stride,
                                address_bits,
                            });
                        }
                    }
                    self.cache[element]
                        .array_type(u32::try_from(*count).map_err(|_| Error::ArrayTooLarge {
                            ty: id,
                            count: *count,
                        })?)
                        .into()
                }
                TypeKind::String | TypeKind::Slice(_) => self.descriptor().into(),
                TypeKind::DynamicArray(_) => {
                    let pointer = self.context.ptr_type(AddressSpace::default()).into();
                    let count = self.context.i64_type().into();
                    let allocator = self
                        .types
                        .allocator_schema()
                        .map(|schema| self.cache[&schema.ty()])
                        .unwrap_or_else(|| {
                            self.context.struct_type(&[pointer, pointer], false).into()
                        });
                    self.context
                        .struct_type(&[count, pointer, count, allocator], false)
                        .into()
                }
                TypeKind::Distinct(distinct) => {
                    self.cache[&self.types.distinct(*distinct)?.representation]
                }
                TypeKind::Record(_) | TypeKind::Any(_) => {
                    let definition = self.types.record_storage_definition(id)?;
                    let fields: Vec<_> = definition
                        .fields
                        .iter()
                        .map(|field| self.cache[field])
                        .collect();
                    let kind = definition.kind;
                    let placed = definition
                        .layout
                        .field_placements
                        .iter()
                        .any(Option::is_some);
                    let custom_union = kind == RecordKind::Union
                        && definition.layout != jai_types::RecordLayout::default();
                    let target = self.target.ok_or(Error::MissingRecordTarget(id))?;
                    let expected = self.semantic_layout(id)?;
                    let address_bits = self
                        .context
                        .ptr_sized_int_type(target, None)
                        .get_bit_width();
                    let maximum = u64::MAX
                        .checked_shr(64u32.saturating_sub(address_bits))
                        .unwrap_or(0);
                    if address_bits > 64 || expected.size > maximum {
                        return Err(Error::RecordStorageTooLarge {
                            ty: id,
                            size: expected.size,
                            address_bits,
                        });
                    }
                    let fields = if placed || custom_union {
                        let size =
                            u32::try_from(expected.size).map_err(|_| Error::InvalidCustomLayout)?;
                        vec![
                            crate::records::alignment_carrier(
                                self.context,
                                target,
                                expected.alignment,
                            )?,
                            self.context.i8_type().array_type(size).into(),
                        ]
                    } else if kind == RecordKind::Union {
                        crate::unions::body(self.context, target, &fields, &expected).map_err(
                            |error| Error::Union {
                                ty: id,
                                error,
                            },
                        )?
                    } else {
                        crate::records::body(self.context, target, &fields, &expected)?
                    };
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

    /// Internal LLVM signatures. An implicit context requires a bound checked
    /// context-pointer schema; foreign conventions use the separate C classifier.
    /// Multiple internal results use a literal LLVM struct in declaration order.
    pub fn function(&mut self, ty: TypeId) -> Result<FunctionType<'ctx>, Error> {
        let TypeKind::Procedure(id) = self.types.kind(ty)? else {
            return Err(Error::NotProcedure(ty));
        };
        let signature = self.types.procedure(*id)?;
        if signature.convention != CallingConvention::Jai
            || (signature.context == ContextMode::Implicit && self.context_pointer.is_none())
        {
            return Err(Error::UnsupportedSignature(ty));
        }
        let mut parameters = signature
            .parameters
            .iter()
            .map(|&ty| self.basic(ty).map(BasicMetadataTypeEnum::from))
            .collect::<Result<Vec<_>, _>>()?;
        if signature.context == ContextMode::Implicit {
            parameters.insert(
                0,
                self.basic(
                    self.context_pointer
                        .ok_or(Error::UnsupportedSignature(ty))?,
                )?
                .into(),
            );
        }
        Ok(match signature.results.as_ref() {
            [] => self.context.void_type().fn_type(&parameters, false),
            [result] => self.basic(*result)?.fn_type(&parameters, false),
            results => {
                let fields = results
                    .iter()
                    .map(|&result| self.basic(result))
                    .collect::<Result<Vec<_>, _>>()?;
                self.context
                    .struct_type(&fields, false)
                    .fn_type(&parameters, false)
            }
        })
    }

    /// Build an exact named physical record constant from semantic fields.
    /// This narrow public path requires constant values of the canonical LLVM
    /// field types; internal relocation-aware constants may use equivalent
    /// alternate physical types through the common record adapter directly.
    pub fn record_constant(
        &mut self,
        ty: TypeId,
        fields: &[BasicValueEnum<'ctx>],
    ) -> Result<StructValue<'ctx>, crate::Error> {
        let record = self
            .types
            .record_storage_definition(ty)
            .map_err(Error::from)?;
        if record.kind != RecordKind::Struct {
            return Err(crate::Error::Invariant);
        }
        if record.layout.field_placements.iter().any(Option::is_some) {
            return Err(Error::UnsupportedPlacement(ty).into());
        }
        let expected = record.fields.to_vec();
        if fields.len() != expected.len() {
            return Err(crate::Error::Invariant);
        }
        for (&value, expected) in fields.iter().zip(expected) {
            if !value.is_const() || value.get_type() != self.basic(expected)? {
                return Err(crate::Error::Invariant);
            }
        }
        let target = self.target.ok_or(Error::MissingRecordTarget(ty))?;
        let layout = self.verify_layout(ty, target)?;
        let storage = self.basic(ty)?.into_struct_type();
        let value = crate::records::constant(self.context, target, storage, fields, &layout)?;
        if value.get_type() != storage.into() {
            return Err(crate::Error::Invariant);
        }
        Ok(value.into_struct_value())
    }

    /// Resolve a source-owned field to the two-level explicit payload path.
    /// Physical indices are not source field ordinals and cannot cross owners.
    pub fn record_field_path(
        &mut self,
        ty: TypeId,
        field: jai_types::FieldId,
    ) -> Result<[u32; 2], Error> {
        self.types.validate_field(ty, field)?;
        let record = self.types.record_storage_definition(ty)?;
        if record.kind != RecordKind::Struct {
            return Err(Error::InvalidCustomLayout);
        }
        if record.layout.field_placements.iter().any(Option::is_some) {
            return Err(Error::UnsupportedPlacement(ty));
        }
        let source_fields = record.fields.to_vec();
        let target = self.target.ok_or(Error::MissingRecordTarget(ty))?;
        let layout = self.verify_layout(ty, target)?;
        let fields = source_fields
            .iter()
            .map(|&field| self.basic(field))
            .collect::<Result<Vec<_>, _>>()?;
        let (_, ordinals) = crate::records::payload(self.context, target, &fields, &layout)?;
        let ordinal = *ordinals
            .get(field.index())
            .ok_or(Error::InvalidCustomLayout)?;
        Ok([1, ordinal])
    }

    /// Derive primitive policy from the actual LLVM TargetData, then compare
    /// storage size, alignment, field offsets, and array stride with the registry.
    pub fn verify_layout(&mut self, ty: TypeId, target: &TargetData) -> Result<Layout, Error> {
        let lowered = self.basic(ty)?;
        let policy = layout_policy(self.context, target)?;
        let expected = LayoutEngine::new(self.types, policy).layout(ty)?.clone();
        let mut representation = ty;
        while let TypeKind::Distinct(distinct) = self.types.kind(representation)? {
            representation = self.types.distinct(*distinct)?.representation;
        }
        let semantic_fields = match self.types.kind(representation)? {
            TypeKind::Record(record) if self.types.record(*record)?.kind == RecordKind::Union => {
                Some(vec![0; self.types.record(*record)?.fields.len()])
            }
            TypeKind::Record(record)
                if self
                    .types
                    .record(*record)?
                    .layout
                    .field_placements
                    .iter()
                    .any(Option::is_some) =>
            {
                let storage = lowered.into_struct_type();
                let base = target
                    .offset_of_element(&storage, 1)
                    .ok_or(Error::InvalidCustomLayout)?;
                if base != 0 {
                    return Err(Error::InvalidCustomLayout);
                }
                Some(expected.field_offsets.to_vec())
            }
            TypeKind::Record(record) | TypeKind::Any(record) => {
                let source_fields = self.types.record(*record)?.fields.to_vec();
                let fields = source_fields
                    .iter()
                    .map(|&field| self.basic(field))
                    .collect::<Result<Vec<_>, _>>()?;
                let (_, ordinals) =
                    crate::records::payload(self.context, target, &fields, &expected)?;
                let structure = lowered.into_struct_type();
                let payload = structure
                    .get_field_type_at_index(1)
                    .ok_or(Error::InvalidCustomLayout)?
                    .into_struct_type();
                let base = target
                    .offset_of_element(&structure, 1)
                    .ok_or(Error::InvalidCustomLayout)?;
                Some(
                    ordinals
                        .into_iter()
                        .map(|ordinal| {
                            target
                                .offset_of_element(&payload, ordinal)
                                .and_then(|offset| base.checked_add(offset))
                                .ok_or(Error::InvalidCustomLayout)
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                )
            }
            _ => None,
        };
        let fields = if let Some(fields) = semantic_fields {
            fields
        } else {
            match lowered {
                BasicTypeEnum::StructType(structure) => (0..structure.count_fields())
                    .map(|index| {
                        target
                            .offset_of_element(&structure, index)
                            .expect("validated LLVM field index")
                    })
                    .collect(),
                _ => vec![],
            }
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
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let target = native_data();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target);
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
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target);
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
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let foreign = registry
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let implicit = registry
            .procedure(ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let mut lowerer = TypeLowerer::new(&context, &types);
        assert!(matches!(lowerer.basic(void), Err(Error::NoStorage(id)) if id == void));
        assert!(lowerer.basic(meta).unwrap().is_pointer_type());
        assert!(matches!(lowerer.basic(union), Err(Error::UnsupportedUnion(id)) if id == union));
        assert!(matches!(lowerer.basic(huge), Err(Error::ArrayTooLarge { ty, .. }) if ty == huge));
        assert_eq!(
            lowerer
                .function(multi)
                .unwrap()
                .get_return_type()
                .unwrap()
                .into_struct_type()
                .count_fields(),
            2
        );
        for id in [foreign, implicit] {
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
        let target = native_data();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target);
        assert!(lowerer.basic(root).unwrap().is_struct_type());
    }
    #[test]
    fn placement_byte_storage_matches_the_checked_overlapping_offsets() {
        let mut registry = TypeRegistry::new();
        let word = registry.scalar(ScalarType::Int(IntegerType::U64));
        let record = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record_with_placements(
                record,
                [word, word],
                jai_types::RecordLayout::default(),
                [None, Some(0)],
            )
            .unwrap();
        let pointer = registry.pointer(record).unwrap();
        let array = registry.fixed_array(record, 3).unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let target = native_data();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target);
        let layout = lowerer.verify_layout(record, &target).unwrap();
        assert_eq!(layout.field_offsets.as_ref(), &[0, 0]);
        assert_eq!((layout.size, layout.alignment), (8, 8));
        assert_eq!(lowerer.verify_layout(array, &target).unwrap().size, 24);
        assert!(lowerer.basic(pointer).unwrap().is_pointer_type());
    }

    #[test]
    fn custom_record_payload_offsets_alignments_and_distinct_views_match_target() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let word = registry.scalar(ScalarType::Int(IntegerType::U64));
        let packed = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record_with_layout(
                packed,
                [byte, word],
                jai_types::RecordLayout {
                    packed: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let reduced = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record_with_layout(
                reduced,
                [byte, word],
                jai_types::RecordLayout {
                    packed: true,
                    field_alignments: Box::new([None, Some(4)]),
                    ..Default::default()
                },
            )
            .unwrap();
        let aligned = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record_with_layout(
                aligned,
                [byte, packed],
                jai_types::RecordLayout {
                    packed: true,
                    minimum_alignment: Some(32),
                    ..Default::default()
                },
            )
            .unwrap();
        let union = registry.reserve_record(RecordKind::Union);
        registry
            .define_record_with_layout(
                union,
                [byte, word],
                jai_types::RecordLayout {
                    packed: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let distinct = registry.reserve_distinct(jai_types::DistinctKind::Distinct);
        registry.define_distinct(distinct, reduced).unwrap();
        let array = registry.fixed_array(aligned, 2).unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let target = native_data();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target);
        for (ty, size, alignment, offsets) in [
            (packed, 9, 1, vec![0, 1]),
            (reduced, 12, 4, vec![0, 4]),
            (aligned, 32, 32, vec![0, 1]),
            (union, 8, 1, vec![0, 0]),
            (distinct, 12, 4, vec![0, 4]),
        ] {
            let layout = lowerer.verify_layout(ty, &target).unwrap();
            assert_eq!(
                (
                    layout.size,
                    layout.alignment,
                    layout.field_offsets.into_vec()
                ),
                (size, alignment, offsets)
            );
        }
        assert_eq!(
            lowerer.verify_layout(array, &target).unwrap().array_stride,
            Some(32)
        );
    }
    #[test]
    fn bound_target_union_layout_preserves_all_members_at_zero_and_array_stride() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let word = registry.scalar(ScalarType::Int(IntegerType::U32));
        let words = registry.fixed_array(word, 3).unwrap();
        let union = registry.reserve_record(RecordKind::Union);
        registry.define_record(union, [byte, words]).unwrap();
        let sequence = registry.fixed_array(union, 2).unwrap();
        let empty = registry.reserve_record(RecordKind::Union);
        registry.define_record(empty, []).unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let target = TargetData::create("e-p:32:32-i64:32-f64:32");
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target);
        let layout = lowerer.verify_layout(union, &target).unwrap();
        assert_eq!(layout.size, 12);
        assert_eq!(layout.alignment, 4);
        assert_eq!(layout.field_offsets.as_ref(), &[0, 0]);
        assert_eq!(
            lowerer
                .verify_layout(sequence, &target)
                .unwrap()
                .array_stride,
            Some(12)
        );
        assert_eq!(lowerer.verify_layout(empty, &target).unwrap().size, 0);
        let storage = lowerer.basic(union).unwrap().into_struct_type();
        assert_eq!(target.offset_of_element(&storage, 1), Some(0));
        let module = context.create_module("union.layout");
        module.set_data_layout(&target.get_data_layout());
        module.add_global(storage, None, "union");
        module.verify().unwrap();
    }
    #[test]
    fn implicit_internal_signatures_require_the_checked_context_schema() {
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let context_record = registry.reserve_record(RecordKind::Struct);
        registry.define_record(context_record, [byte]).unwrap();
        let pointer = registry.pointer(context_record).unwrap();
        let signature = registry
            .procedure(ProcedureType {
                parameters: vec![byte].into_boxed_slice(),
                results: vec![byte].into_boxed_slice(),
                return_abi: jai_types::ForeignReturnAbi::Natural,
                convention: CallingConvention::Jai,
                context: ContextMode::Implicit,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let types = registry.freeze().unwrap();
        let context = Context::create();
        let mut lowerer = TypeLowerer::new(&context, &types);
        assert!(matches!(
            lowerer.function(signature),
            Err(Error::UnsupportedSignature(_))
        ));
        assert!(lowerer.set_context_pointer(Some(context_record)).is_err());
        lowerer.set_context_pointer(Some(pointer)).unwrap();
        let function = lowerer.function(signature).unwrap();
        assert_eq!(function.count_param_types(), 2);
        assert!(function.get_param_types()[0].is_pointer_type());
        assert_eq!(function.get_param_types()[1], context.i8_type().into());
        lowerer.set_context_pointer(None).unwrap();
        assert!(lowerer.function(signature).is_err());
    }
}
