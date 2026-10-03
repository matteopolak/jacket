//! The common generator resolves address expressions once before each SIMD operation.
use super::*;
use jai_ir::{SimdBlock, SimdInstruction};

impl<'ctx> Generator<'ctx, '_, '_> {
    pub(super) fn simd(&mut self, block: &SimdBlock) -> Result<(), Error> {
        let target = self.target;
        let triple = target.triple.as_str().to_string_lossy();
        if block.features() != jai_ir::SimdFeatures::default()
            || block.instructions().iter().any(|instruction| {
                !matches!(
                    instruction,
                    SimdInstruction::DebugTrap | SimdInstruction::Arm64DebugTrap
                )
            })
        {
            native_simd::check_target(
                &triple,
                &target.machine.get_cpu().to_string(),
                &target.machine.get_feature_string().to_string_lossy(),
                block.features(),
            )
            .map_err(Error::Simd)?;
        }
        for instruction in block.instructions() {
            match instruction {
                SimdInstruction::DebugTrap => {
                    native_simd::check_architecture(&triple).map_err(Error::Simd)?
                }
                SimdInstruction::Arm64DebugTrap => {
                    native_simd::check_arm64_architecture(&triple).map_err(Error::Simd)?
                }
                SimdInstruction::Load { .. }
                | SimdInstruction::Store { .. }
                | SimdInstruction::Add { .. } => {}
            }
        }
        block
            .validate(self.types)
            .map_err(native_simd::Error::from)
            .map_err(Error::Simd)?;
        let mut registers = native_simd::Registers::new(block);
        for instruction in block.instructions() {
            match instruction {
                SimdInstruction::DebugTrap => {
                    let trap = Intrinsic::find("llvm.debugtrap")
                        .and_then(|intrinsic| intrinsic.get_declaration(self.module, &[]))
                        .ok_or(Error::Invariant)?;
                    self.builder.build_call(trap, &[], "")?;
                }
                SimdInstruction::Arm64DebugTrap => {
                    native_simd::arm64_debug_trap(self.context, &self.builder)
                        .map_err(Error::Simd)?;
                }
                SimdInstruction::Load {
                    destination,
                    interpretation,
                    address,
                } => {
                    let pointer = self.value(address)?.into_pointer_value();
                    let width = block
                        .width(*destination)
                        .map_err(native_simd::Error::from)
                        .map_err(Error::Simd)?;
                    let valid =
                        native_simd::address_valid(self.context, &self.builder, pointer, width)
                            .map_err(Error::Simd)?;
                    self.check_cast(Bit(valid))?;
                    registers
                        .load(
                            self.context,
                            &self.builder,
                            block,
                            *destination,
                            *interpretation,
                            pointer,
                        )
                        .map_err(Error::Simd)?;
                }
                SimdInstruction::Store {
                    source,
                    interpretation,
                    address,
                } => {
                    let pointer = self.value(address)?.into_pointer_value();
                    let width = block
                        .width(*source)
                        .map_err(native_simd::Error::from)
                        .map_err(Error::Simd)?;
                    let valid =
                        native_simd::address_valid(self.context, &self.builder, pointer, width)
                            .map_err(Error::Simd)?;
                    self.check_cast(Bit(valid))?;
                    registers
                        .store(
                            self.context,
                            &self.builder,
                            block,
                            *source,
                            *interpretation,
                            pointer,
                        )
                        .map_err(Error::Simd)?;
                }
                SimdInstruction::Add {
                    destination,
                    left,
                    right,
                    interpretation,
                } => {
                    registers
                        .add(
                            self.context,
                            &self.builder,
                            block,
                            (*destination, *left, *right),
                            *interpretation,
                        )
                        .map_err(Error::Simd)?;
                }
            }
        }
        Ok(())
    }
}
