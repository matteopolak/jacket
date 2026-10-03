//! Assembly registers are private typed block identities, independent of Math records.
use super::*;
use jai_ir::{
    SimdBlock, SimdBuilder, SimdFeatures, SimdInstruction, SimdInterpretation, SimdRegisterId,
    SimdWidth,
};

#[derive(Default)]
struct Registers {
    names: HashMap<Symbol, usize>,
    widths: Vec<Option<SimdWidth>>,
    ids: Vec<SimdRegisterId>,
}
impl Registers {
    fn declare(&mut self, name: Symbol, span: Span) -> Result<(), Diagnostic> {
        if self.names.contains_key(&name) {
            return Err(Diagnostic::new(
                span,
                "SIMD register is already declared in this block",
            ));
        }
        if self.widths.len() == jai_ir::MAX_SIMD_REGISTERS {
            return Err(Diagnostic::new(span, "SIMD block exceeds 256 registers"));
        }
        self.names.insert(name, self.widths.len());
        self.widths.push(None);
        Ok(())
    }
    fn operand(
        &mut self,
        operand: &syntax::SimdOperand,
        width: SimdWidth,
    ) -> Result<(), Diagnostic> {
        if let syntax::SimdOperandKind::Register { name, introduce } = operand.kind {
            if introduce {
                self.declare(name, operand.span)?;
            }
            let index = self.names.get(&name).copied().ok_or_else(|| {
                Diagnostic::new(operand.span, "SIMD register must be declared before use")
            })?;
            if self.widths[index].is_some_and(|existing| existing != width) {
                return Err(Diagnostic::new(
                    operand.span,
                    "SIMD register cannot change width within a block",
                ));
            }
            self.widths[index] = Some(width);
        }
        Ok(())
    }
    fn id(&self, operand: &syntax::SimdOperand) -> Result<SimdRegisterId, Diagnostic> {
        let syntax::SimdOperandKind::Register { name, .. } = operand.kind else {
            return Err(Diagnostic::new(
                operand.span,
                "SIMD instruction requires a register operand",
            ));
        };
        self.names
            .get(&name)
            .and_then(|index| self.ids.get(*index))
            .copied()
            .ok_or_else(|| {
                Diagnostic::new(operand.span, "SIMD register is unavailable in this block")
            })
    }
}
fn width(width: syntax::SimdWidth) -> SimdWidth {
    match width {
        syntax::SimdWidth::X => SimdWidth::X128,
        syntax::SimdWidth::Y => SimdWidth::Y256,
    }
}
fn interpretation(opcode: syntax::SimdOpcode) -> SimdInterpretation {
    match opcode {
        syntax::SimdOpcode::Movups | syntax::SimdOpcode::Addps => SimdInterpretation::F32,
        syntax::SimdOpcode::Movdqu | syntax::SimdOpcode::Paddb => SimdInterpretation::U8,
    }
}
fn instruction_shape(
    instruction: &syntax::SimdInstruction,
    features: SimdFeatures,
) -> Result<(), Diagnostic> {
    if instruction.width == syntax::SimdWidth::Y {
        if !features.requires_avx() {
            return Err(Diagnostic::new(
                instruction.span,
                "256-bit SIMD requires declared AVX support",
            ));
        }
        if interpretation(instruction.opcode) == SimdInterpretation::U8 && !features.avx2 {
            return Err(Diagnostic::new(
                instruction.span,
                "256-bit integer SIMD requires declared AVX2 support",
            ));
        }
    }
    let operands = &instruction.operands;
    match instruction.opcode {
        syntax::SimdOpcode::Movups | syntax::SimdOpcode::Movdqu => {
            if operands.len() != 2
                || !matches!(
                    (&operands[0].kind, &operands[1].kind),
                    (
                        syntax::SimdOperandKind::Register { .. },
                        syntax::SimdOperandKind::Memory(_)
                    ) | (
                        syntax::SimdOperandKind::Memory(_),
                        syntax::SimdOperandKind::Register { .. }
                    )
                )
            {
                return Err(Diagnostic::new(
                    instruction.span,
                    "SIMD moves require exactly one register and one memory operand",
                ));
            }
        }
        syntax::SimdOpcode::Addps | syntax::SimdOpcode::Paddb => {
            if !(2..=3).contains(&operands.len())
                || operands.iter().any(|operand| {
                    !matches!(operand.kind, syntax::SimdOperandKind::Register { .. })
                })
            {
                return Err(Diagnostic::new(
                    instruction.span,
                    "SIMD adds require two or three register operands",
                ));
            }
            if operands.len() == 3 && !features.requires_avx() {
                return Err(Diagnostic::new(
                    instruction.span,
                    "three-operand SIMD adds require declared AVX support",
                ));
            }
        }
    }
    Ok(())
}

impl Resolver<'_> {
    pub(crate) fn machine_bytes(
        &self,
        source: &syntax::InstructionBytes,
    ) -> Result<SimdBlock, Diagnostic> {
        if source.bytes.as_ref() != [0x20, 0x00, 0x20, 0xd4] {
            return Err(Diagnostic::new(
                source.span,
                "machine instruction bytes are unsupported; the closed ARM64 trap profile accepts only BRK #1",
            ));
        }
        let mut builder = SimdBuilder::new(SimdFeatures::default())
            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
        builder
            .instruction(SimdInstruction::Arm64DebugTrap, self.types)
            .map_err(|error| Diagnostic::new(source.span, error.to_string()))?;
        Ok(builder.finish())
    }
    pub(crate) fn simd_block(
        &mut self,
        source: &syntax::SimdBlock,
    ) -> Result<SimdBlock, Diagnostic> {
        let mut features = SimdFeatures::default();
        for requirement in &source.features {
            let selected = match requirement.feature {
                syntax::SimdFeature::Avx => &mut features.avx,
                syntax::SimdFeature::Avx2 => &mut features.avx2,
                syntax::SimdFeature::Unsupported(name) => {
                    return Err(Diagnostic::new(
                        requirement.span,
                        format!(
                            "assembly feature '{}' is unsupported by the selected compilation profile",
                            self.symbols.name(name)
                        ),
                    ));
                }
            };
            if *selected {
                return Err(Diagnostic::new(
                    requirement.span,
                    "duplicate SIMD feature requirement",
                ));
            }
            *selected = true;
        }
        let mut registers = Registers::default();
        if source
            .statements
            .iter()
            .filter(|statement| {
                matches!(
                    statement,
                    syntax::SimdStatement::Instruction(_) | syntax::SimdStatement::DebugTrap { .. }
                )
            })
            .count()
            > jai_ir::MAX_SIMD_INSTRUCTIONS
        {
            return Err(Diagnostic::new(
                source.span,
                "SIMD block exceeds 4096 instructions",
            ));
        }
        // Collect shapes without evaluating any ordinary expression. This preserves
        // source order when address expressions are resolved in the second pass.
        for statement in &source.statements {
            match statement {
                syntax::SimdStatement::Unsupported { span } => {
                    return Err(Diagnostic::new(
                        *span,
                        "assembly instruction is unsupported by the selected compilation profile",
                    ));
                }
                syntax::SimdStatement::Interrupt { vector, span } => {
                    return Err(Diagnostic::new(
                        *span,
                        format!(
                            "interrupt int {vector:#04x} requires an unsupported platform interrupt capability"
                        ),
                    ));
                }
                syntax::SimdStatement::DebugTrap { .. } => {}
                syntax::SimdStatement::RegisterDeclaration { name, span } => {
                    registers.declare(*name, *span)?
                }
                syntax::SimdStatement::Instruction(instruction) => {
                    instruction_shape(instruction, features)?;
                    for operand in &instruction.operands {
                        registers.operand(operand, width(instruction.width))?;
                    }
                }
            }
        }
        let error = |span, error: jai_ir::SimdError| Diagnostic::new(span, error.to_string());
        let mut builder = SimdBuilder::new(features).map_err(|e| error(source.span, e))?;
        for register in &registers.widths {
            registers.ids.push(
                builder
                    .register(register.unwrap_or(SimdWidth::X128))
                    .map_err(|e| error(source.span, e))?,
            );
        }
        for statement in &source.statements {
            if let syntax::SimdStatement::DebugTrap { span } = statement {
                builder
                    .instruction(SimdInstruction::DebugTrap, self.types)
                    .map_err(|e| error(*span, e))?;
                continue;
            }
            let syntax::SimdStatement::Instruction(instruction) = statement else {
                continue;
            };
            let operands = &instruction.operands;
            let interpretation = interpretation(instruction.opcode);
            let operation = match instruction.opcode {
                syntax::SimdOpcode::Movups | syntax::SimdOpcode::Movdqu => {
                    match (&operands[0].kind, &operands[1].kind) {
                        (
                            syntax::SimdOperandKind::Register { .. },
                            syntax::SimdOperandKind::Memory(address),
                        ) => SimdInstruction::Load {
                            destination: registers.id(&operands[0])?,
                            interpretation,
                            address: self.simd_address(address, false)?,
                        },
                        (
                            syntax::SimdOperandKind::Memory(address),
                            syntax::SimdOperandKind::Register { .. },
                        ) => SimdInstruction::Store {
                            source: registers.id(&operands[1])?,
                            interpretation,
                            address: self.simd_address(address, true)?,
                        },
                        _ => {
                            return Err(Diagnostic::new(
                                instruction.span,
                                "unsupported SIMD move operands",
                            ));
                        }
                    }
                }
                syntax::SimdOpcode::Addps | syntax::SimdOpcode::Paddb => {
                    let destination = registers.id(&operands[0])?;
                    let (left, right) = if operands.len() == 2 {
                        (destination, registers.id(&operands[1])?)
                    } else {
                        (registers.id(&operands[1])?, registers.id(&operands[2])?)
                    };
                    SimdInstruction::Add {
                        destination,
                        left,
                        right,
                        interpretation,
                    }
                }
            };
            builder
                .instruction(operation, self.types)
                .map_err(|e| error(instruction.span, e))?;
        }
        Ok(builder.finish())
    }
    fn simd_address(
        &mut self,
        source: &syntax::Expression,
        writable: bool,
    ) -> Result<ValueExpr, Diagnostic> {
        let Expr::Pointer { value, .. } = self.expr(source)? else {
            return Err(Diagnostic::new(
                source.span,
                "SIMD memory operand requires a typed pointer",
            ));
        };
        if writable {
            let place = self
                .places
                .dereference(value.clone(), self.types)
                .map_err(|e| Diagnostic::new(source.span, e.to_string()))?;
            self.reject_iteration_write(place, source.span)?;
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn block(text: &str) -> syntax::SimdBlock {
        let module = syntax::parse(text).unwrap();
        let syntax::StatementKind::Simd(block) = &module.procedures()[0].body[0].kind else {
            panic!("expected assembly block")
        };
        block.clone()
    }

    fn collect(source: &syntax::SimdBlock) -> Result<Registers, Diagnostic> {
        let mut registers = Registers::default();
        for statement in &source.statements {
            match statement {
                syntax::SimdStatement::Unsupported { .. }
                | syntax::SimdStatement::DebugTrap { .. }
                | syntax::SimdStatement::Interrupt { .. } => {}
                syntax::SimdStatement::RegisterDeclaration { name, span } => {
                    registers.declare(*name, *span)?;
                }
                syntax::SimdStatement::Instruction(instruction) => {
                    for operand in &instruction.operands {
                        registers.operand(operand, width(instruction.width))?;
                    }
                }
            }
        }
        Ok(registers)
    }

    #[test]
    fn declarations_and_inline_destinations_preserve_source_register_order() {
        let source = block("main::(){#asm AVX {a:vec; movups.x a,[p]; addps.x b:,a,a;}} ");
        let registers = collect(&source).unwrap();
        assert_eq!(registers.widths, [Some(SimdWidth::X128); 2]);
        let syntax::SimdStatement::RegisterDeclaration { name, .. } = source.statements[0] else {
            panic!("expected declaration")
        };
        assert_eq!(registers.names[&name], 0);
        let syntax::SimdStatement::Instruction(instruction) = &source.statements[2] else {
            panic!("expected addition")
        };
        let syntax::SimdOperandKind::Register { name, .. } = instruction.operands[0].kind else {
            panic!("expected destination")
        };
        assert_eq!(registers.names[&name], 1);
    }

    #[test]
    fn use_before_declaration_duplicates_and_width_changes_are_rejected() {
        for text in [
            "main::(){#asm {movups.x a,[p]; a:vec;}}",
            "main::(){#asm {a:vec; movups.x a:,[p];}}",
            "main::(){#asm AVX {a:vec; movups.x a,[p]; movups.y a,[q];}}",
        ] {
            assert!(collect(&block(text)).is_err(), "{text}");
        }
    }

    #[test]
    fn feature_proof_distinguishes_wide_float_and_integer_operations() {
        let source = block("main::(){#asm AVX {addps.y a:,b,c; paddb.y d:,b,c;}} ");
        let syntax::SimdStatement::Instruction(float) = &source.statements[0] else {
            panic!("expected float addition")
        };
        let syntax::SimdStatement::Instruction(integer) = &source.statements[1] else {
            panic!("expected integer addition")
        };
        let avx = SimdFeatures {
            avx: true,
            avx2: false,
        };
        assert!(instruction_shape(float, SimdFeatures::default()).is_err());
        instruction_shape(float, avx).unwrap();
        assert!(instruction_shape(integer, avx).is_err());
        instruction_shape(
            integer,
            SimdFeatures {
                avx: false,
                avx2: true,
            },
        )
        .unwrap();
    }

    #[test]
    fn memory_moves_and_additions_reject_invalid_operand_shapes() {
        for text in [
            "main::(){#asm {movups.x a:,b;}}",
            "main::(){#asm {addps.x a:,[p];}}",
            "main::(){#asm {addps.x a:,b,c;}}",
        ] {
            let source = block(text);
            let syntax::SimdStatement::Instruction(instruction) = &source.statements[0] else {
                panic!("expected instruction")
            };
            assert!(
                instruction_shape(instruction, SimdFeatures::default()).is_err(),
                "{text}"
            );
        }
    }
}
