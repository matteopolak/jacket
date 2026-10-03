//! Selected-target wrappers around the independently authored pool ledger.
use super::*;
#[path = "native_pools/ledger.rs"]
mod ledger;
#[cfg(test)]
#[path = "native_pools/tests.rs"]
mod tests;

pub(super) fn prepare<'ctx>(
    module: &Module<'ctx>,
    context: &'ctx Context,
    target: &target::NativeTarget,
) -> Result<(), Error> {
    if module.get_function("jai.pool.get").is_some() {
        let generated = ["get", "reset", "release"].into_iter().all(|name| {
            module
                .get_function(&format!("jai.pool.{name}"))
                .is_some_and(|function| {
                    function.count_basic_blocks() > 0
                        && function.get_linkage() == inkwell::module::Linkage::LinkOnceODR
                        && function.as_global_value().get_visibility()
                            == inkwell::GlobalVisibility::Hidden
                })
        }) && ["head", "lock"].into_iter().all(|name| {
            module
                .get_global(&format!("jai.pool.{name}"))
                .is_some_and(|global| {
                    global.get_initializer().is_some()
                        && global.get_linkage() == inkwell::module::Linkage::LinkOnceODR
                        && global.get_visibility() == inkwell::GlobalVisibility::Hidden
                })
        });
        return if generated {
            Ok(())
        } else {
            Err(symbol_conflict())
        };
    }
    let mut function = module.get_first_function();
    while let Some(existing) = function {
        if existing.get_name().to_bytes().starts_with(b"jai.pool.") {
            return Err(symbol_conflict());
        }
        function = existing.get_next_function();
    }
    if ["jai.pool.head", "jai.pool.lock"]
        .into_iter()
        .any(|name| module.get_global(name).is_some())
    {
        return Err(symbol_conflict());
    }
    target.c_platform()?;
    ledger::define(context, module, target)
}

fn symbol_conflict() -> Error {
    Error::RuntimeIntrinsic(jai_ir::RuntimeIntrinsicError::Signature(
        "reserved native pool runtime symbol conflicts with a source declaration",
    ))
}

pub(super) fn emit<'ctx>(
    module: &Module<'ctx>,
    lowerer: &mut types::TypeLowerer<'ctx, '_>,
    target: &target::NativeTarget,
    builder: &Builder<'ctx>,
    operation: RuntimeIntrinsic,
    arguments: &[BasicValueEnum<'ctx>],
) -> Result<(), Error> {
    let (pool, flat, name) = match operation {
        RuntimeIntrinsic::PoolGet { pool } => (pool, false, "jai.pool.get"),
        RuntimeIntrinsic::FlatPoolGet { pool } => (pool, true, "jai.pool.get"),
        RuntimeIntrinsic::PoolReset { pool } => (pool, false, "jai.pool.reset"),
        RuntimeIntrinsic::FlatPoolReset { pool } => (pool, true, "jai.pool.reset"),
        RuntimeIntrinsic::PoolRelease { pool } => (pool, false, "jai.pool.release"),
        RuntimeIntrinsic::FlatPoolFinish { pool } => (pool, true, "jai.pool.release"),
        _ => return Err(Error::Invariant),
    };
    let context = lowerer.context();
    let policy = types::layout_policy(context, &target.data)?;
    let mut layouts = jai_types::LayoutEngine::new(lowerer.registry(), policy);
    let offsets = layouts
        .layout(pool)
        .map_err(types::Error::from)?
        .field_offsets
        .to_vec();
    let owner = arguments[0].into_pointer_value();
    let token_name = format!("jai.pool.nominal.{}", pool.index());
    let nominal = module
        .get_global(&token_name)
        .unwrap_or_else(|| {
            let token = module.add_global(context.i8_type(), None, &token_name);
            token.set_initializer(&context.i8_type().const_zero());
            token.set_constant(true);
            token.set_linkage(inkwell::module::Linkage::Private);
            token
        })
        .as_pointer_value();
    let integer = context.i64_type();
    let field = |index: usize| {
        jai_llvm::gep(
            builder,
            context.i8_type().into(),
            owner,
            &[integer.const_int(offsets[index], false)],
            "pool.field",
        )
        .map_err(Error::from)
    };
    let capacity = field(0)?;
    let left = field(1)?;
    let block = field(2)?;
    let position = field(3)?;
    let alignment = if flat {
        field(4)?
    } else {
        owner.get_type().const_null()
    };
    let scalar_alignment = layouts
        .layout(
            lowerer
                .registry()
                .scalar(jai_types::ScalarType::Int(IntegerType::S64)),
        )
        .map_err(types::Error::from)?
        .alignment;
    let default_alignment = u64::from(policy.pointer().alignment.max(scalar_alignment));
    let mut parameters: Vec<BasicMetadataValueEnum<'ctx>> = vec![
        owner.into(),
        nominal.into(),
        context.bool_type().const_int(u64::from(flat), false).into(),
    ];
    match name {
        "jai.pool.get" => {
            parameters.extend::<[BasicMetadataValueEnum<'ctx>; 8]>([
                arguments[1].into(),
                integer
                    .const_int(if flat { 0 } else { policy.pointer().size }, false)
                    .into(),
                integer.const_int(default_alignment, false).into(),
                capacity.into(),
                left.into(),
                block.into(),
                position.into(),
                alignment.into(),
            ]);
        }
        "jai.pool.reset" => {
            let overwrite = if flat {
                arguments[1].into_int_value()
            } else {
                context.bool_type().const_zero()
            };
            parameters.extend::<[BasicMetadataValueEnum<'ctx>; 7]>([
                overwrite.into(),
                integer
                    .const_int(if flat { 0 } else { policy.pointer().size }, false)
                    .into(),
                integer.const_int(default_alignment, false).into(),
                left.into(),
                block.into(),
                position.into(),
                alignment.into(),
            ]);
        }
        _ => parameters.extend::<[BasicMetadataValueEnum<'ctx>; 3]>([
            left.into(),
            block.into(),
            position.into(),
        ]),
    }
    let call = builder.build_call(
        module.get_function(name).ok_or(Error::Invariant)?,
        &parameters,
        "",
    )?;
    if name == "jai.pool.get" {
        let result = call.try_as_basic_value().basic().ok_or(Error::Invariant)?;
        builder.build_return(Some(&result))?;
    } else {
        builder.build_return(None)?;
    }
    Ok(())
}
