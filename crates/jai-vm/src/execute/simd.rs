//! Checked register execution; arithmetic never invokes a host SIMD instruction.
use super::*;
use crate::{Endian, Error};
use jai_ir::{SimdBlock, SimdInstruction, SimdInterpretation, SimdRegisterId};
use jai_types::{FloatOp, FloatValue};

impl<P: ProcedureProvider + ?Sized, E: CompilerEffects> Vm<'_, P, E> {
    pub(super) fn simd(&mut self, block: &SimdBlock, depth: usize) -> Result<Control> {
        let target = self.memory.target();
        let vector_operations = block.instructions().iter().any(|instruction| {
            !matches!(
                instruction,
                SimdInstruction::DebugTrap | SimdInstruction::Arm64DebugTrap
            )
        });
        if vector_operations
            && (target.policy.pointer().size != 8 || target.endian != Endian::Little)
        {
            return Err(
                Error::InvalidIr("SIMD execution requires a little-endian 64-bit target").into(),
            );
        }
        block
            .validate(self.provider.types())
            .map_err(|error| Error::IrValidation(error.to_string()))?;
        let register_bytes = block.registers().iter().try_fold(0usize, |bytes, width| {
            bytes
                .checked_add(width.bytes())
                .ok_or(Error::Limit(LimitKind::ValueCells))
        })?;
        // One instruction may snapshot a replacement before dropping its old register.
        let reserved = register_bytes
            .checked_add(
                block
                    .registers()
                    .iter()
                    .map(|width| width.bytes())
                    .max()
                    .unwrap_or(0),
            )
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        self.simd_budget(reserved)?;
        let mut registers = vec![None; block.registers().len()];
        for instruction in block.instructions() {
            self.step(depth)?;
            match instruction {
                SimdInstruction::DebugTrap | SimdInstruction::Arm64DebugTrap => {
                    return Err(Error::RuntimeTrap.into());
                }
                SimdInstruction::Load {
                    destination,
                    address,
                    ..
                } => {
                    let address = self.value(address, depth + 1)?.pointer()?.clone();
                    self.simd_budget(reserved)?;
                    let width = block
                        .width(*destination)
                        .map_err(|error| Error::IrValidation(error.to_string()))?;
                    let work = self.memory.intrinsic_work_cost(
                        self.provider.types(),
                        &[(&address, false)],
                        width.bytes(),
                    )?;
                    self.charge_work(
                        usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
                    )?;
                    let bytes = self.memory.simd_read_bytes(
                        self.provider.types(),
                        &address,
                        width.bytes(),
                    )?;
                    registers[destination.index()] = Some(bytes);
                }
                SimdInstruction::Store {
                    source,
                    address,
                    ..
                } => {
                    let address = self.value(address, depth + 1)?.pointer()?.clone();
                    self.simd_budget(reserved)?;
                    let bytes = register(&registers, *source)?;
                    let work = self.memory.intrinsic_work_cost(
                        self.provider.types(),
                        &[(&address, true)],
                        bytes.len(),
                    )?;
                    self.charge_work(
                        usize::try_from(work).map_err(|_| Error::Limit(LimitKind::Fuel))?,
                    )?;
                    self.memory.simd_write_bytes_reserved(
                        self.provider.types(),
                        &address,
                        bytes,
                        reserved,
                    )?;
                }
                SimdInstruction::Add {
                    destination,
                    left,
                    right,
                    interpretation,
                } => {
                    self.simd_budget(reserved)?;
                    let width = block
                        .width(*destination)
                        .map_err(|error| Error::IrValidation(error.to_string()))?;
                    self.charge_work(width.bytes())?;
                    let left = register(&registers, *left)?;
                    let right = register(&registers, *right)?;
                    let bytes = match interpretation {
                        SimdInterpretation::F32 => add_f32(left, right, target.endian)?,
                        SimdInterpretation::U8 => add_u8(left, right)?,
                    };
                    registers[destination.index()] = Some(bytes);
                }
            }
        }
        Ok(Control::Next)
    }

    fn simd_budget(&self, reserved: usize) -> Result<()> {
        self.memory
            .value_cells()
            .checked_add(reserved)
            .filter(|total| *total <= self.limits.value_cells)
            .ok_or(Error::Limit(LimitKind::ValueCells))?;
        Ok(())
    }
}

fn register(
    registers: &[Option<Vec<u8>>],
    register: SimdRegisterId,
) -> std::result::Result<&[u8], Error> {
    registers
        .get(register.index())
        .and_then(Option::as_deref)
        .ok_or(Error::InvalidIr("SIMD reads an unavailable register"))
}

fn add_f32(left: &[u8], right: &[u8], endian: Endian) -> std::result::Result<Vec<u8>, Error> {
    validate_operands(left, right)?;
    let mut output = Vec::with_capacity(left.len());
    for (left, right) in left.as_chunks::<4>().0.iter().zip(right.as_chunks::<4>().0) {
        let left = FloatValue::F32(bits(left, endian));
        let right = FloatValue::F32(bits(right, endian));
        let result = left.binary(FloatOp::Add, right)?.bits() as u32;
        let bytes = match endian {
            Endian::Little => result.to_le_bytes(),
            Endian::Big => result.to_be_bytes(),
        };
        output.extend_from_slice(&bytes);
    }
    Ok(output)
}

fn add_u8(left: &[u8], right: &[u8]) -> std::result::Result<Vec<u8>, Error> {
    validate_operands(left, right)?;
    Ok(left
        .iter()
        .zip(right)
        .map(|(&a, &b)| a.wrapping_add(b))
        .collect())
}

fn validate_operands(left: &[u8], right: &[u8]) -> std::result::Result<(), Error> {
    if left.len() != right.len() || !matches!(left.len(), 16 | 32) {
        Err(Error::InvalidIr(
            "SIMD operands must have matching 128-bit or 256-bit widths",
        ))
    } else {
        Ok(())
    }
}

fn bits(bytes: &[u8; 4], endian: Endian) -> u32 {
    match endian {
        Endian::Little => u32::from_le_bytes(*bytes),
        Endian::Big => u32::from_be_bytes(*bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn floats(lanes: &[f32], endian: Endian) -> Vec<u8> {
        lanes
            .iter()
            .flat_map(|value| match endian {
                Endian::Little => value.to_bits().to_le_bytes(),
                Endian::Big => value.to_bits().to_be_bytes(),
            })
            .collect()
    }

    #[test]
    fn float_add_rounds_each_lane_at_both_register_widths() {
        for endian in [Endian::Little, Endian::Big] {
            for lanes in [4, 8] {
                let left = [16_777_216.0, -8.0, 0.5, -0.0, 3.5, 1.25, -2.5, 0.0];
                let right = [1.0, 7.0, 0.25, -0.0, 0.25, -2.0, 2.5, 1.0];
                let expected = [16_777_216.0, -1.0, 0.75, -0.0, 3.75, -0.75, 0.0, 1.0];
                assert_eq!(
                    add_f32(
                        &floats(&left[..lanes], endian),
                        &floats(&right[..lanes], endian),
                        endian
                    )
                    .unwrap(),
                    floats(&expected[..lanes], endian)
                );
            }
        }
    }

    #[test]
    fn float_add_preserves_ieee_special_values() {
        let left = floats(
            &[
                f32::INFINITY,
                f32::INFINITY,
                f32::NAN,
                f32::MIN_POSITIVE / 2.0,
            ],
            Endian::Little,
        );
        let right = floats(
            &[1.0, f32::NEG_INFINITY, 1.0, f32::MIN_POSITIVE / 2.0],
            Endian::Little,
        );
        let output = add_f32(&left, &right, Endian::Little).unwrap();
        let lanes: Vec<_> = output
            .as_chunks::<4>()
            .0
            .iter()
            .map(|lane| f32::from_bits(bits(lane, Endian::Little)))
            .collect();
        assert_eq!(lanes[0], f32::INFINITY);
        assert!(lanes[1].is_nan());
        assert!(lanes[2].is_nan());
        assert_eq!(lanes[3], f32::MIN_POSITIVE);
    }

    #[test]
    fn byte_add_wraps_without_carrying_between_lanes() {
        let mut left = vec![255; 16];
        left[1] = 1;
        let mut right = vec![1; 16];
        right[1] = 254;
        let mut expected = vec![0; 16];
        expected[1] = 255;
        assert_eq!(add_u8(&left, &right).unwrap(), expected);
    }

    #[test]
    fn arithmetic_rejects_width_mismatches() {
        assert!(add_u8(&[0; 16], &[0; 32]).is_err());
        assert!(add_f32(&[0; 4], &[0; 4], Endian::Little).is_err());
    }
}

#[cfg(test)]
mod execution_tests;
