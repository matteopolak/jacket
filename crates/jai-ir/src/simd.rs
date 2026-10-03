//! Closed x86 SIMD register operations; registers never acquire source nominal types.
use crate::ValueExpr;
use jai_types::{TypeKind, TypeView};
use std::{
    fmt,
    sync::atomic::{AtomicU64, Ordering},
};

pub const MAX_SIMD_REGISTERS: usize = 256;
pub const MAX_SIMD_INSTRUCTIONS: usize = 4096;
static NEXT_ARENA: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimdWidth {
    X128,
    Y256,
}
impl SimdWidth {
    pub fn bytes(self) -> usize {
        match self {
            Self::X128 => 16,
            Self::Y256 => 32,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimdInterpretation {
    F32,
    U8,
}
impl SimdInterpretation {
    pub fn lanes(self, width: SimdWidth) -> usize {
        width.bytes()
            / match self {
                Self::F32 => 4,
                Self::U8 => 1,
            }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SimdFeatures {
    pub avx: bool,
    pub avx2: bool,
}
impl SimdFeatures {
    pub fn requires_avx(self) -> bool {
        self.avx || self.avx2
    }
}

/// Local register identities carry their builder arena as well as their ordinal.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SimdRegisterId {
    arena: u64,
    index: usize,
}
impl SimdRegisterId {
    pub fn index(self) -> usize {
        self.index
    }
}
#[derive(Clone, Debug)]
pub enum SimdInstruction {
    DebugTrap,
    Arm64DebugTrap,
    Load {
        destination: SimdRegisterId,
        interpretation: SimdInterpretation,
        address: ValueExpr,
    },
    Store {
        source: SimdRegisterId,
        interpretation: SimdInterpretation,
        address: ValueExpr,
    },
    Add {
        destination: SimdRegisterId,
        left: SimdRegisterId,
        right: SimdRegisterId,
        interpretation: SimdInterpretation,
    },
}
#[derive(Clone, Debug)]
pub struct SimdBlock {
    arena: u64,
    features: SimdFeatures,
    registers: Box<[SimdWidth]>,
    instructions: Vec<SimdInstruction>,
}
impl SimdBlock {
    pub fn features(&self) -> SimdFeatures {
        self.features
    }
    pub fn registers(&self) -> &[SimdWidth] {
        &self.registers
    }
    pub fn instructions(&self) -> &[SimdInstruction] {
        &self.instructions
    }
    pub(crate) fn take_instructions(&mut self) -> Vec<SimdInstruction> {
        std::mem::take(&mut self.instructions)
    }
    pub fn width(&self, register: SimdRegisterId) -> Result<SimdWidth, SimdError> {
        checked_width(self.arena, &self.registers, register)
    }
    /// Recheck staging data at the immutable program boundary. Address children
    /// must additionally pass the caller's ordinary expression verifier.
    pub fn validate(&self, types: &dyn TypeView) -> Result<(), SimdError> {
        if self.registers.len() > MAX_SIMD_REGISTERS {
            return Err(SimdError::RegisterLimit);
        }
        if self.instructions.len() > MAX_SIMD_INSTRUCTIONS {
            return Err(SimdError::InstructionLimit);
        }
        let mut initialized = vec![false; self.registers.len()];
        for instruction in &self.instructions {
            validate_instruction(
                self.arena,
                self.features,
                &self.registers,
                &initialized,
                instruction,
                types,
            )?;
            if let SimdInstruction::Load { destination, .. }
            | SimdInstruction::Add { destination, .. } = instruction
            {
                initialized[destination.index()] = true;
            }
        }
        Ok(())
    }
}
impl Drop for SimdBlock {
    fn drop(&mut self) {
        crate::disposal::simd_instructions(self.take_instructions());
    }
}

#[derive(Debug)]
pub enum SimdError {
    ArenaExhausted,
    RegisterLimit,
    InstructionLimit,
    ForeignRegister,
    UnknownRegister,
    WidthMismatch,
    UninitializedRegister,
    MissingAvx,
    MissingAvx2,
    PointerRequired,
}
impl fmt::Display for SimdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ArenaExhausted => "SIMD block identity space exhausted",
            Self::RegisterLimit => "SIMD block exceeds 256 registers",
            Self::InstructionLimit => "SIMD block exceeds 4096 instructions",
            Self::ForeignRegister => "SIMD register belongs to a different block",
            Self::UnknownRegister => "SIMD register is not declared in this block",
            Self::WidthMismatch => "SIMD register widths do not match",
            Self::UninitializedRegister => "SIMD instruction reads an uninitialized register",
            Self::MissingAvx => "256-bit SIMD requires declared AVX support",
            Self::MissingAvx2 => "256-bit integer SIMD requires declared AVX2 support",
            Self::PointerRequired => "SIMD memory operand requires a typed pointer",
        })
    }
}
impl std::error::Error for SimdError {}

pub struct SimdBuilder {
    arena: u64,
    features: SimdFeatures,
    registers: Vec<SimdWidth>,
    initialized: Vec<bool>,
    instructions: Vec<SimdInstruction>,
}
impl SimdBuilder {
    pub fn new(features: SimdFeatures) -> Result<Self, SimdError> {
        let arena = NEXT_ARENA
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| SimdError::ArenaExhausted)?;
        Ok(Self {
            arena,
            features,
            registers: vec![],
            initialized: vec![],
            instructions: vec![],
        })
    }
    pub fn register(&mut self, width: SimdWidth) -> Result<SimdRegisterId, SimdError> {
        if self.registers.len() == MAX_SIMD_REGISTERS {
            return Err(SimdError::RegisterLimit);
        }
        let id = SimdRegisterId {
            arena: self.arena,
            index: self.registers.len(),
        };
        self.registers.push(width);
        self.initialized.push(false);
        Ok(id)
    }
    pub fn instruction(
        &mut self,
        instruction: SimdInstruction,
        types: &dyn TypeView,
    ) -> Result<(), SimdError> {
        if self.instructions.len() == MAX_SIMD_INSTRUCTIONS {
            crate::disposal::simd_instructions(vec![instruction]);
            return Err(SimdError::InstructionLimit);
        }
        if let Err(error) = validate_instruction(
            self.arena,
            self.features,
            &self.registers,
            &self.initialized,
            &instruction,
            types,
        ) {
            crate::disposal::simd_instructions(vec![instruction]);
            return Err(error);
        }
        if let SimdInstruction::Load { destination, .. }
        | SimdInstruction::Add { destination, .. } = &instruction
        {
            self.initialized[destination.index()] = true;
        }
        self.instructions.push(instruction);
        Ok(())
    }
    pub fn finish(mut self) -> SimdBlock {
        SimdBlock {
            arena: self.arena,
            features: self.features,
            registers: std::mem::take(&mut self.registers).into(),
            instructions: std::mem::take(&mut self.instructions),
        }
    }
}
impl Drop for SimdBuilder {
    fn drop(&mut self) {
        crate::disposal::simd_instructions(std::mem::take(&mut self.instructions));
    }
}
fn checked_width(
    arena: u64,
    widths: &[SimdWidth],
    register: SimdRegisterId,
) -> Result<SimdWidth, SimdError> {
    if register.arena != arena {
        return Err(SimdError::ForeignRegister);
    }
    widths
        .get(register.index)
        .copied()
        .ok_or(SimdError::UnknownRegister)
}
fn validate_instruction(
    arena: u64,
    features: SimdFeatures,
    widths: &[SimdWidth],
    initialized: &[bool],
    instruction: &SimdInstruction,
    types: &dyn TypeView,
) -> Result<(), SimdError> {
    let (register, interpretation, address) = match instruction {
        SimdInstruction::DebugTrap | SimdInstruction::Arm64DebugTrap => return Ok(()),
        SimdInstruction::Load {
            destination,
            interpretation,
            address,
        } => (*destination, *interpretation, Some(address)),
        SimdInstruction::Store {
            source,
            interpretation,
            address,
        } => (*source, *interpretation, Some(address)),
        SimdInstruction::Add {
            destination,
            interpretation,
            ..
        } => (*destination, *interpretation, None),
    };
    let width = checked_width(arena, widths, register)?;
    if width == SimdWidth::Y256 {
        if !features.requires_avx() {
            return Err(SimdError::MissingAvx);
        }
        if interpretation == SimdInterpretation::U8 && !features.avx2 {
            return Err(SimdError::MissingAvx2);
        }
    }
    if let Some(address) = address
        && !matches!(types.kind(address.type_id(types)), Ok(TypeKind::Pointer(_)))
    {
        return Err(SimdError::PointerRequired);
    }
    let read = |id: SimdRegisterId| -> Result<(), SimdError> {
        if checked_width(arena, widths, id)? != width {
            return Err(SimdError::WidthMismatch);
        }
        if !initialized[id.index()] {
            return Err(SimdError::UninitializedRegister);
        }
        Ok(())
    };
    match instruction {
        SimdInstruction::Load { .. }
        | SimdInstruction::DebugTrap
        | SimdInstruction::Arm64DebugTrap => {}
        SimdInstruction::Store { source, .. } => read(*source)?,
        SimdInstruction::Add { left, right, .. } => {
            read(*left)?;
            read(*right)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use jai_types::{IntegerType, ScalarType, TypeRegistry};
    fn types() -> (jai_types::Types, ValueExpr) {
        let mut types = TypeRegistry::new();
        let byte = types.scalar(ScalarType::Int(IntegerType::U8));
        let pointer = types.pointer(byte).unwrap();
        (types.freeze().unwrap(), ValueExpr::Zero(pointer))
    }
    #[test]
    fn block_registers_are_branded_initialized_and_width_checked() {
        let (types, address) = types();
        let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
        let register = builder.register(SimdWidth::X128).unwrap();
        let add = SimdInstruction::Add {
            destination: register,
            left: register,
            right: register,
            interpretation: SimdInterpretation::F32,
        };
        assert!(matches!(
            builder.instruction(add.clone(), &types),
            Err(SimdError::UninitializedRegister)
        ));
        builder
            .instruction(
                SimdInstruction::Load {
                    destination: register,
                    interpretation: SimdInterpretation::F32,
                    address,
                },
                &types,
            )
            .unwrap();
        builder.instruction(add, &types).unwrap();
        let other = SimdBuilder::new(SimdFeatures::default())
            .unwrap()
            .register(SimdWidth::X128)
            .unwrap();
        assert!(matches!(
            builder.instruction(
                SimdInstruction::Add {
                    destination: register,
                    left: register,
                    right: other,
                    interpretation: SimdInterpretation::F32
                },
                &types
            ),
            Err(SimdError::ForeignRegister)
        ));
        builder.finish().validate(&types).unwrap();
    }
    #[test]
    fn wide_operations_require_their_declared_isa_features() {
        let (types, address) = types();
        for (features, interpretation, expected) in [
            (
                SimdFeatures::default(),
                SimdInterpretation::F32,
                "AVX support",
            ),
            (
                SimdFeatures {
                    avx: true,
                    avx2: false,
                },
                SimdInterpretation::U8,
                "AVX2 support",
            ),
        ] {
            let mut builder = SimdBuilder::new(features).unwrap();
            let register = builder.register(SimdWidth::Y256).unwrap();
            let error = builder
                .instruction(
                    SimdInstruction::Load {
                        destination: register,
                        interpretation,
                        address: address.clone(),
                    },
                    &types,
                )
                .unwrap_err();
            assert!(error.to_string().contains(expected));
        }
        let mut builder = SimdBuilder::new(SimdFeatures {
            avx: false,
            avx2: true,
        })
        .unwrap();
        let register = builder.register(SimdWidth::Y256).unwrap();
        builder
            .instruction(
                SimdInstruction::Load {
                    destination: register,
                    interpretation: SimdInterpretation::U8,
                    address,
                },
                &types,
            )
            .unwrap();
        builder.finish().validate(&types).unwrap();
    }
    #[test]
    fn mixed_width_reads_and_failed_initialization_are_rejected() {
        let (types, address) = types();
        let mut builder = SimdBuilder::new(SimdFeatures {
            avx: true,
            avx2: false,
        })
        .unwrap();
        let narrow = builder.register(SimdWidth::X128).unwrap();
        let wide = builder.register(SimdWidth::Y256).unwrap();
        assert!(matches!(
            builder.instruction(
                SimdInstruction::Load {
                    destination: narrow,
                    interpretation: SimdInterpretation::F32,
                    address: ValueExpr::Bool(crate::BoolExpr::Constant(true))
                },
                &types
            ),
            Err(SimdError::PointerRequired)
        ));
        assert!(matches!(
            builder.instruction(
                SimdInstruction::Add {
                    destination: narrow,
                    left: narrow,
                    right: narrow,
                    interpretation: SimdInterpretation::F32
                },
                &types
            ),
            Err(SimdError::UninitializedRegister)
        ));
        for destination in [narrow, wide] {
            builder
                .instruction(
                    SimdInstruction::Load {
                        destination,
                        interpretation: SimdInterpretation::F32,
                        address: address.clone(),
                    },
                    &types,
                )
                .unwrap();
        }
        assert!(matches!(
            builder.instruction(
                SimdInstruction::Add {
                    destination: narrow,
                    left: narrow,
                    right: wide,
                    interpretation: SimdInterpretation::F32
                },
                &types
            ),
            Err(SimdError::WidthMismatch)
        ));
        builder.finish().validate(&types).unwrap();
    }
    #[test]
    fn register_and_instruction_growth_is_bounded_before_mutation() {
        let (types, address) = types();
        let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
        let register = builder.register(SimdWidth::X128).unwrap();
        for _ in 1..MAX_SIMD_REGISTERS {
            builder.register(SimdWidth::X128).unwrap();
        }
        assert!(matches!(
            builder.register(SimdWidth::X128),
            Err(SimdError::RegisterLimit)
        ));
        builder
            .instruction(
                SimdInstruction::Load {
                    destination: register,
                    interpretation: SimdInterpretation::U8,
                    address,
                },
                &types,
            )
            .unwrap();
        let add = SimdInstruction::Add {
            destination: register,
            left: register,
            right: register,
            interpretation: SimdInterpretation::U8,
        };
        for _ in 1..MAX_SIMD_INSTRUCTIONS {
            builder.instruction(add.clone(), &types).unwrap();
        }
        assert!(matches!(
            builder.instruction(add, &types),
            Err(SimdError::InstructionLimit)
        ));
        let block = builder.finish();
        assert_eq!(block.instructions().len(), MAX_SIMD_INSTRUCTIONS);
        block.validate(&types).unwrap();
    }

    #[test]
    fn rejected_and_abandoned_address_trees_drop_iteratively_on_a_small_stack() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                let (types, address) = types();
                let pointer = address.type_id(&types);
                let deep_address = || {
                    let mut value = ValueExpr::Zero(pointer);
                    for _ in 0..20_000 {
                        value = ValueExpr::PointerCast {
                            value: Box::new(value),
                            ty: pointer,
                            mode: jai_types::CastMode::Checked,
                        };
                    }
                    value
                };
                for finish in [false, true] {
                    let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
                    let register = builder.register(SimdWidth::X128).unwrap();
                    builder
                        .instruction(
                            SimdInstruction::Load {
                                destination: register,
                                interpretation: SimdInterpretation::U8,
                                address: deep_address(),
                            },
                            &types,
                        )
                        .unwrap();
                    if finish {
                        drop(builder.finish());
                    }
                }
                let mut other = SimdBuilder::new(SimdFeatures::default()).unwrap();
                let foreign = other.register(SimdWidth::X128).unwrap();
                let mut builder = SimdBuilder::new(SimdFeatures::default()).unwrap();
                assert!(matches!(
                    builder.instruction(
                        SimdInstruction::Load {
                            destination: foreign,
                            interpretation: SimdInterpretation::U8,
                            address: deep_address(),
                        },
                        &types,
                    ),
                    Err(SimdError::ForeignRegister)
                ));
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
