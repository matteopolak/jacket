//! Owned compiler events cross scheduler boundaries; VM pointers never do.
use super::WorkspaceId;
use jai_types::{FieldId, TypeId};

/// Source nominal fields verified once by the semantic Compiler API binder.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompilerMessageSchema {
    pub message: TypeId,
    pub kind: FieldId,
    pub workspace: FieldId,
    pub phase: TypeId,
    pub phase_base: FieldId,
    pub phase_kind: FieldId,
    pub pending_count: FieldId,
    pub complete: TypeId,
    pub complete_base: FieldId,
    pub completion_error: FieldId,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct InterceptFlags(u32);
impl InterceptFlags {
    pub const NONE: Self = Self(0);
    pub const SKIP_ALL: Self = Self(0x3f);
    pub const PERFORMANCE_POLYMORPHS: Self = Self(0x1000);
    pub const PERFORMANCE_RUNS: Self = Self(0x2000);
    pub fn from_bits(bits: u32) -> Option<Self> {
        (bits & !0x303f == 0).then_some(Self(bits))
    }
    pub fn bits(self) -> u32 {
        self.0
    }
    pub fn requests_ast(self) -> bool {
        self.0 & Self::SKIP_ALL.0 != Self::SKIP_ALL.0
    }
    pub fn requests_performance(self) -> bool {
        self.0 & 0x3000 != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerPhase {
    SourceParsed,
    Typechecked { pending_count: u32 },
    TargetCodeBuilt,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerCompletion {
    None,
    CompilationFailed,
    CompilerShutdown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilerEvent {
    Phase {
        workspace: WorkspaceId,
        phase: CompilerPhase,
    },
    Complete {
        workspace: WorkspaceId,
        error: CompilerCompletion,
    },
}
impl CompilerEvent {
    pub fn workspace(&self) -> WorkspaceId {
        match *self {
            Self::Phase { workspace, .. } | Self::Complete { workspace, .. } => workspace,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn intercept_flags_preserve_known_bits_and_reject_unknown_bits() {
        assert!(InterceptFlags::NONE.requests_ast());
        assert!(!InterceptFlags::SKIP_ALL.requests_ast());
        let reports = InterceptFlags::from_bits(0x303f).unwrap();
        assert_eq!(reports.bits(), 0x303f);
        assert!(reports.requests_performance());
        assert!(InterceptFlags::from_bits(0x40).is_none());
        assert!(InterceptFlags::from_bits(u32::MAX).is_none());
    }
}
