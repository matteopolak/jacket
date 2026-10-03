use crate::{
    Context,
    debug::{
        DebugInformation,
        variables::tests::{Scratch, clang_command, debug_tool_command},
    },
    target::{NativeTarget, TargetOptions},
};
use jai_ir::*;
use jai_source::{SourceMap, SourceSpan, Span};
use jai_types::*;
use std::process::Command;

fn constant(value: i128) -> IntExpr {
    IntExpr::constant(Integer::checked(IntegerType::S64, value).unwrap())
}

#[test]
fn checked_loop_local_declares_at_parent_with_child_lexical_scope() {
    let scratch = Scratch::new();
    let text = "main :: () -> int {\n  result := 40;\n  for index: 0..0 {\n    result += 2;\n  }\n  return result;\n}\n";
    std::fs::write(scratch.0.join("range.jai"), text).unwrap();
    let mut map = SourceMap::default();
    let source_id = map.insert(scratch.0.join("range.jai"), text.into());
    let source = map.get(source_id).unwrap();
    let mut sources = DebugSources::default();
    let location = |sources: &mut DebugSources, text_part: &str| {
        let start = text.find(text_part).unwrap();
        sources
            .source_location(
                source,
                SourceSpan {
                    source: source_id,
                    span: Span::new(start, start + text_part.len()),
                },
            )
            .unwrap()
    };
    let id = ProcedureId::new(27);
    let root = BlockPath::procedure(id);
    let body_path = root.child(1, DebugBranch::Range);
    let function_source = location(&mut sources, "main");
    sources.insert(
        id,
        ProcedureSource {
            name: "main".into(),
            location: function_source.clone(),
        },
    );
    sources.insert_block(root.clone(), function_source);
    let init = location(&mut sources, "result := 40");
    sources.insert_statement(root.statement(0), init.clone());
    let loop_source = location(&mut sources, "for index");
    sources.insert_statement(root.statement(1), loop_source.clone());
    sources.insert_block(body_path.clone(), loop_source.clone());
    let step = location(&mut sources, "result += 2");
    sources.insert_statement(body_path.statement(0), step);
    let returned = location(&mut sources, "return result");
    sources.insert_statement(root.statement(2), returned);
    let mut types = TypeRegistry::new();
    let int = types.scalar(ScalarType::Int(IntegerType::S64));
    let result = Local::new_typed(id, 0, int, &types)
        .unwrap()
        .integer(&types)
        .unwrap();
    let index = Local::new_typed(id, 1, int, &types)
        .unwrap()
        .integer(&types)
        .unwrap();
    sources.insert_local(
        result.local().id(),
        LocalSource {
            name: "result".into(),
            location: init,
            scope: root.clone(),
            declaration: LocalDeclaration::Statement(root.statement(0)),
        },
    );
    sources.insert_local(
        index.local().id(),
        LocalSource {
            name: "index".into(),
            location: loop_source,
            scope: body_path,
            declaration: LocalDeclaration::Statement(root.statement(1)),
        },
    );
    let signature = types
        .procedure(ProcedureType {
            parameters: Box::new([]),
            results: Box::new([int]),
            convention: CallingConvention::Jai,
            context: ContextMode::None,
            variadic: Variadic::None,
        })
        .unwrap();
    let procedure = Procedure {
        id,
        signature,
        parameters: vec![],
        locals: vec![result.local(), index.local()],
        cleanups: vec![],
        body: Block {
            flow: Flow::Terminates,
            statements: vec![
                Statement::StoreInt(result.place(), constant(40)),
                Statement::Range(RangeLoop {
                    id: LoopId::new(0),
                    iterator: index,
                    start: constant(0),
                    end: constant(0),
                    direction: Direction::Forward,
                    body: Block {
                        flow: Flow::FallsThrough,
                        statements: vec![Statement::StoreInt(
                            result.place(),
                            IntExpr::new(
                                IntegerType::S64,
                                IntExprKind::Binary(
                                    IntOp::Add,
                                    Box::new(IntExpr::load(result.place())),
                                    Box::new(constant(2)),
                                ),
                            ),
                        )],
                    },
                }),
                Statement::Exit(Exit {
                    cleanups: vec![],
                    transfer: Transfer::ReturnInt(IntExpr::load(result.place())),
                }),
            ],
        },
    };
    let program = ProgramBuilder::new(types.freeze().unwrap())
        .debug_sources(sources)
        .procedures(vec![procedure])
        .finish(EntryPoint::Int(id))
        .unwrap();
    let target = NativeTarget::select(&TargetOptions {
        debug: DebugInformation::Variables,
        ..Default::default()
    })
    .unwrap();
    let context = Context::create();
    let module = crate::lower_for_target(&context, &program, &target).unwrap();
    let object = scratch.0.join("range.o");
    target.write_object(&module, &object).unwrap();
    let output = debug_tool_command("llvm-dwarfdump")
        .args(["--debug-info", "--debug-line"])
        .arg(&object)
        .output()
        .unwrap();
    assert!(output.status.success());
    let dwarf = String::from_utf8(output.stdout).unwrap();
    assert!(dwarf.contains("DW_AT_name\t(\"index\")"), "{dwarf}");
    assert!(dwarf.contains("DW_TAG_lexical_block"), "{dwarf}");
    let executable = scratch.0.join("range");
    let linked = clang_command()
        .arg(&object)
        .arg("-o")
        .arg(&executable)
        .output()
        .unwrap();
    assert!(
        linked.status.success(),
        "{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    assert_eq!(Command::new(executable).status().unwrap().code(), Some(42));
}
