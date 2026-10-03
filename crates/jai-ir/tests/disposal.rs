use jai_ir::*;
use jai_source::{SourceMap, SourceSpan, Span};
use jai_types::{
    CallingConvention, ContextMode, Integer, IntegerType, TypeId, TypeRegistry, Types, Variadic,
};
use std::{collections::HashMap, process::Command};

const CHILD_TEST: &str = "JAI_IR_DISPOSAL_CHILD_TEST";
const RAW_DEPTH: usize = 100_000;

// An overflowing child must fail this test instead of aborting the full suite.
// The child executes the same exact test on a fixed-size thread stack.
fn isolated(test_name: &str, regression: fn()) {
    if std::env::var(CHILD_TEST).as_deref() == Ok(test_name) {
        std::thread::Builder::new()
            .name("staging-disposal".into())
            .stack_size(4 * 1024 * 1024)
            .spawn(regression)
            .unwrap()
            .join()
            .unwrap();
        return;
    }
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
        .env(CHILD_TEST, test_name)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "disposal subprocess failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn types_and_signature() -> (Types, TypeId) {
    let mut types = TypeRegistry::new();
    let signature = types
        .procedure(jai_types::ProcedureType {
            parameters: Box::new([]),
            results: Box::new([]),
            return_abi: jai_types::ForeignReturnAbi::Natural,
            convention: CallingConvention::Jai,
            context: ContextMode::Implicit,
            variadic: Variadic::None,
        })
        .unwrap();
    (types.freeze().unwrap(), signature)
}

fn block(statements: Vec<Statement>) -> Block {
    Block {
        statements,
        flow: Flow::FallsThrough,
    }
}

fn procedure(signature: TypeId, body: Block) -> Procedure {
    Procedure {
        id: ProcedureId::new(7),
        signature,
        parameters: vec![],
        locals: vec![],
        body,
        cleanups: vec![],
    }
}

fn deep_block() -> Block {
    let mut body = block(vec![]);
    for _ in 0..RAW_DEPTH {
        body = block(vec![Statement::Block(body)]);
    }
    body
}

fn assert_borrowed_depth(types: &Types, procedure: &Procedure) {
    assert!(matches!(
        verify_procedure(
            types,
            procedure,
            &HashMap::from([(procedure.id, procedure.signature)]),
            &[],
            &Places::default()
        ),
        Err(IrError::VerificationDepth)
    ));
    eprintln!("borrowed verification returned VerificationDepth");
}

#[test]
fn consumed_builder_disposes_deep_block_after_depth_error() {
    isolated(
        "consumed_builder_disposes_deep_block_after_depth_error",
        || {
            let (types, signature) = types_and_signature();
            let procedure = procedure(signature, deep_block());
            assert_borrowed_depth(&types, &procedure);
            assert!(matches!(
                ProgramBuilder::new(types)
                    .procedures(vec![procedure])
                    .finish_library(),
                Err(IrError::VerificationDepth)
            ));
        },
    );
}

#[test]
fn consumed_builder_disposes_deep_expression_after_depth_error() {
    isolated(
        "consumed_builder_disposes_deep_expression_after_depth_error",
        || {
            let (types, signature) = types_and_signature();
            let mut expression = IntExpr::constant(Integer::checked(IntegerType::S64, 1).unwrap());
            for _ in 0..RAW_DEPTH {
                expression =
                    IntExpr::new(IntegerType::S64, IntExprKind::Negate(Box::new(expression)));
            }
            let procedure = procedure(signature, block(vec![Statement::DiscardInt(expression)]));
            assert_borrowed_depth(&types, &procedure);
            assert!(matches!(
                ProgramBuilder::new(types)
                    .procedures(vec![procedure])
                    .finish_library(),
                Err(IrError::VerificationDepth)
            ));
        },
    );
}

#[test]
fn consumed_builder_disposes_deep_staging_after_debug_metadata_error() {
    isolated(
        "consumed_builder_disposes_deep_staging_after_debug_metadata_error",
        || {
            let (types, signature) = types_and_signature();
            let procedure = procedure(signature, deep_block());
            assert_borrowed_depth(&types, &procedure);
            let mut source_map = SourceMap::default();
            let source_id = source_map.insert("disposal.jai".into(), "main :: () {}".into());
            let source = source_map.get(source_id).unwrap();
            let mut debug = DebugSources::default();
            debug.retain_source(source);
            debug.insert(
                ProcedureId::new(999),
                ProcedureSource {
                    name: "unknown_procedure".into(),
                    location: DebugSourceLocation::from_source(
                        source,
                        SourceSpan {
                            source: source_id,
                            span: Span::new(0, source.text().len()),
                        },
                    )
                    .unwrap(),
                },
            );
            assert!(matches!(
                ProgramBuilder::new(types)
                    .procedures(vec![procedure])
                    .debug_sources(debug)
                    .finish_library(),
                Err(IrError::UnknownIdentity {
                    kind: "debug procedure",
                    index: 999
                })
            ));
        },
    );
}
