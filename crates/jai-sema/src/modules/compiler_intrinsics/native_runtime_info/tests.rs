use super::*;
use jai_ir::{Block, Exit, Flow, Local, LocalExternalDataIndex, Places, Procedure};
use jai_source::{SourceMap, SourceSpan, Span};
use jai_types::{
    CallingConvention, ContextMode, Integer, ProcedureType, RecordKind, ScalarType, TypeRegistry,
};
use std::collections::HashMap;

struct Fixture {
    types: TypeRegistry,
    procedure: Procedure,
    globals: Vec<jai_ir::Global>,
    source: SourceProcedureIdentity,
    binding: CompilerProcedure,
}

fn fixture(owner: ProcedureId) -> Fixture {
    let mut types = TypeRegistry::new();
    let header = types.reserve_record(RecordKind::Struct);
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
    let tag = types.reserve_enum(IntegerType::U16);
    types
        .define_enum(
            tag,
            [0, 1, 2, 3, 5].map(|value| Integer::checked(IntegerType::U16, value).unwrap()),
        )
        .unwrap();
    let bytes = types
        .slice(types.scalar(ScalarType::Int(IntegerType::U8)))
        .unwrap();
    let segment = types.reserve_record(RecordKind::Struct);
    types.define_record(segment, [tag, bytes]).unwrap();
    let segments = types.slice(segment).unwrap();
    let global_data = types.reserve_record(RecordKind::Struct);
    types
        .define_record(
            global_data,
            [types.scalar(ScalarType::Int(IntegerType::U64)), segments],
        )
        .unwrap();
    let header_pointer = types.pointer(header).unwrap();
    let table = types.slice(header_pointer).unwrap();
    let global_pointer = types.pointer(global_data).unwrap();
    let info = types.reserve_record(RecordKind::Struct);
    types.define_record(info, [table, global_pointer]).unwrap();
    let schema = RuntimeInfoSchema::validate(&types, info, global_data).unwrap();
    let workspace_type = types.scalar(ScalarType::Int(IntegerType::S64));
    let signature = types
        .procedure(ProcedureType {
            parameters: vec![workspace_type].into(),
            results: vec![info].into(),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let id = ProcedureId::new(8);
    let mut sources = SourceMap::default();
    let text = "info :: (w: s64) -> Runtime_Info #compiler \"get_runtime_info\" { fresh_program_catalog: Runtime_Info #elsewhere; return fresh_program_catalog; }";
    let source_id = sources.insert("own-runtime-info.jai".into(), text.into());
    let source = SourceProcedureIdentity::new(
        sources.get(source_id).unwrap(),
        SourceSpan {
            source: source_id,
            span: Span::new(0, text.len()),
        },
    )
    .unwrap();
    let declaration_start = text.find("fresh_program_catalog:").unwrap();
    let declaration_end =
        declaration_start + "fresh_program_catalog: Runtime_Info #elsewhere;".len();
    let data = ExternalData::new(
        ExternalDataId::Local {
            procedure: owner,
            index: LocalExternalDataIndex::new(0),
        },
        info,
        ExternalDataSource::Program,
        "fresh_program_catalog".into(),
        SourceSpan {
            source: source_id,
            span: Span::new(declaration_start, declaration_end),
        },
        &types,
    )
    .unwrap();
    let global = jai_ir::Global::new_external(0, data);
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![Local::new_typed(id, 0, workspace_type, &types).unwrap()],
        locals: vec![Local::new_typed(id, 0, workspace_type, &types).unwrap()],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::Block(Block {
                    flow: Flow::FallsThrough,
                    statements: vec![],
                }),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnValues(vec![ValueExpr::Load(global.place())]),
                }),
            ],
        },
    };
    Fixture {
        types,
        procedure,
        globals: vec![global],
        source,
        binding: CompilerProcedure {
            signature,
            intrinsic: CompilerIntrinsic::SourceRuntimeInfo {
                current_workspace: WorkspaceId::from_raw(73).unwrap(),
                schema,
            },
        },
    }
}

fn role(fixture: &Fixture) -> Result<Option<NativeRuntimeInfoRole>, String> {
    let signatures = HashMap::from([(fixture.procedure.id, fixture.procedure.signature)]);
    let places = Places::default();
    let checked = jai_ir::verify_procedure(
        &fixture.types,
        &fixture.procedure,
        &signatures,
        &fixture.globals,
        &places,
    )
    .unwrap();
    NativeRuntimeInfoRole::from_checked_fallback(&checked, fixture.binding, fixture.source.clone())
}

#[test]
fn genuine_program_external_receipt_retains_actual_symbol_and_source_owner() {
    let fixture = fixture(ProcedureId::new(8));
    let role = role(&fixture).unwrap().unwrap();
    assert_eq!(role.procedure(), fixture.procedure.id);
    assert_eq!(role.global(), fixture.globals[0].id());
    assert_eq!(role.data().symbol(), "fresh_program_catalog");
    assert_eq!(role.schema().ty(), fixture.globals[0].ty());
    assert_eq!(role.workspace(), WorkspaceId::from_raw(73).unwrap());
    assert_eq!(role.source().body_text(), fixture.source.body_text());
}

#[test]
fn another_procedure_external_cannot_become_the_compiler_table() {
    let fixture = fixture(ProcedureId::new(9));
    assert!(
        role(&fixture)
            .unwrap_err()
            .contains("same source procedure")
    );
}

#[test]
fn ordinary_capability_never_grants_a_runtime_info_role() {
    let mut fixture = fixture(ProcedureId::new(8));
    fixture.binding.intrinsic = CompilerIntrinsic::SourceCurrentWorkspace {
        current_workspace: WorkspaceId::from_raw(73).unwrap(),
    };
    assert!(role(&fixture).unwrap().is_none());
}

#[test]
fn checked_owned_zero_fallback_is_not_native_program_data() {
    let mut fixture = fixture(ProcedureId::new(8));
    fixture.globals[0] = jai_ir::Global::new_typed(
        0,
        jai_ir::ConstantValue {
            ty: fixture.globals[0].ty(),
            kind: jai_ir::ConstantKind::Zero,
        },
        &fixture.types,
    )
    .unwrap();
    assert!(role(&fixture).unwrap_err().contains("owned initializer"));
}

#[test]
fn fallback_with_an_extra_executed_read_is_not_silently_rewritten() {
    let mut fixture = fixture(ProcedureId::new(8));
    fixture.procedure.body.statements.insert(
        0,
        Statement::DiscardValue(ValueExpr::Load(fixture.globals[0].place())),
    );
    assert!(role(&fixture).unwrap_err().contains("direct return"));
}

#[test]
fn external_outside_the_retained_source_fallback_has_no_native_role() {
    let mut fixture = fixture(ProcedureId::new(8));
    let GlobalInitializer::External(data) = fixture.globals[0].initializer() else {
        panic!("external fixture");
    };
    let mut location = data.location();
    location.span = Span::new(
        fixture.source.location().span.end + 1,
        fixture.source.location().span.end + 2,
    );
    let moved = ExternalData::new(
        data.id(),
        data.ty(),
        data.source().clone(),
        data.symbol().into(),
        location,
        &fixture.types,
    )
    .unwrap();
    fixture.globals[0] = jai_ir::Global::new_external(0, moved);
    assert!(
        role(&fixture)
            .unwrap_err()
            .contains("outside its retained source")
    );
}
