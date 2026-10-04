use super::*;
use jai_ir::{ConstantKind, ConstantValue, GlobalInitializer, StaticDataBuilder, StaticDataLimits};
use jai_types::{Integer, IntegerType, ScalarType, TypeRegistry};
use std::path::Path;

#[test]
fn parsed_foreign_alias_cannot_claim_an_immutable_static_object_symbol() {
    let mut storage = StaticDataBuilder::new();
    let temporary = TypeRegistry::new();
    let candidate = storage
        .reserve(
            temporary.scalar(ScalarType::Int(IntegerType::U64)),
            &temporary,
        )
        .unwrap();
    let symbol = format!(
        "jai.static.{}.{}",
        candidate.arena_identity(),
        candidate.index()
    );
    // Preserve the arena while reserving the actual object below in the source
    // compilation's canonical type registry, rather than rebinding TypeIds.
    storage.discard_unpublished();
    let root = Path::new("/jai-authored-static-symbol/main.jai");
    let mut sources = jai_modules::SourceOverlay::new();
    sources.insert(root, format!(
        "libc::#system_library \"c\"; forged:u64 #elsewhere libc \"{symbol}\"; main::()->int{{return 42;}}"
    ).into_bytes()).unwrap();
    let graph =
        jai_modules::ModuleGraph::load_with_provider(root, Default::default(), &sources).unwrap();
    let target = target::NativeTarget::new().unwrap();
    let program = jai_sema::resolve_graph_with_options(
        &graph,
        &jai_sema::ResolveOptions {
            layout: Some(target.layout_policy().unwrap()),
            ..Default::default()
        },
        &mut jai_vm::NoEffects,
    )
    .unwrap();
    let foreign = program
        .globals()
        .iter()
        .find_map(|global| {
            if let GlobalInitializer::External(data) = global.initializer() {
                Some(data)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(foreign.symbol(), symbol);
    assert!(matches!(
        foreign.source(),
        jai_ir::ExternalDataSource::Library(_)
    ));
    let object = storage.reserve(foreign.ty(), program.types()).unwrap();
    assert_eq!(
        format!("jai.static.{}.{}", object.arena_identity(), object.index()),
        symbol
    );
    storage
        .define(
            object,
            StaticValue::constant(ConstantValue {
                ty: foreign.ty(),
                kind: ConstantKind::Int(Integer::checked(IntegerType::U64, 37).unwrap()),
            }),
        )
        .unwrap();
    let data = storage
        .publish(program.types(), StaticDataLimits::default())
        .unwrap();
    let context = Context::create();
    let module = lower_for_target(&context, &program, &target).unwrap();
    let declared = module.get_global(&symbol).unwrap();
    assert!(declared.get_initializer().is_none());
    let mut lowerer = types::TypeLowerer::with_target(&context, program.types(), &target.data);
    let functions = HashMap::new();
    let signatures = HashMap::new();
    {
        let mut emitter = StaticEmitter {
            types: program.types(),
            lowerer: &mut lowerer,
            context: &context,
            module: &module,
            functions: &functions,
            signatures: &signatures,
        };
        assert!(matches!(
            reserve(&mut emitter, &data),
            Err(Error::Invariant)
        ));
    }
    assert!(
        declared.get_initializer().is_none(),
        "foreign data acquired owned storage"
    );
    module.verify().unwrap();

    // LLVM shares one symbol namespace for functions and data. A conflict
    // must not create silently renamed globals on successive reservations.
    let function_module = context.create_module("authored_function_static_collision");
    let function =
        function_module.add_function(&symbol, context.void_type().fn_type(&[], false), None);
    {
        let mut emitter = StaticEmitter {
            types: program.types(),
            lowerer: &mut lowerer,
            context: &context,
            module: &function_module,
            functions: &functions,
            signatures: &signatures,
        };
        for _ in 0..2 {
            assert!(matches!(
                reserve(&mut emitter, &data),
                Err(Error::Invariant)
            ));
        }
    }
    assert_eq!(function_module.get_globals().count(), 0);
    assert_eq!(function_module.get_function(&symbol), Some(function));
    function_module.verify().unwrap();

    // A genuine reservation remains reusable with the selected ABI alignment.
    let owned_module = context.create_module("authored_owned_static");
    let mut emitter = StaticEmitter {
        types: program.types(),
        lowerer: &mut lowerer,
        context: &context,
        module: &owned_module,
        functions: &functions,
        signatures: &signatures,
    };
    let globals = reserve(&mut emitter, &data).unwrap();
    let owned = globals[&object];
    let value = constant(
        &mut emitter,
        &data,
        &globals,
        data.object(object).unwrap().value(),
    )
    .unwrap();
    owned.set_initializer(&value);
    assert_eq!(reserve(&mut emitter, &data).unwrap()[&object], owned);
    assert!(owned.is_constant());
    assert_eq!(owned.get_linkage(), Linkage::Private);
    assert_eq!(
        owned.get_alignment(),
        target.data.get_abi_alignment(&owned.get_value_type())
    );
    owned_module.verify().unwrap();
}
