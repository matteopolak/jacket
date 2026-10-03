//! C declarations and value-preserving argument/result marshaling through LLVM.
use crate::{
    abi::{self, Error, Signature, Value},
    types::TypeLowerer,
    unions,
};
use inkwell::{
    builder::Builder,
    context::Context,
    module::Module,
    targets::TargetData,
    types::BasicTypeEnum,
    values::{
        BasicMetadataValueEnum, BasicValue, BasicValueEnum, CallSiteValue, FunctionValue,
        PointerValue,
    },
};
use jai_types::{FloatType, IntegerType, TypeId, TypeKind, Types, Variadic};

pub struct Function<'ctx> {
    pub value: FunctionValue<'ctx>,
    pub signature: Signature<'ctx>,
}
pub struct CallResult<'ctx> {
    pub site: CallSiteValue<'ctx>,
    pub value: Option<BasicValueEnum<'ctx>>,
}
#[derive(Clone, Copy)]
pub enum Callee<'ctx> {
    Direct(FunctionValue<'ctx>),
    Indirect(PointerValue<'ctx>),
}

pub fn declare<'ctx, 'types>(
    module: &Module<'ctx>,
    types: &'types Types,
    lowerer: &mut TypeLowerer<'ctx, 'types>,
    platform: abi::Platform,
    target: &TargetData,
    signature: TypeId,
    symbol: &str,
) -> Result<Function<'ctx>, Error> {
    let context = lowerer.context();
    if symbol.is_empty() || symbol.contains('\0') || symbol.starts_with("jai.pool.") {
        return Err(Error::InvalidSymbol(symbol.into()));
    }
    let procedure = types.procedure_definition(signature)?;
    let variadic = match procedure.variadic {
        Variadic::None => false,
        Variadic::C { .. } => true,
        Variadic::Jai { .. } => return Err(Error::InvalidSignature(signature)),
    };
    let signature = Signature::classify(
        context, types, lowerer, platform, target, signature, variadic,
    )?;
    let value = if let Some(existing) = module.get_function(symbol) {
        if existing.get_type() != signature.llvm {
            return Err(Error::InvalidSymbol(symbol.into()));
        }
        existing
    } else {
        module.add_function(symbol, signature.llvm, None)
    };
    signature.attributes_on_function(value);
    Ok(Function { value, signature })
}

impl<'ctx> Function<'ctx> {
    pub fn call<'types>(
        &self,
        builder: &Builder<'ctx>,
        types: &'types Types,
        lowerer: &mut TypeLowerer<'ctx, 'types>,
        target: &TargetData,
        arguments: &[(TypeId, BasicValueEnum<'ctx>)],
    ) -> Result<CallResult<'ctx>, Error> {
        call(
            builder,
            types,
            lowerer,
            target,
            &self.signature,
            Callee::Direct(self.value),
            arguments,
        )
    }
}

pub fn call<'ctx, 'types>(
    builder: &Builder<'ctx>,
    types: &'types Types,
    lowerer: &mut TypeLowerer<'ctx, 'types>,
    target: &TargetData,
    signature: &Signature<'ctx>,
    callee: Callee<'ctx>,
    arguments: &[(TypeId, BasicValueEnum<'ctx>)],
) -> Result<CallResult<'ctx>, Error> {
    let context = lowerer.context();
    if arguments.len() < signature.source_parameters.len()
        || (!signature.llvm.is_var_arg() && arguments.len() != signature.source_parameters.len())
    {
        return Err(Error::InvalidCarrier);
    }
    let mut llvm_arguments: Vec<BasicMetadataValueEnum<'ctx>> = vec![];
    let return_pointer = match signature.result {
        Value::Indirect {
            storage, alignment, ..
        } => {
            let pointer = unions::entry_alloca(context, builder, storage, "foreign.result")?;
            set_pointer_alignment(pointer, alignment)?;
            llvm_arguments.push(pointer.into());
            Some(pointer)
        }
        _ => None,
    };
    for (index, (&source_ty, abi)) in signature
        .source_parameters
        .iter()
        .zip(&signature.parameters)
        .enumerate()
    {
        let (argument_ty, argument) = arguments[index];
        if argument_ty != source_ty || argument.get_type() != lowerer.basic(source_ty)? {
            return Err(Error::InvalidCarrier);
        }
        llvm_arguments.extend(marshal_argument(context, builder, target, argument, abi)?);
    }
    for &(source_ty, argument) in &arguments[signature.source_parameters.len()..] {
        abi::validate_storage_graph(types, source_ty)?;
        if argument.get_type() != lowerer.basic(source_ty)? {
            return Err(Error::InvalidCarrier);
        }
        llvm_arguments.push(promote(context, builder, types, source_ty, argument)?.into());
    }
    let name = if signature.llvm.get_return_type().is_some() {
        "foreign.call"
    } else {
        ""
    };
    let site = match callee {
        Callee::Direct(function) => builder.build_call(function, &llvm_arguments, name)?,
        Callee::Indirect(pointer) => {
            builder.build_indirect_call(signature.llvm, pointer, &llvm_arguments, name)?
        }
    };
    signature.attributes_on_call(site);
    let result = match signature.source_result {
        None => None,
        Some(source_ty) => {
            let storage = lowerer.basic(source_ty)?;
            Some(match (&signature.result, return_pointer) {
                (Value::Ignore, _) => storage.const_zero(),
                (Value::Indirect { .. }, Some(pointer)) => {
                    builder.build_load(storage, pointer, "foreign.result")?
                }
                (Value::Direct { .. }, _) => site
                    .try_as_basic_value()
                    .basic()
                    .ok_or(Error::InvalidCarrier)?,
                (Value::Coerce { pieces, carrier }, _) => {
                    let returned = site
                        .try_as_basic_value()
                        .basic()
                        .ok_or(Error::InvalidCarrier)?;
                    let size = target
                        .get_abi_size(&storage)
                        .max(target.get_abi_size(&returned.get_type()));
                    let buffer = buffer(context, builder, size)?;
                    if carrier.is_some() || pieces.len() == 1 {
                        store_unaligned(builder, buffer, returned)?;
                    } else {
                        let returned = returned.into_struct_value();
                        for (index, piece) in pieces.iter().enumerate() {
                            let value = builder.build_extract_value(
                                returned,
                                u32::try_from(index).map_err(|_| Error::InvalidCarrier)?,
                                "foreign.result.piece",
                            )?;
                            let pointer = byte_offset(context, builder, buffer, piece.offset)?;
                            store_unaligned(builder, pointer, value)?;
                        }
                    }
                    load_unaligned(builder, storage, buffer, "foreign.result")?
                }
                _ => return Err(Error::InvalidCarrier),
            })
        }
    };
    Ok(CallResult {
        site,
        value: result,
    })
}

fn marshal_argument<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    target: &TargetData,
    argument: BasicValueEnum<'ctx>,
    abi: &Value<'ctx>,
) -> Result<Vec<BasicMetadataValueEnum<'ctx>>, Error> {
    Ok(match abi {
        Value::Ignore => vec![],
        Value::Direct { ty, .. } => {
            if argument.get_type() != *ty {
                return Err(Error::InvalidCarrier);
            }
            vec![argument.into()]
        }
        Value::Indirect {
            storage, alignment, ..
        } => {
            let pointer = unions::entry_alloca(context, builder, *storage, "foreign.argument")?;
            set_pointer_alignment(pointer, *alignment)?;
            builder.build_store(pointer, argument)?;
            vec![pointer.into()]
        }
        Value::Coerce { pieces, carrier } => {
            let carrier_size = match carrier {
                Some(carrier) => target.get_abi_size(carrier),
                None => pieces
                    .iter()
                    .map(|piece| piece.offset + target.get_abi_size(&piece.ty))
                    .max()
                    .unwrap_or(0),
            };
            let size = target.get_abi_size(&argument.get_type()).max(carrier_size);
            let buffer = buffer(context, builder, size)?;
            store_unaligned(builder, buffer, argument)?;
            match carrier {
                Some(carrier) => {
                    vec![load_unaligned(builder, *carrier, buffer, "foreign.argument")?.into()]
                }
                None => pieces
                    .iter()
                    .map(|piece| {
                        let pointer = byte_offset(context, builder, buffer, piece.offset)?;
                        Ok(
                            load_unaligned(builder, piece.ty, pointer, "foreign.argument.piece")?
                                .into(),
                        )
                    })
                    .collect::<Result<Vec<_>, Error>>()?,
            }
        }
    })
}

/// The packed prefix supplies a constant byte offset through the safe struct-GEP
/// API. Callers bound offsets by the allocated ABI buffer before invoking this.
pub(crate) fn byte_offset<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    pointer: PointerValue<'ctx>,
    offset: u64,
) -> Result<PointerValue<'ctx>, Error> {
    if offset == 0 {
        return Ok(pointer);
    }
    let prefix = context.struct_type(
        &[
            context
                .i8_type()
                .array_type(u32::try_from(offset).map_err(|_| Error::InvalidCarrier)?)
                .into(),
            context.i8_type().into(),
        ],
        true,
    );
    Ok(builder.build_struct_gep(prefix, pointer, 1, "foreign.byte.offset")?)
}
fn buffer<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    size: u64,
) -> Result<PointerValue<'ctx>, Error> {
    let ty = context
        .i8_type()
        .array_type(u32::try_from(size).map_err(|_| Error::InvalidCarrier)?);
    let pointer = unions::entry_alloca(context, builder, ty.into(), "foreign.coercion")?;
    builder.build_store(pointer, ty.const_zero())?;
    Ok(pointer)
}
fn set_pointer_alignment(pointer: PointerValue<'_>, alignment: u32) -> Result<(), Error> {
    pointer
        .as_instruction_value()
        .ok_or(Error::InvalidCarrier)?
        .set_alignment(alignment)
        .map_err(|error| Error::Alignment(error.to_string()))
}
fn store_unaligned(
    builder: &Builder<'_>,
    pointer: PointerValue<'_>,
    value: BasicValueEnum<'_>,
) -> Result<(), Error> {
    builder
        .build_store(pointer, value)?
        .set_alignment(1)
        .map_err(|error| Error::Alignment(error.to_string()))
}
fn load_unaligned<'ctx>(
    builder: &Builder<'ctx>,
    ty: BasicTypeEnum<'ctx>,
    pointer: PointerValue<'ctx>,
    name: &str,
) -> Result<BasicValueEnum<'ctx>, Error> {
    let value = builder.build_load(ty, pointer, name)?;
    value
        .as_instruction_value()
        .ok_or(Error::InvalidCarrier)?
        .set_alignment(1)
        .map_err(|error| Error::Alignment(error.to_string()))?;
    Ok(value)
}
fn promote<'ctx>(
    context: &'ctx Context,
    builder: &Builder<'ctx>,
    types: &Types,
    mut ty: TypeId,
    argument: BasicValueEnum<'ctx>,
) -> Result<BasicValueEnum<'ctx>, Error> {
    while let TypeKind::Distinct(distinct) = types.kind(ty)? {
        ty = types.distinct(*distinct)?.representation;
    }
    let integer = match types.kind(ty)? {
        TypeKind::Bool => {
            return Ok(builder
                .build_int_z_extend(
                    argument.into_int_value(),
                    context.i32_type(),
                    "foreign.promote.bool",
                )?
                .into());
        }
        TypeKind::Integer(integer) => Some(*integer),
        TypeKind::Enum(enumeration) => Some(types.enumeration(*enumeration)?.representation),
        TypeKind::Float(FloatType::F32) => {
            return Ok(builder
                .build_float_ext(
                    argument.into_float_value(),
                    context.f64_type(),
                    "foreign.promote.float",
                )?
                .into());
        }
        TypeKind::Float(FloatType::F64) | TypeKind::Pointer(_) | TypeKind::Procedure(_) => {
            return Ok(argument);
        }
        _ => return Err(Error::UnsupportedType(ty)),
    };
    match integer {
        Some(integer) if integer.bits() < 32 => Ok(if integer.signed() {
            builder.build_int_s_extend(
                argument.into_int_value(),
                context.i32_type(),
                "foreign.promote.int",
            )?
        } else {
            builder.build_int_z_extend(
                argument.into_int_value(),
                context.i32_type(),
                "foreign.promote.int",
            )?
        }
        .into()),
        Some(IntegerType::S32 | IntegerType::U32 | IntegerType::S64 | IntegerType::U64) => {
            Ok(argument)
        }
        _ => Err(Error::InvalidCarrier),
    }
}

/// Recover each checked source parameter from the ABI's physical carriers.
pub fn parameters<'ctx, 'types>(
    builder: &Builder<'ctx>,
    types: &'types Types,
    lowerer: &mut TypeLowerer<'ctx, 'types>,
    target: &TargetData,
    signature: &Signature<'ctx>,
    function: FunctionValue<'ctx>,
) -> Result<Vec<BasicValueEnum<'ctx>>, Error> {
    let context = lowerer.context();
    let mut index = u32::from(matches!(signature.result, Value::Indirect { .. }));
    let mut values = Vec::with_capacity(signature.parameters.len());
    for (&source, abi) in signature
        .source_parameters
        .iter()
        .zip(&signature.parameters)
    {
        types.kind(source)?;
        let storage = lowerer.basic(source)?;
        let value = match abi {
            Value::Ignore => storage.const_zero(),
            Value::Direct { .. } => {
                let value = function.get_nth_param(index).ok_or(Error::InvalidCarrier)?;
                index += 1;
                value
            }
            Value::Indirect { .. } => {
                let pointer = function
                    .get_nth_param(index)
                    .ok_or(Error::InvalidCarrier)?
                    .into_pointer_value();
                index += 1;
                builder.build_load(storage, pointer, "foreign.parameter")?
            }
            Value::Coerce { pieces, carrier } => {
                let carrier_size = match carrier {
                    Some(ty) => target.get_abi_size(ty),
                    None => pieces
                        .iter()
                        .map(|piece| piece.offset + target.get_abi_size(&piece.ty))
                        .max()
                        .unwrap_or(0),
                };
                let temporary = buffer(
                    context,
                    builder,
                    target.get_abi_size(&storage).max(carrier_size),
                )?;
                if carrier.is_some() {
                    let value = function.get_nth_param(index).ok_or(Error::InvalidCarrier)?;
                    index += 1;
                    store_unaligned(builder, temporary, value)?;
                } else {
                    for piece in pieces {
                        let value = function.get_nth_param(index).ok_or(Error::InvalidCarrier)?;
                        index += 1;
                        let pointer = byte_offset(context, builder, temporary, piece.offset)?;
                        store_unaligned(builder, pointer, value)?;
                    }
                }
                load_unaligned(builder, storage, temporary, "foreign.parameter")?
            }
        };
        if value.get_type() != storage {
            return Err(Error::InvalidCarrier);
        }
        values.push(value);
    }
    Ok(values)
}

/// Return an already captured source value through a C definition's ABI.
pub fn return_value<'ctx>(
    builder: &Builder<'ctx>,
    context: &'ctx Context,
    target: &TargetData,
    signature: &Signature<'ctx>,
    function: FunctionValue<'ctx>,
    value: Option<BasicValueEnum<'ctx>>,
) -> Result<(), Error> {
    if signature.source_result.is_none() {
        if value.is_some() {
            return Err(Error::InvalidCarrier);
        }
        builder.build_return(None)?;
        return Ok(());
    }
    let value = value.ok_or(Error::InvalidCarrier)?;
    match &signature.result {
        Value::Ignore => {
            builder.build_return(None)?;
        }
        Value::Indirect { .. } => {
            let pointer = function
                .get_first_param()
                .ok_or(Error::InvalidCarrier)?
                .into_pointer_value();
            builder.build_store(pointer, value)?;
            builder.build_return(None)?;
        }
        Value::Direct { .. } => {
            builder.build_return(Some(&value))?;
        }
        Value::Coerce { carrier, pieces } => {
            let physical = marshal_argument(context, builder, target, value, &signature.result)?;
            let mut values = physical
                .into_iter()
                .map(|value| match value {
                    BasicMetadataValueEnum::IntValue(value) => Ok(value.into()),
                    BasicMetadataValueEnum::FloatValue(value) => Ok(value.into()),
                    BasicMetadataValueEnum::PointerValue(value) => Ok(value.into()),
                    BasicMetadataValueEnum::StructValue(value) => Ok(value.into()),
                    BasicMetadataValueEnum::ArrayValue(value) => Ok(value.into()),
                    BasicMetadataValueEnum::VectorValue(value) => Ok(value.into()),
                    _ => Err(Error::InvalidCarrier),
                })
                .collect::<Result<Vec<BasicValueEnum<'ctx>>, Error>>()?;
            let result = if carrier.is_some() || pieces.len() == 1 {
                values.pop().ok_or(Error::InvalidCarrier)?
            } else {
                let structure = signature
                    .result
                    .return_type(context)
                    .ok_or(Error::InvalidCarrier)?
                    .into_struct_type();
                let mut result = structure.const_zero();
                for (index, value) in values.into_iter().enumerate() {
                    result = builder
                        .build_insert_value(
                            result,
                            value,
                            u32::try_from(index).map_err(|_| Error::InvalidCarrier)?,
                            "foreign.return",
                        )?
                        .into_struct_value();
                }
                result.into()
            };
            builder.build_return(Some(&result))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod reserved_symbols {
    use super::*;
    #[test]
    fn source_foreign_symbols_cannot_declare_or_adopt_private_pool_helpers() {
        let context = inkwell::context::Context::create();
        let module = context.create_module("own.private.runtime.symbols");
        let target = crate::target::NativeTarget::new().unwrap();
        let mut registry = jai_types::TypeRegistry::new();
        let signature = registry
            .procedure(jai_types::ProcedureType {
                parameters: Box::new([]),
                results: Box::new([]),
                convention: jai_types::CallingConvention::C,
                context: jai_types::ContextMode::None,
                variadic: jai_types::Variadic::None,
            })
            .unwrap();
        let types = registry.freeze().unwrap();
        let mut lowerer = TypeLowerer::with_target(&context, &types, &target.data);
        let name = "jai.pool.alloc";
        for preexisting in [false, true] {
            if preexisting {
                module.add_function(name, context.void_type().fn_type(&[], false), None);
            }
            assert!(matches!(declare(&module, &types, &mut lowerer,
                target.c_platform().unwrap(), &target.data, signature, name), Err(Error::InvalidSymbol(symbol)) if symbol == name));
            assert_eq!(module.get_function(name).is_some(), preexisting);
        }
        assert!(jai_ir::NativeSymbol::new(name).is_err());
    }
}
