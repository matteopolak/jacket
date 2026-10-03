use crate::*;
use jai_ir::ProcedureId;
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, ProcedureType, ScalarType, TypeId,
    TypeRegistry, TypeView, Variadic,
};
use std::collections::HashMap;

struct Provider {
    types: TypeRegistry,
    declarations: HashMap<ProcedureId, TypeId>,
    runtime: Option<RuntimeProcedure>,
    compiler: Option<CompilerProcedure>,
}
impl ProcedureProvider for Provider {
    fn types(&self) -> &dyn TypeView {
        &self.types
    }
    fn signatures(&self) -> &HashMap<ProcedureId, TypeId> {
        &self.declarations
    }
    fn procedure(&self, _: ProcedureId) -> ProcedureAvailability<'_> {
        match (self.runtime, self.compiler) {
            (Some(procedure), _) => ProcedureAvailability::Runtime(procedure),
            (_, Some(procedure)) => ProcedureAvailability::Compiler(procedure),
            _ => ProcedureAvailability::Missing,
        }
    }
}
fn signature(types: &mut TypeRegistry, parameters: Vec<TypeId>, results: Vec<TypeId>) -> TypeId {
    types
        .procedure(ProcedureType {
            parameters: parameters.into(),
            results: results.into(),
            context: ContextMode::None,
            convention: CallingConvention::Jai,
            variadic: Variadic::None,
        })
        .unwrap()
}
#[test]
fn runtime_provider_cannot_replace_declared_result_type_or_omit_declaration() {
    let mut types = TypeRegistry::new();
    let pointer = types.pointer(types.void()).unwrap();
    let long = types.scalar(ScalarType::Int(IntegerType::S64));
    let short = types.scalar(ScalarType::Int(IntegerType::S16));
    let declared = signature(&mut types, vec![pointer, pointer, long], vec![long]);
    let actual = signature(&mut types, vec![pointer, pointer, long], vec![short]);
    let void = types.void();
    let mut provider = Provider {
        types,
        declarations: [(ProcedureId::new(0), declared)].into(),
        runtime: Some(RuntimeProcedure {
            signature: actual,
            intrinsic: jai_ir::RuntimeIntrinsic::MemoryCompare,
        }),
        compiler: None,
    };
    let arguments = vec![
        Value::Pointer(Pointer::null(void)),
        Value::Pointer(Pointer::null(void)),
        Value::Int(Integer::wrapping(IntegerType::S64, 0)),
    ];
    for remove in [false, true] {
        if remove {
            provider.declarations.clear();
        }
        let mut vm = Vm::new(&provider, NoEffects, Limits::default()).unwrap();
        assert_eq!(
            vm.execute(ProcedureId::new(0), arguments.clone()).outcome,
            Outcome::Failed(Error::InvalidIr(
                "provider procedure signature differs from declaration"
            ))
        );
    }
}
#[test]
fn compiler_provider_signature_is_checked_before_request() {
    let mut types = TypeRegistry::new();
    let text = types.string();
    let long = types.scalar(ScalarType::Int(IntegerType::S64));
    let declared = signature(&mut types, vec![text], vec![long]);
    let actual = signature(&mut types, vec![text], vec![]);
    let provider = Provider {
        types,
        declarations: [(ProcedureId::new(0), declared)].into(),
        runtime: None,
        compiler: Some(CompilerProcedure {
            signature: actual,
            intrinsic: CompilerIntrinsic::Message(MessageLevel::Info),
        }),
    };
    // NoEffects rejects every request, so InvalidIr proves the request was never reached.
    assert_eq!(
        Vm::new(&provider, NoEffects, Limits::default())
            .unwrap()
            .execute(ProcedureId::new(0), vec![Value::String(b"test".to_vec())])
            .outcome,
        Outcome::Failed(Error::InvalidIr(
            "provider procedure signature differs from declaration"
        ))
    );
}
