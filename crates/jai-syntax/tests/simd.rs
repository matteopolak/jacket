use jai_source::{SourceMap, Symbols};
#[path = "support/optional_sources.rs"]
mod optional_sources;
use jai_syntax::{
    ExpressionKind, FileDeclarationKind, FileItem, SimdFeature, SimdOpcode, SimdOperandKind,
    SimdStatement, SimdWidth, StatementKind,
};

#[test]
fn source_corpus_simd_blocks_preserve_features_registers_operands_and_spans() {
    let Some(source_text) = optional_sources::read_optional(
        "corpus/upstream/withlang-dev--open-jai/examples/28/28.6_simd.jai",
    ) else {
        return;
    };
    let text = source_text.as_str();
    let mut sources = SourceMap::default();
    let source = sources.insert("simd.jai".into(), text.into());
    let mut symbols = Symbols::default();
    let file = jai_syntax::parse_file(sources.get(source).unwrap(), &mut symbols).unwrap();
    let main = file
        .items()
        .iter()
        .find_map(|item| match item {
            FileItem::Declaration(declaration) => match &declaration.kind {
                FileDeclarationKind::Procedure(procedure) => Some(procedure),
                _ => None,
            },
            _ => None,
        })
        .unwrap();
    let blocks = main
        .body
        .iter()
        .filter_map(|statement| match &statement.kind {
            StatementKind::Simd(block) => {
                assert_eq!(statement.span, block.span);
                assert!(block.span.text(text).starts_with("#asm"));
                assert!(block.span.text(text).ends_with('}'));
                Some(block)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 3);
    assert!(blocks[0].features.is_empty());
    assert_eq!(blocks[1].features[0].feature, SimdFeature::Avx);
    assert_eq!(blocks[1].features[0].span.text(text), "AVX");
    assert_eq!(blocks[2].features[1].feature, SimdFeature::Avx2);
    assert_eq!(blocks[2].features[1].span.text(text), "AVX2");
    let SimdStatement::RegisterDeclaration {
        name,
        span,
    } = &blocks[0].statements[0]
    else {
        panic!("expected vec declaration")
    };
    assert_eq!(symbols.name(*name), "v");
    assert_eq!(span.text(text), "v: vec;");
    let SimdStatement::Instruction(load) = &blocks[0].statements[1] else {
        panic!("expected load")
    };
    assert_eq!(load.opcode, SimdOpcode::Movups);
    assert_eq!(load.width, SimdWidth::X);
    assert_eq!(load.span.text(text), "movups.x v, [ptr];");
    assert!(
        matches!(load.operands[0].kind, SimdOperandKind::Register { name: register, introduce: false } if register == *name)
    );
    let SimdOperandKind::Memory(address) = &load.operands[1].kind else {
        panic!("expected address")
    };
    assert_eq!(load.operands[1].span.text(text), "[ptr]");
    assert_eq!(address.span.text(text), "ptr");
    assert!(
        matches!(address.kind, ExpressionKind::Name(register) if symbols.name(register) == "ptr")
    );
    let SimdStatement::Instruction(add) = &blocks[1].statements[2] else {
        panic!("expected AVX add")
    };
    assert_eq!(add.opcode, SimdOpcode::Addps);
    assert_eq!(add.width, SimdWidth::Y);
    assert_eq!(add.operands.len(), 3);
    let SimdStatement::Instruction(add) = &blocks[2].statements[2] else {
        panic!("expected byte add")
    };
    assert_eq!(add.opcode, SimdOpcode::Paddb);
    assert_eq!(
        add.operands
            .iter()
            .map(|operand| operand.span.text(text))
            .collect::<Vec<_>>(),
        ["v3:", "v1", "v2"]
    );
    assert!(matches!(
        add.operands[0].kind,
        SimdOperandKind::Register {
            introduce: true,
            ..
        }
    ));
}

#[test]
fn self_authored_simd_widths_and_register_introductions_always_parse() {
    let source = r#"
        kernel :: () {
            #asm { lane:vec; movups.x lane,[input]; addps.x lane,lane; movups.x [output],lane; }
            #asm AVX { movups.y wide:,[input]; addps.y doubled:,wide,wide; movups.y [output],doubled; }
            #asm AVX,AVX2 { movdqu.x bytes:,[input]; paddb.x sum:,bytes,bytes; movdqu.x [output],sum; }
        }
    "#;
    let module = jai_syntax::parse(source).unwrap();
    let blocks = module.procedures()[0]
        .body
        .iter()
        .map(|statement| {
            let StatementKind::Simd(block) = &statement.kind else {
                panic!("expected SIMD block")
            };
            block
        })
        .collect::<Vec<_>>();
    assert_eq!(blocks.len(), 3);
    for (block, width, opcode) in [
        (blocks[0], SimdWidth::X, SimdOpcode::Addps),
        (blocks[1], SimdWidth::Y, SimdOpcode::Addps),
        (blocks[2], SimdWidth::X, SimdOpcode::Paddb),
    ] {
        let instruction = block
            .statements
            .iter()
            .find_map(|statement| match statement {
                SimdStatement::Instruction(instruction) if instruction.opcode == opcode => {
                    Some(instruction)
                }
                _ => None,
            })
            .unwrap();
        assert_eq!(instruction.width, width);
        assert!(
            instruction.span.text(source).contains(".x")
                || instruction.span.text(source).contains(".y")
        );
    }
    assert!(blocks[0].features.is_empty());
    assert_eq!(blocks[1].features[0].feature, SimdFeature::Avx);
    assert_eq!(blocks[2].features[1].feature, SimdFeature::Avx2);
}

#[test]
fn bracketed_addresses_use_the_standard_expression_parser() {
    let text = "main :: () { #asm { movups.x v:, [ptr + offset]; } }";
    let module = jai_syntax::parse(text).unwrap();
    let StatementKind::Simd(block) = &module.procedures()[0].body[0].kind else {
        panic!("expected SIMD block")
    };
    let SimdStatement::Instruction(instruction) = &block.statements[0] else {
        panic!("expected instruction")
    };
    let SimdOperandKind::Memory(address) = &instruction.operands[1].kind else {
        panic!("expected address")
    };
    assert!(matches!(address.kind, ExpressionKind::Binary { .. }));
    assert_eq!(address.span.text(text), "ptr + offset");
}

#[test]
fn unsupported_assembly_syntax_has_targeted_source_diagnostics() {
    for (assembly, token, message) in [
        ("#asm AVX, { }", "{", "feature list requires identifiers"),
        ("#asm 123 { }", "123", "feature list requires identifiers"),
        (
            "#asm AVX, ,AVX2 { }",
            ",",
            "feature list requires identifiers",
        ),
        (
            "#asm { movups.z v, [ptr]; }",
            "z",
            "unsupported #asm width modifier",
        ),
        (
            "#asm { movups.x 42, v; }",
            "42",
            "register or bracketed memory operand",
        ),
    ] {
        let text = format!("main :: () {{ {assembly} }}");
        let diagnostic = jai_syntax::parse(&text).unwrap_err();
        assert_eq!(diagnostic.span.text(&text), token);
        assert!(
            diagnostic.message.contains(message),
            "{}",
            diagnostic.message
        );
    }
}

#[test]
fn unterminated_assembly_block_reports_eof() {
    let text = "main :: () { #asm { movups.x v:, [ptr];";
    let diagnostic = jai_syntax::parse(text).unwrap_err();
    assert!(diagnostic.message.contains("unterminated #asm block"));
    assert_eq!(diagnostic.span.start, text.len());
}

#[test]
fn unsupported_balanced_statements_are_retained_without_register_or_expression_resolution() {
    let text = "main :: () { #asm { rax: gpr; obscure [address((1 + 2))], .{ field := .[1,2]; }; int3; int 0x41; } }";
    let module = jai_syntax::parse(text).unwrap();
    let StatementKind::Simd(block) = &module.procedures()[0].body[0].kind else {
        panic!("expected assembly block")
    };
    assert_eq!(block.statements.len(), 4);
    let SimdStatement::Unsupported {
        span,
    } = block.statements[0]
    else {
        panic!("expected unsupported register type")
    };
    assert_eq!(span.text(text), "rax: gpr;");
    let SimdStatement::Unsupported {
        span,
    } = block.statements[1]
    else {
        panic!("expected unsupported opcode")
    };
    assert_eq!(
        span.text(text),
        "obscure [address((1 + 2))], .{ field := .[1,2]; };"
    );
    assert!(module.symbols().find("rax").is_none());
    assert!(module.symbols().find("address").is_none());
    let SimdStatement::DebugTrap {
        span,
    } = block.statements[2]
    else {
        panic!("expected debug trap")
    };
    assert_eq!(span.text(text), "int3;");
    let SimdStatement::Interrupt {
        vector,
        span,
    } = block.statements[3]
    else {
        panic!("expected interrupt")
    };
    assert_eq!(vector, 0x41);
    assert_eq!(span.text(text), "int 0x41;");
}

#[test]
fn unknown_instructions_inside_inactive_source_branches_do_not_fail_parsing() {
    let text = "main :: () { #if false { #asm { v: general; arbitrary [never_resolved]; } } }";
    let mut sources = SourceMap::default();
    let source = sources.insert("inactive-asm.jai".into(), text.into());
    let file =
        jai_syntax::parse_file(sources.get(source).unwrap(), &mut Symbols::default()).unwrap();
    let FileItem::Declaration(declaration) = &file.items()[0] else {
        panic!("expected declaration")
    };
    let FileDeclarationKind::Procedure(procedure) = &declaration.kind else {
        panic!("expected procedure")
    };
    let StatementKind::CompileTimeIf {
        then_body, ..
    } = &procedure.body[0].kind
    else {
        panic!("expected compile-time branch")
    };
    let StatementKind::Simd(block) = &then_body[0].kind else {
        panic!("expected retained assembly")
    };
    assert!(
        block
            .statements
            .iter()
            .all(|statement| matches!(statement, SimdStatement::Unsupported { .. }))
    );
}

#[test]
fn unsupported_assembly_still_requires_balanced_and_terminated_syntax() {
    for assembly in [
        "#asm { mystery [ptr); }",
        "#asm { mystery (ptr; }",
        "#asm { mystery [ptr] }",
        "#asm { rax: custom { nested; }; mystery [ptr;",
    ] {
        let text = format!("main :: () {{ {assembly} }}");
        let diagnostic = jai_syntax::parse(&text).unwrap_err();
        assert!(
            diagnostic.message.contains("unsupported #asm statement"),
            "{diagnostic}"
        );
    }
}

#[test]
fn interrupts_accept_only_u8_integer_literals_and_exact_traps_require_a_terminator() {
    for literal in ["0", "255", "0xff", "0b01000001"] {
        let text = format!("main :: () {{ #asm {{ int {literal}; }} }}");
        let module = jai_syntax::parse(&text).unwrap();
        let StatementKind::Simd(block) = &module.procedures()[0].body[0].kind else {
            panic!("expected assembly")
        };
        assert!(matches!(
            block.statements[0],
            SimdStatement::Interrupt { .. }
        ));
    }
    for literal in ["256", "0x100", "1.0", "-1", "name"] {
        let text = format!("main :: () {{ #asm {{ int {literal}; }} }}");
        let diagnostic = jai_syntax::parse(&text).unwrap_err();
        assert!(
            diagnostic.message.contains("u8 integer literal"),
            "{diagnostic}"
        );
    }
    for assembly in [
        "#asm { int3 extra; }",
        "#asm { int3 }",
        "#asm { int 65 extra; }",
    ] {
        let text = format!("main :: () {{ {assembly} }}");
        assert!(
            jai_syntax::parse(&text)
                .unwrap_err()
                .message
                .contains("expected ';'")
        );
    }
}

#[test]
fn unsupported_feature_identifiers_keep_symbol_identity_and_requirement_spans() {
    let text =
        "main :: () { #asm SYSCALL_SYSRET, AVX, SYSCALL_SYSRET { syscall temporary:, source; } }";
    let module = jai_syntax::parse(text).unwrap();
    let StatementKind::Simd(block) = &module.procedures()[0].body[0].kind else {
        panic!("expected assembly block")
    };
    let SimdFeature::Unsupported(first) = block.features[0].feature else {
        panic!("expected retained unsupported feature")
    };
    let SimdFeature::Unsupported(last) = block.features[2].feature else {
        panic!("expected retained unsupported feature")
    };
    assert_eq!(first, last);
    assert_eq!(module.symbols().name(first), "SYSCALL_SYSRET");
    assert_eq!(block.features[0].span.text(text), "SYSCALL_SYSRET");
    assert_eq!(block.features[1].feature, SimdFeature::Avx);
    assert_eq!(block.features[2].span.text(text), "SYSCALL_SYSRET");
    assert!(matches!(
        block.statements[0],
        SimdStatement::Unsupported { .. }
    ));
}
