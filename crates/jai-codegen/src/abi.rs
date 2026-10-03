//! Explicit target C ABIs; aggregate representation follows each platform's rules.
mod aapcs64;
mod platform;
mod webassembly;
mod windows;
use crate::types::{self, TypeLowerer};
use inkwell::{
    AddressSpace,
    attributes::{Attribute, AttributeLoc},
    context::Context,
    targets::{ByteOrdering, TargetData},
    types::{AnyType, BasicMetadataTypeEnum, BasicType, BasicTypeEnum, FunctionType},
    values::{CallSiteValue, FunctionValue},
};
use jai_types::{
    ContextMode, FloatType, Layout, LayoutEngine, RecordKind, TypeError, TypeId, TypeKind, Types,
};
pub use platform::Platform;
use std::{collections::HashSet, fmt, num::NonZeroU32};

/// Bound unique type/offset visits independently of source field multiplicity.
const MAX_CLASSIFICATION_NODES: usize = 65_536;

pub use crate::target::NativeTarget;

#[derive(Debug)]
pub enum Error {
    Storage(types::Error),
    Type(TypeError),
    Build(inkwell::builder::BuilderError),
    Target(String),
    UnsupportedTarget(String),
    UnsupportedType(TypeId),
    UnsupportedPlacedRecord(TypeId),
    InvalidSignature(TypeId),
    MultipleForeignResults(TypeId),
    InvalidCarrier,
    InvalidSymbol(String),
    Alignment(String),
    ClassificationLimit { ty: TypeId, limit: usize },
}
impl From<types::Error> for Error {
    fn from(error: types::Error) -> Self {
        Self::Storage(error)
    }
}
impl From<TypeError> for Error {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<inkwell::builder::BuilderError> for Error {
    fn from(error: inkwell::builder::BuilderError) -> Self {
        Self::Build(error)
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "foreign storage lowering failed: {error}"),
            Self::Type(error) => write!(f, "foreign type identity failed: {error}"),
            Self::Build(error) => write!(f, "foreign instruction lowering failed: {error}"),
            Self::Target(error) => write!(f, "LLVM target selection failed: {error}"),
            Self::UnsupportedTarget(target) => {
                write!(f, "C ABI classification is unsupported for {target}")
            }
            Self::UnsupportedPlacedRecord(ty) => write!(
                f,
                "placed record {ty:?} has no proven foreign by-value alias classification"
            ),
            Self::UnsupportedType(ty) => {
                write!(f, "type {ty:?} has unsupported C ABI classification")
            }
            Self::InvalidSignature(ty) => {
                write!(f, "signature {ty:?} is not a context-free C procedure")
            }
            Self::MultipleForeignResults(ty) => {
                write!(f, "foreign signature {ty:?} has multiple language results")
            }
            Self::InvalidCarrier => {
                f.write_str("foreign ABI carrier does not match its checked storage value")
            }
            Self::InvalidSymbol(symbol) => write!(f, "foreign symbol {symbol:?} cannot be emitted"),
            Self::ClassificationLimit { ty, limit } => write!(
                f,
                "C ABI classification for {ty:?} exceeds {limit} unique type/offset nodes"
            ),
            Self::Alignment(error) => {
                write!(f, "foreign memory alignment construction failed: {error}")
            }
        }
    }
}
impl std::error::Error for Error {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Extension {
    Sign,
    Zero,
}
#[derive(Clone, Copy, Debug)]
pub struct Piece<'ctx> {
    pub ty: BasicTypeEnum<'ctx>,
    pub offset: u64,
}
#[derive(Clone, Debug)]
pub enum Value<'ctx> {
    Ignore,
    Direct {
        ty: BasicTypeEnum<'ctx>,
        extension: Option<Extension>,
    },
    Coerce {
        pieces: Vec<Piece<'ctx>>,
        carrier: Option<BasicTypeEnum<'ctx>>,
    },
    Indirect {
        storage: BasicTypeEnum<'ctx>,
        alignment: u32,
        by_value: bool,
    },
}
impl<'ctx> Value<'ctx> {
    fn parameter_types(&self, context: &'ctx Context) -> Vec<BasicMetadataTypeEnum<'ctx>> {
        match self {
            Self::Ignore => vec![],
            Self::Direct { ty, .. } => vec![(*ty).into()],
            Self::Coerce {
                carrier: Some(ty), ..
            } => vec![(*ty).into()],
            Self::Coerce {
                pieces,
                carrier: None,
            } => pieces.iter().map(|piece| piece.ty.into()).collect(),
            Self::Indirect { .. } => vec![context.ptr_type(AddressSpace::default()).into()],
        }
    }
    pub fn return_type(&self, context: &'ctx Context) -> Option<BasicTypeEnum<'ctx>> {
        match self {
            Self::Ignore | Self::Indirect { .. } => None,
            Self::Direct { ty, .. } => Some(*ty),
            Self::Coerce {
                carrier: Some(ty), ..
            } => Some(*ty),
            Self::Coerce {
                pieces,
                carrier: None,
            } => Some(match pieces.as_slice() {
                [piece] => piece.ty,
                pieces => context
                    .struct_type(
                        &pieces.iter().map(|piece| piece.ty).collect::<Vec<_>>(),
                        false,
                    )
                    .into(),
            }),
        }
    }
}

#[derive(Debug)]
pub struct Signature<'ctx> {
    pub llvm: FunctionType<'ctx>,
    pub parameters: Vec<Value<'ctx>>,
    pub result: Value<'ctx>,
    pub source_parameters: Vec<TypeId>,
    pub source_result: Option<TypeId>,
    attributes: Vec<(AttributeLoc, Attribute)>,
}
impl<'ctx> Signature<'ctx> {
    pub fn classify<'types>(
        context: &'ctx Context,
        types: &'types Types,
        lowerer: &mut TypeLowerer<'ctx, 'types>,
        platform: Platform,
        target: &TargetData,
        signature: TypeId,
        variadic: bool,
    ) -> Result<Self, Error> {
        if target.get_pointer_byte_size(None) != platform.pointer_bytes()
            || target.get_byte_ordering() != ByteOrdering::LittleEndian
        {
            return Err(Error::UnsupportedTarget(
                "C target pointer width or byte order does not match its platform ABI".into(),
            ));
        }
        let procedure = types.procedure_definition(signature)?;
        crate::cpp_methods::validate(types, signature, procedure, platform)?;
        if !procedure.convention.uses_c_abi() || procedure.context != ContextMode::None {
            return Err(Error::InvalidSignature(signature));
        }
        if variadic != matches!(procedure.variadic, jai_types::Variadic::C { .. }) {
            return Err(Error::InvalidSignature(signature));
        }
        if procedure.results.len() > 1 {
            return Err(Error::MultipleForeignResults(signature));
        }
        let mut classifier = Classifier {
            context,
            types,
            lowerer,
            platform,
            layouts: LayoutEngine::new(
                types,
                types::layout_policy(context, target).map_err(Error::from)?,
            ),
            integer_registers: 6,
            float_registers: 8,
            variadic,
        };
        let source_result = procedure.results.first().copied();
        let result = match source_result {
            Some(ty) => classifier.value(ty, true)?,
            None => Value::Ignore,
        };
        let mut parameters = vec![];
        let mut llvm_parameters = vec![];
        let mut attributes = vec![];
        if let Value::Indirect {
            storage, alignment, ..
        } = result
        {
            llvm_parameters.push(context.ptr_type(AddressSpace::default()).into());
            attributes.push((
                AttributeLoc::Param(0),
                type_attribute(context, "sret", storage),
            ));
            attributes.push((
                AttributeLoc::Param(0),
                enum_attribute(context, "align", u64::from(alignment)),
            ));
            attributes.push((
                AttributeLoc::Param(0),
                enum_attribute(context, "noalias", 0),
            ));
            if platform.is_sysv_x86_64() {
                classifier.integer_registers -= 1;
            }
        }
        if let Value::Direct {
            extension: Some(extension),
            ..
        } = result
        {
            attributes.push((
                AttributeLoc::Return,
                extension_attribute(context, extension),
            ));
        }
        for &ty in &procedure.parameters {
            let value = classifier.value(ty, false)?;
            let index = u32::try_from(llvm_parameters.len()).map_err(|_| Error::InvalidCarrier)?;
            if let Some(alignment) = classifier.stack_alignment(ty, &value)? {
                attributes.push((
                    AttributeLoc::Param(index),
                    enum_attribute(context, "alignstack", u64::from(alignment)),
                ));
            }
            match value {
                Value::Direct {
                    extension: Some(extension),
                    ..
                } => attributes.push((
                    AttributeLoc::Param(index),
                    extension_attribute(context, extension),
                )),
                Value::Indirect {
                    storage,
                    alignment,
                    by_value: true,
                } => {
                    attributes.push((
                        AttributeLoc::Param(index),
                        type_attribute(context, "byval", storage),
                    ));
                    attributes.push((
                        AttributeLoc::Param(index),
                        enum_attribute(context, "align", u64::from(alignment)),
                    ));
                }
                _ => {}
            }
            llvm_parameters.extend(value.parameter_types(context));
            parameters.push(value);
        }
        let llvm = match result.return_type(context) {
            Some(ty) => ty.fn_type(&llvm_parameters, variadic),
            None => context.void_type().fn_type(&llvm_parameters, variadic),
        };
        Ok(Self {
            llvm,
            parameters,
            result,
            source_parameters: procedure.parameters.to_vec(),
            source_result,
            attributes,
        })
    }
    pub fn attributes_on_function(&self, function: FunctionValue<'ctx>) {
        function.set_call_conventions(0);
        for &(location, attribute) in &self.attributes {
            function.add_attribute(location, attribute);
        }
    }
    pub fn attributes_on_call(&self, call: CallSiteValue<'ctx>) {
        call.set_call_convention(0);
        for &(location, attribute) in &self.attributes {
            call.add_attribute(location, attribute);
        }
    }
}
fn enum_attribute(context: &Context, name: &str, value: u64) -> Attribute {
    context.create_enum_attribute(Attribute::get_named_enum_kind_id(name), value)
}
fn type_attribute(context: &Context, name: &str, ty: BasicTypeEnum<'_>) -> Attribute {
    context.create_type_attribute(
        Attribute::get_named_enum_kind_id(name),
        ty.as_any_type_enum(),
    )
}
fn extension_attribute(context: &Context, extension: Extension) -> Attribute {
    enum_attribute(
        context,
        match extension {
            Extension::Sign => "signext",
            Extension::Zero => "zeroext",
        },
        0,
    )
}

struct Classifier<'ctx, 'types, 'lowerer> {
    context: &'ctx Context,
    types: &'types Types,
    lowerer: &'lowerer mut TypeLowerer<'ctx, 'types>,
    platform: Platform,
    layouts: LayoutEngine<'types>,
    integer_registers: u8,
    float_registers: u8,
    variadic: bool,
}
#[derive(Clone, Copy, Debug)]
struct Leaf<'ctx> {
    offset: u64,
    size: u64,
    ty: BasicTypeEnum<'ctx>,
    float: Option<FloatType>,
}
impl<'ctx> Classifier<'ctx, '_, '_> {
    fn layout(&mut self, ty: TypeId) -> Result<Layout, Error> {
        self.layouts
            .layout(ty)
            .cloned()
            .map_err(|error| Error::Storage(error.into()))
    }
    fn value(&mut self, mut ty: TypeId, result: bool) -> Result<Value<'ctx>, Error> {
        while let TypeKind::Distinct(distinct) = self.types.kind(ty)? {
            ty = self.types.distinct(*distinct)?.representation;
        }
        validate_storage_graph(self.types, ty)?;
        let storage = self.lowerer.basic(ty)?;
        let scalar = match self.types.kind(ty)? {
            TypeKind::Bool => Some((Some(Extension::Zero), false)),
            TypeKind::Integer(integer) => Some((
                if integer.bits() < 32 {
                    Some(if integer.signed() {
                        Extension::Sign
                    } else {
                        Extension::Zero
                    })
                } else {
                    None
                },
                false,
            )),
            TypeKind::Enum(id) => {
                let integer = self.types.enumeration(*id)?.representation;
                Some((
                    if integer.bits() < 32 {
                        Some(if integer.signed() {
                            Extension::Sign
                        } else {
                            Extension::Zero
                        })
                    } else {
                        None
                    },
                    false,
                ))
            }
            TypeKind::Float(_) => Some((None, true)),
            TypeKind::Type | TypeKind::Pointer(_) | TypeKind::Procedure(_) => Some((None, false)),
            _ => None,
        };
        if let Some((extension, float)) = scalar {
            if !result && self.platform.is_sysv_x86_64() {
                let registers = if float {
                    &mut self.float_registers
                } else {
                    &mut self.integer_registers
                };
                *registers = registers.saturating_sub(1);
            }
            return Ok(Value::Direct {
                ty: storage,
                extension: match self.platform {
                    Platform::LinuxArm64 | Platform::AndroidArm64 | Platform::WindowsArm64 => None,
                    Platform::WindowsX86_64 if !matches!(self.types.kind(ty)?, TypeKind::Bool) => {
                        None
                    }
                    _ => extension,
                },
            });
        }
        let layout = self.layout(ty)?;
        if layout.size == 0 {
            return Ok(Value::Ignore);
        }
        if self.platform == Platform::WindowsX86_64 {
            return self.windows_x86_64_aggregate(storage, &layout, result);
        }
        if self.platform.is_webassembly() {
            return self.webassembly_aggregate(ty, storage, &layout, result);
        }
        let platform = self.platform;
        let indirect = || Value::Indirect {
            storage,
            // SysV rounds argument stack slots up to eight-byte alignment,
            // even when a packed record's own alignment is smaller.
            alignment: if !result && platform.is_sysv_x86_64() {
                layout.alignment.max(8)
            } else {
                layout.alignment
            },
            by_value: !result && platform.is_sysv_x86_64(),
        };
        if self.platform.is_sysv_x86_64() && layout.size > 16 {
            return Ok(indirect());
        }
        if self.platform.is_arm64() && matches!(self.types.kind(ty)?, TypeKind::DynamicArray(_)) {
            return Ok(indirect());
        }
        // At most four f64 leaves can form an HFA. Larger Apple composites
        // are indirect, so never expand a large fixed array just to disprove HFA.
        if self.platform.is_arm64() && layout.size > 32 {
            return Ok(indirect());
        }
        let (leaves, unaligned) = self.leaves(ty, 0)?;
        if self.platform.is_arm64() {
            let homogeneous = homogeneous(&leaves, layout.size);
            if let Some((float, homogeneous_leaves)) = homogeneous
                .filter(|_| result || self.platform != Platform::WindowsArm64 || !self.variadic)
            {
                let element = match float {
                    FloatType::F32 => self.context.f32_type(),
                    FloatType::F64 => self.context.f64_type(),
                };
                return Ok(Value::Coerce {
                    pieces: homogeneous_leaves
                        .iter()
                        .map(|leaf| Piece {
                            ty: element.into(),
                            offset: leaf.offset,
                        })
                        .collect(),
                    carrier: Some(if result {
                        self.context
                            .struct_type(&vec![element.into(); homogeneous_leaves.len()], false)
                            .into()
                    } else {
                        element
                            .array_type(
                                u32::try_from(homogeneous_leaves.len())
                                    .map_err(|_| Error::InvalidCarrier)?,
                            )
                            .into()
                    }),
                });
            }
            if layout.size > 16 {
                return Ok(indirect());
            }
            let argument_alignment = if !result && self.platform.is_aapcs64() {
                self.unadjusted_aggregate_alignment(ty)?
            } else {
                layout.alignment
            };
            if argument_alignment >= 16 {
                // AAPCS64 advances a 16-byte-aligned composite to an even
                // register pair. One i128 carrier preserves that alignment.
                let integer = self.context.i128_type().into();
                return Ok(Value::Coerce {
                    pieces: vec![Piece {
                        ty: integer,
                        offset: 0,
                    }],
                    carrier: Some(integer),
                });
            }
            if result && layout.size <= 8 {
                let integer = integer_width(
                    self.context,
                    u32::try_from(layout.size * 8).map_err(|_| Error::InvalidCarrier)?,
                )?;
                return Ok(Value::Coerce {
                    pieces: vec![Piece {
                        ty: integer,
                        offset: 0,
                    }],
                    carrier: Some(integer),
                });
            }
            let count = layout.size.div_ceil(8);
            // Pointer-only aggregate arguments preserve provenance in their
            // carrier. Returns retain the platform's integer return carrier.
            let pointers = !result
                && leaves.iter().all(|leaf| {
                    leaf.ty.is_pointer_type() && leaf.size == 8 && leaf.offset.is_multiple_of(8)
                })
                && (0..count).all(|index| leaves.iter().any(|leaf| leaf.offset == index * 8));
            let element: BasicTypeEnum<'ctx> = if pointers {
                self.context.ptr_type(AddressSpace::default()).into()
            } else {
                self.context.i64_type().into()
            };
            let pieces = (0..count)
                .map(|index| Piece {
                    ty: element,
                    offset: index * 8,
                })
                .collect();
            let carrier = if count == 1 {
                element
            } else {
                element
                    .array_type(u32::try_from(count).map_err(|_| Error::InvalidCarrier)?)
                    .into()
            };
            return Ok(Value::Coerce {
                pieces,
                carrier: Some(carrier),
            });
        }
        if unaligned {
            return Ok(indirect());
        }
        let mut pieces = vec![];
        let mut integer_count = 0;
        let mut float_count = 0;
        for index in 0..layout.size.div_ceil(8) {
            let start = index * 8;
            let inside: Vec<_> = leaves
                .iter()
                .filter(|leaf| leaf.offset < start + 8 && leaf.offset + leaf.size > start)
                .collect();
            if inside.is_empty() {
                continue;
            }
            if inside.iter().any(|leaf| leaf.float.is_none()) {
                integer_count += 1;
                let natural = match inside.as_slice() {
                    leaves
                        if leaves.iter().all(|leaf| {
                            leaf.ty.is_pointer_type() && leaf.offset == start && leaf.size == 8
                        }) =>
                    {
                        Some(self.context.ptr_type(AddressSpace::default()).into())
                    }
                    [leaf] if leaf.offset == start && leaf.float.is_none() && leaf.size <= 8 => {
                        Some(if leaf.ty.is_pointer_type() {
                            leaf.ty
                        } else {
                            integer_width(
                                self.context,
                                u32::try_from(leaf.size * 8).map_err(|_| Error::InvalidCarrier)?,
                            )?
                        })
                    }
                    _ => None,
                };
                let ty = match natural {
                    Some(ty) => ty,
                    None => integer_width(
                        self.context,
                        u32::try_from((layout.size - start).min(8) * 8)
                            .map_err(|_| Error::InvalidCarrier)?,
                    )?,
                };
                pieces.push(Piece { ty, offset: start });
            } else {
                float_count += 1;
                let ty = if inside.iter().any(|leaf| leaf.float == Some(FloatType::F64)) {
                    self.context.f64_type().into()
                } else if inside.iter().any(|leaf| leaf.offset - start == 4) {
                    self.context.f32_type().vec_type(2).into()
                } else {
                    self.context.f32_type().into()
                };
                pieces.push(Piece { ty, offset: start });
            }
        }
        if !result {
            if integer_count > self.integer_registers || float_count > self.float_registers {
                return Ok(indirect());
            }
            self.integer_registers -= integer_count;
            self.float_registers -= float_count;
        }
        Ok(Value::Coerce {
            pieces,
            carrier: None,
        })
    }

    fn leaves(&mut self, root: TypeId, offset: u64) -> Result<(Vec<Leaf<'ctx>>, bool), Error> {
        let mut pending = vec![(root, offset)];
        let mut leaves = vec![];
        let mut visited = HashSet::from([(root, offset)]);
        let mut unaligned = false;
        while let Some((ty, offset)) = pending.pop() {
            let layout = self.layout(ty)?;
            if layout.size == 0 {
                continue;
            }
            // Test absolute offsets against the field type's natural layout,
            // not the containing record's reduced/packed field alignment.
            // Aligned fields inside a packed record can still use registers.
            unaligned |= !offset.is_multiple_of(u64::from(layout.alignment));
            match self.types.kind(ty)? {
                TypeKind::Integer(_)
                | TypeKind::Bool
                | TypeKind::Enum(_)
                | TypeKind::Type
                | TypeKind::Pointer(_)
                | TypeKind::Procedure(_)
                | TypeKind::Float(_) => {
                    let float = match self.types.kind(ty)? {
                        TypeKind::Float(float) => Some(*float),
                        _ => None,
                    };
                    leaves.push(Leaf {
                        offset,
                        size: layout.size,
                        ty: self.lowerer.basic(ty)?,
                        float,
                    });
                }
                TypeKind::Distinct(distinct) => {
                    schedule(
                        &mut pending,
                        &mut visited,
                        (self.types.distinct(*distinct)?.representation, offset),
                        root,
                    )?;
                }
                TypeKind::Record(_) | TypeKind::Any(_) => {
                    let definition = self.types.record_storage_definition(ty)?;
                    for (index, &field) in definition.fields.iter().enumerate().rev() {
                        schedule(
                            &mut pending,
                            &mut visited,
                            (
                                field,
                                offset
                                    + if definition.kind == RecordKind::Union {
                                        0
                                    } else {
                                        layout.field_offsets[index]
                                    },
                            ),
                            root,
                        )?;
                    }
                }
                TypeKind::FixedArray { element, count } => {
                    let stride = layout.array_stride.ok_or(Error::InvalidCarrier)?;
                    if stride == 0 {
                        continue;
                    }
                    for index in (0..*count).rev() {
                        schedule(
                            &mut pending,
                            &mut visited,
                            (*element, offset + stride * index),
                            root,
                        )?;
                    }
                }
                TypeKind::String | TypeKind::Slice(_) => {
                    leaves.push(Leaf {
                        offset,
                        size: 8,
                        ty: self.context.i64_type().into(),
                        float: None,
                    });
                    leaves.push(Leaf {
                        offset: offset + 8,
                        size: 8,
                        ty: self.context.ptr_type(AddressSpace::default()).into(),
                        float: None,
                    });
                }
                TypeKind::DynamicArray(_) | TypeKind::Void | TypeKind::Code => {
                    return Err(Error::UnsupportedType(ty));
                }
            }
        }
        Ok((leaves, unaligned))
    }
}
/// Bound the type graph before LLVM storage construction, including custom
/// record fields. Pointer and descriptor pointees remain opaque to the C ABI.
pub(crate) fn validate_storage_graph(types: &Types, root: TypeId) -> Result<(), Error> {
    let mut pending = vec![(root, 0)];
    let mut seen = HashSet::from([(root, 0)]);
    while let Some((ty, _)) = pending.pop() {
        match types.kind(ty)? {
            TypeKind::Record(_) | TypeKind::Any(_) => {
                let definition = types.record_storage_definition(ty)?;
                if definition
                    .layout
                    .field_placements
                    .iter()
                    .any(Option::is_some)
                {
                    return Err(Error::UnsupportedPlacedRecord(ty));
                }
                for &field in &definition.fields {
                    schedule(&mut pending, &mut seen, (field, 0), root)?;
                }
            }
            TypeKind::FixedArray { element, .. } => {
                schedule(&mut pending, &mut seen, (*element, 0), root)?
            }
            TypeKind::Distinct(id) => schedule(
                &mut pending,
                &mut seen,
                (types.distinct(*id)?.representation, 0),
                root,
            )?,
            _ => {}
        }
    }
    Ok(())
}

fn schedule(
    pending: &mut Vec<(TypeId, u64)>,
    visited: &mut HashSet<(TypeId, u64)>,
    node: (TypeId, u64),
    root: TypeId,
) -> Result<(), Error> {
    if visited.contains(&node) {
        return Ok(());
    }
    if visited.len() >= MAX_CLASSIFICATION_NODES {
        return Err(Error::ClassificationLimit {
            ty: root,
            limit: MAX_CLASSIFICATION_NODES,
        });
    }
    visited.insert(node);
    pending.push(node);
    Ok(())
}

fn homogeneous<'ctx>(leaves: &[Leaf<'ctx>], size: u64) -> Option<(FloatType, Vec<Leaf<'ctx>>)> {
    let mut unique = leaves.to_vec();
    unique.sort_by_key(|leaf| (leaf.offset, leaf.size));
    unique.dedup_by(|a, b| a.offset == b.offset && a.size == b.size && a.float == b.float);
    if unique.is_empty() || unique.len() > 4 {
        return None;
    }
    let float = unique[0].float?;
    let width = match float {
        FloatType::F32 => 4,
        FloatType::F64 => 8,
    };
    if size != width * u64::try_from(unique.len()).ok()? {
        return None;
    }
    unique
        .iter()
        .enumerate()
        .all(|(index, leaf)| {
            leaf.float == Some(float) && leaf.offset == u64::try_from(index).unwrap() * width
        })
        .then_some((float, unique))
}
pub(crate) fn integer_width(context: &Context, bits: u32) -> Result<BasicTypeEnum<'_>, Error> {
    let bits = NonZeroU32::new(bits).ok_or(Error::InvalidCarrier)?;
    context
        .custom_width_int_type(bits)
        .map(Into::into)
        .map_err(|_| Error::InvalidCarrier)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{
        CallingConvention, IntegerType, ProcedureType, ScalarType, TypeRegistry, Variadic,
    };
    #[test]
    fn classification_budget_limits_distinct_overlapping_union_members() {
        let context = Context::create();
        let target = NativeTarget::new().unwrap();
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let mut fields = Vec::with_capacity(MAX_CLASSIFICATION_NODES);
        for _ in 0..MAX_CLASSIFICATION_NODES {
            let field = registry.reserve_record(RecordKind::Struct);
            registry.define_record(field, [byte]).unwrap();
            fields.push(field);
        }
        let union = registry.reserve_record(RecordKind::Union);
        registry.define_record(union, fields).unwrap();
        let procedure = registry
            .procedure(ProcedureType {
                parameters: vec![union].into_boxed_slice(),
                results: Box::new([]),
                convention: CallingConvention::C,
                context: ContextMode::None,
                variadic: Variadic::None,
            })
            .unwrap();
        let types = registry.freeze().unwrap();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
        assert!(
            matches!(Signature::classify(&context,&types,&mut lowerer,target.c_platform().unwrap(),&target.data,procedure,false),Err(Error::ClassificationLimit {ty,limit}) if ty==union && limit==MAX_CLASSIFICATION_NODES)
        );
    }
    #[test]
    fn custom_record_storage_follows_by_value_edges_and_pointer_edges_stay_opaque() {
        let context = Context::create();
        let target = NativeTarget::new().unwrap();
        let mut registry = TypeRegistry::new();
        let byte = registry.scalar(ScalarType::Int(IntegerType::U8));
        let custom = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record_with_layout(
                custom,
                [byte],
                jai_types::RecordLayout {
                    packed: true,
                    ..Default::default()
                },
            )
            .unwrap();
        let nested = registry.reserve_record(RecordKind::Struct);
        registry.define_record(nested, [custom]).unwrap();
        let array = registry.fixed_array(nested, 1).unwrap();
        let pointer = registry.pointer(custom).unwrap();
        let mut signatures = vec![];
        for ty in [custom, nested, array, pointer] {
            signatures.push(
                registry
                    .procedure(ProcedureType {
                        parameters: vec![ty].into_boxed_slice(),
                        results: Box::new([]),
                        convention: CallingConvention::C,
                        context: ContextMode::None,
                        variadic: Variadic::None,
                    })
                    .unwrap(),
            );
        }
        let types = registry.freeze().unwrap();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
        for &signature in &signatures {
            assert!(
                Signature::classify(
                    &context,
                    &types,
                    &mut lowerer,
                    target.c_platform().unwrap(),
                    &target.data,
                    signature,
                    false
                )
                .is_ok()
            );
        }
    }
    #[test]
    fn placed_records_are_rejected_through_by_value_edges_but_pointer_edges_stay_opaque() {
        let context = Context::create();
        let target = NativeTarget::new().unwrap();
        let mut registry = TypeRegistry::new();
        let word = registry.scalar(ScalarType::Int(IntegerType::U32));
        let placed = registry.reserve_record(RecordKind::Struct);
        registry
            .define_record_with_placements(
                placed,
                [word, word],
                Default::default(),
                [None, Some(0)],
            )
            .unwrap();
        let nested = registry.reserve_record(RecordKind::Struct);
        registry.define_record(nested, [placed]).unwrap();
        let array = registry.fixed_array(nested, 2).unwrap();
        let distinct = registry.reserve_distinct(jai_types::DistinctKind::Distinct);
        registry.define_distinct(distinct, array).unwrap();
        let pointer = registry.pointer(placed).unwrap();
        let mut signatures = vec![];
        for ty in [placed, nested, array, distinct, pointer] {
            signatures.push(
                registry
                    .procedure(ProcedureType {
                        parameters: vec![ty].into_boxed_slice(),
                        results: Box::new([]),
                        convention: CallingConvention::C,
                        context: ContextMode::None,
                        variadic: Variadic::None,
                    })
                    .unwrap(),
            );
        }
        let types = registry.freeze().unwrap();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
        for &signature in &signatures[..4] {
            assert!(
                matches!(Signature::classify(&context, &types, &mut lowerer, target.c_platform().unwrap(), &target.data, signature, false), Err(Error::UnsupportedPlacedRecord(ty)) if ty == placed)
            );
        }
        assert!(
            Signature::classify(
                &context,
                &types,
                &mut lowerer,
                target.c_platform().unwrap(),
                &target.data,
                signatures[4],
                false
            )
            .is_ok()
        );
    }
}
