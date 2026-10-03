//! Preserve source cast policies and their compiler flag identities.
use crate::{CastMode, StorageBitcastStrength};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CastModifier {
    NoBoundsCheck,
    Truncate,
    Force(StorageBitcastStrength),
}

impl CastModifier {
    pub const fn compiler_flag(self) -> u32 {
        match self {
            Self::NoBoundsCheck => 0x8,
            Self::Truncate => 0x10,
            Self::Force(strength) => strength.compiler_flag(),
        }
    }
}

/// At most one explicit policy can be selected; invalid flag combinations
/// cannot be retained after a parser reports an error.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct CastModifiers(Option<CastModifier>);

impl CastModifiers {
    pub const fn checked() -> Self {
        Self(None)
    }

    pub fn insert(&mut self, modifier: CastModifier) -> Result<(), CastModifiersError> {
        match self.0 {
            Some(previous) if previous == modifier => Err(CastModifiersError::Duplicate(modifier)),
            Some(previous) => Err(CastModifiersError::Conflicting {
                previous,
                requested: modifier,
            }),
            None => {
                self.0 = Some(modifier);
                Ok(())
            }
        }
    }

    pub const fn mode(self) -> CastMode {
        match self.0 {
            None => CastMode::Checked,
            Some(CastModifier::NoBoundsCheck) => CastMode::Unchecked,
            Some(CastModifier::Truncate) => CastMode::Truncate,
            Some(CastModifier::Force(strength)) => CastMode::Force(strength),
        }
    }

    pub const fn compiler_flags(self) -> u32 {
        match self.0 {
            None => 0,
            Some(modifier) => modifier.compiler_flag(),
        }
    }

    pub const fn from_mode(mode: CastMode) -> Self {
        Self(match mode {
            CastMode::Checked => None,
            CastMode::Unchecked => Some(CastModifier::NoBoundsCheck),
            CastMode::Truncate => Some(CastModifier::Truncate),
            CastMode::Force(strength) => Some(CastModifier::Force(strength)),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CastModifiersError {
    Duplicate(CastModifier),
    Conflicting {
        previous: CastModifier,
        requested: CastModifier,
    },
}

impl fmt::Display for CastModifiersError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Duplicate(_) => f.write_str("duplicate cast modifier"),
            Self::Conflicting { .. } => f.write_str("conflicting cast modifiers"),
        }
    }
}

impl std::error::Error for CastModifiersError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policies_round_trip_without_collapsing_compiler_flags() {
        for (mode, flags) in [
            (CastMode::Checked, 0),
            (CastMode::Unchecked, 0x8),
            (CastMode::Truncate, 0x10),
            (CastMode::Force(StorageBitcastStrength::EqualSize), 0x80),
            (CastMode::Force(StorageBitcastStrength::Prefix), 0x100),
        ] {
            let modifiers = CastModifiers::from_mode(mode);
            assert_eq!(modifiers.mode(), mode);
            assert_eq!(modifiers.compiler_flags(), flags);
        }
        assert_eq!(CastModifiers::default(), CastModifiers::checked());
    }

    #[test]
    fn duplicate_and_conflicting_insertions_preserve_the_selected_policy() {
        for (first, second) in [
            (CastModifier::NoBoundsCheck, CastModifier::Truncate),
            (CastModifier::Truncate, CastModifier::NoBoundsCheck),
        ] {
            let mut modifiers = CastModifiers::checked();
            modifiers.insert(first).unwrap();
            let selected = modifiers;
            assert_eq!(
                modifiers.insert(first),
                Err(CastModifiersError::Duplicate(first))
            );
            assert_eq!(modifiers, selected);
            assert_eq!(
                modifiers.insert(second),
                Err(CastModifiersError::Conflicting {
                    previous: first,
                    requested: second,
                })
            );
            assert_eq!(modifiers, selected);
        }
    }
}
