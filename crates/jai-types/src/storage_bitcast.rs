//! Target-bound storage extents for explicit byte reinterpretation.
use crate::{LayoutEngine, LayoutError, LayoutPolicy, TypeError, TypeId, TypeView};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StorageBitcastStrength {
    /// Lowercase `force` requires equal source and destination byte extents.
    EqualSize,
    /// Uppercase `FORCE` may read a smaller destination from the source prefix.
    Prefix,
}
impl StorageBitcastStrength {
    pub const fn compiler_flag(self) -> u32 {
        match self {
            Self::EqualSize => 0x80,
            Self::Prefix => 0x100,
        }
    }
}

/// Layout evidence only. Initialized bytes, relocations, and value validity must
/// still be checked by the consumer; this proof cannot issue pointer provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct StorageBitcast {
    source: TypeId,
    target: TypeId,
    policy: LayoutPolicy,
    strength: StorageBitcastStrength,
    source_size: u64,
    target_size: u64,
    scratch_alignment: u32,
}

#[derive(Debug)]
pub enum StorageBitcastError {
    Type(TypeError),
    Layout(LayoutError),
    SizeMismatch {
        strength: StorageBitcastStrength,
        source_size: u64,
        target_size: u64,
    },
    TargetPolicyMismatch,
    StorageLayoutChanged,
}
impl From<TypeError> for StorageBitcastError {
    fn from(error: TypeError) -> Self {
        Self::Type(error)
    }
}
impl From<LayoutError> for StorageBitcastError {
    fn from(error: LayoutError) -> Self {
        Self::Layout(error)
    }
}
impl fmt::Display for StorageBitcastError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Type(error) => error.fmt(formatter),
            Self::Layout(error) => error.fmt(formatter),
            Self::SizeMismatch {
                strength,
                source_size,
                target_size,
            } => {
                let constraint = match strength {
                    StorageBitcastStrength::EqualSize => "equal source and destination sizes",
                    StorageBitcastStrength::Prefix => "a destination no larger than the source",
                };
                write!(
                    formatter,
                    "storage cast requires {constraint} (source {source_size} bytes, destination {target_size} bytes)"
                )
            }
            Self::TargetPolicyMismatch => {
                formatter.write_str("storage cast belongs to another target layout policy")
            }
            Self::StorageLayoutChanged => {
                formatter.write_str("storage cast proof does not match the current type layouts")
            }
        }
    }
}
impl std::error::Error for StorageBitcastError {}

impl StorageBitcast {
    pub fn prove(
        types: &dyn TypeView,
        policy: LayoutPolicy,
        source: TypeId,
        target: TypeId,
        strength: StorageBitcastStrength,
    ) -> Result<Self, StorageBitcastError> {
        // Check arena ownership before interpreting either target layout.
        types.kind(source)?;
        types.kind(target)?;
        let mut engine = LayoutEngine::new(types, policy);
        let source_layout = engine.layout(source)?.clone();
        let target_layout = engine.layout(target)?.clone();
        let compatible = match strength {
            StorageBitcastStrength::EqualSize => source_layout.size == target_layout.size,
            StorageBitcastStrength::Prefix => target_layout.size <= source_layout.size,
        };
        if !compatible {
            return Err(StorageBitcastError::SizeMismatch {
                strength,
                source_size: source_layout.size,
                target_size: target_layout.size,
            });
        }
        Ok(Self {
            source,
            target,
            policy,
            strength,
            source_size: source_layout.size,
            target_size: target_layout.size,
            scratch_alignment: source_layout.alignment.max(target_layout.alignment),
        })
    }

    pub fn revalidate(
        self,
        types: &dyn TypeView,
        policy: LayoutPolicy,
    ) -> Result<(), StorageBitcastError> {
        types.kind(self.source)?;
        types.kind(self.target)?;
        if self.policy != policy {
            return Err(StorageBitcastError::TargetPolicyMismatch);
        }
        let current = Self::prove(types, policy, self.source, self.target, self.strength)?;
        if current != self {
            return Err(StorageBitcastError::StorageLayoutChanged);
        }
        Ok(())
    }
    pub fn source_type(self) -> TypeId {
        self.source
    }
    pub fn target_type(self) -> TypeId {
        self.target
    }
    pub fn layout_policy(self) -> LayoutPolicy {
        self.policy
    }
    pub fn strength(self) -> StorageBitcastStrength {
        self.strength
    }
    pub fn source_size(self) -> u64 {
        self.source_size
    }
    pub fn target_size(self) -> u64 {
        self.target_size
    }
    pub fn scratch_alignment(self) -> u32 {
        self.scratch_alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{IntegerType as I, RecordKind, ScalarLayout, ScalarType, TypeRegistry};
    fn integer(types: &TypeRegistry, ty: I) -> TypeId {
        types.scalar(ScalarType::Int(ty))
    }
    fn record(types: &mut TypeRegistry, fields: &[TypeId]) -> TypeId {
        let ty = types.reserve_record(RecordKind::Struct);
        types.define_record(ty, fields.to_vec()).unwrap();
        ty
    }
    fn ilp32() -> LayoutPolicy {
        let lp64 = LayoutPolicy::lp64();
        LayoutPolicy::new(
            ScalarLayout::new(4, 4),
            [
                lp64.integer(I::U8),
                lp64.integer(I::U16),
                lp64.integer(I::U32),
                lp64.integer(I::U64),
            ],
            [
                lp64.float(crate::FloatType::F32),
                lp64.float(crate::FloatType::F64),
            ],
            lp64.boolean(),
        )
        .unwrap()
    }
    #[test]
    fn signed_and_unsigned_int128_records_keep_distinct_nominal_identities() {
        let mut types = TypeRegistry::new();
        let word = integer(&types, I::U64);
        let signed = integer(&types, I::S64);
        let source = record(&mut types, &[word, signed]);
        let target = record(&mut types, &[word, word]);
        let proof = StorageBitcast::prove(
            &types,
            LayoutPolicy::lp64(),
            source,
            target,
            StorageBitcastStrength::EqualSize,
        )
        .unwrap();
        assert_ne!(source, target);
        assert_eq!(proof.source_type(), source);
        assert_eq!(proof.target_type(), target);
        assert_eq!(
            (
                proof.source_size(),
                proof.target_size(),
                proof.scratch_alignment()
            ),
            (16, 16, 8)
        );
        proof
            .revalidate(&types.freeze().unwrap(), LayoutPolicy::lp64())
            .unwrap();
    }
    #[test]
    fn scratch_alignment_covers_a_more_aligned_destination_and_cannot_be_weakened() {
        let mut types = TypeRegistry::new();
        let byte = integer(&types, I::U8);
        let bytes = types.fixed_array(byte, 8).unwrap();
        let word = integer(&types, I::U64);
        let mut proof = StorageBitcast::prove(
            &types,
            LayoutPolicy::lp64(),
            bytes,
            word,
            StorageBitcastStrength::EqualSize,
        )
        .unwrap();
        assert_eq!(proof.scratch_alignment(), 8);
        let inverse = StorageBitcast::prove(
            &types,
            LayoutPolicy::lp64(),
            word,
            bytes,
            StorageBitcastStrength::EqualSize,
        )
        .unwrap();
        assert_eq!(inverse.scratch_alignment(), 8);
        proof.scratch_alignment = 1;
        assert!(matches!(
            proof.revalidate(&types, LayoutPolicy::lp64()),
            Err(StorageBitcastError::StorageLayoutChanged)
        ));
    }
    #[test]
    fn uppercase_force_admits_only_a_destination_prefix() {
        let types = TypeRegistry::new();
        let byte = integer(&types, I::U8);
        let word = integer(&types, I::U64);
        let proof = StorageBitcast::prove(
            &types,
            LayoutPolicy::lp64(),
            word,
            byte,
            StorageBitcastStrength::Prefix,
        )
        .unwrap();
        assert_eq!((proof.source_size(), proof.target_size()), (8, 1));
        assert!(matches!(
            StorageBitcast::prove(
                &types,
                LayoutPolicy::lp64(),
                word,
                byte,
                StorageBitcastStrength::EqualSize
            ),
            Err(StorageBitcastError::SizeMismatch { .. })
        ));
        assert!(matches!(
            StorageBitcast::prove(
                &types,
                LayoutPolicy::lp64(),
                byte,
                word,
                StorageBitcastStrength::Prefix
            ),
            Err(StorageBitcastError::SizeMismatch { .. })
        ));
        assert_eq!(StorageBitcastStrength::EqualSize.compiler_flag(), 0x80);
        assert_eq!(StorageBitcastStrength::Prefix.compiler_flag(), 0x100);
    }
    #[test]
    fn pointer_width_and_target_policy_remain_bound_to_the_proof() {
        let mut types = TypeRegistry::new();
        let byte = integer(&types, I::U8);
        let pointer = types.pointer(byte).unwrap();
        let word = integer(&types, I::U64);
        let proof = StorageBitcast::prove(
            &types,
            LayoutPolicy::lp64(),
            pointer,
            word,
            StorageBitcastStrength::EqualSize,
        )
        .unwrap();
        assert!(matches!(
            proof.revalidate(&types, ilp32()),
            Err(StorageBitcastError::TargetPolicyMismatch)
        ));
        assert!(matches!(
            StorageBitcast::prove(
                &types,
                ilp32(),
                pointer,
                word,
                StorageBitcastStrength::EqualSize
            ),
            Err(StorageBitcastError::SizeMismatch {
                source_size: 4,
                target_size: 8,
                ..
            })
        ));
        let narrow = integer(&types, I::U32);
        assert_eq!(
            StorageBitcast::prove(
                &types,
                ilp32(),
                pointer,
                narrow,
                StorageBitcastStrength::EqualSize
            )
            .unwrap()
            .target_size(),
            4
        );
    }
    #[test]
    fn foreign_arenas_do_not_reuse_equal_size_proofs() {
        let types = TypeRegistry::new();
        let source = integer(&types, I::S64);
        let target = integer(&types, I::U64);
        let proof = StorageBitcast::prove(
            &types,
            LayoutPolicy::lp64(),
            source,
            target,
            StorageBitcastStrength::EqualSize,
        )
        .unwrap();
        let foreign = TypeRegistry::new();
        let foreign_target = integer(&foreign, I::U64);
        assert!(matches!(
            StorageBitcast::prove(
                &types,
                LayoutPolicy::lp64(),
                source,
                foreign_target,
                StorageBitcastStrength::EqualSize,
            ),
            Err(StorageBitcastError::Type(TypeError::ForeignType(id))) if id == foreign_target
        ));
        assert!(
            matches!(proof.revalidate(&foreign, LayoutPolicy::lp64()), Err(StorageBitcastError::Type(TypeError::ForeignType(id))) if id==source)
        );
        assert!(
            matches!(proof.revalidate(&foreign.freeze().unwrap(), LayoutPolicy::lp64()), Err(StorageBitcastError::Type(TypeError::ForeignType(id))) if id==source)
        );
    }
    #[test]
    fn readiness_and_zero_byte_runtime_values_are_not_guessed() {
        let mut types = TypeRegistry::new();
        let empty = record(&mut types, &[]);
        let pending = types.reserve_record(RecordKind::Struct);
        assert!(matches!(
            StorageBitcast::prove(
                &types,
                LayoutPolicy::lp64(),
                pending,
                empty,
                StorageBitcastStrength::EqualSize
            ),
            Err(StorageBitcastError::Layout(LayoutError::Type(
                TypeError::Incomplete(_)
            )))
        ));
        types.define_record(pending, []).unwrap();
        let proof = StorageBitcast::prove(
            &types,
            LayoutPolicy::lp64(),
            pending,
            empty,
            StorageBitcastStrength::EqualSize,
        )
        .unwrap();
        assert_eq!(
            (
                proof.source_size(),
                proof.target_size(),
                proof.scratch_alignment()
            ),
            (0, 0, 1)
        );
        for no_storage in [types.code_type(), types.void()] {
            assert!(
                matches!(StorageBitcast::prove(&types, LayoutPolicy::lp64(), no_storage, empty, StorageBitcastStrength::Prefix), Err(StorageBitcastError::Layout(LayoutError::Unsized(id))) if id==no_storage)
            );
            assert!(
                matches!(StorageBitcast::prove(&types, LayoutPolicy::lp64(), empty, no_storage, StorageBitcastStrength::EqualSize), Err(StorageBitcastError::Layout(LayoutError::Unsized(id))) if id==no_storage)
            );
        }
    }
}
